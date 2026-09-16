//! Button function records: the 4-byte struct at `96 + 4 * index` that binds a physical button to
//! an action.
//!
//! `docs/research/mouse-protocol-v2.md` section 8.1 (`Ts` enum) and 8.2 (the record layout,
//! identical in both drivers). Types 8 through 11 (`LightSwitch`, `ProfileSwitch`, `DPILock`,
//! `UpDownRoll`) have no menu entry and no documented param encoding anywhere in either bundle;
//! they, and any byte no type here covers, round-trip through [`ButtonAction::Unknown`] instead
//! of being silently dropped.

use crate::encoding::struct_check;

/// A physical mouse button, the type 1 (`MouseKey`) param.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// `0x0100`.
    Left,
    /// `0x0200`.
    Right,
    /// `0x0400`, the wheel click.
    Middle,
    /// `0x0800`.
    Backward,
    /// `0x1000`.
    Forward,
}

impl MouseButton {
    const fn param(self) -> u16 {
        match self {
            Self::Left => 0x0100,
            Self::Right => 0x0200,
            Self::Middle => 0x0400,
            Self::Backward => 0x0800,
            Self::Forward => 0x1000,
        }
    }

    const fn from_param(param: u16) -> Option<Self> {
        match param {
            0x0100 => Some(Self::Left),
            0x0200 => Some(Self::Right),
            0x0400 => Some(Self::Middle),
            0x0800 => Some(Self::Backward),
            0x1000 => Some(Self::Forward),
            _ => None,
        }
    }
}

/// A DPI action, the type 2 (`DPISwitch`) param.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DpiAction {
    /// `0x0100`: cycle through the configured DPI stages.
    Loop,
    /// `0x0200`.
    Increase,
    /// `0x0300`.
    Decrease,
}

impl DpiAction {
    const fn param(self) -> u16 {
        match self {
            Self::Loop => 0x0100,
            Self::Increase => 0x0200,
            Self::Decrease => 0x0300,
        }
    }

    const fn from_param(param: u16) -> Option<Self> {
        match param {
            0x0100 => Some(Self::Loop),
            0x0200 => Some(Self::Increase),
            0x0300 => Some(Self::Decrease),
            _ => None,
        }
    }
}

/// A scroll direction, the type 3 (`LeftRightRoll`) param.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    /// `0x0100`.
    Left,
    /// `0x0200`.
    Right,
}

impl ScrollDirection {
    const fn param(self) -> u16 {
        match self {
            Self::Left => 0x0100,
            Self::Right => 0x0200,
        }
    }

    const fn from_param(param: u16) -> Option<Self> {
        match param {
            0x0100 => Some(Self::Left),
            0x0200 => Some(Self::Right),
            _ => None,
        }
    }
}

/// How many times a bound macro repeats, the low byte of a type 6 (`Macro`) param.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroCycles {
    /// 1..=250: play the macro this many times.
    Times(u8),
    /// 254: repeat for as long as the button stays held.
    UntilReleased,
    /// 255: repeat until any button is pressed.
    UntilAnyPress,
}

impl MacroCycles {
    #[must_use]
    const fn to_byte(self) -> u8 {
        match self {
            Self::Times(n) => n,
            Self::UntilReleased => 254,
            Self::UntilAnyPress => 255,
        }
    }

    #[must_use]
    const fn from_byte(byte: u8) -> Self {
        match byte {
            254 => Self::UntilReleased,
            255 => Self::UntilAnyPress,
            n => Self::Times(n),
        }
    }
}

/// What a button does when pressed.
///
/// Section 8.1. [`Self::Media`] and [`Self::Keystroke`] encode identically (type 5, param 0): the
/// record itself carries no room for a media usage id, only the button index does, and the
/// keystroke slot at that same index (see [`crate::keystroke`]) carries the actual content. A
/// decoded type 5 record is always [`Self::Keystroke`]; `Media` exists so a caller composing a
/// button and its keystroke slot together can say what they mean without inspecting bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonAction {
    /// Type 0: no action. The UI's `NaN` also stores as this.
    Disabled,
    /// Type 1.
    Mouse(MouseButton),
    /// Type 2.
    Dpi(DpiAction),
    /// Type 3.
    Scroll(ScrollDirection),
    /// Type 4 (`FireKey`): repeats a click.
    Fire {
        /// Repeat count, 0..=3; 0 means repeat while held. Unit not documented for the interval.
        times: u8,
        /// Interval between repeats, 10..=255.
        interval_ms: u8,
    },
    /// Type 5 (`ShortcutKey`): the keystroke slot at the button's own index carries the chord.
    Keystroke,
    /// Type 6.
    Macro {
        /// Index of the macro slot to play.
        slot: u8,
        /// How many times it repeats.
        cycles: MacroCycles,
    },
    /// Type 7 (`ReportRateSwitch`).
    PollingCycle,
    /// See the type note above: encodes exactly like [`Self::Keystroke`].
    Media(u16),
    /// Any other type byte, including 8..11 (`LightSwitch`, `ProfileSwitch`, `DPILock`,
    /// `UpDownRoll`), which have no documented param encoding.
    Unknown {
        /// Raw type byte.
        kind: u8,
        /// Raw param, big-endian on the wire.
        param: u16,
    },
}

