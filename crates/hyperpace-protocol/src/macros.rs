//! Macro slot codec: the 384-byte buffer at `768 + 384 * index` a type 6 button plays.
//!
//! `docs/research/mouse-protocol-v2.md` section 8.7:
//!
//! ```text
//! +0        name length in UTF-8 bytes, valid range 1..=30
//! +1..+30   UTF-8 name, 0xFF padded, no check byte of its own
//! +31       event count, valid range 0..=70
//! +32 + 5k  event k: [(status << 6) + kind, value lo, value hi, delay hi, delay lo]
//! last      check byte over the count byte plus every event
//! ```
//!
//! A name length outside 1..=30 makes the vendor reader return null forever (section 8.7); an
//! all-`0xFF` slot, the state connect leaves an unanswered read in, has length 255 and hits the
//! same rule. [`MacroSlot::decode`] treats both as "no macro programmed" rather than an error,
//! matching every host driver's own null handling instead of the null-dereference bug they carry
//! alongside it.

use crate::encoding::struct_check;
use crate::response::ProtocolError;

/// Largest UTF-8 byte length a macro name may have.
pub const MAX_NAME_LEN: usize = 30;
/// Largest number of events a macro slot may hold (33 + 5 * 70 = 383 bytes, section 8.7).
pub const MAX_EVENTS: usize = 70;

/// What kind of input one macro event replays.
///
/// Section 8.8: keyboard recording only ever produces [`Self::Modifier`] and [`Self::Key`] (plus
/// the keymap's lone context-menu entry, which lands in [`Self::Other`]); Insert Command
/// additionally reaches [`Self::Mouse`]. [`Self::Consumer`] (media) is a valid slot kind that no
/// web-authored macro happens to use. [`Self::Other`] keeps any other 4-bit value round-trippable
/// instead of losing it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroEventKind {
    /// Kind 0: a modifier bitmask, the same values as [`crate::keystroke::Modifier`].
    Modifier,
    /// Kind 1: a HID keyboard page usage id.
    Key,
    /// Kind 2: a Consumer Control usage id.
    Consumer,
    /// Kind 4: a mouse button bitmask (`0x0001` left, `0x0002` right, `0x0004` middle, `0x0008`
    /// backward, `0x0010` forward), reachable only through Insert Command.
    Mouse,
    /// Any other 4-bit value.
    Other(u8),
}

impl MacroEventKind {
    const fn to_nibble(self) -> u8 {
        match self {
            Self::Modifier => 0,
            Self::Key => 1,
            Self::Consumer => 2,
            Self::Mouse => 4,
            Self::Other(n) => n & 0x0f,
        }
    }

    const fn from_nibble(nibble: u8) -> Self {
        match nibble & 0x0f {
            0 => Self::Modifier,
            1 => Self::Key,
            2 => Self::Consumer,
            4 => Self::Mouse,
            other => Self::Other(other),
        }
    }
}

/// One recorded macro event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacroEvent {
    /// `true` for a press, `false` for a release. The device firmware's decoder maps only the
    /// bit pattern `0b10` to press and every other pattern (including an erased `0xFF` byte) to
    /// release (section 8.7); decoding matches that behavior.
    pub press: bool,
    /// What the event replays.
    pub kind: MacroEventKind,
    /// The key, modifier bit, usage id or mouse bitmask, little-endian on the wire.
    pub value: u16,
    /// Delay before the next event, big-endian on the wire. Unit is not documented in code.
    pub delay_ms: u16,
}

impl MacroEvent {
    /// The 5-byte wire form of this event.
    #[must_use]
    fn to_bytes(self) -> [u8; 5] {
        let status = if self.press { 0b10 } else { 0b01 };
        let status_kind = (status << 6) | self.kind.to_nibble();
        let [value_lo, value_hi] = self.value.to_le_bytes();
        let [delay_hi, delay_lo] = self.delay_ms.to_be_bytes();
        [status_kind, value_lo, value_hi, delay_hi, delay_lo]
    }

