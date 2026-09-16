//! The state a connected device can push to its subscribers, and the write permission gate.

use hyperpace_protocol::{Battery, DeviceIdentity, StatusChanged};

/// Whether a [`crate::DeviceHandle`] may write to the device it owns.
///
/// [`Access::ReadOnly`] is the default every [`crate::spawn`] caller gets unless it asks for
/// [`Access::ReadWrite`] explicitly; a write attempted under [`Access::ReadOnly`] returns
/// [`crate::DeviceError::ReadOnly`] without the owner thread ever touching the transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Access {
    /// Reads and pushes only; every write is refused before it reaches the transport.
    #[default]
    ReadOnly,
    /// Reads, pushes and writes.
    ReadWrite,
}

/// Something the owner thread observed about its device, delivered to every subscriber
/// registered through [`crate::DeviceHandle::events`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceEvent {
    /// The connect sequence finished: the device answered its handshake and the base settings
    /// walk completed.
    Connected(DeviceIdentity),
    /// A battery poll (every 5 seconds once connected) read a fresh value.
    Battery(Battery),
    /// The device pushed a `StatusChanged` (command 10) frame, unprompted.
    Changed(StatusChanged),
    /// The transport reported the device unreachable; the owner thread has exited and this
    /// handle's requests will now fail with [`crate::DeviceError::Disconnected`].
    Disconnected,
    /// A battery poll's online probe came back negative, or timed out; the device is asleep or
    /// out of range. The owner thread returns to the connect sequence and will emit another
    /// [`DeviceEvent::Connected`] once the device answers again.
    Offline,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn access_defaults_to_read_only() {
        assert_eq!(Access::default(), Access::ReadOnly);
    }
}
