//! Reply parsers: typed values decoded from a [`Frame`] the device sent back.
//!
//! `docs/research/mouse-protocol-v2.md` section 5. Every parser here checks the command byte and
//! the status byte before reading a payload, unlike either vendor driver, which accepts a
//! status-1 (unsupported) reply as success before any check (section 4.6, corrected per 4.11).

use core::fmt;

use crate::frame::{Frame, Status};

/// Why a reply could not be decoded, or a value could not be encoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolError {
    /// The frame's command byte was not the one this parser decodes.
    WrongCommand {
        /// Command byte the parser expected.
        expected: u8,
        /// Command byte the frame actually carried.
        got: u8,
    },
    /// The device reported it does not implement this command (status byte 1).
    Unsupported {
        /// Command byte the device rejected.
        command: u8,
    },
    /// Not enough bytes were available to decode a variable-length record.
    Truncated {
        /// Bytes the decode needed.
        needed: usize,
        /// Bytes actually available.
        got: usize,
    },
    /// A field held a value this codec has no mapping for.
    InvalidValue {
        /// Name of the field that held the bad value.
        field: &'static str,
    },
    /// A macro name was empty or longer than [`crate::macros::MAX_NAME_LEN`] bytes.
    NameTooLong {
        /// The name's actual UTF-8 byte length.
        len: usize,
    },
    /// A macro had more than [`crate::macros::MAX_EVENTS`] events.
    TooManyEvents {
        /// The actual event count.
        count: usize,
    },
    /// A macro name's declared bytes were not valid UTF-8.
    InvalidUtf8,
    /// A requested DPI value is below every range, or above the range that would cover it.
    DpiOutOfRange {
        /// The DPI value that was out of range.
        dpi: u32,
    },
    /// A requested DPI value is not reachable on the model's step grid.
    DpiNotRepresentable {
        /// The DPI value that could not be represented.
        dpi: u32,
    },
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongCommand { expected, got } => {
                write!(
                    f,
                    "expected a reply to command {expected}, got command {got}"
                )
            }
            Self::Unsupported { command } => {
                write!(f, "the device does not support command {command}")
            }
            Self::Truncated { needed, got } => {
                write!(f, "needed at least {needed} bytes, got {got}")
            }
            Self::InvalidValue { field } => write!(f, "invalid value in field {field}"),
            Self::NameTooLong { len } => {
                write!(f, "a macro name must be 1 to 30 UTF-8 bytes, got {len}")
            }
            Self::TooManyEvents { count } => {
                write!(f, "a macro slot holds at most 70 events, got {count}")
            }
            Self::InvalidUtf8 => write!(f, "macro name bytes are not valid UTF-8"),
            Self::DpiOutOfRange { dpi } => write!(f, "{dpi} DPI is outside every table range"),
            Self::DpiNotRepresentable { dpi } => {
                write!(f, "{dpi} DPI does not land on the model's step grid")
            }
        }
    }
}

impl core::error::Error for ProtocolError {}

/// Check that `frame` answers `command` and was decoded, not rejected as unsupported.
fn expect_reply(frame: &Frame, command: u8) -> Result<(), ProtocolError> {
    if frame.command != command {
        return Err(ProtocolError::WrongCommand {
            expected: command,
            got: frame.command,
        });
    }
    if frame.status() == Status::Unsupported {
        return Err(ProtocolError::Unsupported { command });
    }
    Ok(())
}

/// The link a device reported, byte 11 of the handshake reply (section 3.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkType {
    /// Byte 0: 2.4 GHz wireless, host caps polling at 1000 Hz.
    Wireless1k,
    /// Byte 1: 2.4 GHz wireless, 4000 Hz.
    Wireless4k,
    /// Byte 2: wired, 1000 Hz.
    Wired1k,
    /// Byte 3: wired, 8000 Hz.
    Wired8k,
    /// Byte 4: 2.4 GHz wireless, 2000 Hz.
    Wireless2k,
    /// Byte 5: 2.4 GHz wireless, 8000 Hz.
    Wireless8k,
    /// Any other byte; the vendor host treats it as wireless and keeps the previous cap.
    Unknown(u8),
}

impl LinkType {
    /// The polling rate cap the host applies for this link.
    #[must_use]
    pub fn max_polling_hz(self) -> u16 {
        match self {
            Self::Wireless1k | Self::Wired1k | Self::Unknown(_) => 1000,
            Self::Wireless4k => 4000,
            Self::Wired8k | Self::Wireless8k => 8000,
            Self::Wireless2k => 2000,
        }
    }
}

impl From<u8> for LinkType {
    fn from(byte: u8) -> Self {
        match byte {
            0 => Self::Wireless1k,
            1 => Self::Wireless4k,
            2 => Self::Wired1k,
            3 => Self::Wired8k,
            4 => Self::Wireless2k,
            5 => Self::Wireless8k,
            other => Self::Unknown(other),
        }
    }
}

/// The identity a device reports in its handshake reply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceIdentity {
    /// Component id, selects a [`crate::model::ModelTable`] together with `mid`.
    pub cid: u8,
    /// Module id.
    pub mid: u8,
    /// Link type and its polling rate cap.
    pub link: LinkType,
}

