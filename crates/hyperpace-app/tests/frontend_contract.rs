//! Guards the Tauri IPC boundary between this crate and `ui/`, the specific defect class
//! documented in `docs/architecture/api-contract.md`: the frontend's `invoke` call sites silently
//! drifting from the real `#[tauri::command]` signatures, since nothing in the Rust build checks
//! that boundary.
//!
//! Two layers, both cheap (no new dependency; `serde_json` is already a dependency):
//!
//! 1. `frontend_command_list_matches_the_registered_handlers`: `ui/src/lib/generated/commands.ts`
//!    is a checked-in companion to [`command_list::COMMANDS`], kept in sync by hand; this test
//!    parses it back out and asserts the two lists are identical, so a command renamed, added or
//!    removed on the Rust side without a matching frontend edit fails here instead of at runtime.
//!    `ui/src/lib/tauri.ts`'s `invoke` also only accepts that file's `CommandName` union, so a
//!    frontend call site with a misspelled command name additionally fails to type-check.
//! 2. The `*_matches_a_frontend_call` tests: each deserializes a JSON literal written to mirror
//!    exactly what `ui/src/lib/device.svelte.ts` sends for that command, into the real request
//!    type the command handler receives, and asserts the fields landed as the frontend intended.
//!    These catch the argument-name and payload-shape drift a command-name list alone cannot (the
//!    class of bug this suite exists for: `connect`, `delete_macro`, `write_setting` and
//!    `set_button` all had a correct command name but a mismatched payload).

// This is a test binary, not a production path: the doctrine's no-panic rule (`CLAUDE.md`
// section 3) reserves panics for tests, and an integration test crate is not gated by a
// `#[cfg(test)] mod tests` the way this crate's own unit tests are, so the same allowances are
// declared explicitly here (see crates/hyperpace-firmware/tests/archive.rs).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use hyperpace_app::dto::{
    AccessDto, AppSettingsRequest, ButtonActionDto, ConnectRequest, DpiIndicatorModeDto,
    MouseButtonDto, SaveMacroRequest, SetButtonRequest, WriteSettingRequest,
};

#[path = "../src/command_list.rs"]
mod command_list;

/// Every command name declared in `ui/src/lib/generated/commands.ts`'s `COMMAND_NAMES` array,
/// parsed out of its checked-in source. A tiny hand-rolled parser rather than a JS/TS runtime
/// dependency in the Rust test suite: the file's shape (one quoted string per line inside a
/// `[...]`) is fixed by convention with the frontend, not by a shared schema.
fn frontend_command_names() -> Vec<String> {
    let source = include_str!("../../../ui/src/lib/generated/commands.ts");
    let start = source
        .find('[')
        .expect("commands.ts must declare COMMAND_NAMES as an array literal");
    let end = source[start..]
        .find(']')
        .expect("commands.ts's COMMAND_NAMES array must be closed")
        + start;
    source[start + 1..end]
        .lines()
        .filter_map(|line| {
            let line = line.trim().trim_end_matches(',');
            line.strip_prefix('\'')
                .and_then(|rest| rest.strip_suffix('\''))
                .map(str::to_owned)
        })
        .collect()
}

#[test]
fn frontend_command_list_matches_the_registered_handlers() {
    let frontend = frontend_command_names();
    assert_eq!(
        frontend,
        command_list::COMMANDS,
        "ui/src/lib/generated/commands.ts has drifted from command_list::COMMANDS; a command was \
         renamed, added or removed on one side without updating the other"
    );
}

