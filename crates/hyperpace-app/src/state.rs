//! The connection lifecycle: [`AppState`] itself, the current connection (`ActiveDevice`), and
//! the background thread that drains its events, updates the cached state, fans them out to
//! every subscribed window, and drives the tray.
//!
//! This module holds no protocol knowledge of its own (`docs/architecture/api-contract.md`): it
//! only calls [`hyperpace_device::spawn`], [`hyperpace_device::HidTransport`] and
//! [`hyperpace_device::SimTransport`], and reads the [`hyperpace_protocol::DeviceIdentity`] and
//! [`hyperpace_protocol::Battery`] those already decode. Everything that interprets settings
//! bytes lives in `commands::device`, which uses this module only to reach a connected
//! [`hyperpace_device::DeviceHandle`] and its resolved [`hyperpace_protocol::ModelTable`].

use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::Duration;

use hyperpace_device::{Access, DeviceEvent, DeviceHandle, HidTransport, SimTransport};
use hyperpace_protocol::{Battery, DeviceIdentity, ModelTable, Transport, table_for};
use hyperpace_store::Store;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager};

use crate::dto::{AccessDto, DeviceBackendDto, DeviceEventPayload, DeviceStateDto};
use crate::error::AppError;
use crate::tray;

/// Identity the simulator reports when `connect` is not asked for the real device: cid 62 / mid
/// 1 (`hyperpace_protocol::model::CID_62_MID_1`), 2.4 GHz wireless capped at 2000 Hz (link byte 4,
/// `docs/research/mouse-protocol-v2.md` section 3.4).
const SIM_CID: u8 = 62;
const SIM_MID: u8 = 1;
const SIM_LINK_BYTE: u8 = 4;

/// How long a command waits for one device request/reply exchange, used by every command in
/// `commands::device` and `commands::firmware` that sends a request directly.
pub const REQUEST_TIMEOUT: Duration = Duration::from_millis(1500);

/// Default `low_battery_threshold_percent` app setting when the store has none yet.
pub const DEFAULT_LOW_BATTERY_THRESHOLD: u8 = 15;
/// Key [`crate::commands::settings::app_settings`] stores the threshold under.
pub const LOW_BATTERY_THRESHOLD_KEY: &str = "low_battery_threshold_percent";

/// Whether a battery reading crossed the low-battery threshold, for the caller deciding whether
/// to raise or clear a notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LowBatteryEdge {
    /// Nothing changed since the last reading.
    None,
    /// The reading just dropped to or below the threshold for the first time this spell.
    Entered(u8),
    /// The reading just rose back above the threshold, or the device started charging.
    Cleared,
}

/// The part of a connection this crate tracks from the device's own event stream, kept separate
/// from [`ActiveDevice`] so its edge-detection logic is unit-tested without a real
/// [`DeviceHandle`].
#[derive(Debug, Default, Clone, Copy)]
struct TrackedState {
    identity: Option<DeviceIdentity>,
    battery: Option<Battery>,
    online: bool,
    low_battery_notified: bool,
}

impl TrackedState {
    /// Apply one event, returning whether the low-battery state changed.
    fn apply(&mut self, event: DeviceEvent, threshold: u8) -> LowBatteryEdge {
        match event {
            DeviceEvent::Connected(identity) => {
                self.identity = Some(identity);
                self.online = true;
                LowBatteryEdge::None
            }
            DeviceEvent::Battery(battery) => {
                self.battery = Some(battery);
                let low = battery.percent <= threshold && !battery.charging;
                if low && !self.low_battery_notified {
                    self.low_battery_notified = true;
                    LowBatteryEdge::Entered(battery.percent)
                } else if !low && self.low_battery_notified {
                    self.low_battery_notified = false;
                    LowBatteryEdge::Cleared
                } else {
                    LowBatteryEdge::None
                }
            }
            DeviceEvent::Changed(_) => LowBatteryEdge::None,
            DeviceEvent::Disconnected | DeviceEvent::Offline => {
                self.online = false;
                LowBatteryEdge::None
            }
        }
    }
}

/// One live connection: its handle, what backend and access it has, and the state
/// [`spawn_tracker`] has observed from it so far.
struct ActiveDevice {
    handle: DeviceHandle,
    backend: DeviceBackendDto,
    access: Access,
    tracked: TrackedState,
}

