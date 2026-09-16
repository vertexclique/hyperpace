//! Hyperpace desktop application: commands, tray, notifications and window lifecycle.
//!
//! `docs/architecture/api-contract.md`: this crate depends on every other crate in the workspace
//! plus Tauri, and holds no protocol knowledge of its own; every command in [`commands`] converts
//! a request, calls one [`state::AppState`] or [`hyperpace_store::Store`] method, and converts
//! the result back.
//!
//! Device access defaults to read-only ([`hyperpace_device::Access::default`]); a caller of the
//! `connect` command must explicitly ask for write access, and the simulator backend is used
//! unless it explicitly asks for the real device (`commands::device::connect`,
//! [`dto::ConnectRequest`]).
//!
//! # Layers
//!
//! - [`error`]: [`error::AppError`], the one error type every command converts to `String` at its
//!   boundary.
//! - [`dto`]: every request and response shape crossing the IPC boundary.
//! - [`blocking`]: runs a device or store call off the async executor.
//! - [`state`]: [`state::AppState`], the current connection, and the background thread that
//!   tracks its events, fans them out to every subscribed window, and drives the tray.
//! - [`icon`]: renders the battery percentage into the tray icon's own pixels.
//! - [`tray`]: builds the tray icon and its menu, and updates them as the device state changes.
//! - [`window`]: the main window's lifecycle, including hide-to-tray with the webview destroyed.
//! - [`commands`]: every Tauri command in the API contract.

pub mod blocking;
pub mod commands;
pub mod dto;
pub mod error;
pub mod icon;
pub mod state;
pub mod tray;
pub mod window;

use hyperpace_store::Store;
use tauri::Manager;

use error::AppError;
use state::AppState;

/// Build and run the Hyperpace desktop application. Blocks until the app exits.
///
/// # Errors
///
/// Returns [`AppError::Store`] when the local store cannot be opened, and [`AppError::Tauri`]
/// when the Tauri application itself fails to build (a malformed bundled config, or the runtime
/// failing to initialize). The caller (`main.rs`) decides how to report that; this function never
/// panics on it.
pub fn run() -> Result<(), AppError> {
    let store_root = Store::default_root()?;
    let store = Store::open(&store_root)?;

    let app = tauri::Builder::default()
        .runtime(tauri_runtime_wry::Wry::default())
        // Must be the first plugin registered: plugins run in registration order, and this one
        // needs to intercept a second launch before anything else initializes
        // (tauri-plugin-single-instance's own README).
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // A second launch focuses the existing window instead of starting a second instance.
            if let Err(error) = window::show_main_window(app) {
                eprintln!("hyperpace: could not show the main window: {error}");
            }
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .invoke_handler(tauri::generate_handler![
            commands::device::list_devices,
            commands::device::connect,
            commands::device::disconnect,
            commands::device::device_state,
            commands::device::read_settings,
            commands::device::write_setting,
            commands::device::set_button,
            commands::macros::save_macro,
            commands::macros::list_macros,
            commands::macros::delete_macro,
            commands::device::set_profile,
            commands::device::factory_reset,
            commands::device::pair_receiver,
            commands::device::receiver_light,
            commands::device::export_config,
            commands::device::import_config,
            commands::firmware::firmware_list,
            commands::firmware::firmware_import,
            commands::firmware::firmware_install,
            commands::firmware::firmware_check_for_updates,
            commands::settings::app_settings,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            app.manage(AppState::new(store, store_root, handle.clone()));
            tray::build(&handle)?;
            window::show_main_window(&handle)?;
            Ok(())
        })
        .build(tauri::generate_context!())?;

    app.run(|_app_handle, _event| {});
    Ok(())
}
