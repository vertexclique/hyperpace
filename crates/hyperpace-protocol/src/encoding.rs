//! Value conversions between host units and device bytes.
//!
//! Every conversion here has a documented inverse and a property test round-tripping it.
//! Worked examples are taken verbatim from `docs/research/mouse-protocol-v2.md` section 7 and
//! kept as golden-vector tests.

use crate::model::ModelTable;
use crate::response::ProtocolError;

/// The two-byte encoding of a scalar setting: `[value, 0x55 - value]`.
///
/// Section 7.1. Neither driver ever checks the complement byte on read; it exists only so the
/// device can validate a write. Shared by every scalar setter and by the structured-record check
/// byte's sibling, [`struct_check`].
#[must_use]
pub fn scalar_pair(value: u8) -> [u8; 2] {
    [value, 0x55u8.wrapping_sub(value)]
}

/// The check byte of a structured record: `(0x55 - sum(data)) mod 256`.
///
/// Section 7.2. The same formula as the frame checksum in [`crate::frame::checksum`] but without
/// subtracting the report id. Used by the DPI value and color records, the key function record,
/// the lighting struct, keystroke buffers and macro contexts: one function so every writer agrees
/// on the same byte.
#[must_use]
pub fn struct_check(data: &[u8]) -> u8 {
    let sum = data.iter().fold(0u8, |acc, byte| acc.wrapping_add(*byte));
    0x55u8.wrapping_sub(sum)
}

/// Polling rate, in Hz, to its one-byte flash code.
///
/// Section 7.3. Only the seven rates the setter and decoder agree on are accepted; the shared
/// `ReportRate_To_FlashData` helper used by profile imports produces codes (32, 64, 128) that the
/// decoder reads back doubled or that no decoder recognizes at all, so Hyperpace never uses it.
#[must_use]
pub fn polling_to_byte(hz: u16) -> Option<u8> {
    match hz {
        1000 => Some(1),
        500 => Some(2),
        250 => Some(4),
        125 => Some(8),
        2000 => Some(16),
        4000 => Some(32),
        8000 => Some(64),
        _ => None,
    }
}

/// The inverse of [`polling_to_byte`].
#[must_use]
pub fn polling_from_byte(byte: u8) -> Option<u16> {
    match byte {
        1 => Some(1000),
        2 => Some(500),
        4 => Some(250),
        8 => Some(125),
        16 => Some(2000),
        32 => Some(4000),
        64 => Some(8000),
        _ => None,
    }
}

/// Doubling divisor encoded in a DPI record's flag byte (section 7.4).
///
/// Bit 0 doubles the index once, bit 1 doubles it again; only the low nibble of the flag byte is
/// read back, so this is the same computation on encode and on decode.
fn dpi_doubling(flag_bits: u8) -> u32 {
    let mut divisor = 1u32;
    if flag_bits & 0x01 != 0 {
        divisor *= 2;
    }
    if flag_bits & 0x02 != 0 {
        divisor *= 2;
    }
    divisor
}