#[test]
fn connect_request_matches_a_frontend_call() {
    // device.svelte.ts's `connect`: always sends both fields explicitly.
    let request: ConnectRequest =
        serde_json::from_str(r#"{"realDevice":true,"access":"readWrite"}"#).unwrap();
    assert_eq!(
        request,
        ConnectRequest {
            real_device: true,
            access: AccessDto::ReadWrite,
        }
    );
}

#[test]
fn write_setting_request_matches_frontend_calls() {
    // PerformanceScreen/LightingScreen's `diffSettings`, a boolean-field variant.
    let motion_sync: WriteSettingRequest =
        serde_json::from_str(r#"{"key":"motionSync","on":true}"#).unwrap();
    assert_eq!(motion_sync, WriteSettingRequest::MotionSync { on: true });

    // ...and a variant carrying a tuple-encoded color.
    let dpi_stage: WriteSettingRequest =
        serde_json::from_str(r#"{"key":"dpiStage","index":2,"dpi":1600,"color":[10,20,30]}"#)
            .unwrap();
    assert_eq!(
        dpi_stage,
        WriteSettingRequest::DpiStage {
            index: 2,
            dpi: 1600,
            color: (10, 20, 30),
        }
    );

    // PerformanceScreen's long range toggle: a device command pair, not a flash scalar, but the
    // same `{key, on}` shape as every other boolean field.
    let long_range: WriteSettingRequest =
        serde_json::from_str(r#"{"key":"longRange","on":true}"#).unwrap();
    assert_eq!(long_range, WriteSettingRequest::LongRange { on: true });

    // LightingScreen's DPI indicator panel: the nested-tag variant...
    let dpi_indicator_mode: WriteSettingRequest =
        serde_json::from_str(r#"{"key":"dpiIndicatorMode","value":{"mode":"breathing"}}"#).unwrap();
    assert_eq!(
        dpi_indicator_mode,
        WriteSettingRequest::DpiIndicatorMode {
            value: DpiIndicatorModeDto::Breathing,
        }
    );

    // ...and its three plain scalar siblings.
    let dpi_indicator_brightness: WriteSettingRequest =
        serde_json::from_str(r#"{"key":"dpiIndicatorBrightness","level":5}"#).unwrap();
    assert_eq!(
        dpi_indicator_brightness,
        WriteSettingRequest::DpiIndicatorBrightness { level: 5 }
    );

    let dpi_indicator_speed: WriteSettingRequest =
        serde_json::from_str(r#"{"key":"dpiIndicatorSpeed","speed":7}"#).unwrap();
    assert_eq!(
        dpi_indicator_speed,
        WriteSettingRequest::DpiIndicatorSpeed { speed: 7 }
    );

    let dpi_indicator_on: WriteSettingRequest =
        serde_json::from_str(r#"{"key":"dpiIndicatorOn","on":false}"#).unwrap();
    assert_eq!(
        dpi_indicator_on,
        WriteSettingRequest::DpiIndicatorOn { on: false }
    );
}

#[test]
fn set_button_request_matches_a_frontend_call() {
    // ButtonsScreen's `apply`: sends `{ request: { index, action } }`, no `slot` field.
    let request: SetButtonRequest =
        serde_json::from_str(r#"{"index":3,"action":{"type":"mouse","button":"middle"}}"#).unwrap();
    assert_eq!(request.index, 3);
    assert_eq!(
        request.action,
        ButtonActionDto::Mouse {
            button: MouseButtonDto::Middle
        }
    );
    assert!(request.keystroke.is_none());
}

#[test]
fn save_macro_request_matches_a_frontend_call() {
    // MacrosScreen's `save`: an id-less request creates a new store record.
    let request: SaveMacroRequest = serde_json::from_str(
        r#"{"name":"burst","slot":2,"events":[{"press":true,"kind":"Key","value":4,"delayMs":10}]}"#,
    )
    .unwrap();
    assert_eq!(request.id, None);
    assert_eq!(request.name, "burst");
    assert_eq!(request.slot, Some(2));
    assert_eq!(request.events.len(), 1);
    assert_eq!(request.events[0].delay_ms, 10);
}

#[test]
fn app_settings_request_matches_a_frontend_call() {
    // device.svelte.ts's `writeAppSettings`: one `Set` request per changed key.
    let request: AppSettingsRequest =
        serde_json::from_str(r#"{"action":"set","key":"autostart","value":true}"#).unwrap();
    assert_eq!(
        request,
        AppSettingsRequest::Set {
            key: "autostart".to_owned(),
            value: serde_json::json!(true),
        }
    );
}
