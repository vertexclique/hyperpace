//! Settings snapshot, button action and write-request shapes.

use hyperpace_protocol::{
    ButtonAction, DpiAction, DpiStage, Keystroke, LightMode, Lighting, Lod, MacroCycles, Modifier,
    MouseButton, Performance, ScrollDirection, Settings, SleepTime,
};
use serde::{Deserialize, Serialize};

/// Mirrors [`Lod`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "value", rename_all = "camelCase")]
pub enum LodDto {
    /// See [`Lod::OneMillimeter`].
    OneMillimeter,
    /// See [`Lod::TwoMillimeters`].
    TwoMillimeters,
    /// See [`Lod::PointSevenMillimeters`].
    PointSevenMillimeters,
    /// See [`Lod::Other`].
    Other {
        /// The raw, unrecognized value.
        byte: u8,
    },
}

impl From<Lod> for LodDto {
    fn from(lod: Lod) -> Self {
        match lod {
            Lod::OneMillimeter => Self::OneMillimeter,
            Lod::TwoMillimeters => Self::TwoMillimeters,
            Lod::PointSevenMillimeters => Self::PointSevenMillimeters,
            Lod::Other(byte) => Self::Other { byte },
        }
    }
}

impl From<LodDto> for Lod {
    fn from(lod: LodDto) -> Self {
        match lod {
            LodDto::OneMillimeter => Self::OneMillimeter,
            LodDto::TwoMillimeters => Self::TwoMillimeters,
            LodDto::PointSevenMillimeters => Self::PointSevenMillimeters,
            LodDto::Other { byte } => Self::Other(byte),
        }
    }
}

/// Mirrors [`SleepTime`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "value", rename_all = "camelCase")]
pub enum SleepTimeDto {
    /// See [`SleepTime::TenSeconds`].
    TenSeconds,
    /// See [`SleepTime::ThirtySeconds`].
    ThirtySeconds,
    /// See [`SleepTime::OneMinute`].
    OneMinute,
    /// See [`SleepTime::TwoMinutes`].
    TwoMinutes,
    /// See [`SleepTime::FiveMinutes`].
    FiveMinutes,
    /// See [`SleepTime::TenMinutes`].
    TenMinutes,
    /// See [`SleepTime::FifteenMinutes`].
    FifteenMinutes,
    /// See [`SleepTime::Other`].
    Other {
        /// The raw, unrecognized value.
        byte: u8,
    },
}

impl From<SleepTime> for SleepTimeDto {
    fn from(value: SleepTime) -> Self {
        match value {
            SleepTime::TenSeconds => Self::TenSeconds,
            SleepTime::ThirtySeconds => Self::ThirtySeconds,
            SleepTime::OneMinute => Self::OneMinute,
            SleepTime::TwoMinutes => Self::TwoMinutes,
            SleepTime::FiveMinutes => Self::FiveMinutes,
            SleepTime::TenMinutes => Self::TenMinutes,
            SleepTime::FifteenMinutes => Self::FifteenMinutes,
            SleepTime::Other(byte) => Self::Other { byte },
        }
    }
}

impl From<SleepTimeDto> for SleepTime {
    fn from(value: SleepTimeDto) -> Self {
        match value {
            SleepTimeDto::TenSeconds => Self::TenSeconds,
            SleepTimeDto::ThirtySeconds => Self::ThirtySeconds,
            SleepTimeDto::OneMinute => Self::OneMinute,
            SleepTimeDto::TwoMinutes => Self::TwoMinutes,
            SleepTimeDto::FiveMinutes => Self::FiveMinutes,
            SleepTimeDto::TenMinutes => Self::TenMinutes,
            SleepTimeDto::FifteenMinutes => Self::FifteenMinutes,
            SleepTimeDto::Other { byte } => Self::Other(byte),
        }
    }
}

