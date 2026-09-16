//! Firmware archive and install commands.
//!
//! `docs/plans/hyperpace.md`: the firmware engine is built to the verified spec and ships
//! enabled, but no genuine package exists for this hardware on any channel reached, and flashing
//! has never been observed on real hardware. Nothing here claims otherwise: [`firmware_install`]
//! refuses outright against the simulator (which never models the update bootloader) rather than
//! fabricate a result, and against a real device it follows the documented procedure, including
//! the one gap the research itself names: the device's boot-mode USB identity is unknown, so the
//! reconnect step below can open only a device still reporting its normal-mode identity, and
//! fails loudly, on a bounded budget, if that is all it ever finds.

use std::fmt::Write as _;
use std::time::{Duration, Instant};

use hyperpace_device::{Access, HidTransport};
use hyperpace_firmware::{
    FirmwareError, Match, Package, Progress, RECEIVER_PRODUCT_ID, UsbIds, VENDOR_ID,
    WIRED_PRODUCT_ID, flash, preflight,
};
use hyperpace_protocol::{Command, DeviceIdentity, LinkType, response};
use hyperpace_store::FirmwareRecord;
use sha2::{Digest, Sha256};
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager};

use crate::blocking::blocking;
use crate::dto::{DeviceBackendDto, FirmwareProgressPayload, FirmwareRecordDto};
use crate::error::AppError;
use crate::error::to_command_result;
use crate::state::{AppState, REQUEST_TIMEOUT};

/// How long `firmware_install` polls for the device to re-enumerate after entering update mode,
/// matching the vendor watchdog's own ~10 s budget (`docs/research/firmware-update-spec.md`
/// section 10's R1).
const RECONNECT_BUDGET: Duration = Duration::from_secs(10);
/// How often the reconnect poll retries [`HidTransport::open_first`].
const RECONNECT_POLL_INTERVAL: Duration = Duration::from_millis(300);

/// Every package in the local firmware archive.
///
/// # Errors
///
/// Returns an error message when the store cannot be reached.
#[tauri::command]
pub async fn firmware_list(app: AppHandle) -> Result<Vec<FirmwareRecordDto>, String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            let records = state.store.firmware().list()?;
            Ok(records
                .into_iter()
                .map(|(id, record)| FirmwareRecordDto::from_record(id, record))
                .collect())
        })
        .await,
    )
}

/// Import a firmware package into the local archive. Requires a connected device: the package's
/// declared identity must resolve to [`Match::Target`] against it, per the plan's acquisition
/// route ("an import path that accepts an operator-supplied package only when its identity
/// markers match this hardware").
///
/// # Errors
///
/// Returns an error message when no device is connected, `bytes` is not a valid firmware
/// package, its declared identity does not resolve to [`Match::Target`], or the archive file
/// could not be written.
#[tauri::command]
pub async fn firmware_import(app: AppHandle, bytes: Vec<u8>) -> Result<FirmwareRecordDto, String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            let (_handle, _table, identity, _access, _battery) = state.connected_model()?;

            let package = Package::parse(&bytes)?;
            let first = package.images.first().ok_or(FirmwareError::Empty)?;
            let usb_ids = derive_usb_ids(identity);
            let identity_match = package.matches(&identity, &usb_ids);
            if identity_match != Match::Target {
                return Err(FirmwareError::NotTarget {
                    found: identity_match,
                }
                .into());
            }

            let sha256 = hex_sha256(&bytes);
            let path = format!("firmware/{sha256}.bin");
            let full_path = state.store_root().join(&path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&full_path, &bytes)?;

            let record = FirmwareRecord {
                product: first.product.clone(),
                version: format_version(first.version.major, first.version.minor),
                cid: first.cid,
                mid: first.mid,
                path,
                sha256,
                imported_at: unix_now(),
            };
            let id = state.store.firmware().create(&record)?;
            Ok(FirmwareRecordDto::from_record(id, record))
        })
        .await,
    )
}