/// Encode a DPI value for `table`'s sensor, as the 4-byte record at `12 + 4 * stage`.
///
/// Section 7.4. The vendor encoder always selects the highest range whose `min` is at or below
/// `dpi`, divides by that range's doubling factor, then indexes using the *first* range's step,
/// even when a later range was selected. Verified against every worked example in the document
/// for both model tables (100, 800, 30000, 30100, 32000, 40000 DPI).
///
/// # Errors
///
/// Returns [`ProtocolError::DpiOutOfRange`] when `dpi` is below the table's lowest range or above
/// the selected range's maximum, and [`ProtocolError::DpiNotRepresentable`] when `dpi` is not an
/// exact multiple of the selected range's doubling divisor and step.
pub fn dpi_to_bytes(dpi: u32, table: &ModelTable) -> Result<[u8; 4], ProtocolError> {
    let Some(base_step) = table.dpi_ranges.first().map(|range| range.step) else {
        return Err(ProtocolError::DpiOutOfRange { dpi });
    };
    // The vendor encoder selects by `min` alone and never checks `max` (section 7.4): NEW's
    // single range tops out at 30000 yet the document's own worked example encodes 30100 against
    // it without error, and an index past a table's declared maximum is explicitly undefined
    // rather than rejected. Only "below every range" is a real out-of-range condition here.
    let selected = table
        .dpi_ranges
        .iter()
        .filter(|range| dpi >= range.min)
        .max_by_key(|range| range.min)
        .ok_or(ProtocolError::DpiOutOfRange { dpi })?;
    let divisor = dpi_doubling(selected.flags);
    if !dpi.is_multiple_of(divisor) {
        return Err(ProtocolError::DpiNotRepresentable { dpi });
    }
    let scaled = dpi / divisor;
    if !scaled.is_multiple_of(base_step) {
        return Err(ProtocolError::DpiNotRepresentable { dpi });
    }
    let steps = scaled / base_step;
    let Some(idx) = steps.checked_sub(1) else {
        return Err(ProtocolError::DpiNotRepresentable { dpi });
    };
    if idx > 0x3ff {
        return Err(ProtocolError::DpiNotRepresentable { dpi });
    }
    let idx_low = (idx & 0xff) as u8;
    let idx_high = ((idx >> 8) & 0x3) as u8;
    let low_doubling = selected.flags & 0x03;
    let flags = low_doubling | (idx_high << 2) | (low_doubling << 4) | (idx_high << 6);
    let check = struct_check(&[idx_low, idx_low, flags]);
    Ok([idx_low, idx_low, flags, check])
}

/// Decode a DPI record produced by [`dpi_to_bytes`] or read from the device.
///
/// The decoder reads only byte 0 and the low nibble of the flag byte (byte 2), always scaling by
/// the table's first range step, matching section 7.4 exactly, including its edge case: an erased
/// `FF FF FF FF` record decodes as 204800, not an error, because the firmware never rejects it.
///
/// # Errors
///
/// Returns [`ProtocolError::DpiOutOfRange`] only when `table` declares no DPI ranges at all,
/// which never happens for a real model table and exists so this function cannot panic on one
/// that does.
pub fn dpi_from_bytes(record: &[u8; 4], table: &ModelTable) -> Result<u32, ProtocolError> {
    let Some(base_step) = table.dpi_ranges.first().map(|range| range.step) else {
        return Err(ProtocolError::DpiOutOfRange { dpi: 0 });
    };
    let idx_high = u32::from((record[2] >> 2) & 0x3);
    let idx = u32::from(record[0]) | (idx_high << 8);
    let divisor = dpi_doubling(record[2] & 0x03);
    Ok((idx + 1) * divisor * base_step)
}

/// DPI indicator brightness, level 1..=10, to its raw flash byte.
///
/// Section 7.6. `level` is clamped into 1..=10, since the function is infallible and levels
/// outside that range have no vendor meaning.
#[must_use]
pub fn indicator_brightness_to_byte(level: u8) -> u8 {
    const RAW: [u8; 10] = [16, 30, 60, 90, 128, 150, 180, 210, 230, 255];
    RAW[usize::from(level.clamp(1, 10)) - 1]
}