/// Mirrors [`LightMode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "camelCase")]
pub enum LightModeDto {
    /// See [`LightMode::Off`].
    Off,
    /// See [`LightMode::Rainbow`].
    Rainbow,
    /// See [`LightMode::SingleColorBreath`].
    SingleColorBreath,
    /// See [`LightMode::Fixed`].
    Fixed,
    /// See [`LightMode::Neon`].
    Neon,
    /// See [`LightMode::RainbowBreath`].
    RainbowBreath,
    /// See [`LightMode::Other`].
    Other {
        /// The raw, unrecognized value.
        byte: u8,
    },
}

impl From<LightMode> for LightModeDto {
    fn from(mode: LightMode) -> Self {
        match mode {
            LightMode::Off => Self::Off,
            LightMode::Rainbow => Self::Rainbow,
            LightMode::SingleColorBreath => Self::SingleColorBreath,
            LightMode::Fixed => Self::Fixed,
            LightMode::Neon => Self::Neon,
            LightMode::RainbowBreath => Self::RainbowBreath,
            LightMode::Other(byte) => Self::Other { byte },
        }
    }
}

impl From<LightModeDto> for LightMode {
    fn from(mode: LightModeDto) -> Self {
        match mode {
            LightModeDto::Off => Self::Off,
            LightModeDto::Rainbow => Self::Rainbow,
            LightModeDto::SingleColorBreath => Self::SingleColorBreath,
            LightModeDto::Fixed => Self::Fixed,
            LightModeDto::Neon => Self::Neon,
            LightModeDto::RainbowBreath => Self::RainbowBreath,
            LightModeDto::Other { byte } => Self::Other(byte),
        }
    }
}

/// Mirrors [`Performance`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceDto {
    /// Whether highest performance mode is enabled.
    pub on: bool,
    /// Idle timeout before it disengages.
    pub timeout: SleepTimeDto,
}

impl From<Performance> for PerformanceDto {
    fn from(performance: Performance) -> Self {
        Self {
            on: performance.on,
            timeout: performance.timeout.into(),
        }
    }
}

/// Mirrors [`DpiStage`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DpiStageDto {
    /// DPI value.
    pub dpi: u32,
    /// RGB indicator color for this stage.
    pub color: (u8, u8, u8),
}

impl From<DpiStage> for DpiStageDto {
    fn from(stage: DpiStage) -> Self {
        Self {
            dpi: stage.dpi,
            color: stage.color,
        }
    }
}

/// Mirrors [`Lighting`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LightingDto {
    /// Effect mode.
    pub mode: LightModeDto,
    /// RGB color.
    pub color: (u8, u8, u8),
    /// Effect speed, 0..=9.
    pub speed: u8,
    /// Effect brightness, 0..=9.
    pub brightness: u8,
    /// Whether lighting is on.
    pub on: bool,
}

impl From<Lighting> for LightingDto {
    fn from(lighting: Lighting) -> Self {
        Self {
            mode: lighting.mode.into(),
            color: lighting.color,
            speed: lighting.speed,
            brightness: lighting.brightness,
            on: lighting.on,
        }
    }
}

/// Mirrors [`MouseButton`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MouseButtonDto {
    /// See [`MouseButton::Left`].
    Left,
    /// See [`MouseButton::Right`].
    Right,
    /// See [`MouseButton::Middle`].
    Middle,
    /// See [`MouseButton::Backward`].
    Backward,
    /// See [`MouseButton::Forward`].
    Forward,
}

impl From<MouseButton> for MouseButtonDto {
    fn from(button: MouseButton) -> Self {
        match button {
            MouseButton::Left => Self::Left,
            MouseButton::Right => Self::Right,
            MouseButton::Middle => Self::Middle,
            MouseButton::Backward => Self::Backward,
            MouseButton::Forward => Self::Forward,
        }
    }
}