/// Shared state behind every Tauri command in this crate.
pub struct AppState {
    /// The local store: macros, profiles, app settings, the firmware archive and the event log.
    pub store: Store,
    /// The root directory `store` was opened with, so `commands::firmware` can locate a firmware
    /// package's file by [`hyperpace_store::FirmwareRecord::path`], which is relative to it; the
    /// store's own API exposes no path accessor once it is open.
    store_root: std::path::PathBuf,
    app: AppHandle,
    device: Mutex<Option<ActiveDevice>>,
    /// Window label -> the channel streaming that window's device events.
    channels: Mutex<HashMap<String, Channel<DeviceEventPayload>>>,
    /// Bumped on every `connect` and `disconnect`; a tracker thread stops touching `device` once
    /// it observes this no longer matches the id it was spawned with, so a connection superseded
    /// mid-flight can never clobber the one that replaced it.
    connection_id: AtomicU64,
    /// Cached [`LOW_BATTERY_THRESHOLD_KEY`] app setting. Plain atomic, not the store: read on
    /// every battery event and written rarely by `app_settings`, never contended enough to need a
    /// lock (concurrency doctrine: process-local, infrequently-written state is plain std).
    low_battery_threshold: AtomicU8,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl AppState {
    /// Build the app state, opened at `store_root`. `app` is used only to reach the tray and the
    /// notification plugin from the background tracker thread a `connect` call starts.
    #[must_use]
    pub fn new(store: Store, store_root: std::path::PathBuf, app: AppHandle) -> Self {
        let threshold = read_low_battery_threshold(&store);
        Self {
            store,
            store_root,
            app,
            device: Mutex::new(None),
            channels: Mutex::new(HashMap::new()),
            connection_id: AtomicU64::new(0),
            low_battery_threshold: AtomicU8::new(threshold),
        }
    }

    /// The root directory the store was opened with.
    pub(crate) fn store_root(&self) -> &std::path::Path {
        &self.store_root
    }

    /// Connect to the simulator, or to the operator's real device when `real_device` is set,
    /// requesting `access`. Replaces any existing connection.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Device`] when the transport could not be opened or the owner thread
    /// could not be started.
    pub fn connect(
        &self,
        real_device: bool,
        access: AccessDto,
    ) -> Result<DeviceStateDto, AppError> {
        let backend = if real_device {
            DeviceBackendDto::RealDevice
        } else {
            DeviceBackendDto::Simulator
        };
        let access: Access = access.into();
        let handle = open_connection(backend, access)?;

        let id = self.connection_id.fetch_add(1, Ordering::SeqCst) + 1;
        spawn_tracker(self.app.clone(), id, &handle)?;
        *lock(&self.device) = Some(ActiveDevice {
            handle,
            backend,
            access,
            tracked: TrackedState::default(),
        });
        Ok(self.snapshot())
    }

    /// Drop the current connection, if any. Its owner thread exits once every clone of its
    /// handle is dropped, and its tracker thread exits on the next event (or immediately, once it
    /// notices `connection_id` no longer matches).
    pub fn disconnect(&self) -> DeviceStateDto {
        self.connection_id.fetch_add(1, Ordering::SeqCst);
        *lock(&self.device) = None;
        self.snapshot()
    }

    /// The current connection state.
    #[must_use]
    pub fn snapshot(&self) -> DeviceStateDto {
        match lock(&self.device).as_ref() {
            Some(active) => DeviceStateDto {
                connected: true,
                backend: Some(active.backend),
                access: Some(active.access.into()),
                identity: active.tracked.identity.map(Into::into),
                battery: active.tracked.battery.map(Into::into),
                online: active.tracked.online,
            },
            None => DeviceStateDto {
                connected: false,
                backend: None,
                access: None,
                identity: None,
                battery: None,
                online: false,
            },
        }
    }

    /// Register `channel` to receive every device event from now on for `window_label`,
    /// replacing any channel already registered for that window (a window recreated after being
    /// destroyed re-subscribes under the same label).
    pub fn subscribe(&self, window_label: String, channel: Channel<DeviceEventPayload>) {
        lock(&self.channels).insert(window_label, channel);
    }

    /// Stop streaming to `window_label`, if it was subscribed.
    pub fn unsubscribe(&self, window_label: &str) {
        lock(&self.channels).remove(window_label);
    }

    /// A clone of the current device handle.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::NotConnected`] when no device is connected.
    pub(crate) fn handle(&self) -> Result<DeviceHandle, AppError> {
        lock(&self.device)
            .as_ref()
            .map(|active| active.handle.clone())
            .ok_or(AppError::NotConnected)
    }

    /// The current device handle, its resolved model table, its declared identity and access, and
    /// the last known battery percent.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::NotConnected`] when no device is connected or its identity has not
    /// resolved yet, and [`AppError::UnknownModel`] when the reported `cid`/`mid` matches no
    /// [`ModelTable`].
    pub(crate) fn connected_model(
        &self,
    ) -> Result<
        (
            DeviceHandle,
            &'static ModelTable,
            DeviceIdentity,
            Access,
            Option<u8>,
        ),
        AppError,
    > {
        let guard = lock(&self.device);
        let active = guard.as_ref().ok_or(AppError::NotConnected)?;
        let identity = active.tracked.identity.ok_or(AppError::NotConnected)?;
        let table = table_for(identity.cid, identity.mid).ok_or(AppError::UnknownModel {
            cid: identity.cid,
            mid: identity.mid,
        })?;
        Ok((
            active.handle.clone(),
            table,
            identity,
            active.access,
            active.tracked.battery.map(|battery| battery.percent),
        ))
    }

    /// The current connection's backend, for a caller (firmware install) that must refuse to
    /// proceed against the simulator.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::NotConnected`] when no device is connected.
    pub(crate) fn backend(&self) -> Result<DeviceBackendDto, AppError> {
        lock(&self.device)
            .as_ref()
            .map(|active| active.backend)
            .ok_or(AppError::NotConnected)
    }

    /// The cached low-battery threshold percent.
    #[must_use]
    pub fn low_battery_threshold(&self) -> u8 {
        self.low_battery_threshold.load(Ordering::Relaxed)
    }

    /// Update the cached low-battery threshold; called by `app_settings` after it persists the
    /// new value.
    pub fn set_low_battery_threshold(&self, percent: u8) {
        self.low_battery_threshold.store(percent, Ordering::Relaxed);
    }

    /// A clone of every channel currently subscribed, for the tracker thread to fan an event out
    /// to without holding the channel map locked while it calls into each one.
    fn subscribed_channels(&self) -> Vec<(String, Channel<DeviceEventPayload>)> {
        lock(&self.channels)
            .iter()
            .map(|(label, channel)| (label.clone(), channel.clone()))
            .collect()
    }
}

/// Read [`LOW_BATTERY_THRESHOLD_KEY`] from `store`, falling back to
/// [`DEFAULT_LOW_BATTERY_THRESHOLD`] when it is absent or malformed. Never fails: a store error
/// here should not block the app from starting, and a missing setting is an expected first run,
/// not a problem to surface.
fn read_low_battery_threshold(store: &Store) -> u8 {
    store
        .settings()
        .list()
        .unwrap_or_default()
        .into_iter()
        .find(|(_, setting)| setting.key == LOW_BATTERY_THRESHOLD_KEY)
        .and_then(|(_, setting)| setting.value.as_u64())
        .and_then(|value| u8::try_from(value).ok())
        .unwrap_or(DEFAULT_LOW_BATTERY_THRESHOLD)
}

/// Open a transport for `backend` and spawn its owner thread.
fn open_connection(backend: DeviceBackendDto, access: Access) -> Result<DeviceHandle, AppError> {
    let transport: Box<dyn Transport> = match backend {
        DeviceBackendDto::Simulator => {
            let (transport, _controller) = SimTransport::new(SIM_CID, SIM_MID, SIM_LINK_BYTE);
            Box::new(transport)
        }
        DeviceBackendDto::RealDevice => Box::new(HidTransport::open_first()?),
    };
    Ok(hyperpace_device::spawn(transport, access)?)
}

/// Start the background thread that drains `handle`'s events, updates the tracked state, fans
/// each event out to every subscribed window, and drives the tray.
///
/// # Errors
///
/// Returns [`AppError::Io`] when the OS could not start the thread.
fn spawn_tracker(app: AppHandle, id: u64, handle: &DeviceHandle) -> Result<(), AppError> {
    let events = handle.events();
    thread::Builder::new()
        .name("hyperpace-app-device-tracker".to_owned())
        .spawn(move || {
            while let Ok(event) = events.recv() {
                let Some(state) = app.try_state::<AppState>() else {
                    return; // state not managed yet/anymore; nothing to update
                };
                if state.connection_id.load(Ordering::SeqCst) != id {
                    return; // superseded by a later connect/disconnect
                }
                let threshold = state.low_battery_threshold();
                let edge = {
                    let mut guard = lock(&state.device);
                    let Some(active) = guard.as_mut() else {
                        return; // disconnected out from under this tracker
                    };
                    active.tracked.apply(event, threshold)
                };

                let payload = DeviceEventPayload::from(event);
                for (label, channel) in state.subscribed_channels() {
                    if channel.send(payload).is_err() {
                        // The window behind this channel is gone (destroyed to the tray, or
                        // closing); drop it so a future subscribe for the same label replaces it
                        // cleanly instead of piling up dead entries.
                        state.unsubscribe(&label);
                    }
                }

                tray::on_state_changed(&app, state.snapshot());
                match edge {
                    LowBatteryEdge::Entered(percent) => tray::notify_low_battery(&app, percent),
                    LowBatteryEdge::None | LowBatteryEdge::Cleared => {}
                }
            }
        })
        .map_err(AppError::from)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyperpace_protocol::LinkType;

    fn identity() -> DeviceIdentity {
        DeviceIdentity {
            cid: 62,
            mid: 1,
            link: LinkType::Wireless2k,
        }
    }

    fn battery(percent: u8, charging: bool) -> Battery {
        Battery {
            percent,
            charging,
            millivolts: 3900,
        }
    }

    #[test]
    fn a_connected_event_records_the_identity_and_goes_online() {
        let mut tracked = TrackedState::default();
        let edge = tracked.apply(DeviceEvent::Connected(identity()), 15);
        assert_eq!(edge, LowBatteryEdge::None);
        assert_eq!(tracked.identity, Some(identity()));
        assert!(tracked.online);
    }

    #[test]
    fn crossing_the_threshold_fires_once_then_stays_quiet() {
        let mut tracked = TrackedState::default();
        assert_eq!(
            tracked.apply(DeviceEvent::Battery(battery(15, false)), 15),
            LowBatteryEdge::Entered(15)
        );
        assert_eq!(
            tracked.apply(DeviceEvent::Battery(battery(14, false)), 15),
            LowBatteryEdge::None,
            "already notified this spell; must not fire again"
        );
    }

    #[test]
    fn recovering_above_the_threshold_clears_the_flag_and_can_fire_again() {
        let mut tracked = TrackedState::default();
        tracked.apply(DeviceEvent::Battery(battery(10, false)), 15);
        assert_eq!(
            tracked.apply(DeviceEvent::Battery(battery(50, false)), 15),
            LowBatteryEdge::Cleared
        );
        assert_eq!(
            tracked.apply(DeviceEvent::Battery(battery(5, false)), 15),
            LowBatteryEdge::Entered(5),
            "a fresh drop after recovering must notify again"
        );
    }

    #[test]
    fn charging_never_counts_as_low_even_below_the_threshold() {
        let mut tracked = TrackedState::default();
        assert_eq!(
            tracked.apply(DeviceEvent::Battery(battery(5, true)), 15),
            LowBatteryEdge::None
        );
        assert!(!tracked.low_battery_notified);
    }

    #[test]
    fn disconnect_and_offline_clear_online_without_touching_battery() {
        let mut tracked = TrackedState::default();
        tracked.apply(DeviceEvent::Battery(battery(80, false)), 15);
        tracked.apply(DeviceEvent::Connected(identity()), 15);
        tracked.apply(DeviceEvent::Offline, 15);
        assert!(!tracked.online);
        assert_eq!(tracked.battery, Some(battery(80, false)));
    }
}
