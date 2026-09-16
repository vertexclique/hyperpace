//! The real transport: hidapi's vendor configuration collection, on the operator's own hardware.
//!
//! Never exercised by this crate's own test suite (every test in this crate uses
//! [`crate::SimTransport`]). The application opens it, and so do the `probe`, `read_state` and
//! `dump_settings` examples, which have been run against the operator's own device.

use core::time::Duration;

use hidapi::{HidApi, HidDevice, HidError};
use hyperpace_protocol::{FRAME_LEN, REPORT_ID, Transport, TransportError};

use crate::error::DeviceError;

/// Vendor id shared by every link type (protocol reference section 3.1).
const VENDOR_ID: u16 = 0x3554;
/// Product id when connected through the 2.4 GHz receiver.
const PRODUCT_ID_RECEIVER: u16 = 0xFB16;
/// Product id when connected by cable.
const PRODUCT_ID_WIRED: u16 = 0xFB14;
/// Usage page of the vendor configuration collection (protocol reference section 4.1).
const USAGE_PAGE: u16 = 0xFF02;
/// Usage of the vendor configuration collection within that page.
const USAGE: u16 = 0x0002;

/// What the mouse's cable connection can do right now, from [`HidTransport::cable_state`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CableState {
    /// No cable connection is attached.
    NotAttached,
    /// A cable connection is attached and this app may open it.
    Usable,
    /// A cable connection is attached but the operating system refuses this app access to it. On
    /// Linux that means the device access rule Hyperpace ships has not been installed.
    PermissionDenied,
}

/// The vendor configuration collection, opened read-write over hidapi.
///
/// Selects the collection by usage page `USAGE_PAGE` and usage `USAGE` (the only collection
/// on this hardware whose output report id is 8, protocol reference section 4.1), never by
/// interface index, so the same code serves the wireless receiver and the wired link. macOS opens
/// this non-exclusively via the `macos-shared-device` Cargo feature, which this crate enables for
/// that target; Windows and Linux use their respective native hidapi backends.
pub struct HidTransport {
    device: HidDevice,
    /// A link description with no vendor name, model name or branding, for
    /// [`Transport::description`].
    link: &'static str,
}

/// Whether `info` is this product's vendor configuration collection: [`VENDOR_ID`], either
/// product id, and the usage page and usage that identify the config channel (protocol reference
/// section 4.1). Shared with [`crate::hotplug`], so the two never drift apart on what counts.
pub(crate) fn is_vendor_collection(info: &hidapi::DeviceInfo) -> bool {
    info.vendor_id() == VENDOR_ID
        && matches!(info.product_id(), PRODUCT_ID_RECEIVER | PRODUCT_ID_WIRED)
        && info.usage_page() == USAGE_PAGE
        && info.usage() == USAGE
}

impl HidTransport {
    /// Open the best attached configuration collection: the cable when the mouse is plugged in,
    /// otherwise the receiver.
    ///
    /// The cable wins because a mouse on its cable stops talking to the receiver: the receiver's
    /// collection stays attached but its mouse reports offline, so a connection through it would
    /// show the mouse as asleep and could never read its battery or charging state.
    ///
    /// # Errors
    ///
    /// Returns [`DeviceError::Disconnected`] when no such collection is currently attached, or
    /// [`DeviceError::Io`] when the HID subsystem could not be reached or the matching collection
    /// could not be opened.
    pub fn open_first() -> Result<Self, DeviceError> {
        let api = HidApi::new().map_err(|error| DeviceError::Io(error.to_string()))?;
        let candidates: Vec<_> = api
            .device_list()
            .filter(|info| is_vendor_collection(info))
            .collect();
        if candidates.is_empty() {
            return Err(DeviceError::Disconnected);
        }
        // Cable first, receiver second. A candidate that cannot be opened (on Linux, usually a node
        // the access rule has not been applied to yet) falls through to the next one rather than
        // leaving the app with no connection at all; the error from the preferred one is kept,
        // because that is the one the operator can act on.
        let mut ordered: Vec<_> = candidates.iter().collect();
        ordered.sort_by_key(|info| info.product_id() != PRODUCT_ID_WIRED);
        let mut first_error = None;
        for candidate in ordered {
            match candidate.open_device(&api) {
                Ok(device) => {
                    let link = if candidate.product_id() == PRODUCT_ID_WIRED {
                        "wired"
                    } else {
                        "wireless via receiver"
                    };
                    return Ok(Self { device, link });
                }
                Err(error) => {
                    first_error.get_or_insert_with(|| DeviceError::Io(error.to_string()));
                }
            }
        }
        Err(first_error.unwrap_or(DeviceError::Disconnected))
    }

    /// Whether this connection goes over the cable rather than the receiver.
    #[must_use]
    pub fn is_wired(&self) -> bool {
        self.link == "wired"
    }

    /// Whether the mouse's own cable collection is attached right now. Enumeration only; nothing
    /// is opened.
    ///
    /// # Errors
    ///
    /// Returns [`DeviceError::Io`] when the HID subsystem could not be reached.
    pub fn wired_attached() -> Result<bool, DeviceError> {
        let api = HidApi::new().map_err(|error| DeviceError::Io(error.to_string()))?;
        Ok(api
            .device_list()
            .any(|info| is_vendor_collection(info) && info.product_id() == PRODUCT_ID_WIRED))
    }

