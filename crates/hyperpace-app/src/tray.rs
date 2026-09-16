//! The system tray: the dynamic battery icon, the status and battery lines in its menu, and
//! low-battery notifications.
//!
//! `docs/plans/hyperpace.md`: "the tray owns battery display; the window may be closed without
//! exiting" and "the tray icon is redrawn only when the displayed bucket changes, because on this
//! platform every icon update writes a file and crosses D-Bus." [`TrayHandles`] caches the last
//! rendered `(percent, charging)` pair so [`on_state_changed`] skips the redraw when nothing the
//! icon shows has actually changed.
//!
//! Every line of text the tray shows is derived from one [`TrayStatus`], so the tooltip, the menu
//! and the icon can never disagree about what the device is doing. None of them ever shows a
//! number the device has not reported: a connection whose battery has not been read yet says so.

use std::sync::{Mutex, PoisonError};

use tauri::image::Image;
use tauri::menu::{MenuBuilder, MenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, DynRuntime, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::dto::DeviceStateDto;
use crate::icon::render_battery_icon;
use crate::window;

/// Fixed id for the app's one tray icon.
pub const TRAY_ID: &str = "hyperpace-tray";

const MENU_SHOW: &str = "show";
const MENU_STATUS: &str = "status";
const MENU_BATTERY: &str = "battery";
const MENU_QUIT: &str = "quit";

/// Where the mouse is, as one value: a connection that is not answering is a different thing
/// from no connection at all, and the tray has to say which.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Presence {
    /// Nothing is connected.
    #[default]
    Missing,
    /// Connected, but the last online probe came back negative: asleep or out of range.
    Asleep,
    /// Connected and answering.
    Awake,
}

/// The link the connection resolved, once the device has reported its identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Link {
    /// Whether this is a wired link.
    wired: bool,
}

/// Everything the tray displays about the device, derived once from a [`DeviceStateDto`] so the
/// icon, the tooltip and the two menu lines all read the same facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TrayStatus {
    /// Where the mouse is.
    presence: Presence,
    /// Its link, once known.
    link: Option<Link>,
    /// The last battery percent read, if one has been.
    percent: Option<u8>,
    /// Whether that reading said the device is charging.
    charging: bool,
}

impl From<DeviceStateDto> for TrayStatus {
    fn from(state: DeviceStateDto) -> Self {
        Self {
            presence: match (state.connected, state.online) {
                (false, _) => Presence::Missing,
                (true, false) => Presence::Asleep,
                (true, true) => Presence::Awake,
            },
            link: state.identity.map(|identity| Link {
                wired: identity.wired,
            }),
            percent: state.battery.map(|battery| battery.percent),
            charging: state.battery.is_some_and(|battery| battery.charging),
        }
    }
}

impl TrayStatus {
    /// The `(percent, charging)` pair the icon pixels are drawn from; [`on_state_changed`] skips
    /// the redraw when this has not changed.
    fn icon_key(self) -> (Option<u8>, bool) {
        (self.percent, self.charging)
    }

    /// One plain sentence naming what the mouse is doing right now.
    fn status_line(self) -> String {
        match (self.presence, self.link) {
            (Presence::Missing, _) => "No mouse found".to_owned(),
            (Presence::Asleep, _) => "Mouse asleep".to_owned(),
            // The link, not its polling ceiling: "up to 8000 Hz" reads as the rate the mouse is set
            // to, and the tray has no way to know the rate the operator actually chose.
            (Presence::Awake, None) => "Connected".to_owned(),
            (Presence::Awake, Some(link)) if link.wired => "Connected by cable".to_owned(),
            (Presence::Awake, Some(_)) => "Connected over 2.4 GHz".to_owned(),
        }
    }

    /// The battery line. Never invents a percent: a connection that has not produced a reading
    /// yet says exactly that.
    fn battery_line(self) -> String {
        match (self.percent, self.charging) {
            (Some(percent), true) => format!("Battery: {percent}% (charging)"),
            (Some(percent), false) => format!("Battery: {percent}%"),
            (None, _) => match self.presence {
                Presence::Missing => "Battery: unknown".to_owned(),
                Presence::Asleep | Presence::Awake => "Battery: not read yet".to_owned(),
            },
        }
    }

    /// The hover tooltip: the same facts as the menu, in one line.
    fn tooltip(self) -> String {
        match self.presence {
            Presence::Missing => "Hyperpace: no mouse found".to_owned(),
            Presence::Asleep => "Hyperpace: mouse asleep".to_owned(),
            Presence::Awake => match (self.percent, self.charging) {
                (Some(percent), true) => format!("Hyperpace: battery {percent}% (charging)"),
                (Some(percent), false) => format!("Hyperpace: battery {percent}%"),
                (None, _) => "Hyperpace: connected, battery not read yet".to_owned(),
            },
        }
    }
}

/// The icon and the two menu lines kept alive for the app's lifetime so [`on_state_changed`] can
/// update them later, plus the last `(percent, charging)` rendered, to skip a redundant redraw.
///
/// Managed as Tauri state by [`build`]; every later update goes through
/// `app.try_state::<TrayHandles>()`.
pub struct TrayHandles {
    icon: TrayIcon<DynRuntime>,
    status_item: MenuItem<DynRuntime>,
    battery_item: MenuItem<DynRuntime>,
    last_rendered: Mutex<Option<(Option<u8>, bool)>>,
}

