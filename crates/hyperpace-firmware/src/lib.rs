//! Firmware update engine for Hyperpace.
//!
//! Implements `docs/research/firmware-update-spec.md` end to end: the `ComUsbUpgradeFile`
//! container (a 720 byte header, its checksum, and multi-image chaining), the identity checks
//! that decide whether a package targets this hardware, the preflight guards a caller must clear
//! before flashing, and the flash state machine that streams a package to a device already
//! sitting in its bootloader.
//!
//! # No genuine package exists for this hardware
//!
//! `docs/plans/hyperpace.md`: no HYPACE firmware image has been found on any channel reached.
//! Every number this crate acts on comes from thirty-five archived *sibling* images (other
//! vendors on the same ODM platform, `firmware/archive/other-devices/`) and from source code that
//! never ran against this device. This crate's own tests parse those sibling images directly and
//! prove they classify as [`package::Match::ForeignDevice`] for this hardware; none of them is
//! flashable to it, and nothing in this crate has ever been exercised against real hardware. See
//! [`mod@flash`] for the further, specific gap this leaves in the flash state machine's fidelity to
//! the vendor's own wire format.
//!
//! # Never opens a device
//!
//! No function here calls into hidapi or touches `/dev/hidraw*` or `/dev/bus/usb`; every
//! interaction with a device goes through [`hyperpace_protocol::Transport`], a pure trait a
//! caller already connected. See [`mod@flash`]'s module documentation for exactly what this crate
//! assumes has already happened before it is called.
//!
//! # Layers
//!
//! - [`header`]: one image's 720 byte header, its checksum, and its on-wire command bytes.
//! - [`package`]: a parsed package (one image, or a chain), and the identity checks that decide
//!   whether it targets this hardware.
//! - [`mod@guards`]: the preflight checks a caller runs before ever flashing.
//! - [`mod@flash`]: the state machine that streams a preflighted package.

pub mod error;
pub mod flash;
pub mod guards;
pub mod header;
pub mod package;

#[cfg(test)]
mod test_support;

pub use error::FirmwareError;
pub use flash::{Progress, flash};
pub use guards::{Guards, MIN_BATTERY_PERCENT, guards, preflight};
pub use header::{DeviceType, ImageHeader, UsbEndpoint};
pub use package::{
    Match, Package, RECEIVER_PRODUCT_ID, UsbIds, VENDOR_ID, WIRED_PRODUCT_ID,
    order_for_multi_target,
};