    /// What the cable connection can do right now.
    ///
    /// The cable is the only way to reach a mouse that is charging or set to wired: while it is
    /// plugged in, the mouse stops answering through the receiver. On Linux its device nodes belong
    /// to root until the access rule Hyperpace ships is installed, and without it the app can see
    /// the cable but not open it, which must be reported as exactly that rather than as no mouse.
    ///
    /// # Errors
    ///
    /// Returns [`DeviceError::Io`] when the HID subsystem could not be reached.
    pub fn cable_state() -> Result<CableState, DeviceError> {
        let api = HidApi::new().map_err(|error| DeviceError::Io(error.to_string()))?;
        let Some(candidate) = api
            .device_list()
            .find(|info| is_vendor_collection(info) && info.product_id() == PRODUCT_ID_WIRED)
        else {
            return Ok(CableState::NotAttached);
        };
        // Opened and dropped at once, with nothing sent: this asks the operating system whether the
        // app may use the cable, it does not talk to the mouse.
        match candidate.open_device(&api) {
            Ok(_) => Ok(CableState::Usable),
            Err(HidError::IoError { error })
                if error.kind() == std::io::ErrorKind::PermissionDenied =>
            {
                Ok(CableState::PermissionDenied)
            }
            // hidapi's Linux backend reports a refused open as a plain message rather than an
            // io error kind, so the message is what says permission when the kind does not.
            Err(error) if error.to_string().to_lowercase().contains("permission") => {
                Ok(CableState::PermissionDenied)
            }
            Err(error) => Err(DeviceError::Io(error.to_string())),
        }
    }
}

/// Map a hidapi error to the coarser [`TransportError`] the rest of this crate reasons about.
///
/// hidapi has no dedicated "the device was unplugged" variant, so this leans on the [`std::io`]
/// error kind the native backends surface for a device that is gone. Opening and reading from the
/// operator's own device has been exercised by hand (see the crate docs), but this mapping has
/// not: doing so means unplugging the device mid-session, which nothing here has done. A kind this
/// does not recognize falls back to a generic, non-fatal [`TransportError::Io`].
fn map_hid_error(error: &HidError) -> TransportError {
    if let HidError::IoError { error: io_error } = error {
        use std::io::ErrorKind::{BrokenPipe, ConnectionAborted, ConnectionReset, NotFound};
        if matches!(
            io_error.kind(),
            NotFound | BrokenPipe | ConnectionAborted | ConnectionReset
        ) {
            return TransportError::Disconnected;
        }
    }
    TransportError::Io(error.to_string())
}

impl Transport for HidTransport {
    fn send(&mut self, frame: &[u8; FRAME_LEN]) -> Result<(), TransportError> {
        let mut report = [0u8; FRAME_LEN + 1];
        report[0] = REPORT_ID;
        report[1..].copy_from_slice(frame);
        self.device
            .write(&report)
            .map_err(|error| map_hid_error(&error))?;
        Ok(())
    }

    fn recv(&mut self, timeout: Duration) -> Result<Option<[u8; FRAME_LEN]>, TransportError> {
        let timeout_ms = i32::try_from(timeout.as_millis()).unwrap_or(i32::MAX);
        let mut report = [0u8; FRAME_LEN + 1];
        let read = self
            .device
            .read_timeout(&mut report, timeout_ms)
            .map_err(|error| map_hid_error(&error))?;
        match read {
            0 => Ok(None),
            n if n == report.len() => {
                let mut frame = [0u8; FRAME_LEN];
                frame.copy_from_slice(&report[1..]);
                Ok(Some(frame))
            }
            n => Err(TransportError::Io(format!("short HID read: {n} bytes"))),
        }
    }

    fn description(&self) -> String {
        format!("Hyperpace mouse ({})", self.link)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    // No test in this module opens or enumerates a real HID device (workspace rule); these cover
    // only the pure, host-independent pieces: identity constants and the error classifier.

    #[test]
    fn identity_constants_match_the_protocol_reference() {
        assert_eq!(VENDOR_ID, 0x3554);
        assert_eq!(PRODUCT_ID_RECEIVER, 0xFB16);
        assert_eq!(PRODUCT_ID_WIRED, 0xFB14);
        assert_eq!(USAGE_PAGE, 0xFF02);
        assert_eq!(USAGE, 0x0002);
    }

    #[test]
    fn a_device_gone_io_kind_maps_to_disconnected() {
        for kind in [
            std::io::ErrorKind::NotFound,
            std::io::ErrorKind::BrokenPipe,
            std::io::ErrorKind::ConnectionAborted,
            std::io::ErrorKind::ConnectionReset,
        ] {
            let error = HidError::IoError {
                error: std::io::Error::from(kind),
            };
            assert_eq!(map_hid_error(&error), TransportError::Disconnected);
        }
    }

    #[test]
    fn an_unrecognized_io_kind_falls_back_to_a_generic_io_error() {
        let error = HidError::IoError {
            error: std::io::Error::from(std::io::ErrorKind::TimedOut),
        };
        assert!(matches!(map_hid_error(&error), TransportError::Io(_)));
    }

    #[test]
    fn a_plain_hidapi_error_is_a_generic_io_error() {
        let error = HidError::HidApiError {
            message: "example".to_owned(),
        };
        assert!(matches!(map_hid_error(&error), TransportError::Io(_)));
    }
}
