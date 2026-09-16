//! Keeps the app attached to the operator's own mouse without anyone having to press connect.
//!
//! The app is a configurator, so a window that shows nothing until the operator walks to Settings
//! and picks a backend is a window that shows nothing. This module connects to the real device at
//! startup and follows [`hyperpace_device::watch`] from then on, so plugging the receiver in
//! brings the tray and every open screen to life on their own.
//!
//! What it will not do:
//!
//! - It never connects to the simulator. That stays an explicit choice in Settings.
//! - It never replaces a connection that already exists, so an operator who deliberately
//!   connected to the simulator keeps it.
//! - It never writes. Connecting performs the device's own handshake and settings walk, which are
//!   reads; the access it asks for only decides whether a later write the operator makes in the
//!   interface is allowed to reach the device at all.

use std::thread;

use hyperpace_device::{DeviceError, DeviceEvent, HotplugEvent};
use tauri::{AppHandle, Manager};

use crate::dto::{AccessDto, DeviceBackendDto, DeviceEventPayload};
use crate::error::AppError;
use crate::state::AppState;

/// The access an automatic connection asks for.
///
/// [`AccessDto::ReadWrite`], because every control in the interface is a write and a connection
/// that cannot make one leaves the whole app disabled. Nothing is written by connecting itself;
/// the first write is the one the operator makes.
const ACCESS: AccessDto = AccessDto::ReadWrite;

/// Start the watcher thread. Called once from `setup`; never fails the app's startup, because an
/// app that cannot watch for the device is still usable through Settings.
pub fn spawn(app: AppHandle) {
    let started = thread::Builder::new()
        .name("hyperpace-app-autoconnect".to_owned())
        .spawn(move || run(&app));
    if let Err(error) = started {
        tracing::error!(%error, "could not start the device watcher; connect from Settings instead");
    }
}

fn run(app: &AppHandle) {
    // The watcher goes first, then the initial attempt: a device that appeared in the gap between
    // an enumeration and starting a watcher would otherwise go unreported until the next replug
    // (`hyperpace_device::watch`'s own contract).
    let events = match hyperpace_device::watch() {
        Ok(events) => Some(events),
        Err(error) => {
            tracing::warn!(
                %error,
                "cannot watch for the mouse being plugged in; connect from Settings instead"
            );
            None
        }
    };

    try_connect(app);

    let Some(events) = events else { return };
    for event in events {
        match event {
            HotplugEvent::Added => try_connect(app),
            HotplugEvent::Removed => {
                // One attached collection went away. Drop what may now be a dead connection, then
                // try once: if the mouse is still here and something else was unplugged, this
                // reconnects it, and if it is gone the attempt simply fails.
                drop_real_connection(app);
                try_connect(app);
            }
        }
    }
}

/// Connect to the real device, unless something is already connected.
fn try_connect(app: &AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return; // state not managed yet; the setup hook manages it before spawning this thread
    };
    if state.is_connected() {
        return;
    }
    match state.connect(true, ACCESS) {
        Ok(_) => tracing::info!("connected to the mouse"),
        // `Disconnected` means nothing matching is attached, which is the normal state of an app
        // sitting in the tray with the receiver unplugged: not a problem to report, and the
        // watcher tries again the moment one appears. Anything else means the collection was
        // found and the open itself failed, which the operator can usually act on.
        Err(AppError::Device(DeviceError::Disconnected)) => {
            tracing::debug!("no mouse attached yet");
        }
        Err(error) => tracing::warn!(
            %error,
            "the mouse is attached but could not be opened; on Linux this usually means the \
             device access rule the packages install is missing"
        ),
    }
}

/// Drop the current connection if it is a real device, and tell the tray and every open window.
/// A simulator session is left alone: nothing about a USB event concerns it.
fn drop_real_connection(app: &AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    if !matches!(state.backend(), Ok(DeviceBackendDto::RealDevice)) {
        return;
    }
    state.disconnect();
    state.announce(DeviceEventPayload::from(DeviceEvent::Disconnected));
    tracing::info!("the mouse was unplugged");
}
