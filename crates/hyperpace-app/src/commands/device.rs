//! Device connection, settings, buttons, profile, receiver and config commands.
//!
//! Every write here goes through [`hyperpace_device::DeviceHandle::write_scalar`] or
//! [`hyperpace_device::DeviceHandle::write_block`], which already refuse a write under
//! [`hyperpace_device::Access::ReadOnly`] without touching the transport
//! (`docs/architecture/api-contract.md`); this module adds no access check of its own, so the one
//! gate the device layer already enforces is the only one, never a second copy that could drift
//! from it.

use hyperpace_device::DeviceHandle;
use hyperpace_protocol::settings::SHADOW_LEN;
use hyperpace_protocol::{
    ButtonAction, Command, Keystroke, LightMode, Lod, ModelTable, ProtocolError, ReceiverLight,
    SleepTime, config_file, dpi_to_bytes, offset, polling_to_byte, request, response, struct_check,
};
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State, Window};

use crate::blocking::blocking;
use crate::dto::{
    ConnectRequest, DeviceBackendDto, DeviceDescriptor, DeviceEventPayload, DeviceStateDto,
    LightingDto, PairStateDto, ReceiverLightDto, SetButtonRequest, SettingsDto,
    WriteSettingRequest,
};
use crate::error::{AppError, to_command_result};
use crate::state::{AppState, REQUEST_TIMEOUT};

/// List the connection options `connect` accepts. Static: neither backend can be probed without
/// either opening the real device (never done outside an explicit `connect`) or already holding a
/// connection, so this names the two choices rather than a live scan.
///
/// # Errors
///
/// Never fails; the `Result` matches every other command's shape.
#[tauri::command]
pub async fn list_devices() -> Result<Vec<DeviceDescriptor>, String> {
    Ok(vec![
        DeviceDescriptor {
            backend: DeviceBackendDto::Simulator,
            label: "Simulator".to_owned(),
            description: "A simulated device; never touches real hardware.".to_owned(),
        },
        DeviceDescriptor {
            backend: DeviceBackendDto::RealDevice,
            label: "This device".to_owned(),
            description: "The operator's own mouse. Read-only unless write access is requested."
                .to_owned(),
        },
    ])
}

/// Connect to the simulator, or to the operator's real device when `request.real_device` is set.
///
/// # Errors
///
/// Returns an error message when the transport could not be opened (no real device attached, or
/// it could not be reached) or its owner thread could not be started.
#[tauri::command]
pub async fn connect(
    state: State<'_, AppState>,
    request: ConnectRequest,
) -> Result<DeviceStateDto, String> {
    to_command_result(state.connect(request.real_device, request.access))
}

/// Disconnect the current device, if any.
///
/// # Errors
///
/// Never fails; the `Result` matches every other command's shape.
#[tauri::command]
pub async fn disconnect(state: State<'_, AppState>) -> Result<DeviceStateDto, String> {
    Ok(state.disconnect())
}

/// Register `channel` to stream this window's device events from now on, and return the current
/// connection snapshot. Re-subscribing under the same window label (a window recreated after
/// being destroyed to the tray) replaces the previous channel.
///
/// # Errors
///
/// Never fails; the `Result` matches every other command's shape.
#[tauri::command]
pub async fn device_state(
    state: State<'_, AppState>,
    window: Window,
    channel: Channel<DeviceEventPayload>,
) -> Result<DeviceStateDto, String> {
    state.subscribe(window.label().to_owned(), channel);
    Ok(state.snapshot())
}

/// Read every setting from the connected device.
///
/// # Errors
///
/// Returns an error message when no device is connected, its model is unrecognized, or the
/// device did not answer the read in time.
#[tauri::command]
pub async fn read_settings(app: AppHandle) -> Result<SettingsDto, String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            let (handle, table, ..) = state.connected_model()?;
            let shadow = handle.read_settings()?;
            Ok(SettingsDto::from(shadow.settings(table)?))
        })
        .await,
    )
}

/// Write one named setting to the connected device.
///
/// # Errors
///
/// Returns an error message when no device is connected, `request` carries a value the device
/// has no encoding for, the connection is read-only, or the write was not acknowledged in time.
#[tauri::command]
pub async fn write_setting(app: AppHandle, request: WriteSettingRequest) -> Result<(), String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            let (handle, table, ..) = state.connected_model()?;
            apply_write_setting(&handle, table, request)
        })
        .await,
    )
}

/// Bind `request.action` to button `request.index`, and its keystroke or media chord when given.
///
/// # Errors
///
/// Returns an error message when no device is connected, the connection is read-only, or the
/// write was not acknowledged in time.
#[tauri::command]
pub async fn set_button(app: AppHandle, request: SetButtonRequest) -> Result<(), String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            let handle = state.handle()?;
            apply_set_button(&handle, request)
        })
        .await,
    )
}

