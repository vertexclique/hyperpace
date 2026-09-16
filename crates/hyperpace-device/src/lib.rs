//! Device access for Hyperpace: one owner thread per device, the hotplug watcher, and the
//! simulator every test in this workspace drives instead of real hardware.
//!
//! This crate implements [`hyperpace_protocol::Transport`] twice: [`HidTransport`] over hidapi,
//! for the operator's real device, and [`SimTransport`], the simulator. Neither this crate nor
//! anything it runs opens, reads or writes a real `/dev/hidraw*` node or USB device on its own;
//! [`HidTransport`] is complete and correct as written but is exercised only when the application
//! the operator launches by hand calls it. Every test here, and every fact this crate asserts
//! about itself, comes from [`SimTransport`].
//!
//! # Layers
//!
//! - [`error`]: [`DeviceError`], the error every fallible call in this crate returns.
//! - [`event`]: [`Access`], the write permission gate, and [`DeviceEvent`], what a connected
//!   device can push to its subscribers.
//! - [`handle`]: [`DeviceHandle`], the cheap-to-clone handle every caller talks to.
//! - [`owner`]: the owner thread itself: [`spawn`], the connect sequence, request and reply
//!   matching, battery polling, and push dispatch.
//! - [`sim`]: [`SimTransport`] and [`SimController`], the device simulator and its test-side
//!   remote control.
//! - [`hid`]: [`HidTransport`], the real transport over hidapi's vendor configuration collection.
//! - [`hotplug`]: [`watch`] and [`HotplugEvent`], the polling hotplug watcher hidapi itself has no
//!   notification for.

pub mod error;
pub mod event;
pub mod handle;
pub mod hid;
pub mod hotplug;
pub mod owner;
pub mod sim;

pub use error::DeviceError;
pub use event::{Access, DeviceEvent};
pub use handle::DeviceHandle;
pub use hid::HidTransport;
pub use hotplug::{HotplugEvent, watch};
pub use owner::spawn;
pub use sim::{SimController, SimTransport};