    /// Decode one 5-byte event record.
    #[must_use]
    fn from_bytes(bytes: &[u8]) -> Self {
        let status_kind = bytes[0];
        let press = (status_kind >> 6) & 0b11 == 0b10;
        let kind = MacroEventKind::from_nibble(status_kind);
        let value = u16::from_le_bytes([bytes[1], bytes[2]]);
        let delay_ms = u16::from_be_bytes([bytes[3], bytes[4]]);
        Self {
            press,
            kind,
            value,
            delay_ms,
        }
    }
}

/// One macro: its name and its recorded events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroSlot {
    /// Macro name, at most [`MAX_NAME_LEN`] UTF-8 bytes.
    pub name: String,
    /// Recorded events, at most [`MAX_EVENTS`].
    pub events: Vec<MacroEvent>,
}

impl MacroSlot {
    /// Encode this macro as the slot's data bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::NameTooLong`] when the name is empty or longer than
    /// [`MAX_NAME_LEN`] UTF-8 bytes, and [`ProtocolError::TooManyEvents`] when there are more than
    /// [`MAX_EVENTS`].
    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        let name_bytes = self.name.as_bytes();
        if name_bytes.is_empty() || name_bytes.len() > MAX_NAME_LEN {
            return Err(ProtocolError::NameTooLong {
                len: name_bytes.len(),
            });
        }
        if self.events.len() > MAX_EVENTS {
            return Err(ProtocolError::TooManyEvents {
                count: self.events.len(),
            });
        }

        let mut data = Vec::with_capacity(33 + 5 * self.events.len());
        data.push(u8::try_from(name_bytes.len()).unwrap_or(u8::MAX));
        data.extend_from_slice(name_bytes);
        data.extend(std::iter::repeat_n(0xffu8, MAX_NAME_LEN - name_bytes.len()));

