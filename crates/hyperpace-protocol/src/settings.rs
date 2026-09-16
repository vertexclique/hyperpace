//! The device memory map and a shadow copy of it.
//!
//! `docs/research/mouse-protocol-v2.md` section 6.1. Hyperpace standardizes on the NEW driver's
//! 16 KiB shadow for every model, since both connect walks it supersets read into the same
//! address space and nothing in this crate is model-specific about the shadow's size, only about
//! how particular offsets decode (that lives in [`crate::model::ModelTable`]).

use crate::buttons::ButtonAction;
use crate::encoding::dpi_from_bytes;
use crate::model::ModelTable;
use crate::response::ProtocolError;

/// Byte offsets into the shadow, section 6.1.
pub mod offset {
    /// Polling rate, 2-byte scalar.
    pub const POLLING: u16 = 0;
    /// Number of enabled DPI stages, 2-byte scalar.
    pub const DPI_STAGE_COUNT: u16 = 2;
    /// Active DPI stage, 0-based, 2-byte scalar.
    pub const CURRENT_DPI: u16 = 4;
    /// Lift-off distance, 2-byte scalar.
    pub const LOD: u16 = 10;
    /// Base of the 8-entry DPI value array, 4 bytes each.
    pub const DPI_VALUE: u16 = 12;
    /// Base of the 8-entry DPI color array, 4 bytes each.
    pub const DPI_COLOR: u16 = 44;
    /// DPI indicator mode, 2-byte scalar. [`DPI_INDICATOR_BRIGHTNESS`], [`DPI_INDICATOR_SPEED`]
    /// and [`DPI_INDICATOR_ON`] sit at fixed offsets alongside it, section 7.6.
    pub const DPI_INDICATOR: u16 = 76;
    /// DPI indicator brightness, raw byte, 2-byte scalar.
    pub const DPI_INDICATOR_BRIGHTNESS: u16 = 78;
    /// DPI indicator speed, raw, 2-byte scalar.
    pub const DPI_INDICATOR_SPEED: u16 = 80;
    /// DPI indicator on/off, 2-byte scalar.
    pub const DPI_INDICATOR_ON: u16 = 82;
    /// Light power-save flag, 2-byte scalar.
    pub const LIGHT_POWER_SAVE: u16 = 94;
    /// Base of the per-button key function array, 4 bytes each.
    pub const BUTTONS: u16 = 96;
    /// The 7-byte lighting struct: mode, r, g, b, speed, brightness, check.
    pub const LIGHTING: u16 = 160;
    /// Lighting on/off, 2-byte scalar.
    pub const LIGHT_ON: u16 = 167;
    /// Debounce time in milliseconds, 2-byte scalar.
    pub const DEBOUNCE: u16 = 169;
    /// Motion sync on/off, 2-byte scalar.
    pub const MOTION_SYNC: u16 = 171;
    /// Sleep time, tens of seconds, 2-byte scalar.
    pub const SLEEP: u16 = 173;
    /// Angle snap on/off, 2-byte scalar.
    pub const ANGLE: u16 = 175;
    /// Ripple control on/off, 2-byte scalar.
    pub const RIPPLE: u16 = 177;
    /// Lights-off-while-moving on/off, 2-byte scalar.
    pub const LIGHT_OFF_MOVING: u16 = 179;
    /// Highest performance mode on/off, 2-byte scalar.
    pub const PERF_ON: u16 = 181;
    /// Highest performance timeout, tens of seconds, 2-byte scalar.
    pub const PERF_TIMEOUT: u16 = 183;
    /// Sensor mode, 2-byte scalar.
    pub const SENSOR_MODE: u16 = 185;
    /// Base of the per-button keystroke slot array, 32 bytes each.
    pub const KEYSTROKE: u16 = 256;
    /// Base of the per-button macro slot array, 384 bytes each.
    pub const MACRO: u16 = 768;
}

/// Size of the shadow, matching the NEW driver's flash layout (section 2).
pub const SHADOW_LEN: usize = 16384;

