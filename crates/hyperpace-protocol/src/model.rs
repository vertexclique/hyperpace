//! Model tables: the per-hardware constants a DPI encode, a button count or a debounce ceiling
//! must be read from, never hardcoded.
//!
//! `docs/research/mouse-protocol-v2.md` section 3.3: the two vendor deployments disagree about
//! this mouse. The production page's config (`cid.json#mouse[0]`) reports cid 102 with two mid
//! variants and a two-range, 40000 DPI sensor table; the newer chooser's config reports cid 62
//! with a single-range, 32000 DPI table and a sixth button. Which one the hardware actually
//! answers with is unresolved (section 15.2, item 6), so both tables are built in and selected at
//! runtime from the identity the device reports.

use crate::buttons::{ButtonAction, DpiAction, MouseButton};

/// One DPI range: a contiguous span of representable DPI values, its step and its device-side
/// doubling flag.
///
/// Section 7.4. `flags` is the sensor table's `DPIex` value (0 or 17 for sensor 3950); its low
/// two bits select how many times the index doubles before it is written to the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DpiRange {
    /// Lowest DPI value this range covers, inclusive.
    pub min: u32,
    /// Highest DPI value this range covers, inclusive.
    pub max: u32,
    /// DPI increment between adjacent index values in this range.
    pub step: u32,
    /// The sensor table's `DPIex` doubling flag for this range.
    pub flags: u8,
}

/// The constants a single cid/mid pair reports: button count, DPI ceiling and ranges, sensor
/// name, default bindings and the debounce ceiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelTable {
    /// Component id, byte 9 of the handshake reply.
    pub cid: u8,
    /// Module id, byte 10 of the handshake reply.
    pub mid: u8,
    /// Number of remappable buttons this model exposes.
    pub buttons: u8,
    /// Highest DPI value the vendor config advertises for this model.
    pub max_dpi: u32,
    /// Sensor identifier, matching the vendor's `sensor.json` key.
    pub sensor: &'static str,
    /// DPI ranges, ascending by [`DpiRange::min`], used by [`crate::encoding::dpi_to_bytes`] and
    /// [`crate::encoding::dpi_from_bytes`].
    pub dpi_ranges: &'static [DpiRange],
    /// Factory button bindings, one per button index in device order.
    pub default_buttons: &'static [ButtonAction],
    /// Highest debounce value, in milliseconds, the vendor UI allows for this model.
    pub max_debounce_ms: u8,
}

const OLD_3950_RANGES: &[DpiRange] = &[
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

const NEW_3950_RANGES: &[DpiRange] = &[DpiRange {
    min: 50,
    max: 30_000,
    step: 50,
    flags: 0,
}];

const CID_102_DEFAULT_BUTTONS: &[ButtonAction] = &[
    ButtonAction::Mouse(MouseButton::Left),
    ButtonAction::Mouse(MouseButton::Right),
    ButtonAction::Mouse(MouseButton::Middle),
    ButtonAction::Mouse(MouseButton::Backward),
    ButtonAction::Mouse(MouseButton::Forward),
];

const CID_62_DEFAULT_BUTTONS: &[ButtonAction] = &[
    ButtonAction::Mouse(MouseButton::Left),
    ButtonAction::Mouse(MouseButton::Right),
    ButtonAction::Mouse(MouseButton::Middle),
    ButtonAction::Mouse(MouseButton::Backward),
    ButtonAction::Mouse(MouseButton::Forward),
    ButtonAction::Dpi(DpiAction::Loop),
];

/// The production mouse page's table: `cfg.json#mouse[0]`, mid 1.
///
/// 5 buttons, 40000 DPI ceiling, the two-range sensor table (`old:cfg.json`, section 3.3).
pub const CID_102_MID_1: ModelTable = ModelTable {
    cid: 102,
    mid: 1,
    buttons: 5,
    max_dpi: 40_000,
    sensor: "3950",
    dpi_ranges: OLD_3950_RANGES,
    default_buttons: CID_102_DEFAULT_BUTTONS,
    max_debounce_ms: 15,
};

/// The production mouse page's table, mid 2: identical to mid 1 except for the mid the device
/// reports and a factory debounce default the vendor cfg sets to 0 instead of 8 (section 3.3);
/// [`ModelTable`] carries no default-debounce field, so the two tables differ only in `mid`.
pub const CID_102_MID_2: ModelTable = ModelTable {
    mid: 2,
    ..CID_102_MID_1
};

/// The newer chooser page's table: `home/cfg.json#mouse[0]`, mid 1.
///
/// 6 buttons (index 5 defaults to a DPI loop), 32000 DPI ceiling, the single-range sensor table
/// (section 3.3).
pub const CID_62_MID_1: ModelTable = ModelTable {
    cid: 62,
    mid: 1,
    buttons: 6,
    max_dpi: 32_000,
    sensor: "3950",
    dpi_ranges: NEW_3950_RANGES,
    default_buttons: CID_62_DEFAULT_BUTTONS,
    max_debounce_ms: 15,
};

/// The model table for `cid` and `mid`, or `None` when the device reports a pair neither vendor
/// config lists.
#[must_use]
pub fn table_for(cid: u8, mid: u8) -> Option<&'static ModelTable> {
    match (cid, mid) {
        (102, 1) => Some(&CID_102_MID_1),
        (102, 2) => Some(&CID_102_MID_2),
        (62, 1) => Some(&CID_62_MID_1),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_cid_mid_pairs_resolve() {
        assert_eq!(table_for(102, 1), Some(&CID_102_MID_1));
        assert_eq!(table_for(102, 2), Some(&CID_102_MID_2));
        assert_eq!(table_for(62, 1), Some(&CID_62_MID_1));
    }

    #[test]
    fn unknown_pairs_resolve_to_none() {
        assert_eq!(table_for(102, 3), None);
        assert_eq!(table_for(62, 2), None);
        assert_eq!(table_for(0, 0), None);
    }

    #[test]
    fn cid_102_variants_share_every_field_but_mid() {
        assert_eq!(CID_102_MID_1.cid, CID_102_MID_2.cid);
        assert_eq!(CID_102_MID_1.buttons, CID_102_MID_2.buttons);
        assert_eq!(CID_102_MID_1.max_dpi, CID_102_MID_2.max_dpi);
        assert_ne!(CID_102_MID_1.mid, CID_102_MID_2.mid);
    }

    #[test]
    fn default_button_lists_match_the_buttons_field() {
        assert_eq!(
            CID_102_MID_1.default_buttons.len(),
            usize::from(CID_102_MID_1.buttons)
        );
        assert_eq!(
            CID_62_MID_1.default_buttons.len(),
            usize::from(CID_62_MID_1.buttons)
        );
    }
}
