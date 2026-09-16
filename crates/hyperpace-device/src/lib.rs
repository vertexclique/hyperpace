//! Device access for Hyperpace: one owner thread per device, the hotplug watcher, and the
//! simulator every test in this workspace drives instead of real hardware.
//!
//! This crate implements [`hyperpace_protocol::Transport`] twice: [`HidTransport`] over hidapi,
//! for the operator's real device, and [`SimTransport`], the simulator. No test in this crate
//! opens a real `/dev/hidraw*` node: every test here, and every fact this crate asserts about
//! itself, comes from [`SimTransport`]. The real transport is exercised by the application, and
//! by the two examples, which are run by hand:
//!
//! - `cargo run -p hyperpace-device --example probe` lists the attached collections without
//!   opening any of them, answering "the mouse is plugged in, so why is it not found?".
//! - `cargo run -p hyperpace-device --example read_state` connects read-only and prints the
//!   identity and the battery readings as they arrive.
//!
//! Both were run against the operator's own device on 2026-09-16: `probe` found the configuration
//! collection (usage page `0xff02`, usage `0x0002`) exactly where the protocol reference says it
//! is, and `read_state` reported cid 102, mid 1, an 8000 Hz wireless link and a battery reading.
//! No write has ever been exercised against that device.
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
pub use hid::{CableState, HidTransport};
pub use hotplug::{HotplugEvent, watch};
pub use owner::spawn;
pub use sim::{SimController, SimTransport};
