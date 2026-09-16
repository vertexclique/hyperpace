//! The one error type every command in this crate converts to `String` at its boundary.
//!
//! `docs/architecture/api-contract.md` fixes every Tauri command to `Result<T, String>`, so the
//! richer error is only useful inside this crate: it carries enough detail to log or match on,
//! and its `Display` impl is the one place that turns it into the plain sentence the UI shows
//! (the honesty fence in `AGENTS.md`: a user-facing string says what happened, not an internal
//! type name).

use core::fmt;

use hyperpace_device::DeviceError;
use hyperpace_firmware::FirmwareError;
use hyperpace_protocol::{ConfigError, ProtocolError};
use hyperpace_store::StoreError;

/// Why a command in this crate could not complete.
#[derive(Debug)]
pub enum AppError {
    /// A command that needs a connected device was called with none active.
    NotConnected,
    /// The connected device's `cid`/`mid` pair matches no known [`hyperpace_protocol::ModelTable`].
    UnknownModel {
        /// Component id the device reported.
        cid: u8,
        /// Module id the device reported.
        mid: u8,
    },
    /// No document in the local store matches the id a command was given.
    NotFound {
        /// What kind of record was looked up.
        kind: &'static str,
        /// The id that matched nothing.
        id: String,
    },
    /// A firmware install was attempted without the write access it requires.
    FirmwareWriteNotAuthorized,
    /// The simulator backend cannot exercise a firmware install: it never models the update
    /// bootloader (`docs/research/firmware-update-spec.md` section 5 onward), so pretending it
    /// could would be a fabricated result rather than a real one.
    FirmwareSimulatorUnsupported,
    /// The device did not answer the reconnect probe after entering update mode within the
    /// bounded window this crate allows.
    FirmwareReconnectTimeout,
    /// The operator turned the firmware watch route off (`app_settings`'s
    /// `firmware_watch_enabled` key; `commands::firmware_watch::FIRMWARE_WATCH_ENABLED_KEY`).
    FirmwareWatchDisabled,
    /// The HTTP client the firmware watch route needs could not be built, or a request it sent
    /// failed outright (a per-target fetch failure inside a check is not this: it is recorded in
    /// the report's `fetch_errors` instead, so one unreachable target never fails the whole
    /// check).
    Http(reqwest::Error),
    /// The device layer reported an error.
    Device(DeviceError),
    /// The protocol codec reported an error.
    Protocol(ProtocolError),
    /// The `.bin` config file codec reported an error.
    Config(ConfigError),
    /// The firmware engine reported an error.
    Firmware(FirmwareError),
    /// The local store reported an error.
    Store(StoreError),
    /// A filesystem operation on the store's own files (the firmware archive) failed.
    Io(std::io::Error),
    /// The Tauri runtime reported an error building or driving a window, tray or channel.
    Tauri(tauri::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotConnected => write!(f, "no device is connected"),
            Self::UnknownModel { cid, mid } => {
                write!(
                    f,
                    "the device reported cid {cid} mid {mid}, which is not a recognized model"
                )
            }
            Self::NotFound { kind, id } => write!(f, "no {kind} with id \"{id}\""),
            Self::FirmwareWriteNotAuthorized => write!(
                f,
                "firmware install needs write access to the device; reconnect with write access enabled"
            ),
            Self::FirmwareSimulatorUnsupported => write!(
                f,
                "the simulator does not model the update bootloader; firmware install needs a real device"
            ),
            Self::FirmwareReconnectTimeout => write!(
                f,
                "the device did not reappear in update mode before the timeout"
            ),
            Self::FirmwareWatchDisabled => write!(
                f,
                "checking for firmware publication is turned off in app preferences"
            ),
            Self::Http(error) => write!(f, "could not reach the vendor site: {error}"),
            Self::Device(error) => write!(f, "{error}"),
            Self::Protocol(error) => write!(f, "{error}"),
            Self::Config(error) => write!(f, "{error}"),
            Self::Firmware(error) => write!(f, "{error}"),
            Self::Store(error) => write!(f, "{error}"),
            Self::Io(error) => write!(f, "local file error: {error}"),
            Self::Tauri(error) => write!(f, "{error}"),
        }
    }
}

impl core::error::Error for AppError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Device(error) => Some(error),
            Self::Protocol(error) => Some(error),
            Self::Config(error) => Some(error),
            Self::Firmware(error) => Some(error),
            Self::Store(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::Tauri(error) => Some(error),
            Self::Http(error) => Some(error),
            _ => None,
        }
    }
}

impl From<DeviceError> for AppError {
    fn from(error: DeviceError) -> Self {
        Self::Device(error)
    }
}

impl From<ProtocolError> for AppError {
    fn from(error: ProtocolError) -> Self {
        Self::Protocol(error)
    }
}

impl From<ConfigError> for AppError {
    fn from(error: ConfigError) -> Self {
        Self::Config(error)
    }
}

impl From<FirmwareError> for AppError {
    fn from(error: FirmwareError) -> Self {
        Self::Firmware(error)
    }
}

impl From<StoreError> for AppError {
    fn from(error: StoreError) -> Self {
        Self::Store(error)
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<tauri::Error> for AppError {
    fn from(error: tauri::Error) -> Self {
        Self::Tauri(error)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(error: reqwest::Error) -> Self {
        Self::Http(error)
    }
}

/// Turn any command result into the `Result<T, String>` shape the API contract fixes.
///
/// # Errors
///
/// Returns `result`'s own error, rendered through [`AppError`]'s `Display`.
pub fn to_command_result<T>(result: Result<T, AppError>) -> Result<T, String> {
    result.map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_variant_has_a_non_empty_display() {
        let variants: Vec<AppError> = vec![
            AppError::NotConnected,
            AppError::UnknownModel { cid: 1, mid: 2 },
            AppError::NotFound {
                kind: "macro",
                id: "abc".to_owned(),
            },
            AppError::FirmwareWriteNotAuthorized,
            AppError::FirmwareSimulatorUnsupported,
            AppError::FirmwareReconnectTimeout,
            AppError::FirmwareWatchDisabled,
            AppError::Device(DeviceError::Disconnected),
        ];
        for variant in variants {
            assert!(!variant.to_string().is_empty());
        }
    }

    #[test]
    fn to_command_result_maps_the_error_to_its_display_string() {
        let result: Result<(), AppError> = Err(AppError::NotConnected);
        assert_eq!(
            to_command_result(result),
            Err("no device is connected".to_owned())
        );
    }
}