/// The inverse of [`indicator_brightness_to_byte`].
///
/// Decode rule from section 7.6, quoted exactly: multiples of 30 map to `raw / 30 + 1`; the four
/// raw values the vendor calls out specially (16, 128, 230, 255) map to 1, 5, 9 and 10; anything
/// else maps to 5.
#[must_use]
pub fn indicator_brightness_from_byte(raw: u8) -> u8 {
    match raw {
        16 => 1,
        128 => 5,
        230 => 9,
        255 => 10,
        other if other % 30 == 0 => other / 30 + 1,
        _ => 5,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::model::DpiRange;
    use proptest::prelude::*;

    const OLD_3950: &[DpiRange] = &[
        DpiRange {
            min: 100,
            max: 30_000,
            step: 50,
            flags: 0,
        },
        DpiRange {
            min: 30_100,
            max: 60_000,
            step: 100,
            flags: 17,
        },
    ];

    const NEW_3950: &[DpiRange] = &[DpiRange {
        min: 50,
        max: 30_000,
        step: 50,
        flags: 0,
    }];

    fn table_with(ranges: &'static [DpiRange]) -> ModelTable {
        ModelTable {
            cid: 0,
            mid: 0,
            buttons: 0,
            max_dpi: ranges.last().map_or(0, |r| r.max),
            sensor: "3950",
            dpi_ranges: ranges,
            default_buttons: &[],
            max_debounce_ms: 15,
            body_lighting: false,
            dpi_indicator: false,
            long_range: false,
        }
    }

    #[test]
    fn scalar_pair_matches_the_vendor_formula() {
        assert_eq!(scalar_pair(1), [1, 0x54]);
        assert_eq!(scalar_pair(8), [8, 0x4d]);
        assert_eq!(scalar_pair(0), [0, 0x55]);
        assert_eq!(scalar_pair(0x55), [0x55, 0]);
    }

    #[test]
    fn struct_check_matches_the_lighting_worked_example() {
        // Section 7.7: mode 3, rgb(255,0,0), speed 8, brightness 3 -> check 0x48.
        assert_eq!(struct_check(&[0x03, 0xff, 0x00, 0x00, 0x08, 0x03]), 0x48);
    }

    #[test]
    fn struct_check_matches_the_dpi_worked_example() {
        // Section 7.4: 100 DPI, either table -> record 01 01 00 53.
        assert_eq!(struct_check(&[0x01, 0x01, 0x00]), 0x53);
    }

    #[test]
    fn polling_rate_matches_the_documented_table() {
        for (hz, code) in [
            (125u16, 8u8),
            (250, 4),
            (500, 2),
            (1000, 1),
            (2000, 16),
            (4000, 32),
            (8000, 64),
        ] {
            assert_eq!(polling_to_byte(hz), Some(code));
            assert_eq!(polling_from_byte(code), Some(hz));
        }
    }

    #[test]
    fn undocumented_polling_values_are_refused() {
        assert_eq!(polling_to_byte(333), None);
        assert_eq!(polling_from_byte(128), None);
        assert_eq!(polling_from_byte(0), None);
    }

    #[test]
    fn dpi_worked_examples_match_the_document() {
        let old = table_with(OLD_3950);
        let new = table_with(NEW_3950);
        assert_eq!(dpi_to_bytes(100, &old).unwrap(), [0x01, 0x01, 0x00, 0x53]);
        assert_eq!(dpi_to_bytes(100, &new).unwrap(), [0x01, 0x01, 0x00, 0x53]);
        assert_eq!(dpi_to_bytes(800, &old).unwrap(), [0x0f, 0x0f, 0x00, 0x37]);
        assert_eq!(
            dpi_to_bytes(30_000, &old).unwrap(),
            [0x57, 0x57, 0x88, 0x1f]
        );
        assert_eq!(
            dpi_to_bytes(30_000, &new).unwrap(),
            [0x57, 0x57, 0x88, 0x1f]
        );
        assert_eq!(
            dpi_to_bytes(30_100, &new).unwrap(),
            [0x59, 0x59, 0x88, 0x1b]
        );
        assert_eq!(
            dpi_to_bytes(30_100, &old).unwrap(),
            [0x2c, 0x2c, 0x55, 0xa8]
        );
        assert_eq!(
            dpi_to_bytes(32_000, &new).unwrap(),
            [0x7f, 0x7f, 0x88, 0xcf]
        );
        assert_eq!(
            dpi_to_bytes(32_000, &old).unwrap(),
            [0x3f, 0x3f, 0x55, 0x82]
        );
        assert_eq!(
            dpi_to_bytes(40_000, &old).unwrap(),
            [0x8f, 0x8f, 0x55, 0xe2]
        );
        assert_eq!(dpi_to_bytes(50, &new).unwrap(), [0x00, 0x00, 0x00, 0x55]);
    }

    #[test]
    fn both_tables_decode_32000_from_their_own_encoding() {
        // The decoder is the same function in both drivers: both forms read back as 32000.
        let old = table_with(OLD_3950);
        let new = table_with(NEW_3950);
        assert_eq!(
            dpi_from_bytes(&[0x7f, 0x7f, 0x88, 0xcf], &new).unwrap(),
            32_000
        );
        assert_eq!(
            dpi_from_bytes(&[0x3f, 0x3f, 0x55, 0x82], &old).unwrap(),
            32_000
        );
    }

    #[test]
    fn an_erased_dpi_record_decodes_without_error() {
        let table = table_with(NEW_3950);
        assert_eq!(
            dpi_from_bytes(&[0xff, 0xff, 0xff, 0xff], &table).unwrap(),
            204_800
        );
    }

    #[test]
    fn dpi_below_every_range_is_refused() {
        let table = table_with(NEW_3950);
        assert_eq!(
            dpi_to_bytes(10, &table),
            Err(ProtocolError::DpiOutOfRange { dpi: 10 })
        );
    }

    #[test]
    fn a_dpi_past_the_tables_own_maximum_still_encodes() {
        // Section 7.4: the encoder never checks a range's upper bound, only its lower one; the
        // document's own worked example relies on exactly this for 30100 against NEW's single
        // 50..30000 range.
        let table = table_with(NEW_3950);
        assert_eq!(
            dpi_to_bytes(30_100, &table).unwrap(),
            [0x59, 0x59, 0x88, 0x1b]
        );
    }

    #[test]
    fn a_dpi_whose_index_would_exceed_ten_bits_is_refused() {
        let table = table_with(NEW_3950);
        // idx = 100_000/50 - 1 = 1999, past the 10-bit index field's 1023 ceiling.
        assert_eq!(
            dpi_to_bytes(100_000, &table),
            Err(ProtocolError::DpiNotRepresentable { dpi: 100_000 })
        );
    }

    #[test]
    fn dpi_off_the_step_grid_is_refused() {
        let table = table_with(NEW_3950);
        assert_eq!(
            dpi_to_bytes(123, &table),
            Err(ProtocolError::DpiNotRepresentable { dpi: 123 })
        );
    }

    #[test]
    fn indicator_brightness_matches_the_documented_levels() {
        let raw = [16u8, 30, 60, 90, 128, 150, 180, 210, 230, 255];
        for (level, byte) in raw.into_iter().enumerate() {
            let level = u8::try_from(level + 1).unwrap();
            assert_eq!(indicator_brightness_to_byte(level), byte);
            assert_eq!(indicator_brightness_from_byte(byte), level);
        }
    }

    #[test]
    fn indicator_brightness_level_is_clamped_not_panicking() {
        assert_eq!(
            indicator_brightness_to_byte(0),
            indicator_brightness_to_byte(1)
        );
        assert_eq!(
            indicator_brightness_to_byte(255),
            indicator_brightness_to_byte(10)
        );
    }

    proptest! {
        #[test]
        fn scalar_pair_always_sums_to_0x55(value: u8) {
            let [v, complement] = scalar_pair(value);
            prop_assert_eq!(v, value);
            prop_assert_eq!(v.wrapping_add(complement), 0x55);
        }

        #[test]
        fn polling_rate_round_trips_through_its_seven_codes(idx in 0usize..7) {
            let hz = [125u16, 250, 500, 1000, 2000, 4000, 8000][idx];
            let code = polling_to_byte(hz).unwrap();
            prop_assert_eq!(polling_from_byte(code), Some(hz));
        }

        #[test]
        fn indicator_brightness_round_trips_for_every_level(level in 1u8..=10) {
            let raw = indicator_brightness_to_byte(level);
            prop_assert_eq!(indicator_brightness_from_byte(raw), level);
        }

        #[test]
        fn dpi_round_trips_on_the_new_table_grid(steps in 1u32..600) {
            let table = table_with(NEW_3950);
            let dpi = steps * 50;
            let encoded = dpi_to_bytes(dpi, &table).unwrap();
            prop_assert_eq!(dpi_from_bytes(&encoded, &table).unwrap(), dpi);
        }

        #[test]
        fn dpi_round_trips_on_the_old_table_grid_low_range(steps in 2u32..600) {
            let table = table_with(OLD_3950);
            let dpi = steps * 50; // OLD's low range starts at 100 DPI (step 2), not 50.
            let encoded = dpi_to_bytes(dpi, &table).unwrap();
            prop_assert_eq!(dpi_from_bytes(&encoded, &table).unwrap(), dpi);
        }

        #[test]
        fn dpi_round_trips_on_the_old_table_high_range(steps in 1u32..300) {
            let table = table_with(OLD_3950);
            let dpi = 30_100u32 + (steps - 1) * 100;
            let encoded = dpi_to_bytes(dpi, &table).unwrap();
            prop_assert_eq!(dpi_from_bytes(&encoded, &table).unwrap(), dpi);
        }
    }
}
