//! Errors a device-facing call can return.

use core::fmt;

use hyperpace_protocol::{FrameError, TransportError};

/// Why a [`crate::DeviceHandle`] call did not produce the value it asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceError {
    /// A write was attempted while the handle's [`crate::Access`] is [`crate::Access::ReadOnly`];
    /// the transport was never touched.
    ReadOnly,
    /// No matching reply arrived before the caller's deadline.
    Timeout,
    /// The transport reported the device unreachable (closed, unplugged, or the owner thread
    /// already exited).
    Disconnected,
    /// The transport's underlying I/O layer reported an error; its message is kept for
    /// diagnostics.
    Io(String),
    /// A write could not be framed, for example a block write whose chunk length still exceeded
    /// what a single frame carries.
    InvalidWrite(String),
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadOnly => write!(f, "the device connection is read-only"),
            Self::Timeout => write!(f, "no reply from the device before the deadline"),
            Self::Disconnected => write!(f, "the device is disconnected"),
            Self::Io(message) => write!(f, "device I/O error: {message}"),
            Self::InvalidWrite(message) => write!(f, "invalid write: {message}"),
        }
    }
}

impl core::error::Error for DeviceError {}

impl From<TransportError> for DeviceError {
    fn from(error: TransportError) -> Self {
        match error {
            TransportError::Disconnected => Self::Disconnected,
            TransportError::Io(message) => Self::Io(message),
        }
    }
}

impl From<FrameError> for DeviceError {
    fn from(error: FrameError) -> Self {
        Self::InvalidWrite(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_errors_map_to_their_device_error() {
        assert_eq!(
            DeviceError::from(TransportError::Disconnected),
            DeviceError::Disconnected
        );
        assert_eq!(
            DeviceError::from(TransportError::Io("closed".to_owned())),
            DeviceError::Io("closed".to_owned())
        );
    }

    #[test]
    fn frame_errors_map_to_an_invalid_write() {
        let error = DeviceError::from(FrameError::PayloadTooLong(11));
        assert!(matches!(error, DeviceError::InvalidWrite(_)));
    }

    #[test]
    fn display_messages_are_plain_sentences() {
        assert_eq!(
            DeviceError::ReadOnly.to_string(),
            "the device connection is read-only"
        );
        assert_eq!(
            DeviceError::Timeout.to_string(),
            "no reply from the device before the deadline"
        );
    }
}