/// Build the tray icon and its menu, and register the handles later updates need. Called once
/// from `setup`.
///
/// # Errors
///
/// Returns whatever [`tauri::Error`] building the menu items, the menu or the tray icon reports.
pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let initial = TrayStatus::default();
    let show = MenuItem::with_id(app, MENU_SHOW, "Show Hyperpace", true, None::<&str>)?;
    let status_item =
        MenuItem::with_id(app, MENU_STATUS, initial.status_line(), false, None::<&str>)?;
    let battery_item = MenuItem::with_id(
        app,
        MENU_BATTERY,
        initial.battery_line(),
        false,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, MENU_QUIT, "Quit", true, None::<&str>)?;
    let menu = MenuBuilder::new(app)
        .item(&status_item)
        .item(&battery_item)
        .separator()
        .item(&show)
        .item(&quit)
        .build()?;

    let image = render_battery_icon(initial.percent, initial.charging);
    let icon = TrayIconBuilder::with_id(TRAY_ID)
        .icon(Image::new_owned(image.rgba, image.width, image.height))
        .tooltip(initial.tooltip())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            MENU_SHOW => {
                if let Err(error) = window::show_main_window(app) {
                    tracing::error!(%error, "could not show the main window from the tray menu");
                }
            }
            MENU_QUIT => app.exit(0),
            _ => {}
        })
        .build(app)?;

    app.manage(TrayHandles {
        icon,
        status_item,
        battery_item,
        last_rendered: Mutex::new(None),
    });
    Ok(())
}

/// Update the tray icon, tooltip and both menu lines to match `state`. Safe to call on every
/// device event; the icon itself is only redrawn when `(percent, charging)` actually changed.
pub fn on_state_changed(app: &AppHandle, state: DeviceStateDto) {
    let Some(handles) = app.try_state::<TrayHandles>() else {
        return; // tray not built yet; should not happen once setup has run
    };
    let status = TrayStatus::from(state);

    let mut last = handles
        .last_rendered
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let redraw = *last != Some(status.icon_key());
    if redraw {
        *last = Some(status.icon_key());
    }
    drop(last);

    if redraw {
        let image = render_battery_icon(status.percent, status.charging);
        if let Err(error) = handles.icon.set_icon(Some(Image::new_owned(
            image.rgba,
            image.width,
            image.height,
        ))) {
            tracing::warn!(%error, "could not update the tray icon");
        }
    }

    if let Err(error) = handles.icon.set_tooltip(Some(status.tooltip())) {
        tracing::warn!(%error, "could not update the tray tooltip");
    }
    if let Err(error) = handles.status_item.set_text(status.status_line()) {
        tracing::warn!(%error, "could not update the tray status line");
    }
    if let Err(error) = handles.battery_item.set_text(status.battery_line()) {
        tracing::warn!(%error, "could not update the tray battery line");
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
        tracing::warn!(%error, percent, "could not show the low-battery notification");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{BatteryDto, DeviceIdentityDto, LinkTypeDto};

    fn state(connected: bool, online: bool) -> DeviceStateDto {
        DeviceStateDto {
            connected,
            backend: None,
            access: None,
            identity: None,
            battery: None,
            online,
        }
    }

    fn wireless_identity() -> DeviceIdentityDto {
        DeviceIdentityDto {
            cid: 102,
            mid: 1,
            link: LinkTypeDto::Wireless2k,
            max_polling_hz: 2000,
            wired: false,
            capabilities: None,
        }
    }

    #[test]
    fn a_disconnected_tray_says_so_everywhere() {
        let status = TrayStatus::from(state(false, false));
        assert_eq!(status.status_line(), "No mouse found");
        assert_eq!(status.battery_line(), "Battery: unknown");
        assert_eq!(status.tooltip(), "Hyperpace: no mouse found");
    }

    #[test]
    fn a_connection_without_a_reading_never_shows_a_percent() {
        let status = TrayStatus::from(state(true, true));
        assert_eq!(status.battery_line(), "Battery: not read yet");
        assert_eq!(
            status.tooltip(),
            "Hyperpace: connected, battery not read yet"
        );
    }

    #[test]
    fn an_asleep_device_is_distinguished_from_a_missing_one() {
        let status = TrayStatus::from(state(true, false));
        assert_eq!(status.status_line(), "Mouse asleep");
        assert_eq!(status.tooltip(), "Hyperpace: mouse asleep");
    }

    #[test]
    fn the_status_line_names_the_link_once_the_identity_is_known() {
        let mut dto = state(true, true);
        dto.identity = Some(wireless_identity());
        assert_eq!(
            TrayStatus::from(dto).status_line(),
            "Connected over 2.4 GHz"
        );

        let mut wired = state(true, true);
        wired.identity = Some(DeviceIdentityDto {
            link: LinkTypeDto::Wired8k,
            max_polling_hz: 8000,
            wired: true,
            ..wireless_identity()
        });
        assert_eq!(TrayStatus::from(wired).status_line(), "Connected by cable");
    }

    #[test]
    fn a_reading_is_shown_with_its_charging_state() {
        let mut dto = state(true, true);
        dto.battery = Some(BatteryDto {
            percent: 73,
            charging: true,
            millivolts: 4050,
        });
        let status = TrayStatus::from(dto);
        assert_eq!(status.battery_line(), "Battery: 73% (charging)");
        assert_eq!(status.tooltip(), "Hyperpace: battery 73% (charging)");
        assert_eq!(status.icon_key(), (Some(73), true));
    }
}