impl ButtonAction {
    /// Encode this action as the 4-byte record `[type, param hi, param lo, check]`.
    #[must_use]
    pub fn encode(&self) -> [u8; 4] {
        let (kind, param) = match *self {
            Self::Disabled => (0u8, 0u16),
            Self::Mouse(button) => (1, button.param()),
            Self::Dpi(action) => (2, action.param()),
            Self::Scroll(direction) => (3, direction.param()),
            Self::Fire { times, interval_ms } => {
                (4, (u16::from(interval_ms) << 8) | u16::from(times))
            }
            Self::Keystroke | Self::Media(_) => (5, 0),
            Self::Macro { slot, cycles } => {
                (6, (u16::from(slot) << 8) | u16::from(cycles.to_byte()))
            }
            Self::PollingCycle => (7, 0),
            Self::Unknown { kind, param } => (kind, param),
        };
        let param_hi = (param >> 8) as u8;
        let param_lo = (param & 0xff) as u8;
        let check = struct_check(&[kind, param_hi, param_lo]);
        [kind, param_hi, param_lo, check]
    }

    /// Decode a 4-byte key function record.
    ///
    /// Total: every type byte and every param decodes to something, so this never panics or
    /// loses a record's identity, even for firmware-reserved types this document could not fully
    /// recover the meaning of. The check byte is not verified, matching every host driver.
    #[must_use]
    pub fn decode(record: &[u8; 4]) -> Self {
        let kind = record[0];
        let param = (u16::from(record[1]) << 8) | u16::from(record[2]);
        match kind {
            0 => Self::Disabled,
            1 => MouseButton::from_param(param).map_or(Self::Unknown { kind, param }, Self::Mouse),
            2 => DpiAction::from_param(param).map_or(Self::Unknown { kind, param }, Self::Dpi),
            3 => ScrollDirection::from_param(param)
                .map_or(Self::Unknown { kind, param }, Self::Scroll),
            4 => Self::Fire {
                times: (param & 0xff) as u8,
                interval_ms: (param >> 8) as u8,
            },
            5 => Self::Keystroke,
            6 => Self::Macro {
                slot: (param >> 8) as u8,
                cycles: MacroCycles::from_byte((param & 0xff) as u8),
            },
            7 => Self::PollingCycle,
            _ => Self::Unknown { kind, param },
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use proptest::prelude::*;

    #[test]
    fn disabled_matches_the_vendor_zero_record() {
        assert_eq!(ButtonAction::Disabled.encode(), [0, 0, 0, 0x55]);
        assert_eq!(
            ButtonAction::decode(&[0, 0, 0, 0x55]),
            ButtonAction::Disabled
        );
    }

    #[test]
    fn mouse_left_matches_the_documented_param() {
        let encoded = ButtonAction::Mouse(MouseButton::Left).encode();
        assert_eq!(encoded, [1, 0x01, 0x00, struct_check(&[1, 0x01, 0x00])]);
        assert_eq!(
            ButtonAction::decode(&encoded),
            ButtonAction::Mouse(MouseButton::Left)
        );
    }

    #[test]
    fn media_encodes_like_keystroke_and_decodes_as_keystroke() {
        assert_eq!(
            ButtonAction::Media(0x00e9).encode(),
            ButtonAction::Keystroke.encode()
        );
        assert_eq!(
            ButtonAction::decode(&ButtonAction::Media(0x00e9).encode()),
            ButtonAction::Keystroke
        );
    }

    #[test]
    fn unrecognized_types_round_trip_as_unknown() {
        for kind in [8u8, 9, 10, 11, 200] {
            let record = ButtonAction::Unknown {
                kind,
                param: 0x1234,
            }
            .encode();
            assert_eq!(record[0], kind);
            assert_eq!(
                ButtonAction::decode(&record),
                ButtonAction::Unknown {
                    kind,
                    param: 0x1234
                }
            );
        }
    }

    #[test]
    fn a_recognized_type_with_a_garbage_param_decodes_as_unknown() {
        // Type 1 (MouseKey) with a param no MouseButton maps to.
        assert_eq!(
            ButtonAction::decode(&[1, 0x9a, 0xbc, 0]),
            ButtonAction::Unknown {
                kind: 1,
                param: 0x9abc
            }
        );
    }

    #[test]
    fn decode_never_panics_on_any_byte_pattern() {
        for kind in 0u8..=255 {
            for param in [0u16, 1, 0x0100, 0xffff] {
                let hi = (param >> 8) as u8;
                let lo = (param & 0xff) as u8;
                let _ = ButtonAction::decode(&[kind, hi, lo, 0]);
            }
        }
    }

    fn arbitrary_button_action() -> impl Strategy<Value = ButtonAction> {
        prop_oneof![
            Just(ButtonAction::Disabled),
            prop_oneof![
                Just(MouseButton::Left),
                Just(MouseButton::Right),
                Just(MouseButton::Middle),
                Just(MouseButton::Backward),
                Just(MouseButton::Forward),
            ]
            .prop_map(ButtonAction::Mouse),
            prop_oneof![
                Just(DpiAction::Loop),
                Just(DpiAction::Increase),
                Just(DpiAction::Decrease),
            ]
            .prop_map(ButtonAction::Dpi),
            prop_oneof![Just(ScrollDirection::Left), Just(ScrollDirection::Right)]
                .prop_map(ButtonAction::Scroll),
            (0u8..=3, 10u8..=255)
                .prop_map(|(times, interval_ms)| ButtonAction::Fire { times, interval_ms }),
            Just(ButtonAction::Keystroke),
            (0u8..6, 1u8..=250).prop_map(|(slot, n)| ButtonAction::Macro {
                slot,
                cycles: MacroCycles::Times(n)
            }),
            Just(ButtonAction::PollingCycle),
            (8u8..=11, any::<u16>())
                .prop_map(|(kind, param)| ButtonAction::Unknown { kind, param }),
        ]
    }

    proptest! {
        #[test]
        fn every_constructible_action_round_trips(action in arbitrary_button_action()) {
            let encoded = action.encode();
            prop_assert_eq!(ButtonAction::decode(&encoded), action);
        }
    }
}