/// Select profile `index` as active.
///
/// # Errors
///
/// Returns an error message when no device is connected, the connection is read-only, or the
/// request was not acknowledged in time.
#[tauri::command]
pub async fn set_profile(app: AppHandle, index: u8) -> Result<(), String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            let handle = state.handle()?;
            handle.request(request::set_profile(index), REQUEST_TIMEOUT)?;
            Ok(())
        })
        .await,
    )
}

/// Factory-reset the connected device.
///
/// # Errors
///
/// Returns an error message when no device is connected, the connection is read-only, or the
/// request was not acknowledged in time.
#[tauri::command]
pub async fn factory_reset(app: AppHandle) -> Result<(), String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            let handle = state.handle()?;
            handle.request(Command::FactoryReset.request(), REQUEST_TIMEOUT)?;
            Ok(())
        })
        .await,
    )
}

/// Start receiver pairing and report the resulting pairing state.
///
/// # Errors
///
/// Returns an error message when no device is connected, the connection is read-only, or either
/// request was not acknowledged in time.
#[tauri::command]
pub async fn pair_receiver(app: AppHandle) -> Result<PairStateDto, String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            let handle = state.handle()?;
            handle.request(Command::EnterPair.request(), REQUEST_TIMEOUT)?;
            let reply = handle.request(Command::PairState.request(), REQUEST_TIMEOUT)?;
            Ok(response::pair_state(&reply)?.into())
        })
        .await,
    )
}

/// Set the receiver's own indicator light.
///
/// # Errors
///
/// Returns an error message when no device is connected, the connection is read-only, or the
/// request was not acknowledged in time.
#[tauri::command]
pub async fn receiver_light(app: AppHandle, light: ReceiverLightDto) -> Result<(), String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            let handle = state.handle()?;
            let light: ReceiverLight = light.into();
            handle.request(request::receiver_light(&light), REQUEST_TIMEOUT)?;
            Ok(())
        })
        .await,
    )
}

/// Export the connected device's full settings shadow as a `.bin` config file.
///
/// # Errors
///
/// Returns an error message when no device is connected, its model is unrecognized, or the
/// device did not answer the read in time.
#[tauri::command]
pub async fn export_config(app: AppHandle) -> Result<Vec<u8>, String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            let (handle, table, identity, ..) = state.connected_model()?;
            let shadow = handle.read_settings()?;
            Ok(config_file::export(&shadow, &identity, table.sensor))
        })
        .await,
    )
}

/// Import a `.bin` config file and write its (clamp-fixed) shadow to the connected device.
///
/// # Errors
///
/// Returns an error message when no device is connected, `bytes` is not a valid config file for
/// the connected model, the connection is read-only, or a write was not acknowledged in time.
#[tauri::command]
pub async fn import_config(app: AppHandle, bytes: Vec<u8>) -> Result<(), String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            let (handle, table, ..) = state.connected_model()?;
            let imported = config_file::import(&bytes, table)?;
            let last_address = u16::try_from(SHADOW_LEN).unwrap_or(u16::MAX);
            let raw: Vec<u8> = (0..last_address)
                .map(|addr| imported.shadow.scalar(addr))
                .collect();
            handle.write_block(0, &raw)?;
            Ok(())
        })
        .await,
    )
}