/// A full copy of the device's settings flash, filled with `0xFF` until a read fills it in.
///
/// Matches the connect sequence's own pre-fill (section 9.1 step 4), so an offset a connect never
/// reached decodes exactly as it would on a real host: as erased flash, not as zero.
#[derive(Clone, PartialEq, Eq)]
pub struct Shadow {
    data: Box<[u8; SHADOW_LEN]>,
}

impl core::fmt::Debug for Shadow {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Shadow")
            .field("len", &self.data.len())
            .finish_non_exhaustive()
    }
}

impl Default for Shadow {
    fn default() -> Self {
        Self::new()
    }
}

impl Shadow {
    /// A shadow filled entirely with `0xFF`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: Box::new([0xff; SHADOW_LEN]),
        }
    }

    /// The whole shadow as raw bytes, for saving a snapshot of a real device to test the decoder
    /// against offline.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.data.as_slice()
    }

    /// Apply a `ReadFlashData` reply's bytes at `address`.
    ///
    /// Bytes that would land at or past [`SHADOW_LEN`] are dropped rather than aliased, unlike
    /// both vendor hosts, which store an address at or above 65536 modulo 65536 (section 4.9);
    /// no address a real read reaches comes close, so this only ever protects against corrupt
    /// input.
    pub fn apply_read(&mut self, address: u16, data: &[u8]) {
        let start = usize::from(address);
        if start >= SHADOW_LEN {
            return;
        }
        let end = start.saturating_add(data.len()).min(SHADOW_LEN);
        let n = end - start;
        self.data[start..end].copy_from_slice(&data[..n]);
    }

    /// The first byte of a scalar pair at `address`. Neither vendor host ever checks the
    /// complement byte on read (section 7.1), and neither does this. An out-of-range address
    /// reads back `0xFF`, the shadow's own fill value.
    #[must_use]
    pub fn scalar(&self, address: u16) -> u8 {
        self.data.get(usize::from(address)).copied().unwrap_or(0xff)
    }

    /// The raw bytes at `address`, clipped to what the shadow actually holds.
    fn region(&self, address: u16, len: usize) -> &[u8] {
        let start = usize::from(address).min(SHADOW_LEN);
        let end = start.saturating_add(len).min(SHADOW_LEN);
        &self.data[start..end]
    }

    fn record4(&self, address: u16) -> [u8; 4] {
        let region = self.region(address, 4);
        let mut record = [0xffu8; 4];
        record[..region.len()].copy_from_slice(region);
        record
    }

    /// Decode every field [`ModelTable`] describes into a [`Settings`] snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::InvalidValue`] when the polling rate scalar is not one of the
    /// seven codes [`crate::encoding::polling_from_byte`] recognizes, and propagates
    /// [`ProtocolError::DpiOutOfRange`] from a DPI stage record whose table has no ranges (never
    /// true for a real [`ModelTable`]).
    pub fn settings(&self, table: &ModelTable) -> Result<Settings, ProtocolError> {
        let polling_hz = crate::encoding::polling_from_byte(self.scalar(offset::POLLING)).ok_or(
            ProtocolError::InvalidValue {
                field: "polling rate",
            },
        )?;

        let stage_count = self.scalar(offset::DPI_STAGE_COUNT).clamp(1, 8);
        let mut dpi_stages = Vec::with_capacity(usize::from(stage_count));
        for i in 0..u16::from(stage_count) {
            let value = self.record4(offset::DPI_VALUE + 4 * i);
            let color = self.record4(offset::DPI_COLOR + 4 * i);
            dpi_stages.push(DpiStage {
                dpi: dpi_from_bytes(&value, table)?,
                color: (color[0], color[1], color[2]),
            });
        }

        let lighting_bytes = self.region(offset::LIGHTING, 7);
        let lighting = Lighting {
            mode: LightMode::from_byte(lighting_bytes[0]),
            color: (lighting_bytes[1], lighting_bytes[2], lighting_bytes[3]),
            speed: lighting_bytes[4],
            brightness: lighting_bytes[5],
            on: self.scalar(offset::LIGHT_ON) != 0,
        };

        let mut buttons = Vec::with_capacity(usize::from(table.buttons));
        for i in 0..u16::from(table.buttons) {
            let record = self.record4(offset::BUTTONS + 4 * i);
            buttons.push(ButtonAction::decode(&record));
        }

        Ok(Settings {
            polling_hz,
            dpi_stages,
            current_stage: self.scalar(offset::CURRENT_DPI),
            lod: Lod::from_byte(self.scalar(offset::LOD)),
            debounce_ms: self.scalar(offset::DEBOUNCE),
            motion_sync: self.scalar(offset::MOTION_SYNC) != 0,
            angle_snap: self.scalar(offset::ANGLE) != 0,
            ripple: self.scalar(offset::RIPPLE) != 0,
            performance: Performance {
                on: self.scalar(offset::PERF_ON) != 0,
                timeout: SleepTime::from_byte(self.scalar(offset::PERF_TIMEOUT)),
            },
            sleep: SleepTime::from_byte(self.scalar(offset::SLEEP)),
            lighting,
            buttons,
            sensor_mode: self.scalar(offset::SENSOR_MODE),
            dpi_indicator: DpiIndicator {
                mode: DpiIndicatorMode::from_byte(self.scalar(offset::DPI_INDICATOR)),
                brightness: crate::encoding::indicator_brightness_from_byte(
                    self.scalar(offset::DPI_INDICATOR_BRIGHTNESS),
                ),
                speed: self.scalar(offset::DPI_INDICATOR_SPEED),
                on: self.scalar(offset::DPI_INDICATOR_ON) != 0,
            },
        })
    }
}

