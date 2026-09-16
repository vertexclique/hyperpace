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
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hyperpace_device::{Access, DeviceEvent, DeviceHandle, HidTransport, SimTransport};
use hyperpace_protocol::{Battery, DeviceIdentity, ModelTable, Transport, table_for};
use hyperpace_store::{EventRecord, Store, StoreError};
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

/// Hard cap on the device event log ([`EventRecord`]), enforced by [`AppState::record_event`]
/// every time it appends one (`commands::data::list_events` reads this same log; see its own
/// module docs for the Data screen that browses it). Bounds the resource that actually fails
/// (the store's on-disk size and every `list()` this crate does against the collection), not a
/// proxy for it: an unbounded log is exactly the "counting is not bounding" mistake the doctrine
/// warns about.
const EVENT_LOG_CAP: usize = 2000;

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
    /// Whether the transport is the mouse's own cable rather than the receiver. Known from the
    /// moment the transport opens, before the device has said anything.
    wired: bool,
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
    /// Whether a cable is attached that this app is not allowed to open, as last checked by the
    /// watcher. Plain atomic for the same reason the threshold below is: written rarely by one
    /// thread, read by every snapshot.
    cable_blocked: AtomicBool,
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
            cable_blocked: AtomicBool::new(false),
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
        let (handle, wired) = open_connection(backend, access)?;
        tracing::info!(
            link = if wired { "cable" } else { "2.4 GHz receiver" },
            "opened the device"
        );

        let id = self.connection_id.fetch_add(1, Ordering::SeqCst) + 1;
        spawn_tracker(self.app.clone(), id, &handle)?;
        *lock(&self.device) = Some(ActiveDevice {
            handle,
            backend,
            access,
            wired,
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
                cable_blocked: self.cable_blocked.load(Ordering::Relaxed),
                connected: true,
                backend: Some(active.backend),
                access: Some(active.access.into()),
                identity: active.tracked.identity.map(Into::into),
                battery: active.tracked.battery.map(Into::into),
                online: active.tracked.online,
            },
            None => DeviceStateDto {
                cable_blocked: self.cable_blocked.load(Ordering::Relaxed),
                connected: false,
                backend: None,
                access: None,
                identity: None,
                battery: None,
                online: false,
            },
        }
    }

    /// Append one line to the device history the store keeps, and that the Data screen reads
    /// back.
    ///
    /// Only state transitions are recorded, never a poll: the battery is read every 5 seconds, and
    /// a row per poll would bury the events worth keeping under thousands of identical ones and
    /// grow the store without bound. `message` is shown to a person as written, so it says what
    /// happened in one plain sentence.
    ///
    /// A store failure here is logged and dropped rather than propagated: losing a history line
    /// must never take down the connection that produced it.
    pub(crate) fn record_event(&self, kind: &str, message: String) {
        let at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| i64::try_from(since.as_secs()).unwrap_or(0));
        let record = EventRecord {
            at,
            kind: kind.to_owned(),
            message,
            detail: None,
        };
        if let Err(error) = self.store.events().create(&record) {
            tracing::warn!(%error, kind, "could not record a device event in the history");
            return;
        }
        if let Err(error) = prune_event_log(&self.store, EVENT_LOG_CAP) {
            tracing::warn!(%error, "could not prune the device event log");
        }
    }

    /// Record what the cable connection can do, so a mouse that is plugged in but unreachable is
    /// reported as exactly that instead of as no mouse at all.
    pub(crate) fn set_cable_blocked(&self, blocked: bool) {
        let previous = self.cable_blocked.swap(blocked, Ordering::Relaxed);
        if previous != blocked {
            tray::on_state_changed(&self.app, self.snapshot());
            self.announce(DeviceEventPayload::from(DeviceEvent::Offline));
        }
    }

    /// Whether a device is connected right now, for the auto-connect watcher deciding whether
    /// there is anything to do.
    #[must_use]
    pub fn is_connected(&self) -> bool {
        lock(&self.device).is_some()
    }

    /// Drop the current connection without bumping [`Self::connection_id`], used by the tracker
    /// thread when its own device reported [`DeviceEvent::Disconnected`]: the connection is over,
    /// so [`Self::snapshot`] must stop claiming otherwise, but the tracker is still the live one
    /// for that id and has an event left to announce.
    pub(crate) fn clear_connection(&self) {
        *lock(&self.device) = None;
    }

    /// Fan `payload` out to every subscribed window and refresh the tray from the current
    /// snapshot.
    ///
    /// The tracker thread calls this for every event a connected device produces, and
    /// [`crate::autoconnect`] calls it for the one transition no connected device can report,
    /// its own unplug. Both go through here so the windows and the tray can never be told
    /// different stories.
    pub(crate) fn announce(&self, payload: DeviceEventPayload) {
        for (label, channel) in self.subscribed_channels() {
            if channel.send(payload).is_err() {
                // The window behind this channel is gone (destroyed to the tray, or closing);
                // drop it so a future subscribe for the same label replaces it cleanly instead
                // of piling up dead entries.
                self.unsubscribe(&label);
            }
        }
        tray::on_state_changed(&self.app, self.snapshot());
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

    /// Whether the current connection is to the real device over its cable: `Some(true)` for the
    /// cable, `Some(false)` for the receiver, `None` when there is no real-device connection.
    pub(crate) fn real_connection_wired(&self) -> Option<bool> {
        lock(&self.device)
            .as_ref()
            .filter(|active| active.backend == DeviceBackendDto::RealDevice)
            .map(|active| active.wired)
    }

    /// The identity the current connection resolved, if it has one yet.
    pub(crate) fn identity(&self) -> Option<DeviceIdentity> {
        lock(&self.device)
            .as_ref()
            .and_then(|active| active.tracked.identity)
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

/// Prune the event log back down to `cap` when it holds more, deleting the oldest entries first.
/// A free function taking a bare [`Store`], not an [`AppState`] method, so it is directly
/// testable against a temporary store the way [`read_low_battery_threshold`] above is; returns
/// how many entries were actually removed.
///
/// # Errors
///
/// Returns [`StoreError`] when the embedded store cannot be reached. [`AppState::record_event`]
/// logs and drops that rather than let a failed prune take down the write that triggered it.
// vertexia: the embedded query engine has no ORDER BY/LIMIT on a record field
// (`hyperpace_store::collection`'s own doc comment: every collection is one opaque `data: JSON`
// document), so finding the oldest entries means listing the whole collection every time an
// event is appended past the cap. The real ceiling this imposes is one list of at most `cap + 1`
// small `EventRecord` documents (a few hundred KB at `EVENT_LOG_CAP`), not truly unbounded, and
// event appends are rare: only device state transitions (`history_line`), never the 5 second
// battery poll. Upgrade path: give `EventRecord` an indexed `at` field in the schema so the store
// can push the ordering to the query engine instead of sorting here.
fn prune_event_log(store: &Store, cap: usize) -> Result<usize, StoreError> {
    let mut events = store.events().list()?;
    if events.len() <= cap {
        return Ok(0);
    }
    events.sort_by_key(|(_, record)| record.at);
    let excess = events.len() - cap;
    let mut pruned = 0;
    for (id, _) in events.into_iter().take(excess) {
        if store.events().delete(&id)? {
            pruned += 1;
        }
    }
    Ok(pruned)
}

/// Open a transport for `backend` and spawn its owner thread, returning the handle and whether
/// the transport is the mouse's own cable.
fn open_connection(
    backend: DeviceBackendDto,
    access: Access,
) -> Result<(DeviceHandle, bool), AppError> {
    let (transport, wired): (Box<dyn Transport>, bool) = match backend {
        DeviceBackendDto::Simulator => {
            // Development mode with a real device's settings dump reports the production model's
            // identity (cid 102, mid 1, 2.4 GHz at 8000 Hz), since that is the table the dump
            // decodes against.
            let shadow = crate::dev::simulator_shadow();
            let (cid, mid, link_byte) = if shadow.is_some() {
                (102, 1, 5)
            } else {
                (SIM_CID, SIM_MID, SIM_LINK_BYTE)
            };
            let (transport, controller) = SimTransport::new(cid, mid, link_byte);
            if let Some(shadow) = shadow {
                controller.set_flash(0, &shadow);
            }
            (Box::new(transport), false)
        }
        DeviceBackendDto::RealDevice => {
            let transport = HidTransport::open_first()?;
            let wired = transport.is_wired();
            (Box::new(transport), wired)
        }
    };
    Ok((hyperpace_device::spawn(transport, access)?, wired))
}

/// The history line one device event deserves, or `None` for an event that is a poll rather than a
/// transition.
///
/// [`DeviceEvent::Battery`] is deliberately absent: it arrives every 5 seconds and says nothing
/// new. Crossing the low-battery threshold is recorded by the caller instead, because that is a
/// transition.
fn history_line(event: DeviceEvent) -> Option<(&'static str, String)> {
    match event {
        // The link, not its polling ceiling: "up to 8000 Hz" reads as the rate the mouse is set
        // to, which this event does not know.
        DeviceEvent::Connected(identity) => Some((
            "connected",
            if matches!(
                identity.link,
                hyperpace_protocol::LinkType::Wired1k | hyperpace_protocol::LinkType::Wired8k
            ) {
                "Connected by cable.".to_owned()
            } else {
                "Connected over 2.4 GHz.".to_owned()
            },
        )),
        DeviceEvent::Disconnected => Some(("disconnected", "The mouse disconnected.".to_owned())),
        DeviceEvent::Offline => Some((
            "offline",
            "The mouse stopped answering; it is asleep or out of range.".to_owned(),
        )),
        DeviceEvent::Changed(_) => Some((
            "changed",
            "The mouse reported that its settings changed.".to_owned(),
        )),
        DeviceEvent::Battery(_) => None,
    }
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

                // A device that reported its own disconnection is gone: clear the connection
                // before announcing, so the snapshot the windows and the tray receive with this
                // event does not still claim one.
                if matches!(event, DeviceEvent::Disconnected) {
                    state.clear_connection();
                }
                if let Some((kind, message)) = history_line(event) {
                    state.record_event(kind, message);
                }
                state.announce(DeviceEventPayload::from(event));
                match edge {
                    LowBatteryEdge::Entered(percent) => {
                        state
                            .record_event("battery_low", format!("Battery dropped to {percent}%."));
                        tray::notify_low_battery(&app, percent);
                    }
                    LowBatteryEdge::None | LowBatteryEdge::Cleared => {}
                }
            }
        })
        .map_err(AppError::from)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    // Tests may assert: the doctrine bans panics on production paths, not in tests.
    #![allow(clippy::unwrap_used)]

    use super::*;
    use hyperpace_protocol::{LinkType, StatusChanged};

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
    fn a_battery_poll_never_becomes_a_history_line() {
        // Every 5 seconds forever; recording it would bury the real events and grow the store
        // without bound.
        assert_eq!(history_line(DeviceEvent::Battery(battery(90, false))), None);
    }

    #[test]
    fn every_transition_becomes_one_plain_history_line() {
        for event in [
            DeviceEvent::Connected(identity()),
            DeviceEvent::Disconnected,
            DeviceEvent::Offline,
            DeviceEvent::Changed(StatusChanged {
                dpi: true,
                polling: false,
                profile: false,
                dpi_indicator: false,
                lighting: false,
                battery: false,
            }),
        ] {
            let line = history_line(event);
            assert!(line.is_some(), "{event:?} should be recorded");
            let (kind, message) = line.unwrap();
            assert!(!kind.is_empty());
            assert!(
                message.ends_with('.') && message.chars().next().is_some_and(char::is_uppercase),
                "a history line is shown to a person as written: {message:?}"
            );
        }
    }

    #[test]
    fn a_connect_line_names_the_link_the_device_reported() {
        let (kind, message) = history_line(DeviceEvent::Connected(identity())).unwrap();
        assert_eq!(kind, "connected");
        assert_eq!(message, "Connected over 2.4 GHz.");
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

    fn open_temp_store() -> (Store, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path()).unwrap();
        (store, dir)
    }

    fn seed_event(store: &Store, at: i64) {
        store
            .events()
            .create(&EventRecord {
                at,
                kind: "connected".to_owned(),
                message: format!("event at {at}"),
                detail: None,
            })
            .unwrap();
    }

    #[test]
    fn pruning_under_the_cap_does_nothing() {
        let (store, _dir) = open_temp_store();
        for at in 0..5 {
            seed_event(&store, at);
        }
        assert_eq!(prune_event_log(&store, 10).unwrap(), 0);
        assert_eq!(store.events().list().unwrap().len(), 5);
    }

    #[test]
    fn pruning_over_the_cap_deletes_the_oldest_first() {
        let (store, _dir) = open_temp_store();
        for at in 0..10 {
            seed_event(&store, at);
        }
        let pruned = prune_event_log(&store, 4).unwrap();
        assert_eq!(pruned, 6);
        let remaining = store.events().list().unwrap();
        assert_eq!(remaining.len(), 4);
        let mut ats: Vec<i64> = remaining.into_iter().map(|(_, record)| record.at).collect();
        ats.sort_unstable();
        // The four most recent (highest `at`) survive; every older one is gone.
        assert_eq!(ats, vec![6, 7, 8, 9]);
    }

    #[test]
    fn record_event_prunes_the_log_once_it_grows_past_the_cap() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path()).unwrap();
        for at in 0..3 {
            seed_event(&store, at);
        }
        // record_event itself needs an AppHandle this unit test has none of; exercising the same
        // create-then-prune sequence directly against a tiny cap proves the mechanism without
        // needing a running Tauri app.
        store
            .events()
            .create(&EventRecord {
                at: 3,
                kind: "connected".to_owned(),
                message: "newest".to_owned(),
                detail: None,
            })
            .unwrap();
        prune_event_log(&store, 2).unwrap();
        let remaining = store.events().list().unwrap();
        assert_eq!(remaining.len(), 2);
        assert!(
            remaining
                .iter()
                .any(|(_, record)| record.message == "newest")
        );
    }
}