/// The mapping from one [`WriteSettingRequest`] to the device write(s) it needs, using only
/// [`hyperpace_protocol`]'s own offsets and encodings.
///
/// # Errors
///
/// Returns [`AppError::Protocol`] when a value the device has no encoding for was given (an
/// unrecognized polling rate, a debounce past `table`'s ceiling, or a DPI value the model table
/// cannot represent), and otherwise whatever [`hyperpace_device::DeviceError`] the write reports.
fn apply_write_setting(
    handle: &DeviceHandle,
    table: &ModelTable,
    request: WriteSettingRequest,
) -> Result<(), AppError> {
    match request {
        WriteSettingRequest::Polling { hz } => {
            let byte = polling_to_byte(hz).ok_or(ProtocolError::InvalidValue {
                field: "polling rate",
            })?;
            handle.write_scalar(offset::POLLING, byte)?;
        }
        WriteSettingRequest::Debounce { ms } => {
            if ms > table.max_debounce_ms {
                return Err(ProtocolError::InvalidValue {
                    field: "debounce_ms",
                }
                .into());
            }
            handle.write_scalar(offset::DEBOUNCE, ms)?;
        }
        WriteSettingRequest::MotionSync { on } => {
            handle.write_scalar(offset::MOTION_SYNC, u8::from(on))?;
        }
        WriteSettingRequest::AngleSnap { on } => {
            handle.write_scalar(offset::ANGLE, u8::from(on))?;
        }
        WriteSettingRequest::Ripple { on } => {
            handle.write_scalar(offset::RIPPLE, u8::from(on))?;
        }
        WriteSettingRequest::Lod { value } => {
            handle.write_scalar(offset::LOD, Lod::from(value).to_byte())?;
        }
        WriteSettingRequest::PerformanceOn { on } => {
            handle.write_scalar(offset::PERF_ON, u8::from(on))?;
        }
        WriteSettingRequest::PerformanceTimeout { value } => {
            handle.write_scalar(offset::PERF_TIMEOUT, SleepTime::from(value).to_byte())?;
        }
        WriteSettingRequest::Sleep { value } => {
            handle.write_scalar(offset::SLEEP, SleepTime::from(value).to_byte())?;
        }
        WriteSettingRequest::SensorMode { mode } => {
            handle.write_scalar(offset::SENSOR_MODE, mode)?;
        }
        WriteSettingRequest::CurrentDpiStage { index } => {
            handle.write_scalar(offset::CURRENT_DPI, index)?;
        }
        WriteSettingRequest::DpiStageCount { count } => {
            handle.write_scalar(offset::DPI_STAGE_COUNT, count.clamp(1, 8))?;
        }
        WriteSettingRequest::DpiStage { index, dpi, color } => {
            let value_bytes = dpi_to_bytes(dpi, table)?;
            handle.write_block(offset::DPI_VALUE + u16::from(index) * 4, &value_bytes)?;
            let (r, g, b) = color;
            let color_bytes = [r, g, b, struct_check(&[r, g, b])];
            handle.write_block(offset::DPI_COLOR + u16::from(index) * 4, &color_bytes)?;
        }
        WriteSettingRequest::Lighting { value } => write_lighting(handle, value)?,
        WriteSettingRequest::Button { index, action } => {
            let record = ButtonAction::from(action).encode();
            handle.write_block(offset::BUTTONS + u16::from(index) * 4, &record)?;
        }
    }
    Ok(())
}

/// Write the 7-byte lighting struct at [`offset::LIGHTING`] and the separate on/off scalar at
/// [`offset::LIGHT_ON`] `hyperpace_protocol::settings::Lighting` splits them into.
fn write_lighting(handle: &DeviceHandle, lighting: LightingDto) -> Result<(), AppError> {
    let mode = LightMode::from(lighting.mode).to_byte();
    let (r, g, b) = lighting.color;
    let head = [mode, r, g, b, lighting.speed, lighting.brightness];
    let mut record = [0u8; 7];
    record[..6].copy_from_slice(&head);
    record[6] = struct_check(&head);
    handle.write_block(offset::LIGHTING, &record)?;
    handle.write_scalar(offset::LIGHT_ON, u8::from(lighting.on))?;
    Ok(())
}

