//! The device command table.
//!
//! `docs/research/mouse-protocol-v2.md` section 5. The NEW driver's enum has 16 ids; the OLD
//! driver additionally sends two receiver-light commands the NEW driver dropped. Hyperpace keeps
//! every command either driver actually sends, renamed away from vendor terms ("dongle" becomes
//! "receiver").

use crate::frame::Frame;

/// A command byte the device channel understands.
///
/// Byte values match `docs/research/mouse-protocol-v2.md` section 5 exactly; ids the vendor
/// enums declare but never send (11, 12, 16, 17, 20, 21, 176, 177, 240, 241) are not represented.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Command {
    /// `EncryptionData`: challenge and identity handshake.
    Handshake = 1,
    /// `PCDriverStatus`: tells the device whether a host driver is present.
    DriverStatus = 2,
    /// `DeviceOnLine`: online probe, sent before almost every setter.
    Online = 3,
    /// `BatteryLevel`.
    Battery = 4,
    /// `DongleEnterPair`.
    EnterPair = 5,
    /// `GetPairState`.
    PairState = 6,
    /// `WriteFlashData`.
    WriteFlash = 7,
    /// `ReadFlashData`.
    ReadFlash = 8,
    /// `ClearSetting`: factory reset.
    FactoryReset = 9,
    /// `StatusChanged`: device push, never sent by a host.
    StatusChanged = 10,
    /// `EnterUsbUpdateMode`.
    EnterUpdateMode = 13,
    /// `GetCurrentConfig`: active profile.
    GetProfile = 14,
    /// `SetCurrentConfig`.
    SetProfile = 15,
    /// `ReadVersionID`.
    ReadVersion = 18,
    /// `SetLongRangeMode`.
    SetLongRange = 22,
    /// `GetLongRangeMode`.
    GetLongRange = 23,
    /// `SetDongleLight`, renamed: sets the receiver's indicator light.
    SetReceiverLight = 24,
    /// `GetDongleLight`, renamed.
    GetReceiverLight = 25,
    /// `GetDongleVersion`, renamed: the receiver's own firmware version.
    GetReceiverVersion = 29,
}

impl Command {
    /// The command whose byte value is `byte`, or `None` for a byte no command uses.
    #[must_use]
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            1 => Some(Self::Handshake),
            2 => Some(Self::DriverStatus),
            3 => Some(Self::Online),
            4 => Some(Self::Battery),
            5 => Some(Self::EnterPair),
            6 => Some(Self::PairState),
            7 => Some(Self::WriteFlash),
            8 => Some(Self::ReadFlash),
            9 => Some(Self::FactoryReset),
            10 => Some(Self::StatusChanged),
            13 => Some(Self::EnterUpdateMode),
            14 => Some(Self::GetProfile),
            15 => Some(Self::SetProfile),
            18 => Some(Self::ReadVersion),
            22 => Some(Self::SetLongRange),
            23 => Some(Self::GetLongRange),
            24 => Some(Self::SetReceiverLight),
            25 => Some(Self::GetReceiverLight),
            29 => Some(Self::GetReceiverVersion),
            _ => None,
        }
    }

    /// A bare request frame carrying only this command, empty payload.
    #[must_use]
    pub fn request(self) -> Frame {
        Frame::command(self as u8)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn every_documented_byte_round_trips_through_from_byte() {
        let ids = [
            1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 13, 14, 15, 18, 22, 23, 24, 25, 29,
        ];
        for id in ids {
            let command = Command::from_byte(id).unwrap();
            assert_eq!(command as u8, id);
        }
    }

    #[test]
    fn bytes_the_vendor_enum_declares_but_never_sends_are_not_commands() {
        for id in [0u8, 11, 12, 16, 17, 20, 21, 176, 177, 240, 241, 255] {
            assert_eq!(Command::from_byte(id), None, "byte {id} should not decode");
        }
    }

    #[test]
    fn request_builds_a_bare_frame() {
        let frame = Command::Online.request();
        assert_eq!(frame.command, 3);
        assert_eq!(frame.length, 0);
        assert_eq!(frame.payload, [0; crate::frame::PAYLOAD_LEN]);
    }
}