        let count_at = data.len();
        data.push(u8::try_from(self.events.len()).unwrap_or(u8::MAX));
        for event in &self.events {
            data.extend_from_slice(&event.to_bytes());
        }
        data.push(struct_check(&data[count_at..]));
        Ok(data)
    }

    /// Decode a macro slot from its data bytes.
    ///
    /// Returns `Ok(None)` for "no macro programmed": a name length byte outside 1..=30, which
    /// covers both a freshly erased (`0xFF`-filled) slot and the zeroed slot a delete leaves
    /// behind (section 8.7). Event and continuation reads are clamped to `bytes`, bounding the
    /// device's own uncapped count and length bytes (section 8.7's continuation-read table)
    /// instead of trusting them.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::Truncated`] when `bytes` is too short to hold the 32-byte header,
    /// and [`ProtocolError::InvalidUtf8`] when the declared name bytes are not valid UTF-8.
    pub fn decode(bytes: &[u8]) -> Result<Option<Self>, ProtocolError> {
        let &name_len = bytes
            .first()
            .ok_or(ProtocolError::Truncated { needed: 1, got: 0 })?;
        let max_name_len = u8::try_from(MAX_NAME_LEN).unwrap_or(u8::MAX);
        if !(1..=max_name_len).contains(&name_len) {
            return Ok(None);
        }
        if bytes.len() < 32 {
            return Err(ProtocolError::Truncated {
                needed: 32,
                got: bytes.len(),
            });
        }
        let name_bytes = &bytes[1..=usize::from(name_len)];
        let name = str::from_utf8(name_bytes)
            .map_err(|_| ProtocolError::InvalidUtf8)?
            .to_owned();

        let count = usize::from(bytes[31]).min(MAX_EVENTS);
        let mut events = Vec::with_capacity(count);
        for i in 0..count {
            let base = 32 + 5 * i;
            let Some(chunk) = bytes.get(base..base + 5) else {
                break;
            };
            events.push(MacroEvent::from_bytes(chunk));
        }
        Ok(Some(Self { name, events }))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use proptest::prelude::*;

    fn sample_event() -> MacroEvent {
        MacroEvent {
            press: true,
            kind: MacroEventKind::Key,
            value: 4,
            delay_ms: 10,
        }
    }

    #[test]
    fn a_one_event_macro_round_trips() {
        let slot = MacroSlot {
            name: "hi".to_owned(),
            events: vec![sample_event()],
        };
        let encoded = slot.encode().unwrap();
        assert_eq!(encoded[0], 2); // name length
        assert_eq!(&encoded[1..3], b"hi");
        assert_eq!(encoded[31], 1); // event count
        assert_eq!(encoded.len(), 33 + 5);
        assert_eq!(MacroSlot::decode(&encoded).unwrap(), Some(slot));
    }

    #[test]
    fn the_name_field_is_0xff_padded() {
        let slot = MacroSlot {
            name: "ab".to_owned(),
            events: vec![],
        };
        let encoded = slot.encode().unwrap();
        assert_eq!(&encoded[3..31], [0xffu8; 28]);
    }

    #[test]
    fn a_zero_length_name_is_refused() {
        let slot = MacroSlot {
            name: String::new(),
            events: vec![],
        };
        assert_eq!(slot.encode(), Err(ProtocolError::NameTooLong { len: 0 }));
    }

    #[test]
    fn a_name_over_30_bytes_is_refused() {
        let slot = MacroSlot {
            name: "x".repeat(31),
            events: vec![],
        };
        assert_eq!(slot.encode(), Err(ProtocolError::NameTooLong { len: 31 }));
    }

    #[test]
    fn more_than_70_events_is_refused() {
        let slot = MacroSlot {
            name: "many".to_owned(),
            events: vec![sample_event(); 71],
        };
        assert_eq!(
            slot.encode(),
            Err(ProtocolError::TooManyEvents { count: 71 })
        );
    }

    #[test]
    fn an_erased_slot_decodes_to_no_macro() {
        let erased = [0xffu8; 384];
        assert_eq!(MacroSlot::decode(&erased).unwrap(), None);
    }

    #[test]
    fn a_zeroed_slot_from_a_delete_decodes_to_no_macro() {
        let zeroed = [0u8; 384];
        assert_eq!(MacroSlot::decode(&zeroed).unwrap(), None);
    }

    #[test]
    fn an_erased_event_byte_decodes_as_a_release() {
        // Section 8.7: an erased 0xFF byte decodes as a plausible release, not an error.
        let event = MacroEvent::from_bytes(&[0xff, 0xff, 0xff, 0xff, 0xff]);
        assert!(!event.press);
    }

    #[test]
    fn a_slot_shorter_than_the_header_is_truncated_not_a_panic() {
        assert_eq!(
            MacroSlot::decode(&[5]),
            Err(ProtocolError::Truncated { needed: 32, got: 1 })
        );
    }

    #[test]
    fn decode_clamps_the_event_count_to_the_given_slice() {
        // A device-supplied count larger than what the slice can hold must not panic or read OOB.
        let mut bytes = vec![0u8; 32]; // header only: no room for any event
        bytes[0] = 5; // valid name length
        bytes[1..6].copy_from_slice(b"hello");
        bytes[31] = 70; // claims 70 events but the slice holds none
        let decoded = MacroSlot::decode(&bytes).unwrap().unwrap();
        assert!(decoded.events.is_empty());
    }

    #[test]
    fn invalid_utf8_in_the_name_is_reported_not_silently_replaced() {
        let mut bytes = vec![0u8; 32];
        bytes[0] = 1;
        bytes[1] = 0xff; // not valid UTF-8 on its own outside the padding convention
        // 0xFF alone is not valid UTF-8.
        assert_eq!(MacroSlot::decode(&bytes), Err(ProtocolError::InvalidUtf8));
    }

    proptest! {
        #[test]
        fn any_valid_macro_round_trips(
            name in "[a-zA-Z0-9 ]{1,30}",
            events in prop::collection::vec(
                (any::<bool>(), 0u16..=0xffff, 0u16..=0xffff),
                0..=70,
            ),
        ) {
            let events = events
                .into_iter()
                .map(|(press, value, delay_ms)| MacroEvent {
                    press,
                    kind: MacroEventKind::Key,
                    value,
                    delay_ms,
                })
                .collect();
            let slot = MacroSlot { name, events };
            let encoded = slot.encode().unwrap();
            prop_assert_eq!(MacroSlot::decode(&encoded).unwrap(), Some(slot));
        }

        #[test]
        fn decode_never_panics_on_arbitrary_bytes(bytes in prop::collection::vec(any::<u8>(), 0..400)) {
            let _ = MacroSlot::decode(&bytes);
        }
    }
}
