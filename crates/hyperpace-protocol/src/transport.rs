//! The pure transport contract.
//!
//! Contract amendment: `Transport` lives here, not in `hyperpace-device`, so `hyperpace-firmware`
//! can drive one without depending on anything that opens a device. This module performs no I/O
//! itself; it only names the shape a byte-moving implementation must have. `hyperpace-device`
//! provides `HidTransport` and `SimTransport`; `hyperpace-firmware` is generic over any
//! `Transport` implementation for its own 64-byte upgrade packets.

use core::fmt;
use core::time::Duration;

use crate::frame::FRAME_LEN;

/// Why a transport could not send or receive a frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportError {
    /// The device is no longer reachable (closed, unplugged, or the owner thread exited).
    Disconnected,
    /// The underlying I/O layer reported an error; its message is kept for diagnostics.
    Io(String),
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disconnected => write!(f, "the device is disconnected"),
            Self::Io(message) => write!(f, "transport error: {message}"),
        }
    }
}

impl core::error::Error for TransportError {}

/// Moves raw, already-framed bytes to and from a device. Implementations own no protocol
/// knowledge: they neither build nor parse a [`crate::frame::Frame`], only carry its encoded
/// bytes.
pub trait Transport: Send {
    /// Send one encoded frame.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError`] when the frame could not be written.
    fn send(&mut self, frame: &[u8; FRAME_LEN]) -> Result<(), TransportError>;

    /// Wait up to `timeout` for one encoded frame.
    ///
    /// Returns `Ok(None)` on a timeout with nothing received, never an error: a caller distinguishes
    /// "nothing arrived yet" from "the transport is broken" by the `Result`, not by the `Option`.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError`] when the transport itself failed, not when it merely timed out.
    fn recv(&mut self, timeout: Duration) -> Result<Option<[u8; FRAME_LEN]>, TransportError>;

    /// A human-readable description of the underlying device or simulator, for logs and the UI.
    fn description(&self) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A transport that always fails, exercising the trait object contract with no I/O.
    struct AlwaysDisconnected;

    impl Transport for AlwaysDisconnected {
        fn send(&mut self, _frame: &[u8; FRAME_LEN]) -> Result<(), TransportError> {
            Err(TransportError::Disconnected)
        }

        fn recv(&mut self, _timeout: Duration) -> Result<Option<[u8; FRAME_LEN]>, TransportError> {
            Err(TransportError::Disconnected)
        }

        fn description(&self) -> String {
            "always disconnected".to_owned()
        }
    }

    #[test]
    fn a_transport_implementation_is_object_safe_and_send() {
        let mut transport: Box<dyn Transport> = Box::new(AlwaysDisconnected);
        assert_eq!(
            transport.send(&[0; FRAME_LEN]),
            Err(TransportError::Disconnected)
        );
        assert_eq!(
            transport.recv(Duration::from_millis(1)),
            Err(TransportError::Disconnected)
        );
        assert_eq!(transport.description(), "always disconnected");
    }

    #[test]
    fn transport_error_displays_a_plain_message() {
        assert_eq!(
            TransportError::Io("closed".to_owned()).to_string(),
            "transport error: closed"
        );
        assert_eq!(
            TransportError::Disconnected.to_string(),
            "the device is disconnected"
        );
    }
}
