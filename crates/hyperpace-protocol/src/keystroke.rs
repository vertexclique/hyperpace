//! Keystroke slot codec: the 32-byte buffer at `256 + 32 * index` that a type 5 button (or a
//! media key) binds to.
//!
//! `docs/research/mouse-protocol-v2.md` sections 8.4 (chord layout), 8.5 (media keys, stored as a
//! one-entry slot of kind 2) and 8.6 (the modifier bitmask, type 0 of the shared keymap). Layout:
//!
//! ```text
//! [2k, k press entries, the same k entries in reverse as releases, check]
//! entry = [kind | 0x80 (press) or kind | 0x40 (release), value lo, value hi]
//! length = 6k + 2 data bytes, plus the check byte
//! ```
//!
//! A reimplementation must clamp reads to the slot's geometry rather than trust the device's own
//! count byte, which is uncapped on the wire (section 8.4): [`Keystroke::decode`] only ever reads
//! as many entries as fit in the slice it is given.

use crate::encoding::struct_check;
use crate::response::ProtocolError;

/// A held modifier key, the type 0 entries of the shared keymap (section 8.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    /// Bit 0.
    LeftCtrl,
    /// Bit 1.
    LeftShift,
    /// Bit 2.
    LeftAlt,
    /// Bit 3.
    LeftWin,
    /// Bit 4.
    RightCtrl,
    /// Bit 5.
    RightShift,
    /// Bit 6.
    RightAlt,
    /// Bit 7.
    RightWin,
}

impl Modifier {
    const fn bit(self) -> u16 {
        match self {
            Self::LeftCtrl => 1,
            Self::LeftShift => 2,
            Self::LeftAlt => 4,
            Self::LeftWin => 8,
            Self::RightCtrl => 16,
            Self::RightShift => 32,
            Self::RightAlt => 64,
            Self::RightWin => 128,
        }
    }

    const fn from_bit(bit: u16) -> Option<Self> {
        match bit {
            1 => Some(Self::LeftCtrl),
            2 => Some(Self::LeftShift),
            4 => Some(Self::LeftAlt),
            8 => Some(Self::LeftWin),
            16 => Some(Self::RightCtrl),
            32 => Some(Self::RightShift),
            64 => Some(Self::RightAlt),
            128 => Some(Self::RightWin),
            _ => None,
        }
    }
}

/// Entry kind, the masked low nibble of an entry's first byte.
const KIND_MODIFIER: u8 = 0;
const KIND_KEY: u8 = 1;
const KIND_MEDIA: u8 = 2;

/// The bound keystroke, keyboard chord or media key, of one button's slot.
///
/// A chord holds zero or more [`Modifier`] entries plus an optional HID keyboard usage in `key`;
/// a media key holds only `media`, the Consumer Control usage id (section 8.5). Nothing stops a
/// caller from setting more than one, in which case [`Keystroke::encode`] writes every entry it
/// is given, in the order modifiers, then key, then media.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Keystroke {
    /// Modifier keys held as part of the chord.
    pub modifiers: Vec<Modifier>,
    /// HID keyboard page usage id, section 8.6 (4..=99 for the keys the shared keymap covers).
    pub key: Option<u8>,
    /// Consumer Control usage id for a media key binding (section 8.5).
    pub media: Option<u16>,
}

impl Keystroke {
    /// Encode this keystroke as the slot's data bytes: `[2k, presses.., releases.., check]`.
    ///
    /// Not padded to 32 bytes, matching every vendor writer (section 8.4): the caller writes
    /// exactly this many bytes and leaves the rest of the slot untouched. `k` is clamped to fit a
    /// byte (255 entries is already far beyond anything the device UI can produce) so this never
    /// panics.
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let mut entries: Vec<(u8, u16)> = Vec::with_capacity(self.modifiers.len() + 2);
        entries.extend(self.modifiers.iter().map(|m| (KIND_MODIFIER, m.bit())));
        if let Some(key) = self.key {
            entries.push((KIND_KEY, u16::from(key)));
        }
        if let Some(media) = self.media {
            entries.push((KIND_MEDIA, media));
        }
        let k = entries.len();
        let count = u8::try_from(2 * k).unwrap_or(u8::MAX);

