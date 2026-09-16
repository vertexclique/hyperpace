//! Serializable request and response shapes for every Tauri command.
//!
//! This crate holds no protocol knowledge of its own (`docs/architecture/api-contract.md`): every
//! type here is a plain mirror of a `hyperpace_protocol`, `hyperpace_device`, `hyperpace_firmware`
//! or `hyperpace_store` type, with `serde` derives so it can cross the IPC boundary, plus a
//! `From`/`TryFrom` conversion to and from the type it mirrors. No field here is computed from
//! wire bytes anywhere but in those crates.

mod app_settings;
mod device;
mod firmware;
mod macros;
mod settings;

pub use app_settings::{AppSettingsRequest, AppSettingsResponse};
pub use device::{
    AccessDto, BatteryDto, ConnectRequest, DeviceBackendDto, DeviceDescriptor, DeviceEventPayload,
    DeviceIdentityDto, DeviceStateDto, LinkTypeDto, PairPhaseDto, PairStateDto, ReceiverLightDto,
    StatusChangedDto,
};
pub use firmware::{
    FirmwareProgressPayload, FirmwareRecordDto, FirmwareWatchConfigDto, FirmwareWatchDirectoryDto,
    FirmwareWatchFetchErrorDto, FirmwareWatchReportDto,
};
pub use macros::{MacroEventDto, MacroRecordDto, SaveMacroRequest};
pub use settings::{
    ButtonActionDto, DpiActionDto, DpiIndicatorDto, DpiIndicatorModeDto, DpiStageDto, KeystrokeDto,
    LightModeDto, LightingDto, LodDto, LongRangeDto, MacroCyclesDto, ModifierDto, MouseButtonDto,
    PerformanceDto, ProfileDto, ScrollDirectionDto, SetButtonRequest, SettingsDto, SleepTimeDto,
    WriteSettingRequest,
};
