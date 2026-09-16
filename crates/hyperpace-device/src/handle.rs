//! The handle every caller uses to talk to a connected device's owner thread.

use std::sync::mpsc;
use std::time::Duration;

use hyperpace_protocol::{Frame, Shadow};

use crate::error::DeviceError;
use crate::event::{Access, DeviceEvent};

/// One unit of work the owner thread services, in the order its command channel delivers them.
/// Every variant carries its own reply channel, so a caller's method call blocks on that reply
/// rather than on the owner thread's internal state.
pub(crate) enum OwnerCommand {
    /// Send `frame` and wait up to `timeout` for a reply matching its command byte.
    Request {
        frame: Frame,
        timeout: Duration,
        reply: mpsc::Sender<Result<Frame, DeviceError>>,
    },
    /// Return a clone of the owner thread's current settings shadow.
    ReadSettings {
        reply: mpsc::Sender<Result<Shadow, DeviceError>>,
    },
    /// Read `len` bytes starting at `address`, chunked by the owner thread.
    ReadBlock {
        address: u16,
        len: usize,
        reply: mpsc::Sender<Result<Vec<u8>, DeviceError>>,
    },
    /// Write a scalar pair at `address`.
    WriteScalar {
        address: u16,
        value: u8,
        reply: mpsc::Sender<Result<(), DeviceError>>,
    },
    /// Write `data` at `address`, chunked by the owner thread.
    WriteBlock {
        address: u16,
        data: Vec<u8>,
        reply: mpsc::Sender<Result<(), DeviceError>>,
    },
    /// Register a fresh event subscriber and hand back its receiver.
    Subscribe {
        reply: mpsc::Sender<mpsc::Receiver<DeviceEvent>>,
    },
}

/// A cheap-to-clone handle to one device's owner thread.
///
/// Every method sends one `OwnerCommand` and waits for its reply; the owner thread is the only
/// thing that ever touches the [`hyperpace_protocol::Transport`], so writes queued from any
/// number of cloned handles are serialized by construction.
#[derive(Clone)]
pub struct DeviceHandle {
    pub(crate) commands: mpsc::Sender<OwnerCommand>,
    pub(crate) access: Access,
}

/// Map a dropped reply channel (the owner thread exited before answering) to
/// [`DeviceError::Disconnected`], so every public method shares one mapping instead of each
/// repeating it.
fn recv_reply<T>(rx: &mpsc::Receiver<Result<T, DeviceError>>) -> Result<T, DeviceError> {
    rx.recv().unwrap_or(Err(DeviceError::Disconnected))
}

impl DeviceHandle {
    /// Send `frame` and wait up to `timeout` for a reply that echoes its command byte.
    ///
    /// This is the escape hatch for any command [`DeviceHandle`] has no dedicated method for.
    /// When `frame` carries a command that mutates device state, it is subject to the same
    /// [`Access::ReadOnly`] gate as [`DeviceHandle::write_scalar`] and
    /// [`DeviceHandle::write_block`].
    ///
    /// # Errors
    ///
    /// Returns [`DeviceError::ReadOnly`], [`DeviceError::Timeout`] or
    /// [`DeviceError::Disconnected`] as documented on [`DeviceError`].
    pub fn request(&self, frame: Frame, timeout: Duration) -> Result<Frame, DeviceError> {
        let (reply, rx) = mpsc::channel();
        if self
            .commands
            .send(OwnerCommand::Request {
                frame,
                timeout,
                reply,
            })
            .is_err()
        {
            return Err(DeviceError::Disconnected);
        }
        recv_reply(&rx)
    }

    /// A clone of the owner thread's current settings shadow.
    ///
    /// The shadow reflects every flash offset the connect sequence and any write have touched so
    /// far; an offset nothing has read yet decodes as erased flash (`0xFF`), matching the
    /// device's own convention.
    ///
    /// # Errors
    ///
    /// Returns [`DeviceError::Disconnected`] if the owner thread has already exited.
    pub fn read_settings(&self) -> Result<Shadow, DeviceError> {
        let (reply, rx) = mpsc::channel();
        if self
            .commands
            .send(OwnerCommand::ReadSettings { reply })
            .is_err()
        {
            return Err(DeviceError::Disconnected);
        }
        recv_reply(&rx)
    }