/// Every archived package that targets the connected device and declares a version newer than
/// what it currently reports, per `ReadVersionID`. A purely local comparison: no vendor page is
/// reached (`docs/plans/hyperpace.md`'s watcher acquisition route is not implemented here).
///
/// # Errors
///
/// Returns an error message when no device is connected or it did not answer the version
/// request in time.
#[tauri::command]
pub async fn firmware_check_for_updates(app: AppHandle) -> Result<Vec<FirmwareRecordDto>, String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            let (handle, _table, identity, _access, _battery) = state.connected_model()?;
            let reply = handle.request(Command::ReadVersion.request(), REQUEST_TIMEOUT)?;
            let current = response::version(&reply)?;
            let current = (current.major, current.minor);

            let mut updates = Vec::new();
            for (id, record) in state.store.firmware().list()? {
                if record.cid != identity.cid || record.mid != identity.mid {
                    continue;
                }
                if parse_version(&record.version).is_some_and(|version| version > current) {
                    updates.push(FirmwareRecordDto::from_record(id, record));
                }
            }
            Ok(updates)
        })
        .await,
    )
}

/// Flash archived package `id` to the connected device, streaming progress through `channel`.
///
/// Refuses outright unless the connection has write access to a real device
/// (`check_write_authorized`); see the module documentation for what this does and does not
/// prove.
///
/// # Errors
///
/// Returns an error message when no device is connected, write access was not requested at
/// `connect`, the connection is the simulator, `id` names no archived package, the package's
/// identity or a preflight guard fails, the device does not reappear before the reconnect
/// budget runs out, or the flash itself fails partway through.
#[tauri::command]
pub async fn firmware_install(
    app: AppHandle,
    id: String,
    channel: Channel<FirmwareProgressPayload>,
) -> Result<(), String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            let (handle, _table, identity, access, battery_percent) = state.connected_model()?;
            let backend = state.backend()?;
            check_write_authorized(access, backend)?;

            let record = state
                .store
                .firmware()
                .get(&id)?
                .ok_or_else(|| AppError::NotFound {
                    kind: "firmware package",
                    id: id.clone(),
                })?;
            let bytes = std::fs::read(state.store_root().join(&record.path))?;
            let package = Package::parse(&bytes)?;

            let usb_ids = derive_usb_ids(identity);
            let identity_match = package.matches(&identity, &usb_ids);
            if identity_match != Match::Target {
                return Err(FirmwareError::NotTarget {
                    found: identity_match,
                }
                .into());
            }
            preflight(&package, &identity, battery_percent)?;

            // Section 5's recommendation: do not require the reset frame's echo, since whether
            // this device even sends one is unverified; treat any reply, or none, as
            // informational and move straight to polling for re-enumeration.
            let _ = handle.request(Command::EnterUpdateMode.request(), REQUEST_TIMEOUT);
            state.disconnect();

            let mut boot_transport = reconnect_to_bootloader(RECONNECT_BUDGET)?;
            let mut report_progress = |progress: Progress| {
                let _ = channel.send(FirmwareProgressPayload::from(progress));
            };
            flash(&package, &mut boot_transport, &mut report_progress)?;
            Ok(())
        })
        .await,
    )
}

/// The USB vendor and product id the connected device is reachable at, inferred from its link
/// type: [`LinkType::Wired1k`] or [`LinkType::Wired8k`] means the wired product id, every other
/// link means the 2.4 GHz receiver's.
fn derive_usb_ids(identity: DeviceIdentity) -> UsbIds {
    let product_id = if matches!(identity.link, LinkType::Wired1k | LinkType::Wired8k) {
        WIRED_PRODUCT_ID
    } else {
        RECEIVER_PRODUCT_ID
    };
    UsbIds {
        vendor_id: VENDOR_ID,
        product_id,
    }
}

/// Every non-I/O check `firmware_install` must pass before it ever resets the device. Kept
/// separate from the reset/reconnect/flash sequence so it is testable without a real device.
///
/// # Errors
///
/// Returns [`AppError::FirmwareWriteNotAuthorized`] unless `access` is
/// [`Access::ReadWrite`], and [`AppError::FirmwareSimulatorUnsupported`] unless `backend` is
/// [`DeviceBackendDto::RealDevice`].
fn check_write_authorized(access: Access, backend: DeviceBackendDto) -> Result<(), AppError> {
    if access != Access::ReadWrite {
        return Err(AppError::FirmwareWriteNotAuthorized);
    }
    if backend != DeviceBackendDto::RealDevice {
        return Err(AppError::FirmwareSimulatorUnsupported);
    }
    Ok(())
}