impl From<MouseButtonDto> for MouseButton {
    fn from(button: MouseButtonDto) -> Self {
        match button {
            MouseButtonDto::Left => Self::Left,
            MouseButtonDto::Right => Self::Right,
            MouseButtonDto::Middle => Self::Middle,
            MouseButtonDto::Backward => Self::Backward,
            MouseButtonDto::Forward => Self::Forward,
        }
    }
}

/// Mirrors [`DpiAction`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DpiActionDto {
    /// See [`DpiAction::Loop`].
    Loop,
    /// See [`DpiAction::Increase`].
    Increase,
    /// See [`DpiAction::Decrease`].
    Decrease,
}

impl From<DpiAction> for DpiActionDto {
    fn from(action: DpiAction) -> Self {
        match action {
            DpiAction::Loop => Self::Loop,
            DpiAction::Increase => Self::Increase,
            DpiAction::Decrease => Self::Decrease,
        }
    }
}

impl From<DpiActionDto> for DpiAction {
    fn from(action: DpiActionDto) -> Self {
        match action {
            DpiActionDto::Loop => Self::Loop,
            DpiActionDto::Increase => Self::Increase,
            DpiActionDto::Decrease => Self::Decrease,
        }
    }
}

/// Mirrors [`ScrollDirection`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScrollDirectionDto {
    /// See [`ScrollDirection::Left`].
    Left,
    /// See [`ScrollDirection::Right`].
    Right,
}

impl From<ScrollDirection> for ScrollDirectionDto {
    fn from(direction: ScrollDirection) -> Self {
        match direction {
            ScrollDirection::Left => Self::Left,
            ScrollDirection::Right => Self::Right,
        }
    }
}

impl From<ScrollDirectionDto> for ScrollDirection {
    fn from(direction: ScrollDirectionDto) -> Self {
        match direction {
            ScrollDirectionDto::Left => Self::Left,
            ScrollDirectionDto::Right => Self::Right,
        }
    }
}

/// Mirrors [`MacroCycles`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "cycles", rename_all = "camelCase")]
pub enum MacroCyclesDto {
    /// See [`MacroCycles::Times`].
    Times {
        /// Repeat count, 1..=250.
        n: u8,
    },
    /// See [`MacroCycles::UntilReleased`].
    UntilReleased,
    /// See [`MacroCycles::UntilAnyPress`].
    UntilAnyPress,
}

impl From<MacroCycles> for MacroCyclesDto {
    fn from(cycles: MacroCycles) -> Self {
        match cycles {
            MacroCycles::Times(n) => Self::Times { n },
            MacroCycles::UntilReleased => Self::UntilReleased,
            MacroCycles::UntilAnyPress => Self::UntilAnyPress,
        }
    }
}

impl From<MacroCyclesDto> for MacroCycles {
    fn from(cycles: MacroCyclesDto) -> Self {
        match cycles {
            MacroCyclesDto::Times { n } => Self::Times(n),
            MacroCyclesDto::UntilReleased => Self::UntilReleased,
            MacroCyclesDto::UntilAnyPress => Self::UntilAnyPress,
        }
    }
}

/// Mirrors [`ButtonAction`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ButtonActionDto {
    /// See [`ButtonAction::Disabled`].
    Disabled,
    /// See [`ButtonAction::Mouse`].
    Mouse {
        /// The bound mouse button.
        button: MouseButtonDto,
    },
    /// See [`ButtonAction::Dpi`].
    Dpi {
        /// The bound DPI action.
        action: DpiActionDto,
    },
    /// See [`ButtonAction::Scroll`].
    Scroll {
        /// The bound scroll direction.
        direction: ScrollDirectionDto,
    },
    /// See [`ButtonAction::Fire`].
    Fire {
        /// Repeat count, 0..=3.
        times: u8,
        /// Interval between repeats, 10..=255.
        interval_ms: u8,
    },
    /// See [`ButtonAction::Keystroke`]. Pair with [`crate::dto::SetButtonRequest::keystroke`] to
    /// set what it plays.
    Keystroke,
    /// See [`ButtonAction::Macro`].
    Macro {
        /// Index of the macro slot to play.
        slot: u8,
        /// How many times it repeats.
        cycles: MacroCyclesDto,
    },
    /// See [`ButtonAction::PollingCycle`].
    PollingCycle,
    /// See [`ButtonAction::Media`].
    Media {
        /// Consumer Control usage id.
        usage: u16,
    },
    /// See [`ButtonAction::Unknown`].
    Unknown {
        /// Raw type byte.
        kind: u8,
        /// Raw param, big-endian on the wire.
        param: u16,
    },
}