/// Lift-off distance, offset [`offset::LOD`] (section 7.8). Option 3 exists only for sensor 3950.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lod {
    /// Raw value 1.
    OneMillimeter,
    /// Raw value 2.
    TwoMillimeters,
    /// Raw value 3, sensor 3950 only.
    PointSevenMillimeters,
    /// Any other raw value.
    Other(u8),
}

impl Lod {
    /// Decode a raw LOD byte.
    #[must_use]
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            1 => Self::OneMillimeter,
            2 => Self::TwoMillimeters,
            3 => Self::PointSevenMillimeters,
            other => Self::Other(other),
        }
    }

    /// Encode this LOD setting to its raw byte.
    #[must_use]
    pub fn to_byte(self) -> u8 {
        match self {
            Self::OneMillimeter => 1,
            Self::TwoMillimeters => 2,
            Self::PointSevenMillimeters => 3,
            Self::Other(byte) => byte,
        }
    }
}

/// A tens-of-seconds timeout code, shared by sleep time ([`offset::SLEEP`]) and highest
/// performance timeout ([`offset::PERF_TIMEOUT`]), which use the same codes (section 7.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SleepTime {
    /// Raw value 1: 10 seconds.
    TenSeconds,
    /// Raw value 3: 30 seconds.
    ThirtySeconds,
    /// Raw value 6: 1 minute.
    OneMinute,
    /// Raw value 12: 2 minutes.
    TwoMinutes,
    /// Raw value 30: 5 minutes.
    FiveMinutes,
    /// Raw value 60: 10 minutes.
    TenMinutes,
    /// Raw value 90: 15 minutes.
    FifteenMinutes,
    /// Any other raw value.
    Other(u8),
}