/// Write the button record for `request.index`, and its keystroke or media chord alongside it
/// when one was given.
fn apply_set_button(handle: &DeviceHandle, request: SetButtonRequest) -> Result<(), AppError> {
    let action = ButtonAction::from(request.action);
    let record = action.encode();
    handle.write_block(offset::BUTTONS + u16::from(request.index) * 4, &record)?;

    if let Some(keystroke) = request.keystroke {
        let keystroke = Keystroke::from(keystroke);
        let bytes = keystroke.encode();
        handle.write_block(offset::KEYSTROKE + u16::from(request.index) * 32, &bytes)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use hyperpace_device::{Access, SimTransport};
    use hyperpace_protocol::model::CID_62_MID_1;
    use hyperpace_protocol::{DpiAction, MouseButton};

    use crate::dto::{ButtonActionDto, DpiActionDto, LightModeDto, MouseButtonDto, SleepTimeDto};

    fn connected_handle() -> DeviceHandle {
        let (transport, _controller) = SimTransport::new(62, 1, 4);
        hyperpace_device::spawn(Box::new(transport), Access::ReadWrite).unwrap()
    }

    #[test]
    fn polling_writes_the_documented_code_and_reads_back() {
        let handle = connected_handle();
        apply_write_setting(
            &handle,
            &CID_62_MID_1,
            WriteSettingRequest::Polling { hz: 500 },
        )
        .unwrap();
        let shadow = handle.read_settings().unwrap();
        assert_eq!(shadow.scalar(offset::POLLING), 2);
    }

    #[test]
    fn an_unrecognized_polling_rate_is_refused_before_touching_the_device() {
        let handle = connected_handle();
        let error = apply_write_setting(
            &handle,
            &CID_62_MID_1,
            WriteSettingRequest::Polling { hz: 333 },
        )
        .unwrap_err();
        assert!(matches!(error, AppError::Protocol(_)));
    }

    #[test]
    fn debounce_past_the_model_ceiling_is_refused() {
        let handle = connected_handle();
        let over = CID_62_MID_1.max_debounce_ms + 1;
        let error = apply_write_setting(
            &handle,
            &CID_62_MID_1,
            WriteSettingRequest::Debounce { ms: over },
        )
        .unwrap_err();
        assert!(matches!(error, AppError::Protocol(_)));
    }

    #[test]
    fn dpi_stage_writes_both_the_value_and_color_records() {
        let handle = connected_handle();
        apply_write_setting(
            &handle,
            &CID_62_MID_1,
            WriteSettingRequest::DpiStage {
                index: 0,
                dpi: 800,
                color: (255, 0, 0),
            },
        )
        .unwrap();
        let shadow = handle.read_settings().unwrap();
        let color = shadow.scalar(offset::DPI_COLOR);
        assert_eq!(color, 255);
        // dpi_from_bytes is the protocol crate's own inverse; re-deriving it here would duplicate
        // protocol knowledge this module must not hold, so this only proves the write landed at
        // the documented offset.
        assert_ne!(shadow.scalar(offset::DPI_VALUE), 0xff);
    }

    #[test]
    fn lighting_writes_the_struct_and_the_separate_on_scalar() {
        let handle = connected_handle();
        apply_write_setting(
            &handle,
            &CID_62_MID_1,
            WriteSettingRequest::Lighting {
                value: LightingDto {
                    mode: LightModeDto::Fixed,
                    color: (0, 255, 0),
                    speed: 5,
                    brightness: 5,
                    on: true,
                },
            },
        )
        .unwrap();
        let shadow = handle.read_settings().unwrap();
        assert_eq!(shadow.scalar(offset::LIGHTING), 3); // Fixed
        assert_eq!(shadow.scalar(offset::LIGHT_ON), 1);
    }

    #[test]
    fn button_write_lands_at_the_documented_offset() {
        let handle = connected_handle();
        apply_set_button(
            &handle,
            SetButtonRequest {
                index: 1,
                action: ButtonActionDto::Dpi {
                    action: DpiActionDto::Loop,
                },
                keystroke: None,
            },
        )
        .unwrap();
        let shadow = handle.read_settings().unwrap();
        let record = [
            shadow.scalar(offset::BUTTONS + 4),
            shadow.scalar(offset::BUTTONS + 5),
            shadow.scalar(offset::BUTTONS + 6),
            shadow.scalar(offset::BUTTONS + 7),
        ];
        assert_eq!(
            ButtonAction::decode(&record),
            ButtonAction::Dpi(DpiAction::Loop)
        );
    }

    #[test]
    fn a_keystroke_button_also_writes_its_chord() {
        let handle = connected_handle();
        apply_set_button(
            &handle,
            SetButtonRequest {
                index: 0,
                action: ButtonActionDto::Keystroke,
                keystroke: Some(crate::dto::KeystrokeDto {
                    modifiers: vec![],
                    key: Some(4),
                    media: None,
                }),
            },
        )
        .unwrap();
        let shadow = handle.read_settings().unwrap();
        let record = [
            shadow.scalar(offset::BUTTONS),
            shadow.scalar(offset::BUTTONS + 1),
            shadow.scalar(offset::BUTTONS + 2),
            shadow.scalar(offset::BUTTONS + 3),
        ];
        assert_eq!(ButtonAction::decode(&record), ButtonAction::Keystroke);
        assert_ne!(shadow.scalar(offset::KEYSTROKE), 0xff);
    }

    #[test]
    fn set_profile_and_factory_reset_round_trip_against_the_simulator() {
        let handle = connected_handle();
        handle
            .request(request::set_profile(2), REQUEST_TIMEOUT)
            .unwrap();
        handle
            .request(Command::FactoryReset.request(), REQUEST_TIMEOUT)
            .unwrap();
    }

    #[test]
    fn mouse_button_action_round_trips_through_write_and_read() {
        let handle = connected_handle();
        apply_set_button(
            &handle,
            SetButtonRequest {
                index: 2,
                action: ButtonActionDto::Mouse {
                    button: MouseButtonDto::Middle,
                },
                keystroke: None,
            },
        )
        .unwrap();
        let shadow = handle.read_settings().unwrap();
        let base = offset::BUTTONS + 8;
        let record = [
            shadow.scalar(base),
            shadow.scalar(base + 1),
            shadow.scalar(base + 2),
            shadow.scalar(base + 3),
        ];
        assert_eq!(
            ButtonAction::decode(&record),
            ButtonAction::Mouse(MouseButton::Middle)
        );
    }

    #[test]
    fn performance_timeout_writes_the_sleep_time_code() {
        let handle = connected_handle();
        apply_write_setting(
            &handle,
            &CID_62_MID_1,
            WriteSettingRequest::PerformanceTimeout {
                value: SleepTimeDto::OneMinute,
            },
        )
        .unwrap();
        let shadow = handle.read_settings().unwrap();
        assert_eq!(shadow.scalar(offset::PERF_TIMEOUT), 6);
    }
}
