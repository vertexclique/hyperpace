//! Main window lifecycle: creation, and hide-to-tray with the webview destroyed.
//!
//! `docs/architecture/api-contract.md`: "Events to the UI go over one `Channel<DeviceEvent>` per
//! window, re-subscribed when a destroyed window is recreated." Closing the window destroys its
//! webview outright ([`tauri::WebviewWindow::destroy`]) rather than merely hiding it, so the
//! tray's "Show" action always creates a fresh window with a fresh IPC channel; the state's
//! subscription for [`MAIN_WINDOW_LABEL`] is explicitly dropped here rather than left to the next
//! event that happens to fail sending through it, since a disconnected device produces no events
//! to trigger that fallback.

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};

use crate::state::AppState;

/// Label of the app's one window. Reused across every destroy/recreate cycle, since the tray's
/// device-state channel subscription is keyed by this label.
pub const MAIN_WINDOW_LABEL: &str = "main";

/// Create the main window, or bring an already-open one to front.
///
/// # Errors
///
/// Returns whatever [`tauri::Error`] building or focusing the window reports.
pub fn show_main_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }

    // 1180x940: at this size every screen's dense panel grid (Lighting and receiver's three
    // panels above all, the tallest: mouse lighting, the DPI indicator and the receiver,
    // including its pairing and factory-reset rows) renders in full with no scrollbar for its
    // own content; see the responsive rules in `ui/src/app.css` and each screen's own styles.
    // 900x620 is the minimum the same layout genuinely supports without a horizontal scrollbar
    // or a clipped control: below it the sidebar collapses to icons and the two-column screens
    // (Buttons, Macros) stack instead of squeezing side by side (docs/decs/hyperpace_DECS.md).
    // A window shorter than the default, minimum included, scrolls each screen's own content
    // region rather than clipping it (`.content`'s `overflow-y: auto`).
    let window =
        WebviewWindowBuilder::new(app, MAIN_WINDOW_LABEL, WebviewUrl::App("index.html".into()))
            .title("Hyperpace")
            .inner_size(1180.0, 940.0)
            .min_inner_size(900.0, 620.0)
            .build()?;
    wire_close_to_tray(&window);
    Ok(())
}

/// Destroy the window and its webview on close instead of hiding it, and drop its device-event
/// subscription, leaving the app running in the tray.
fn wire_close_to_tray(window: &WebviewWindow) {
    let app = window.app_handle().clone();
    window.on_window_event(move |event| {
        let WindowEvent::CloseRequested { api, .. } = event else {
            return;
        };
        api.prevent_close();

        if let Some(state) = app.try_state::<AppState>() {
            state.unsubscribe(MAIN_WINDOW_LABEL);
        }
        if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL)
            && let Err(error) = window.destroy()
        {
            tracing::warn!(%error, "could not destroy the main window on close");
        }
    });
}