        let mut data = Vec::with_capacity(2 + 6 * k + 1);
        data.push(count);
        for &(kind, value) in &entries {
            data.push(kind | 0x80);
            data.push((value & 0xff) as u8);
            data.push((value >> 8) as u8);
        }
        for &(kind, value) in entries.iter().rev() {
            data.push(kind | 0x40);
            data.push((value & 0xff) as u8);
            data.push((value >> 8) as u8);
        }
        data.push(struct_check(&data));
        data
    }

    /// Decode a keystroke slot from its data bytes.
    ///
    /// Reads only the first `bytes[0] / 2` press entries, the same as every host driver (section
    /// 8.4), and never reads past `bytes`: passing a full 32-byte slot bounds the worst case the
    /// device's uncapped count byte can produce.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::Truncated`] when `bytes` is empty or too short for the entry
    /// count it declares, and [`ProtocolError::InvalidValue`] when an entry's kind is not one
    /// this slot format defines, or a modifier entry's value is not a single documented bit.
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let &count = bytes
            .first()
            .ok_or(ProtocolError::Truncated { needed: 1, got: 0 })?;
        let k = usize::from(count / 2).min(bytes.len().saturating_sub(1) / 3);

        let mut modifiers = Vec::new();
        let mut key = None;
        let mut media = None;
        for i in 0..k {
            let base = 1 + 3 * i;
            let entry = bytes.get(base..base + 3).ok_or(ProtocolError::Truncated {
                needed: base + 3,
                got: bytes.len(),
            })?;
            let kind = entry[0] & 0x0f;
            let value = u16::from(entry[1]) | (u16::from(entry[2]) << 8);
            match kind {
                KIND_MODIFIER => {
                    let modifier =
                        Modifier::from_bit(value).ok_or(ProtocolError::InvalidValue {
                            field: "keystroke modifier",
                        })?;
                    modifiers.push(modifier);
                }
                KIND_KEY => key = Some((value & 0xff) as u8),
                KIND_MEDIA => media = Some(value),
                _ => {
                    return Err(ProtocolError::InvalidValue {
                        field: "keystroke entry kind",
                    });
                }
            }
        }
        Ok(Self {
            modifiers,
            key,
            media,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use proptest::prelude::*;

    #[test]
    fn media_slot_matches_the_volume_up_worked_example() {
        // Section 8.5: volume up (0x00E9) -> 02 82 E9 00 42 E9 00 BD.
        let keystroke = Keystroke {
            modifiers: vec![],
            key: None,
            media: Some(0x00e9),
        };
        assert_eq!(
            keystroke.encode(),
            vec![0x02, 0x82, 0xe9, 0x00, 0x42, 0xe9, 0x00, 0xbd]
        );
    }

    #[test]
    fn media_slot_round_trips() {
        let keystroke = Keystroke {
            modifiers: vec![],
            key: None,
            media: Some(0x00e9),
        };
        let encoded = keystroke.encode();
        assert_eq!(Keystroke::decode(&encoded).unwrap(), keystroke);
    }

    #[test]
    fn a_chord_round_trips() {
        let keystroke = Keystroke {
            modifiers: vec![Modifier::LeftCtrl, Modifier::LeftShift],
            key: Some(4), // 'A'
            media: None,
        };
        let encoded = keystroke.encode();
        // count byte is 2k = 6.
        assert_eq!(encoded[0], 6);
        assert_eq!(Keystroke::decode(&encoded).unwrap(), keystroke);
    }

    #[test]
    fn an_empty_keystroke_round_trips_as_zero_entries() {
        let keystroke = Keystroke::default();
        let encoded = keystroke.encode();
        assert_eq!(encoded, vec![0, struct_check(&[0])]);
        assert_eq!(Keystroke::decode(&encoded).unwrap(), keystroke);
    }

    #[test]
    fn decode_clamps_to_the_given_slice_instead_of_trusting_the_count_byte() {
        // A device-supplied count of 255 (uncapped, section 8.4) must not read past a 32-byte slot.
        let mut slot = vec![0xffu8; 32];
        slot[0] = 255;
        // Must not panic and must not read past the slice.
        let _ = Keystroke::decode(&slot);
    }

    #[test]
    fn an_empty_buffer_is_truncated_not_a_panic() {
        assert_eq!(
            Keystroke::decode(&[]),
            Err(ProtocolError::Truncated { needed: 1, got: 0 })
        );
    }

    proptest! {
        #[test]
        fn a_chord_of_any_size_round_trips(
            mod_bits in prop::collection::hash_set(0u8..8, 0..8),
            has_key in any::<bool>(),
            key in 4u8..=99,
        ) {
            let modifiers = mod_bits
                .into_iter()
                .map(|shift| Modifier::from_bit(1u16 << shift).unwrap())
                .collect();
            let keystroke = Keystroke {
                modifiers,
                key: has_key.then_some(key),
                media: None,
            };
            let encoded = keystroke.encode();
            prop_assert_eq!(Keystroke::decode(&encoded).unwrap(), keystroke);
        }

        #[test]
        fn decode_never_panics_on_arbitrary_bytes(bytes in prop::collection::vec(any::<u8>(), 0..40)) {
            let _ = Keystroke::decode(&bytes);
        }
    }
}