/// Decode an `EncryptionData` (command 1) reply.
///
/// # Errors
///
/// Returns [`ProtocolError::WrongCommand`] if `frame` did not answer command 1.
pub fn identity(frame: &Frame) -> Result<DeviceIdentity, ProtocolError> {
    expect_reply(frame, 1)?;
    Ok(DeviceIdentity {
        cid: frame.payload[4],
        mid: frame.payload[5],
        link: LinkType::from(frame.payload[6]),
    })
}

/// Battery state, a `BatteryLevel` (command 4) reply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Battery {
    /// Charge percent as the device reports it, before any smoothing.
    pub percent: u8,
    /// Whether the device is charging.
    pub charging: bool,
    /// Battery voltage in millivolts.
    pub millivolts: u16,
}

/// Decode a `BatteryLevel` reply.
///
/// Reads the four payload bytes directly; the reply's declared length byte is never read by
/// either vendor host, so this parser does not trust it either (section 5).
///
/// # Errors
///
/// Returns [`ProtocolError::WrongCommand`] if `frame` did not answer command 4.
pub fn battery(frame: &Frame) -> Result<Battery, ProtocolError> {
    expect_reply(frame, 4)?;
    Ok(Battery {
        percent: frame.payload[0],
        charging: frame.payload[1] == 1,
        millivolts: u16::from_be_bytes([frame.payload[2], frame.payload[3]]),
    })
}

/// A firmware version, from `ReadVersionID` (18) or `GetDongleVersion` (29).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    /// Decimal major version.
    pub major: u8,
    /// Hex-encoded minor version.
    pub minor: u8,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}.{:02x}", self.major, self.minor)
    }
}

/// Decode a `ReadVersionID` or `GetDongleVersion` reply; both share the same layout.
///
/// # Errors
///
/// Returns [`ProtocolError::WrongCommand`] if `frame` answered neither command, and
/// [`ProtocolError::Unsupported`] if the device marked the command unsupported.
pub fn version(frame: &Frame) -> Result<Version, ProtocolError> {
    if frame.command != 18 && frame.command != 29 {
        return Err(ProtocolError::WrongCommand {
            expected: 18,
            got: frame.command,
        });
    }
    if frame.status() == Status::Unsupported {
        return Err(ProtocolError::Unsupported {
            command: frame.command,
        });
    }
    Ok(Version {
        major: frame.payload[0],
        minor: frame.payload[1],
    })
}

/// Decode a `DeviceOnLine` (command 3) reply: online flag plus the raw receiver address bytes.
///
/// # Errors
///
/// Returns [`ProtocolError::WrongCommand`] if `frame` did not answer command 3.
pub fn online(frame: &Frame) -> Result<(bool, [u8; 3]), ProtocolError> {
    expect_reply(frame, 3)?;
    Ok((
        frame.payload[0] != 0,
        [frame.payload[1], frame.payload[2], frame.payload[3]],
    ))
}

/// Pairing progress, `GetPairState` (command 6) byte 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairPhase {
    /// 1: pairing in progress.
    Pairing,
    /// 2: pairing failed.
    Failed,
    /// 3: pairing succeeded.
    Succeeded,
    /// Any other byte.
    Other(u8),
}

impl From<u8> for PairPhase {
    fn from(byte: u8) -> Self {
        match byte {
            1 => Self::Pairing,
            2 => Self::Failed,
            3 => Self::Succeeded,
            other => Self::Other(other),
        }
    }
}

/// A `GetPairState` reply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PairState {
    /// Pairing phase.
    pub state: PairPhase,
    /// Seconds left in the pairing window.
    pub seconds_left: u8,
}

/// Decode a `GetPairState` reply.
///
/// # Errors
///
/// Returns [`ProtocolError::WrongCommand`] if `frame` did not answer command 6.
pub fn pair_state(frame: &Frame) -> Result<PairState, ProtocolError> {
    expect_reply(frame, 6)?;
    Ok(PairState {
        state: PairPhase::from(frame.payload[0]),
        seconds_left: frame.payload[1],
    })
}

/// Decode a `GetLongRangeMode` (23) reply: whether long range mode is on.
///
/// Not part of the flash shadow (section 7.9): long range is a dedicated command pair, so this
/// parser, unlike [`crate::settings::Shadow::settings`], is the only way to learn its state.
///
/// # Errors
///
/// Returns [`ProtocolError::WrongCommand`] if `frame` did not answer command 23, and
/// [`ProtocolError::Unsupported`] if the device marked long range unsupported (status 1): a
/// caller must map that to an honest "unsupported" state, never to `false`.
pub fn long_range(frame: &Frame) -> Result<bool, ProtocolError> {
    expect_reply(frame, 23)?;
    Ok(frame.payload[0] == 1)
}