    /// Read `len` bytes starting at `address`, split into [`hyperpace_protocol::PAYLOAD_LEN`]-byte
    /// `ReadFlashData` requests.
    ///
    /// Unlike [`DeviceHandle::read_settings`], this reaches flash offsets the connect walk never
    /// covers (the keystroke and macro regions, `docs/research/mouse-protocol-v2.md` section 6.1
    /// start at offset 256, past the walk's 256-byte span): a caller that needs one of those
    /// addresses reads it directly instead of waiting for a wider walk that never happens. Every
    /// byte read is also mirrored into the owner thread's shadow, so a later [`DeviceHandle::read_settings`]
    /// reflects it too.
    ///
    /// # Errors
    ///
    /// Returns [`DeviceError::Timeout`] or [`DeviceError::Disconnected`] as documented on
    /// [`DeviceError`]. Never refused under [`Access::ReadOnly`]: reading flash is not a write.
    pub fn read_block(&self, address: u16, len: usize) -> Result<Vec<u8>, DeviceError> {
        let (reply, rx) = mpsc::channel();
        if self
            .commands
            .send(OwnerCommand::ReadBlock {
                address,
                len,
                reply,
            })
            .is_err()
        {
            return Err(DeviceError::Disconnected);
        }
        recv_reply(&rx)
    }

    /// Write the two-byte scalar pair for `value` at `address`.
    ///
    /// # Errors
    ///
    /// Returns [`DeviceError::ReadOnly`] under [`Access::ReadOnly`], without touching the
    /// transport; otherwise the errors documented on [`DeviceError`].
    pub fn write_scalar(&self, address: u16, value: u8) -> Result<(), DeviceError> {
        let (reply, rx) = mpsc::channel();
        if self
            .commands
            .send(OwnerCommand::WriteScalar {
                address,
                value,
                reply,
            })
            .is_err()
        {
            return Err(DeviceError::Disconnected);
        }
        recv_reply(&rx)
    }

    /// Write `data` at `address`, split into [`hyperpace_protocol::PAYLOAD_LEN`]-byte frames.
    ///
    /// Stops at the first chunk that fails; the shadow mirrors only the chunks that already
    /// succeeded.
    ///
    /// # Errors
    ///
    /// Returns [`DeviceError::ReadOnly`] under [`Access::ReadOnly`], without touching the
    /// transport; otherwise the errors documented on [`DeviceError`].
    pub fn write_block(&self, address: u16, data: &[u8]) -> Result<(), DeviceError> {
        let (reply, rx) = mpsc::channel();
        if self
            .commands
            .send(OwnerCommand::WriteBlock {
                address,
                data: data.to_vec(),
                reply,
            })
            .is_err()
        {
            return Err(DeviceError::Disconnected);
        }
        recv_reply(&rx)
    }

    /// Subscribe to this device's events from now on.
    ///
    /// Every call returns an independent receiver that observes every event from this point
    /// forward; past events are not replayed. When the owner thread has already exited, the
    /// returned receiver is immediately disconnected, matching what a live subscription sees the
    /// instant the device goes away.
    #[must_use]
    pub fn events(&self) -> mpsc::Receiver<DeviceEvent> {
        let (reply, rx) = mpsc::channel();
        if self
            .commands
            .send(OwnerCommand::Subscribe { reply })
            .is_ok()
            && let Ok(events) = rx.recv()
        {
            return events;
        }
        let (_never_sent, events) = mpsc::channel();
        events
    }

    /// Whether this handle may write to its device.
    #[must_use]
    pub fn access(&self) -> Access {
        self.access
    }
}
