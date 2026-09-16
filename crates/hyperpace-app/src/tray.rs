//! The system tray: the dynamic battery icon, its menu, and low-battery notifications.
//!
//! `docs/plans/hyperpace.md`: "the tray owns battery display; the window may be closed without
//! exiting" and "the tray icon is redrawn only when the displayed bucket changes, because on this
//! platform every icon update writes a file and crosses D-Bus." [`TrayHandles`] caches the last
//! rendered `(percent, charging)` pair so [`on_state_changed`] skips the redraw when nothing the
//! icon shows has actually changed.

use std::sync::{Mutex, PoisonError};

use tauri::image::Image;
use tauri::menu::{MenuBuilder, MenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, DynRuntime, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::dto::DeviceStateDto;
use crate::icon::{render_battery_icon, tray_tooltip};
use crate::window;

/// Fixed id for the app's one tray icon.
pub const TRAY_ID: &str = "hyperpace-tray";

const MENU_SHOW: &str = "show";
const MENU_BATTERY: &str = "battery";
const MENU_QUIT: &str = "quit";

/// The icon and menu line kept alive for the app's lifetime so [`on_state_changed`] can update
/// them later, plus the last `(percent, charging)` rendered, to skip a redundant redraw.
///
/// Managed as Tauri state by [`build`]; every later update goes through
/// `app.try_state::<TrayHandles>()`.
pub struct TrayHandles {
    icon: TrayIcon<DynRuntime>,
    battery_item: MenuItem<DynRuntime>,
    last_rendered: Mutex<Option<(Option<u8>, bool)>>,
}

/// The battery menu item's label for `percent`, or a state name when there is none yet.
fn battery_menu_label(percent: Option<u8>, connected: bool) -> String {
    match percent {
        Some(percent) => format!("Battery: {percent}%"),
        None if connected => "Battery: unknown".to_owned(),
        None => "Not connected".to_owned(),
    }
}

/// Build the tray icon and its menu, and register the handles later updates need. Called once
/// from `setup`.
///
/// # Errors
///
/// Returns whatever [`tauri::Error`] building the menu items, the menu or the tray icon reports.
pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, MENU_SHOW, "Show Hyperpace", true, None::<&str>)?;
    let battery_item = MenuItem::with_id(
        app,
        MENU_BATTERY,
        battery_menu_label(None, false),
        false,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, MENU_QUIT, "Quit", true, None::<&str>)?;
    let menu = MenuBuilder::new(app)
        .item(&show)
        .item(&battery_item)
        .separator()
        .item(&quit)
        .build()?;

    let initial = render_battery_icon(None, false);
    let icon = TrayIconBuilder::with_id(TRAY_ID)
        .icon(Image::new_owned(
            initial.rgba,
            initial.width,
            initial.height,
        ))
        .tooltip(tray_tooltip(None, false))
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            MENU_SHOW => {
                // vertexia: eprintln, not structured logging; no logging crate is set up
                // anywhere in this workspace yet, and adding one is a bigger call than one tray
                // handler. Upgrade path: route through whichever crate the workspace adopts.
                if let Err(error) = window::show_main_window(app) {
                    eprintln!("hyperpace: could not show the main window: {error}");
                }
            }
            MENU_QUIT => app.exit(0),
            _ => {}
        })
        .build(app)?;

    app.manage(TrayHandles {
        icon,
        battery_item,
        last_rendered: Mutex::new(None),
    });
    Ok(())
}

/// Update the tray icon, tooltip and battery menu line to match `state`. Safe to call on every
/// device event; the icon itself is only redrawn when `(percent, charging)` actually changed.
pub fn on_state_changed(app: &AppHandle, state: DeviceStateDto) {
    let Some(handles) = app.try_state::<TrayHandles>() else {
        return; // tray not built yet; should not happen once setup has run
    };

    let percent = state.battery.map(|battery| battery.percent);
    let charging = state.battery.is_some_and(|battery| battery.charging);
    let key = (percent, charging);

    let mut last = handles
        .last_rendered
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if *last == Some(key) {
        drop(last);
    } else {
        *last = Some(key);
        drop(last);
        let image = render_battery_icon(percent, charging);
        if let Err(error) = handles.icon.set_icon(Some(Image::new_owned(
            image.rgba,
            image.width,
            image.height,
        ))) {
            eprintln!("hyperpace: could not update the tray icon: {error}");
        }
        if let Err(error) = handles
            .icon
            .set_tooltip(Some(tray_tooltip(percent, charging)))
        {
            eprintln!("hyperpace: could not update the tray tooltip: {error}");
        }
    }

    if let Err(error) = handles
        .battery_item
        .set_text(battery_menu_label(percent, state.connected))
    {
        eprintln!("hyperpace: could not update the tray menu: {error}");
    }
}

/// Send a low-battery notification through the OS notification center.
pub fn notify_low_battery(app: &AppHandle, percent: u8) {
    let result = app
        .notification()
        .builder()
        .title("Hyperpace")
        .body(format!("Battery at {percent}%. Charge the mouse soon."))
        .show();
    if let Err(error) = result {
        eprintln!("hyperpace: could not show the low-battery notification: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battery_label_shows_the_percent_when_known() {
        assert_eq!(battery_menu_label(Some(42), true), "Battery: 42%");
    }

    #[test]
    fn battery_label_distinguishes_connected_but_unknown_from_disconnected() {
        assert_eq!(battery_menu_label(None, true), "Battery: unknown");
        assert_eq!(battery_menu_label(None, false), "Not connected");
    }
}