impl From<ButtonAction> for ButtonActionDto {
    fn from(action: ButtonAction) -> Self {
        match action {
            ButtonAction::Disabled => Self::Disabled,
            ButtonAction::Mouse(button) => Self::Mouse {
                button: button.into(),
            },
            ButtonAction::Dpi(action) => Self::Dpi {
                action: action.into(),
            },
            ButtonAction::Scroll(direction) => Self::Scroll {
                direction: direction.into(),
            },
            ButtonAction::Fire { times, interval_ms } => Self::Fire { times, interval_ms },
            ButtonAction::Keystroke => Self::Keystroke,
            ButtonAction::Macro { slot, cycles } => Self::Macro {
                slot,
                cycles: cycles.into(),
            },
            ButtonAction::PollingCycle => Self::PollingCycle,
            ButtonAction::Media(usage) => Self::Media { usage },
            ButtonAction::Unknown { kind, param } => Self::Unknown { kind, param },
        }
    }
}

impl From<ButtonActionDto> for ButtonAction {
    fn from(action: ButtonActionDto) -> Self {
        match action {
            ButtonActionDto::Disabled => Self::Disabled,
            ButtonActionDto::Mouse { button } => Self::Mouse(button.into()),
            ButtonActionDto::Dpi { action } => Self::Dpi(action.into()),
            ButtonActionDto::Scroll { direction } => Self::Scroll(direction.into()),
            ButtonActionDto::Fire { times, interval_ms } => Self::Fire { times, interval_ms },
            ButtonActionDto::Keystroke => Self::Keystroke,
            ButtonActionDto::Macro { slot, cycles } => Self::Macro {
                slot,
                cycles: cycles.into(),
            },
            ButtonActionDto::PollingCycle => Self::PollingCycle,
            ButtonActionDto::Media { usage } => Self::Media(usage),
            ButtonActionDto::Unknown { kind, param } => Self::Unknown { kind, param },
        }
    }
}

/// A full settings snapshot, mirroring [`Settings`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDto {
    /// Polling rate in Hz.
    pub polling_hz: u16,
    /// Configured DPI stages, in device order.
    pub dpi_stages: Vec<DpiStageDto>,
    /// Index of the active DPI stage.
    pub current_stage: u8,
    /// Lift-off distance.
    pub lod: LodDto,
    /// Debounce time in milliseconds.
    pub debounce_ms: u8,
    /// Whether motion sync is enabled.
    pub motion_sync: bool,
    /// Whether angle snap is enabled.
    pub angle_snap: bool,
    /// Whether ripple control is enabled.
    pub ripple: bool,
    /// Highest performance mode.
    pub performance: PerformanceDto,
    /// Idle sleep timeout.
    pub sleep: SleepTimeDto,
    /// RGB lighting.
    pub lighting: LightingDto,
    /// Button bindings, in device order.
    pub buttons: Vec<ButtonActionDto>,
    /// Raw sensor mode: 0 low power, 1 high performance.
    pub sensor_mode: u8,
}