impl SleepTime {
    /// Decode a raw tens-of-seconds byte.
    #[must_use]
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            1 => Self::TenSeconds,
            3 => Self::ThirtySeconds,
            6 => Self::OneMinute,
            12 => Self::TwoMinutes,
            30 => Self::FiveMinutes,
            60 => Self::TenMinutes,
            90 => Self::FifteenMinutes,
            other => Self::Other(other),
        }
    }

    /// Encode this timeout to its raw tens-of-seconds byte.
    #[must_use]
    pub fn to_byte(self) -> u8 {
        match self {
            Self::TenSeconds => 1,
            Self::ThirtySeconds => 3,
            Self::OneMinute => 6,
            Self::TwoMinutes => 12,
            Self::FiveMinutes => 30,
            Self::TenMinutes => 60,
            Self::FifteenMinutes => 90,
            Self::Other(byte) => byte,
        }
    }
}

/// Highest performance mode: on/off plus its idle timeout, offsets [`offset::PERF_ON`] and
/// [`offset::PERF_TIMEOUT`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Performance {
    /// Whether highest performance mode is enabled.
    pub on: bool,
    /// Idle timeout before it disengages.
    pub timeout: SleepTime,
}

/// One DPI stage: its value and its indicator color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DpiStage {
    /// DPI value, decoded per the model's sensor table.
    pub dpi: u32,
    /// RGB indicator color for this stage.
    pub color: (u8, u8, u8),
}

/// RGB lighting mode, byte 0 of the lighting struct at [`offset::LIGHTING`] (section 7.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightMode {
    /// 0: lighting off.
    Off,
    /// 1: cycling rainbow.
    Rainbow,
    /// 2: single color, breathing.
    SingleColorBreath,
    /// 3: single fixed color.
    Fixed,
    /// 4: neon.
    Neon,
    /// 5: rainbow, breathing.
    RainbowBreath,
    /// Any other raw value, including 6, which `LightMode_To_Disable` handles but no language
    /// entry names.
    Other(u8),
}

impl LightMode {
    /// Decode a raw lighting mode byte.
    #[must_use]
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0 => Self::Off,
            1 => Self::Rainbow,
            2 => Self::SingleColorBreath,
            3 => Self::Fixed,
            4 => Self::Neon,
            5 => Self::RainbowBreath,
            other => Self::Other(other),
        }
    }

    /// Encode this mode to its raw byte.
    #[must_use]
    pub fn to_byte(self) -> u8 {
        match self {
            Self::Off => 0,
            Self::Rainbow => 1,
            Self::SingleColorBreath => 2,
            Self::Fixed => 3,
            Self::Neon => 4,
            Self::RainbowBreath => 5,
            Self::Other(byte) => byte,
        }
    }
}

/// The 7-byte lighting struct at [`offset::LIGHTING`], plus the separate on/off scalar at
/// [`offset::LIGHT_ON`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lighting {
    /// Effect mode.
    pub mode: LightMode,
    /// RGB color.
    pub color: (u8, u8, u8),
    /// Effect speed, 0..=9.
    pub speed: u8,
    /// Effect brightness, 0..=9.
    pub brightness: u8,
    /// Whether lighting is on.
    pub on: bool,
}

/// DPI indicator effect mode, offset [`offset::DPI_INDICATOR`] (section 7.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DpiIndicatorMode {
    /// Raw value 0: the indicator light is off.
    Off,
    /// Raw value 1: steady.
    Steady,
    /// Raw value 2: breathing.
    Breathing,
    /// Any other raw value.
    Other(u8),
}

impl DpiIndicatorMode {
    /// Decode a raw DPI indicator mode byte.
    #[must_use]
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0 => Self::Off,
            1 => Self::Steady,
            2 => Self::Breathing,
            other => Self::Other(other),
        }
    }

    /// Encode this mode to its raw byte.
    #[must_use]
    pub fn to_byte(self) -> u8 {
        match self {
            Self::Off => 0,
            Self::Steady => 1,
            Self::Breathing => 2,
            Self::Other(byte) => byte,
        }
    }
}

