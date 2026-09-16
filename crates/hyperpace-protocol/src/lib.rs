//! Wire codec for the Hyperpace device channel.
//!
//! This crate is pure data: it builds and parses the 16 byte frames the device exchanges on its
//! vendor HID collection, and it owns every value encoding those frames carry. It performs no
//! I/O, opens no device and spawns no thread, so every rule in it can be tested against vectors
//! taken from the vendor implementation without hardware.
//!
//! The wire format is documented in `docs/research/mouse-protocol-v2.md`; the numbers here are
//! only ever changed together with that document.
//!
//! # Layers
//!
//! - [`frame`]: the transport envelope, its checksum and its status byte.
//! - [`command`]: the command byte table.
//! - [`request`]: one frame builder per command a host sends.
//! - [`response`]: typed reply parsers.
//! - [`model`]: the per-hardware constants (cid 102 and cid 62).
//! - [`settings`]: the device memory map, a shadow copy of it, and the decoded [`settings::Settings`].
//! - [`encoding`]: every value conversion between host units and device bytes.
//! - [`buttons`]: the button function record.
//! - [`keystroke`]: the keystroke slot codec.
//! - [`macros`]: the macro slot codec.
//! - [`config_file`]: the `.bin` export and import format.
//! - [`transport`]: the pure transport contract other crates implement and use.

pub mod buttons;
pub mod command;
pub mod config_file;
pub mod encoding;
pub mod frame;
pub mod keystroke;
pub mod macros;
pub mod model;
pub mod request;
pub mod response;
pub mod settings;
pub mod transport;

pub use buttons::{ButtonAction, DpiAction, MacroCycles, MouseButton, ScrollDirection};
pub use command::Command;
pub use config_file::{ConfigError, ImportedConfig};
pub use encoding::{
    dpi_from_bytes, dpi_to_bytes, indicator_brightness_from_byte, indicator_brightness_to_byte,
    polling_from_byte, polling_to_byte, scalar_pair, struct_check,
};
pub use frame::{FRAME_LEN, Frame, FrameError, PAYLOAD_LEN, REPORT_ID, Status, checksum};
pub use keystroke::{Keystroke, Modifier};
pub use macros::{MacroEvent, MacroEventKind, MacroSlot};
pub use model::{DpiRange, ModelTable, table_for};
pub use response::{
    Battery, DeviceIdentity, LinkType, PairPhase, PairState, ProtocolError, StatusChanged, Version,
    battery, identity, online, pair_state, status_changed, version,
};
pub use settings::{
    DpiStage, LightMode, Lighting, Lod, Performance, ReceiverLight, Settings, Shadow, SleepTime,
    offset,
};
pub use transport::{Transport, TransportError};