impl From<Settings> for SettingsDto {
    fn from(settings: Settings) -> Self {
        Self {
            polling_hz: settings.polling_hz,
            dpi_stages: settings.dpi_stages.into_iter().map(Into::into).collect(),
            current_stage: settings.current_stage,
            lod: settings.lod.into(),
            debounce_ms: settings.debounce_ms,
            motion_sync: settings.motion_sync,
            angle_snap: settings.angle_snap,
            ripple: settings.ripple,
            performance: settings.performance.into(),
            sleep: settings.sleep.into(),
            lighting: settings.lighting.into(),
            buttons: settings.buttons.into_iter().map(Into::into).collect(),
            sensor_mode: settings.sensor_mode,
        }
    }
}

/// Mirrors [`Modifier`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModifierDto {
    /// See [`Modifier::LeftCtrl`].
    LeftCtrl,
    /// See [`Modifier::LeftShift`].
    LeftShift,
    /// See [`Modifier::LeftAlt`].
    LeftAlt,
    /// See [`Modifier::LeftWin`].
    LeftWin,
    /// See [`Modifier::RightCtrl`].
    RightCtrl,
    /// See [`Modifier::RightShift`].
    RightShift,
    /// See [`Modifier::RightAlt`].
    RightAlt,
    /// See [`Modifier::RightWin`].
    RightWin,
}

impl From<ModifierDto> for Modifier {
    fn from(modifier: ModifierDto) -> Self {
        match modifier {
            ModifierDto::LeftCtrl => Self::LeftCtrl,
            ModifierDto::LeftShift => Self::LeftShift,
            ModifierDto::LeftAlt => Self::LeftAlt,
            ModifierDto::LeftWin => Self::LeftWin,
            ModifierDto::RightCtrl => Self::RightCtrl,
            ModifierDto::RightShift => Self::RightShift,
            ModifierDto::RightAlt => Self::RightAlt,
            ModifierDto::RightWin => Self::RightWin,
        }
    }
}

/// A keystroke chord to bind alongside [`ButtonActionDto::Keystroke`] or
/// [`ButtonActionDto::Media`], mirroring [`Keystroke`].
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeystrokeDto {
    /// Modifier keys held as part of the chord.
    pub modifiers: Vec<ModifierDto>,
    /// HID keyboard page usage id.
    pub key: Option<u8>,
    /// Consumer Control usage id for a media key binding.
    pub media: Option<u16>,
}

impl From<KeystrokeDto> for Keystroke {
    fn from(keystroke: KeystrokeDto) -> Self {
        Self {
            modifiers: keystroke.modifiers.into_iter().map(Into::into).collect(),
            key: keystroke.key,
            media: keystroke.media,
        }
    }
}

/// A `set_button` request: the action bound to `index`, and the keystroke chord to bind
/// alongside it when `action` is [`ButtonActionDto::Keystroke`] or [`ButtonActionDto::Media`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetButtonRequest {
    /// Index of the button to bind, in device order.
    pub index: u8,
    /// The action to bind.
    pub action: ButtonActionDto,
    /// The keystroke or media chord this button plays, when `action` needs one.
    #[serde(default)]
    pub keystroke: Option<KeystrokeDto>,
}