/// The DPI indicator light: mode, brightness, speed and on/off, offsets [`offset::DPI_INDICATOR`]
/// onward (section 7.6). Unlike [`ReceiverLight`], this is part of the mouse's own flash shadow,
/// not a separate command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DpiIndicator {
    /// Effect mode.
    pub mode: DpiIndicatorMode,
    /// Brightness level, 1..=10 (see [`crate::encoding::indicator_brightness_from_byte`]).
    pub brightness: u8,
    /// Effect speed, raw; the protocol reference names no mapping for this field.
    pub speed: u8,
    /// Whether the indicator light is on.
    pub on: bool,
}

/// The receiver's own indicator light (command 24/25, section 10.5). Not part of the mouse's
/// flash shadow: it is a separate command, sent directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReceiverLight {
    /// Effect mode; the receiver has no documented mode names.
    pub mode: u8,
    /// RGB color.
    pub color: (u8, u8, u8),
    /// Effect speed.
    pub speed: u8,
    /// Effect brightness.
    pub brightness: u8,
    /// Effect time.
    pub time: u8,
}

/// A fully decoded settings snapshot, built by [`Shadow::settings`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// Polling rate in Hz.
    pub polling_hz: u16,
    /// Configured DPI stages, in device order.
    pub dpi_stages: Vec<DpiStage>,
    /// Index of the active DPI stage.
    pub current_stage: u8,
    /// Lift-off distance.
    pub lod: Lod,
    /// Debounce time in milliseconds.
    pub debounce_ms: u8,
    /// Whether motion sync is enabled.
    pub motion_sync: bool,
    /// Whether angle snap is enabled.
    pub angle_snap: bool,
    /// Whether ripple control is enabled.
    pub ripple: bool,
    /// Highest performance mode.
    pub performance: Performance,
    /// Idle sleep timeout.
    pub sleep: SleepTime,
    /// RGB lighting.
    pub lighting: Lighting,
    /// Button bindings, in device order.
    pub buttons: Vec<ButtonAction>,
    /// Raw sensor mode: 0 low power, 1 high performance.
    pub sensor_mode: u8,
    /// The DPI indicator light.
    pub dpi_indicator: DpiIndicator,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::model::CID_62_MID_1;
    use proptest::prelude::*;

    #[test]
    fn a_fresh_shadow_is_entirely_erased() {
        let shadow = Shadow::new();
        assert_eq!(shadow.scalar(0), 0xff);
        assert_eq!(shadow.scalar(16383), 0xff);
    }

    #[test]
    fn apply_read_writes_the_given_bytes() {
        let mut shadow = Shadow::new();
        shadow.apply_read(offset::POLLING, &[1, 0x54]);
        assert_eq!(shadow.scalar(offset::POLLING), 1);
    }

    #[test]
    fn apply_read_drops_bytes_past_the_shadow_instead_of_aliasing() {
        let mut shadow = Shadow::new();
        shadow.apply_read(65530, &[0xaa; 10]);
        // Must not panic, and must not wrap into low offsets (section 4.9's aliasing hazard).
        assert_eq!(shadow.scalar(0), 0xff);
    }

    #[test]
    fn an_out_of_range_address_reads_the_fill_value() {
        let shadow = Shadow::new();
        assert_eq!(shadow.scalar(u16::MAX), 0xff);
    }

    #[test]
    fn settings_decodes_a_fully_written_shadow() {
        let table = &CID_62_MID_1;
        let mut shadow = Shadow::new();
        shadow.apply_read(offset::POLLING, &[1, 0x54]); // 1000 Hz
        shadow.apply_read(offset::DPI_STAGE_COUNT, &[1, 0x54]);
        shadow.apply_read(offset::CURRENT_DPI, &[0, 0x55]);
        shadow.apply_read(offset::DPI_VALUE, &[0x0f, 0x0f, 0x00, 0x37]); // 800 DPI
        shadow.apply_read(offset::DPI_COLOR, &[0xff, 0x00, 0x00, 0x56]);
        shadow.apply_read(offset::LIGHTING, &[0x03, 0xff, 0, 0, 0x08, 0x03, 0x48]);
        shadow.apply_read(offset::LIGHT_ON, &[1, 0x54]);
        shadow.apply_read(offset::DPI_INDICATOR, &[2, 0x53]); // breathing
        shadow.apply_read(offset::DPI_INDICATOR_BRIGHTNESS, &[128, 0x2d]); // level 5
        shadow.apply_read(offset::DPI_INDICATOR_SPEED, &[7, 0x4e]);
        shadow.apply_read(offset::DPI_INDICATOR_ON, &[1, 0x54]);
        for i in 0..u16::from(table.buttons) {
            let record = ButtonAction::Disabled.encode();
            shadow.apply_read(offset::BUTTONS + 4 * i, &record);
        }

        let settings = shadow.settings(table).unwrap();
        assert_eq!(settings.polling_hz, 1000);
        assert_eq!(settings.dpi_stages.len(), 1);
        assert_eq!(settings.dpi_stages[0].dpi, 800);
        assert_eq!(settings.dpi_stages[0].color, (0xff, 0, 0));
        assert_eq!(settings.lighting.mode, LightMode::Fixed);
        assert!(settings.lighting.on);
        assert_eq!(settings.dpi_indicator.mode, DpiIndicatorMode::Breathing);
        assert_eq!(settings.dpi_indicator.brightness, 5);
        assert_eq!(settings.dpi_indicator.speed, 7);
        assert!(settings.dpi_indicator.on);
        assert_eq!(settings.buttons.len(), usize::from(table.buttons));
        assert!(
            settings
                .buttons
                .iter()
                .all(|b| *b == ButtonAction::Disabled)
        );
    }

    #[test]
    fn settings_reports_an_unrecognized_polling_byte_instead_of_a_wrong_default() {
        let table = &CID_62_MID_1;
        let shadow = Shadow::new(); // erased shadow: polling scalar is 0xFF
        assert_eq!(
            shadow.settings(table),
            Err(ProtocolError::InvalidValue {
                field: "polling rate"
            })
        );
    }

    #[test]
    fn lod_and_sleep_time_round_trip_their_named_codes() {
        for code in [1u8, 2, 3, 4, 7] {
            assert_eq!(Lod::from_byte(code).to_byte(), code);
        }
        for code in [1u8, 3, 6, 12, 30, 60, 90, 5] {
            assert_eq!(SleepTime::from_byte(code).to_byte(), code);
        }
    }

    #[test]
    fn dpi_indicator_mode_round_trips_its_named_codes() {
        for code in [0u8, 1, 2, 9] {
            assert_eq!(DpiIndicatorMode::from_byte(code).to_byte(), code);
        }
        assert_eq!(DpiIndicatorMode::from_byte(0), DpiIndicatorMode::Off);
        assert_eq!(DpiIndicatorMode::from_byte(1), DpiIndicatorMode::Steady);
        assert_eq!(DpiIndicatorMode::from_byte(2), DpiIndicatorMode::Breathing);
        assert_eq!(DpiIndicatorMode::from_byte(9), DpiIndicatorMode::Other(9));
    }

    proptest! {
        #[test]
        fn lod_round_trips_for_every_byte(byte: u8) {
            prop_assert_eq!(Lod::from_byte(byte).to_byte(), byte);
        }

        #[test]
        fn dpi_indicator_mode_round_trips_for_every_byte(byte: u8) {
            prop_assert_eq!(DpiIndicatorMode::from_byte(byte).to_byte(), byte);
        }

        #[test]
        fn sleep_time_round_trips_for_every_byte(byte: u8) {
            prop_assert_eq!(SleepTime::from_byte(byte).to_byte(), byte);
        }

        #[test]
        fn light_mode_round_trips_for_every_byte(byte: u8) {
            prop_assert_eq!(LightMode::from_byte(byte).to_byte(), byte);
        }

        #[test]
        fn apply_read_never_panics(address: u16, data in prop::collection::vec(any::<u8>(), 0..20)) {
            let mut shadow = Shadow::new();
            shadow.apply_read(address, &data);
        }
    }
}