/// A `StatusChanged` (command 10) push, decoded for the mouse's bit assignments (section 9.4);
/// bits 0x10 and 0x80 are keyboard-only and carry no meaning here.
// Six independent, orthogonal change flags, matching the device's own bitmask (section 9.4)
// exactly; the fixed shape comes from `docs/architecture/api-contract.md`, not a design choice.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusChanged {
    /// Bit 0x01: current DPI stage changed.
    pub dpi: bool,
    /// Bit 0x02: polling rate changed.
    pub polling: bool,
    /// Bit 0x04: active profile changed.
    pub profile: bool,
    /// Bit 0x08: DPI indicator settings changed.
    pub dpi_indicator: bool,
    /// Bit 0x20: lighting settings changed.
    pub lighting: bool,
    /// Bit 0x40: battery state changed.
    pub battery: bool,
}

/// Decode a `StatusChanged` push.
///
/// # Errors
///
/// Returns [`ProtocolError::WrongCommand`] if `frame` did not carry command 10.
pub fn status_changed(frame: &Frame) -> Result<StatusChanged, ProtocolError> {
    if frame.command != 10 {
        return Err(ProtocolError::WrongCommand {
            expected: 10,
            got: frame.command,
        });
    }
    let bits = frame.payload[0];
    Ok(StatusChanged {
        dpi: bits & 0x01 != 0,
        polling: bits & 0x02 != 0,
        profile: bits & 0x04 != 0,
        dpi_indicator: bits & 0x08 != 0,
        lighting: bits & 0x20 != 0,
        battery: bits & 0x40 != 0,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::frame::PAYLOAD_LEN;

    fn reply(command: u8, status: u8, payload: &[u8]) -> Frame {
        let mut p = [0u8; PAYLOAD_LEN];
        p[..payload.len()].copy_from_slice(payload);
        Frame {
            command,
            status,
            address: 0,
            length: u8::try_from(payload.len()).unwrap(),
            payload: p,
        }
    }

    #[test]
    fn identity_reads_cid_mid_and_link() {
        let frame = reply(1, 0, &[0, 0, 0, 0, 102, 1, 4]);
        let identity = identity(&frame).unwrap();
        assert_eq!(identity.cid, 102);
        assert_eq!(identity.mid, 1);
        assert_eq!(identity.link, LinkType::Wireless2k);
        assert_eq!(identity.link.max_polling_hz(), 2000);
    }

    #[test]
    fn identity_refuses_a_reply_to_the_wrong_command() {
        let frame = reply(3, 0, &[]);
        assert_eq!(
            identity(&frame),
            Err(ProtocolError::WrongCommand {
                expected: 1,
                got: 3
            })
        );
    }

    #[test]
    fn battery_reads_percent_charging_and_millivolts() {
        let frame = reply(4, 0, &[73, 1, 0x0e, 0x74]);
        let battery = battery(&frame).unwrap();
        assert_eq!(battery.percent, 73);
        assert!(battery.charging);
        assert_eq!(battery.millivolts, 0x0e74);
    }

    #[test]
    fn version_reads_from_either_of_its_two_commands() {
        let mouse = reply(18, 0, &[2, 0x16]);
        assert_eq!(version(&mouse).unwrap().to_string(), "v2.16");
        let receiver = reply(29, 0, &[1, 0x00]);
        assert_eq!(version(&receiver).unwrap().to_string(), "v1.00");
    }

    #[test]
    fn version_reports_unsupported_status() {
        let frame = reply(29, 1, &[0, 0]);
        assert_eq!(
            version(&frame),
            Err(ProtocolError::Unsupported { command: 29 })
        );
    }

    #[test]
    fn online_reads_the_flag_and_the_raw_address() {
        let frame = reply(3, 0, &[1, 0xaa, 0xbb, 0xcc]);
        assert_eq!(online(&frame).unwrap(), (true, [0xaa, 0xbb, 0xcc]));
    }

    #[test]
    fn pair_state_reads_phase_and_seconds() {
        let frame = reply(6, 0, &[1, 18]);
        let state = pair_state(&frame).unwrap();
        assert_eq!(state.state, PairPhase::Pairing);
        assert_eq!(state.seconds_left, 18);
    }

    #[test]
    fn long_range_reads_the_on_flag() {
        let on = reply(23, 0, &[1]);
        assert_eq!(long_range(&on), Ok(true));
        let off = reply(23, 0, &[0]);
        assert_eq!(long_range(&off), Ok(false));
    }

    #[test]
    fn long_range_reports_unsupported_status_not_a_silent_off() {
        let frame = reply(23, 1, &[0]);
        assert_eq!(
            long_range(&frame),
            Err(ProtocolError::Unsupported { command: 23 })
        );
    }

    #[test]
    fn long_range_refuses_a_reply_to_the_wrong_command() {
        let frame = reply(22, 0, &[1]);
        assert_eq!(
            long_range(&frame),
            Err(ProtocolError::WrongCommand {
                expected: 23,
                got: 22
            })
        );
    }

    #[test]
    fn status_changed_decodes_every_mouse_bit() {
        let frame = reply(10, 0, &[0b0110_1111]);
        let changed = status_changed(&frame).unwrap();
        assert!(changed.dpi);
        assert!(changed.polling);
        assert!(changed.profile);
        assert!(changed.dpi_indicator);
        assert!(changed.lighting);
        assert!(changed.battery);
    }
}