/// A `write_setting` request: one named field of [`SettingsDto`] and its new value.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "key", rename_all = "camelCase")]
pub enum WriteSettingRequest {
    /// [`SettingsDto::polling_hz`].
    Polling {
        /// The new rate in Hz; must be one of the seven rates the device recognizes.
        hz: u16,
    },
    /// [`SettingsDto::debounce_ms`].
    Debounce {
        /// The new debounce time in milliseconds.
        ms: u8,
    },
    /// [`SettingsDto::motion_sync`].
    MotionSync {
        /// The new state.
        on: bool,
    },
    /// [`SettingsDto::angle_snap`].
    AngleSnap {
        /// The new state.
        on: bool,
    },
    /// [`SettingsDto::ripple`].
    Ripple {
        /// The new state.
        on: bool,
    },
    /// [`SettingsDto::lod`].
    Lod {
        /// The new lift-off distance.
        value: LodDto,
    },
    /// [`SettingsDto::performance`]'s `on` field.
    PerformanceOn {
        /// The new state.
        on: bool,
    },
    /// [`SettingsDto::performance`]'s `timeout` field.
    PerformanceTimeout {
        /// The new idle timeout.
        value: SleepTimeDto,
    },
    /// [`SettingsDto::sleep`].
    Sleep {
        /// The new idle sleep timeout.
        value: SleepTimeDto,
    },
    /// [`SettingsDto::sensor_mode`].
    SensorMode {
        /// The new raw sensor mode.
        mode: u8,
    },
    /// [`SettingsDto::current_stage`].
    CurrentDpiStage {
        /// The new active DPI stage index.
        index: u8,
    },
    /// The number of enabled entries of [`SettingsDto::dpi_stages`], clamped to 1..=8 to match
    /// the device's own array capacity.
    DpiStageCount {
        /// The new stage count.
        count: u8,
    },
    /// One entry of [`SettingsDto::dpi_stages`].
    DpiStage {
        /// Index of the stage to set, 0-based.
        index: u8,
        /// The new DPI value.
        dpi: u32,
        /// The new indicator color for this stage.
        color: (u8, u8, u8),
    },
    /// [`SettingsDto::lighting`].
    Lighting {
        /// The new lighting configuration.
        value: LightingDto,
    },
    /// One entry of [`SettingsDto::buttons`]. Prefer `set_button` for a button binding: this
    /// variant exists so every [`SettingsDto`] field is reachable through one request shape.
    Button {
        /// Index of the button to bind, in device order.
        index: u8,
        /// The new action.
        action: ButtonActionDto,
    },
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn button_action_dto_round_trips_every_named_variant() {
        let actions = [
            ButtonAction::Disabled,
            ButtonAction::Mouse(MouseButton::Left),
            ButtonAction::Dpi(DpiAction::Loop),
            ButtonAction::Scroll(ScrollDirection::Right),
            ButtonAction::Fire {
                times: 2,
                interval_ms: 40,
            },
            ButtonAction::Keystroke,
            ButtonAction::Macro {
                slot: 3,
                cycles: MacroCycles::UntilReleased,
            },
            ButtonAction::PollingCycle,
            ButtonAction::Media(0x00e9),
            ButtonAction::Unknown {
                kind: 9,
                param: 0x1234,
            },
        ];
        for action in actions {
            let dto = ButtonActionDto::from(action);
            let json = serde_json::to_string(&dto).unwrap();
            let back: ButtonActionDto = serde_json::from_str(&json).unwrap();
            assert_eq!(ButtonAction::from(back), action);
        }
    }

    #[test]
    fn write_setting_request_tags_by_key() {
        let request = WriteSettingRequest::Polling { hz: 1000 };
        let json = serde_json::to_value(request).unwrap();
        assert_eq!(json["key"], "polling");
        assert_eq!(json["hz"], 1000);
    }

    #[test]
    fn settings_dto_round_trips_through_json() {
        let settings = Settings {
            polling_hz: 1000,
            dpi_stages: vec![DpiStage {
                dpi: 800,
                color: (255, 0, 0),
            }],
            current_stage: 0,
            lod: Lod::TwoMillimeters,
            debounce_ms: 8,
            motion_sync: true,
            angle_snap: false,
            ripple: false,
            performance: Performance {
                on: true,
                timeout: SleepTime::OneMinute,
            },
            sleep: SleepTime::FiveMinutes,
            lighting: Lighting {
                mode: LightMode::Fixed,
                color: (0, 255, 0),
                speed: 5,
                brightness: 5,
                on: true,
            },
            buttons: vec![ButtonAction::Mouse(MouseButton::Left)],
            sensor_mode: 1,
        };
        let dto = SettingsDto::from(settings);
        let json = serde_json::to_string(&dto).unwrap();
        let back: SettingsDto = serde_json::from_str(&json).unwrap();
        assert_eq!(back, dto);
    }
}
