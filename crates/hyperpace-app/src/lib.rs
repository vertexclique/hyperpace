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
//! [`dto::ConnectRequest`]). Those are the defaults for a bare request; the app itself connects
//! to the operator's own mouse on startup and on hotplug through [`autoconnect`], which asks for
//! write access because every control in the interface is a write.
//!
//! # Layers
//!
//! - [`error`]: [`error::AppError`], the one error type every command converts to `String` at its
//!   boundary.
//! - [`dto`]: every request and response shape crossing the IPC boundary.
//! - [`blocking`]: runs a device or store call off the async executor.
//! - [`autoconnect`]: connects to the operator's own mouse at startup and follows it across
//!   unplugs and replugs.
//! - [`state`]: [`state::AppState`], the current connection, and the background thread that
//!   tracks its events, fans them out to every subscribed window, and drives the tray.
//! - [`icon`]: renders the battery percentage into the tray icon's own pixels.
//! - [`tray`]: builds the tray icon and its menu, and updates them as the device state changes.
//! - [`window`]: the main window's lifecycle, including hide-to-tray with the webview destroyed.
//! - [`gpu_workaround`]: the Linux WebKitGTK/NVIDIA DMABUF blank-window workaround, applied before
//!   the webview exists.
//! - [`commands`]: every Tauri command in the API contract.

pub mod autoconnect;
pub mod blocking;
pub mod commands;
pub mod dto;
pub mod error;
pub mod gpu_workaround;
pub mod icon;
pub mod state;
pub mod tray;
pub mod window;

use std::path::Path;

use hyperpace_store::Store;
use tauri::Manager;

use error::AppError;
use state::AppState;

/// Install the process-wide structured logging subscriber every `tracing` call in this crate
/// writes through, so the doctrine's log levels (error/warn/info/debug/trace) become visible
/// output instead of a no-op. Reads `RUST_LOG` when set (the standard `tracing-subscriber`
/// directive syntax, e.g. `hyperpace_app=debug`), otherwise defaults to `info`; writes to stderr,
/// matching every background failure this replaces (`eprintln!`, previously). Failing to install
/// it (only possible if something else in the process already installed a global subscriber first,
/// which nothing here does) is reported the one way it can be at this point: a direct stderr print,
/// since `tracing` itself has nothing to log through yet.
fn init_logging() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    if let Err(error) = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .try_init()
    {
        eprintln!("hyperpace: could not initialize logging: {error}");
    }
}

/// Build and run the Hyperpace desktop application. Blocks until the app exits.
///
/// # Errors
///
/// Returns [`AppError::Store`] when the local store cannot be opened, and [`AppError::Tauri`]
/// when the Tauri application itself fails to build (a malformed bundled config, or the runtime
/// failing to initialize). The caller (`main.rs`) decides how to report that; this function never
/// panics on it.
pub fn run() -> Result<(), AppError> {
    init_logging();

    // Must run before `tauri::Builder` (before the webview exists): see `gpu_workaround`'s own
    // doc comment for why this needs a short-lived store handle of its own (read then dropped
    // here) rather than reusing the one opened below, and for why a re-exec, not
    // `std::env::set_var`, is how it changes this process's environment. The closure defers that
    // read until `gpu_workaround::apply` actually needs it (most startups never do: the
    // environment variable or an already-set `WEBKIT_DISABLE_DMABUF_RENDERER` settles it first).
    let store_root = Store::default_root()?;
    gpu_workaround::apply(|| read_gpu_workaround_setting(&store_root));

    // `commands::firmware_watch`'s HTTP client depends on reqwest's "rustls-no-provider" feature
    // (its own default feature pulls in an aws-lc-sys native build step nothing else in this
    // workspace needs), which requires the app to install a process-wide default crypto provider
    // itself, once, before the first TLS connection. A second install attempt returns `Err` and is
    // safe to ignore; nothing else in this process installs one.
    let _ = rustls::crypto::ring::default_provider().install_default();

    let store = Store::open(&store_root)?;

    let app = tauri::Builder::default()
        .runtime(tauri_runtime_wry::Wry::default())
        // Must be the first plugin registered: plugins run in registration order, and this one
        // needs to intercept a second launch before anything else initializes
        // (tauri-plugin-single-instance's own README).
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // A second launch focuses the existing window instead of starting a second instance.
            if let Err(error) = window::show_main_window(app) {
                tracing::error!(%error, "could not show the main window for a second launch");
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
            commands::device::get_button_keystroke,
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
            commands::firmware_watch::firmware_watch_check,
            commands::settings::app_settings,
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            app.manage(AppState::new(store, store_root, handle.clone()));
            tray::build(&handle)?;
            window::show_main_window(&handle)?;
            autoconnect::spawn(handle.clone());
            commands::firmware_watch::spawn_periodic_check(handle);
            Ok(())
        })
        .build(tauri::generate_context!())?;

    app.run(|_app_handle, _event| {});
    Ok(())
}

/// Reads the [`gpu_workaround::APP_SETTING_KEY`] app setting through a store handle opened and
/// dropped entirely within this function, before `run` opens the store it actually uses for the
/// rest of the app's lifetime.
///
/// This has to be a separate, short-lived open rather than reusing `run`'s own `store`: applying
/// the workaround can re-execute this process (see `gpu_workaround`'s doc comment), and the
/// embedded store node holds an exclusive lock on `store_root` for as long as a `Store` handle to
/// it is alive. Re-executing while that handle was still open would carry the lock into the new
/// process image with nothing left in the old one to release it, deadlocking the new process's
/// own `Store::open`. Never fails loudly: an unreadable or not-yet-created store on first launch
/// just means no override is configured yet, which is `None`, the same as an override that was
/// never set.
fn read_gpu_workaround_setting(store_root: &Path) -> Option<String> {
    let store = Store::open(store_root).ok()?;
    store
        .settings()
        .list()
        .ok()?
        .into_iter()
        .find(|(_, setting)| setting.key == gpu_workaround::APP_SETTING_KEY)
        .and_then(|(_, setting)| setting.value.as_str().map(str::to_owned))
    // `store` drops here, closing the embedded node before `gpu_workaround::apply` can re-exec.
}