/// Poll [`HidTransport::open_first`] until it opens a collection or `budget` runs out.
///
/// # Errors
///
/// Returns [`AppError::FirmwareReconnectTimeout`] once `budget` elapses with no successful open.
// vertexia: `HidTransport::open_first` matches only the normal-mode product ids
// (`docs/research/firmware-update-spec.md`: "no real HYPACE boot PID is known"), so on real
// hardware this can structurally never find the rebooted device and will exhaust `budget` every
// time. Ceiling: firmware install cannot complete against real hardware today. Upgrade path: once
// a genuine package's boot PID is observed, give `hyperpace-device` a boot-mode-aware open
// function (matching that PID, not the normal-mode ones `is_vendor_collection` checks) and use it
// here instead.
fn reconnect_to_bootloader(budget: Duration) -> Result<HidTransport, AppError> {
    let deadline = Instant::now() + budget;
    loop {
        if let Ok(transport) = HidTransport::open_first() {
            return Ok(transport);
        }
        if Instant::now() >= deadline {
            return Err(AppError::FirmwareReconnectTimeout);
        }
        std::thread::sleep(RECONNECT_POLL_INTERVAL);
    }
}

/// Format a firmware version the same way [`hyperpace_store::FirmwareRecord::version`] documents:
/// `"{major}.{minor:02x}"`, with no `v` prefix (unlike
/// [`hyperpace_protocol::Version`]'s own `Display`, which this crate must not duplicate the
/// meaning of under a different string shape without saying so).
fn format_version(major: u8, minor: u8) -> String {
    format!("{major}.{minor:02x}")
}

/// The inverse of [`format_version`]: `None` for a string this crate did not write itself.
fn parse_version(text: &str) -> Option<(u8, u8)> {
    let (major, minor) = text.split_once('.')?;
    Some((major.parse().ok()?, u8::from_str_radix(minor, 16).ok()?))
}

/// Hex-encoded sha256 of `bytes`.
fn hex_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest
        .iter()
        .fold(String::with_capacity(64), |mut acc, byte| {
            let _ = write!(acc, "{byte:02x}");
            acc
        })
}

/// Unix seconds, clamped to 0 rather than panicking, for the rare host clock set before 1970.
///
/// `pub(crate)` so `commands::firmware_watch` can stamp a watch report's `checkedAt` with the same
/// clock read the archive import path uses, rather than a second copy of the same three lines.
pub(crate) fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| {
            i64::try_from(duration.as_secs()).unwrap_or(i64::MAX)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_authorization_needs_both_read_write_and_a_real_device() {
        assert!(matches!(
            check_write_authorized(Access::ReadOnly, DeviceBackendDto::RealDevice),
            Err(AppError::FirmwareWriteNotAuthorized)
        ));
        assert!(matches!(
            check_write_authorized(Access::ReadWrite, DeviceBackendDto::Simulator),
            Err(AppError::FirmwareSimulatorUnsupported)
        ));
        assert!(check_write_authorized(Access::ReadWrite, DeviceBackendDto::RealDevice).is_ok());
    }

    #[test]
    fn usb_ids_follow_the_link_types_wired_flag() {
        let wired = DeviceIdentity {
            cid: 0,
            mid: 0,
            link: LinkType::Wired8k,
        };
        let wireless = DeviceIdentity {
            cid: 0,
            mid: 0,
            link: LinkType::Wireless2k,
        };
        assert_eq!(derive_usb_ids(wired).product_id, WIRED_PRODUCT_ID);
        assert_eq!(derive_usb_ids(wireless).product_id, RECEIVER_PRODUCT_ID);
    }

    #[test]
    fn version_formatting_round_trips() {
        assert_eq!(format_version(2, 0x16), "2.16");
        assert_eq!(parse_version("2.16"), Some((2, 0x16)));
        assert_eq!(parse_version("10.00"), Some((10, 0)));
    }

    #[test]
    fn version_comparison_is_numeric_not_lexicographic() {
        // A lexicographic compare would say "10.00" < "2.16"; the tuple form must not.
        assert!(parse_version("10.00") > parse_version("2.16"));
    }

    #[test]
    fn a_malformed_version_string_does_not_parse() {
        assert_eq!(parse_version("not-a-version"), None);
        assert_eq!(parse_version("2.zz"), None);
    }

    #[test]
    fn sha256_is_stable_and_hex_encoded() {
        let digest = hex_sha256(b"hyperpace");
        assert_eq!(digest.len(), 64);
        assert!(digest.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(digest, hex_sha256(b"hyperpace"));
    }
}
