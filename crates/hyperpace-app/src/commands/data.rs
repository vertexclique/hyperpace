//! Data Management commands: browse and manage every local store collection end to end (the
//! overview strip, the device event log, profile snapshots, the firmware archive, app settings,
//! and a whole-store backup), backing the Data screen.
//!
//! Every command here is thin, mirroring every other module in [`crate::commands`]: it pulls
//! what it needs off [`AppState`], calls one small, independently testable function, and converts
//! the result back to `Result<T, String>`. Profile snapshot capture and restore reuse
//! `crate::commands::device::export_config_bytes` and `crate::commands::device::import_config_bytes`
//! rather than a second copy of the `.bin` encode/decode path `export_config`/`import_config`
//! already own.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use hyperpace_store::{
    AppSetting, Collection, EventRecord, FirmwareRecord, MacroRecord, ProfileSnapshot, Record,
    Store,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::blocking::blocking;
use crate::commands::device::{export_config_bytes, import_config_bytes, query_profile};
use crate::commands::firmware::unix_now;
use crate::dto::{
    AppSettingsResponse, DataOverviewDto, EventRecordDto, ImportStoreResponse, ListEventsResponse,
    ProfileDto, ProfileSnapshotDto,
};
use crate::error::{AppError, to_command_result};
use crate::state::{AppState, DEFAULT_LOW_BATTERY_THRESHOLD, LOW_BATTERY_THRESHOLD_KEY};

/// Newest-first event page size [`list_events`] allows in one call, regardless of what a caller
/// asks for.
const EVENT_PAGE_LIMIT: u32 = 200;

/// States plainly, inside the backup file itself, that firmware entries are index metadata only
/// (the honesty fence: a backup that silently dropped package bytes without saying so would read
/// as more complete than it is).
const FIRMWARE_NOTE: &str =
    "Firmware entries are index metadata only; package files are not included in this backup.";

// ---- Overview ----------------------------------------------------------------------------

/// Record counts for all five collections, the store's root directory, and its on-disk size.
///
/// # Errors
///
/// Returns an error message when the store cannot be reached or its root directory cannot be
/// walked.
#[tauri::command]
pub async fn data_overview(app: AppHandle) -> Result<DataOverviewDto, String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            build_overview(&state.store, state.store_root())
        })
        .await,
    )
}

fn build_overview(store: &Store, store_root: &Path) -> Result<DataOverviewDto, AppError> {
    Ok(DataOverviewDto {
        macros: count(&store.macros())?,
        profiles: count(&store.profiles())?,
        settings: count(&store.settings())?,
        firmware: count(&store.firmware())?,
        events: count(&store.events())?,
        store_root: store_root.display().to_string(),
        store_size_bytes: directory_size(store_root)?,
    })
}

fn count<T: Record>(collection: &Collection<T>) -> Result<u64, AppError> {
    Ok(collection.list()?.len() as u64)
}

/// Total bytes every file under `root` occupies. Walked with an explicit stack rather than
/// recursion, bounded by the store's own shallow layout (the embedded node's own files plus one
/// `firmware/` directory of packages). Symlinks are never followed (`DirEntry::metadata` does not
/// traverse them), so a symlink loop cannot make this loop forever.
///
/// # Errors
///
/// Returns the first I/O error a directory or file read reports, rather than silently reporting a
/// partial (and therefore fabricated) total.
fn directory_size(root: &Path) -> Result<u64, AppError> {
    let mut total = 0u64;
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            if metadata.is_dir() {
                stack.push(entry.path());
            } else {
                total += metadata.len();
            }
        }
    }
    Ok(total)
}

// ---- Event log ----------------------------------------------------------------------------

/// One page of the device and app event log, newest first. The log itself is bounded to at most
/// a couple thousand entries (`state::AppState::record_event`'s own retention), so this is always
/// a full list of a small collection, never an unbounded scan.
///
/// # Errors
///
/// Returns an error message when the store cannot be reached.
#[tauri::command]
pub async fn list_events(
    app: AppHandle,
    offset: u32,
    limit: u32,
) -> Result<ListEventsResponse, String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            page_events(&state.store, offset, limit)
        })
        .await,
    )
}

fn page_events(store: &Store, offset: u32, limit: u32) -> Result<ListEventsResponse, AppError> {
    let mut events = store.events().list()?;
    // Newest first; the document id breaks a tie between entries sharing a timestamp, so a page
    // stays stable across calls instead of depending on the store's own iteration order.
    events.sort_by(|(id_a, a), (id_b, b)| b.at.cmp(&a.at).then_with(|| id_b.cmp(id_a)));
    let total = events.len() as u64;
    let limit = limit.min(EVENT_PAGE_LIMIT) as usize;
    let page = events
        .into_iter()
        .skip(offset as usize)
        .take(limit)
        .map(|(id, record)| EventRecordDto {
            id,
            at: record.at,
            kind: record.kind,
            message: record.message,
        })
        .collect();
    Ok(ListEventsResponse {
        events: page,
        total,
    })
}

/// Delete every entry in the device and app event log.
///
/// # Errors
///
/// Returns an error message when the store cannot be reached.
#[tauri::command]
pub async fn clear_events(app: AppHandle) -> Result<u64, String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            clear_all_events(&state.store)
        })
        .await,
    )
}

fn clear_all_events(store: &Store) -> Result<u64, AppError> {
    let mut cleared = 0u64;
    for (id, _) in store.events().list()? {
        if store.events().delete(&id)? {
            cleared += 1;
        }
    }
    Ok(cleared)
}

// ---- Profile snapshots ----------------------------------------------------------------------

/// Every saved profile snapshot.
///
/// # Errors
///
/// Returns an error message when the store cannot be reached.
#[tauri::command]
pub async fn list_profile_snapshots(app: AppHandle) -> Result<Vec<ProfileSnapshotDto>, String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            Ok(state
                .store
                .profiles()
                .list()?
                .into_iter()
                .map(|(id, record)| ProfileSnapshotDto::from_record(id, record))
                .collect())
        })
        .await,
    )
}

/// Capture the connected mouse's current settings, encoded the same way [`export_config`] encodes
/// them, and save them as a new named snapshot.
///
/// [`export_config`]: crate::commands::device::export_config
///
/// # Errors
///
/// Returns an error message when no device is connected, its model is unrecognized, or the
/// device did not answer the read in time.
#[tauri::command]
pub async fn save_profile_snapshot(
    app: AppHandle,
    name: String,
) -> Result<ProfileSnapshotDto, String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            let bytes = export_config_bytes(&state)?;
            let handle = state.handle()?;
            let index = match query_profile(&handle)? {
                ProfileDto::Active { index } => index,
                ProfileDto::Unsupported => 0,
            };
            let record = ProfileSnapshot {
                name,
                index,
                captured_at: unix_now(),
                settings: encode_config_bytes(&bytes),
            };
            let id = state.store.profiles().create(&record)?;
            Ok(ProfileSnapshotDto::from_record(id, record))
        })
        .await,
    )
}

/// Write snapshot `id`'s saved configuration back to the connected device, the same path
/// [`import_config`] writes through.
///
/// [`import_config`]: crate::commands::device::import_config
///
/// # Errors
///
/// Returns an error message when `id` names no snapshot, its saved configuration is corrupted,
/// no device is connected, or the write was not acknowledged in time.
#[tauri::command]
pub async fn restore_profile_snapshot(app: AppHandle, id: String) -> Result<(), String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            let bytes = snapshot_config_bytes(&state.store, &id)?;
            import_config_bytes(&state, &bytes)
        })
        .await,
    )
}

/// Look up snapshot `id` and decode its stored configuration bytes.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] when `id` names no snapshot, and [`AppError::InvalidData`] when
/// its stored bytes are malformed.
fn snapshot_config_bytes(store: &Store, id: &str) -> Result<Vec<u8>, AppError> {
    let record = store
        .profiles()
        .get(id)?
        .ok_or_else(|| AppError::NotFound {
            kind: "profile snapshot",
            id: id.to_owned(),
        })?;
    decode_config_bytes(&record.settings)
}

/// Rename a saved profile snapshot.
///
/// # Errors
///
/// Returns an error message when `id` names no snapshot.
#[tauri::command]
pub async fn rename_profile_snapshot(
    app: AppHandle,
    id: String,
    name: String,
) -> Result<ProfileSnapshotDto, String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            rename_snapshot(&state.store, &id, name)
        })
        .await,
    )
}

fn rename_snapshot(store: &Store, id: &str, name: String) -> Result<ProfileSnapshotDto, AppError> {
    let mut record = store
        .profiles()
        .get(id)?
        .ok_or_else(|| AppError::NotFound {
            kind: "profile snapshot",
            id: id.to_owned(),
        })?;
    record.name = name;
    store.profiles().update(id, &record)?;
    Ok(ProfileSnapshotDto::from_record(id.to_owned(), record))
}

/// Delete a saved profile snapshot. Returns `true` when a snapshot was deleted, `false` when
/// `id` already named none.
///
/// # Errors
///
/// Returns an error message when the store cannot be reached.
#[tauri::command]
pub async fn delete_profile_snapshot(app: AppHandle, id: String) -> Result<bool, String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            Ok(state.store.profiles().delete(&id)?)
        })
        .await,
    )
}

/// Encode `.bin` config file bytes the same way they already cross the Tauri IPC boundary
/// (`export_config`/`import_config`'s own `Vec<u8>` becomes a JSON `number[]` on the wire): a
/// plain JSON array of byte values, so [`ProfileSnapshot::settings`] needs no encoding of its
/// own.
fn encode_config_bytes(bytes: &[u8]) -> serde_json::Value {
    serde_json::Value::Array(
        bytes
            .iter()
            .map(|&byte| serde_json::Value::from(byte))
            .collect(),
    )
}

/// The inverse of [`encode_config_bytes`].
///
/// # Errors
///
/// Returns [`AppError::InvalidData`] when `value` is not a JSON array of byte values, meaning
/// this document was not written by [`encode_config_bytes`] (a corrupted or hand-edited store).
fn decode_config_bytes(value: &serde_json::Value) -> Result<Vec<u8>, AppError> {
    value
        .as_array()
        .and_then(|items| {
            items
                .iter()
                .map(|item| item.as_u64().and_then(|n| u8::try_from(n).ok()))
                .collect::<Option<Vec<u8>>>()
        })
        .ok_or_else(|| {
            AppError::InvalidData(
                "this profile snapshot's saved configuration could not be read".to_owned(),
            )
        })
}

// ---- Firmware archive -----------------------------------------------------------------------

/// Remove a firmware archive entry: its index record and its package file under the store root.
/// Returns `false`, not an error, when `id` already named nothing.
///
/// # Errors
///
/// Returns an error message when the record's stored path would resolve outside the store's own
/// directory, or the package file could not be removed.
#[tauri::command]
pub async fn firmware_delete(app: AppHandle, id: String) -> Result<bool, String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            delete_firmware(&state.store, state.store_root(), &id)
        })
        .await,
    )
}

fn delete_firmware(store: &Store, store_root: &Path, id: &str) -> Result<bool, AppError> {
    let Some(record) = store.firmware().get(id)? else {
        return Ok(false);
    };
    let path = resolve_in_store(store_root, &record.path)?;
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(store.firmware().delete(id)?)
}

/// Resolve `relative` against `store_root`, refusing any path that would land outside it.
/// Lexical (component-based), not [`std::fs::canonicalize`]: the target file may already be gone
/// (a delete retried, or a package removed by hand outside the app), and `canonicalize` requires
/// the path to exist.
///
/// # Errors
///
/// Returns [`AppError::InvalidData`] when `relative` is absolute or its `..` components would
/// net below `store_root`.
fn resolve_in_store(store_root: &Path, relative: &str) -> Result<PathBuf, AppError> {
    let invalid = || {
        AppError::InvalidData("that firmware entry's stored file path is not allowed".to_owned())
    };
    let mut depth: i32 = 0;
    for component in Path::new(relative).components() {
        match component {
            std::path::Component::Normal(_) => depth += 1,
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => depth -= 1,
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                return Err(invalid());
            }
        }
        if depth < 0 {
            return Err(invalid());
        }
    }
    Ok(store_root.join(relative))
}

// ---- App settings ----------------------------------------------------------------------------

/// Delete one stored app setting so the app falls back to its own default the next time it reads
/// that key, then return every app setting as it stands afterward.
///
/// # Errors
///
/// Returns an error message when the store cannot be reached.
#[tauri::command]
pub async fn reset_app_setting(app: AppHandle, key: String) -> Result<AppSettingsResponse, String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            let response = reset_setting(&state.store, &key)?;
            // `low_battery_threshold_percent` is also cached on `AppState` (read on every battery
            // event, never from the store directly): resetting it in the store alone would leave
            // the cache holding the value that was just erased. `gpu_dmabuf_workaround` and
            // `firmware_watch_enabled` need no such update; both are re-read from the store fresh
            // every time they matter (`gpu_workaround::apply`'s own short-lived read at startup,
            // `commands::firmware_watch::watch_enabled` on every check).
            if key == LOW_BATTERY_THRESHOLD_KEY {
                state.set_low_battery_threshold(DEFAULT_LOW_BATTERY_THRESHOLD);
            }
            Ok(response)
        })
        .await,
    )
}

/// Not finding `key` is not an error: resetting a setting that was never written, or was already
/// reset, is idempotent.
fn reset_setting(store: &Store, key: &str) -> Result<AppSettingsResponse, AppError> {
    let existing = store.settings().list()?;
    if let Some((doc_id, _)) = existing.iter().find(|(_, setting)| setting.key == key) {
        store.settings().delete(doc_id)?;
    }
    let settings = store
        .settings()
        .list()?
        .into_iter()
        .map(|(_, setting)| (setting.key, setting.value))
        .collect::<BTreeMap<_, _>>();
    Ok(AppSettingsResponse { settings })
}

// ---- Whole-store backup ----------------------------------------------------------------------

/// The whole local store, serialized for [`export_store`]/[`import_store`]. Every collection's
/// raw records, not their IPC DTOs: a backup round trips through this crate alone, so it reuses
/// the shape `hyperpace_store` already persists rather than inventing a second wire format for
/// the same data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoreBackup {
    exported_at: i64,
    firmware_note: String,
    macros: Vec<MacroRecord>,
    profiles: Vec<ProfileSnapshot>,
    settings: Vec<AppSetting>,
    firmware: Vec<FirmwareRecord>,
    events: Vec<EventRecord>,
}

/// Export the whole local store as JSON bytes: every collection, with firmware entries as index
/// metadata only (see `FIRMWARE_NOTE`, carried in the file itself).
///
/// # Errors
///
/// Returns an error message when the store cannot be reached or the backup could not be built.
#[tauri::command]
pub async fn export_store(app: AppHandle) -> Result<Vec<u8>, String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            let backup = build_backup(&state.store)?;
            serde_json::to_vec(&backup).map_err(|error| {
                tracing::warn!(%error, "could not serialize the store backup");
                AppError::InvalidData("could not build the backup file".to_owned())
            })
        })
        .await,
    )
}

fn build_backup(store: &Store) -> Result<StoreBackup, AppError> {
    Ok(StoreBackup {
        exported_at: unix_now(),
        firmware_note: FIRMWARE_NOTE.to_owned(),
        macros: records_only(&store.macros())?,
        profiles: records_only(&store.profiles())?,
        settings: records_only(&store.settings())?,
        firmware: records_only(&store.firmware())?,
        events: records_only(&store.events())?,
    })
}

fn records_only<T: Record>(collection: &Collection<T>) -> Result<Vec<T>, AppError> {
    Ok(collection
        .list()?
        .into_iter()
        .map(|(_, record)| record)
        .collect())
}

/// Import a backup [`export_store`] produced: creates every record not already present (by exact
/// field equality), skipping duplicates, and reports how many were added per collection. Never
/// touches or replaces an existing record; this is a merge, not a restore-from-scratch.
///
/// # Errors
///
/// Returns an error message when `bytes` is not a valid Hyperpace backup file, or the store
/// cannot be reached.
#[tauri::command]
pub async fn import_store(app: AppHandle, bytes: Vec<u8>) -> Result<ImportStoreResponse, String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            let backup = parse_backup(&bytes)?;
            merge_backup(&state.store, backup)
        })
        .await,
    )
}

/// Decode a backup file [`export_store`] produced.
///
/// # Errors
///
/// Returns [`AppError::InvalidData`] when `bytes` is not a valid Hyperpace backup.
fn parse_backup(bytes: &[u8]) -> Result<StoreBackup, AppError> {
    serde_json::from_slice(bytes).map_err(|error| {
        tracing::warn!(%error, "could not parse an imported store backup");
        AppError::InvalidData("that file is not a valid Hyperpace backup".to_owned())
    })
}

fn merge_backup(store: &Store, backup: StoreBackup) -> Result<ImportStoreResponse, AppError> {
    Ok(ImportStoreResponse {
        macros_added: merge_collection(&store.macros(), backup.macros)?,
        profiles_added: merge_collection(&store.profiles(), backup.profiles)?,
        settings_added: merge_collection(&store.settings(), backup.settings)?,
        firmware_added: merge_collection(&store.firmware(), backup.firmware)?,
        events_added: merge_collection(&store.events(), backup.events)?,
    })
}

/// Create every record in `incoming` that has no exact match already in `collection` (by full
/// field equality, not store id, which is always freshly assigned on create), and return how many
/// were actually added. The one place `import_store` decides what "exact duplicate" means, so a
/// real import and its tests can never disagree about the rule.
fn merge_collection<T: Record + PartialEq>(
    collection: &Collection<T>,
    incoming: Vec<T>,
) -> Result<u64, AppError> {
    let mut existing: Vec<T> = collection
        .list()?
        .into_iter()
        .map(|(_, record)| record)
        .collect();
    let mut added = 0u64;
    for record in incoming {
        if existing.contains(&record) {
            continue;
        }
        collection.create(&record)?;
        existing.push(record);
        added += 1;
    }
    Ok(added)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn open_temp_store() -> (Store, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path()).unwrap();
        (store, dir)
    }

    fn event(at: i64, kind: &str) -> EventRecord {
        EventRecord {
            at,
            kind: kind.to_owned(),
            message: format!("event at {at}"),
            detail: None,
        }
    }

    // ---- overview ----

    #[test]
    fn build_overview_counts_every_collection_and_walks_the_store_root() {
        let (store, dir) = open_temp_store();
        store
            .macros()
            .create(&MacroRecord {
                name: "m".to_owned(),
                slot: None,
                events: vec![],
            })
            .unwrap();
        store.events().create(&event(1, "connected")).unwrap();
        store.events().create(&event(2, "connected")).unwrap();

        let overview = build_overview(&store, dir.path()).unwrap();
        assert_eq!(overview.macros, 1);
        assert_eq!(overview.events, 2);
        assert_eq!(overview.profiles, 0);
        assert_eq!(overview.store_root, dir.path().display().to_string());
        // The embedded node has already written real files by the time a document has been
        // created; this only proves the walk is not silently reporting zero.
        assert!(overview.store_size_bytes > 0);
    }

    #[test]
    fn directory_size_sums_nested_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.bin"), [0u8; 10]).unwrap();
        let sub = dir.path().join("sub");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(sub.join("b.bin"), [0u8; 25]).unwrap();

        assert_eq!(directory_size(dir.path()).unwrap(), 35);
    }

    // ---- event log ----

    #[test]
    fn page_events_orders_newest_first_and_caps_the_limit() {
        let (store, _dir) = open_temp_store();
        for at in 0..5 {
            store.events().create(&event(at, "connected")).unwrap();
        }

        let page = page_events(&store, 0, 10_000).unwrap();
        assert_eq!(page.total, 5);
        assert_eq!(page.events.len(), 5);
        let ats: Vec<i64> = page.events.iter().map(|e| e.at).collect();
        assert_eq!(ats, vec![4, 3, 2, 1, 0]);
    }

    #[test]
    fn page_events_respects_offset_and_limit() {
        let (store, _dir) = open_temp_store();
        for at in 0..5 {
            store.events().create(&event(at, "connected")).unwrap();
        }
        let page = page_events(&store, 2, 2).unwrap();
        assert_eq!(page.total, 5);
        let ats: Vec<i64> = page.events.iter().map(|e| e.at).collect();
        assert_eq!(ats, vec![2, 1]);
    }

    #[test]
    fn page_events_never_exceeds_the_hard_page_limit() {
        let (store, _dir) = open_temp_store();
        for at in 0..3 {
            store.events().create(&event(at, "connected")).unwrap();
        }
        let page = page_events(&store, 0, u32::MAX).unwrap();
        assert!(page.events.len() <= EVENT_PAGE_LIMIT as usize);
    }

    #[test]
    fn clear_all_events_deletes_everything_and_reports_the_count() {
        let (store, _dir) = open_temp_store();
        for at in 0..4 {
            store.events().create(&event(at, "connected")).unwrap();
        }
        assert_eq!(clear_all_events(&store).unwrap(), 4);
        assert!(store.events().list().unwrap().is_empty());
        assert_eq!(clear_all_events(&store).unwrap(), 0);
    }

    // ---- profile snapshots ----

    fn snapshot(name: &str, bytes: &[u8]) -> ProfileSnapshot {
        ProfileSnapshot {
            name: name.to_owned(),
            index: 0,
            captured_at: 1_726_000_000,
            settings: encode_config_bytes(bytes),
        }
    }

    #[test]
    fn config_bytes_round_trip_through_the_json_encoding() {
        let bytes = vec![0u8, 1, 2, 255, 128];
        let encoded = encode_config_bytes(&bytes);
        assert_eq!(decode_config_bytes(&encoded).unwrap(), bytes);
    }

    #[test]
    fn decode_config_bytes_refuses_a_value_that_is_not_a_byte_array() {
        let error = decode_config_bytes(&serde_json::json!({"not": "bytes"})).unwrap_err();
        assert!(matches!(error, AppError::InvalidData(_)));

        let error = decode_config_bytes(&serde_json::json!([1, 2, 999])).unwrap_err();
        assert!(matches!(error, AppError::InvalidData(_)));
    }

    #[test]
    fn snapshot_config_bytes_reports_not_found_honestly() {
        let (store, _dir) = open_temp_store();
        let error = snapshot_config_bytes(&store, "bae-missing").unwrap_err();
        assert!(matches!(error, AppError::NotFound { .. }));
    }

    #[test]
    fn snapshot_config_bytes_decodes_a_saved_snapshot() {
        let (store, _dir) = open_temp_store();
        let bytes = vec![9u8, 8, 7];
        let id = store.profiles().create(&snapshot("a", &bytes)).unwrap();
        assert_eq!(snapshot_config_bytes(&store, &id).unwrap(), bytes);
    }

    #[test]
    fn rename_snapshot_updates_the_name_and_keeps_everything_else() {
        let (store, _dir) = open_temp_store();
        let id = store.profiles().create(&snapshot("old", &[1])).unwrap();
        let dto = rename_snapshot(&store, &id, "new".to_owned()).unwrap();
        assert_eq!(dto.name, "new");
        assert_eq!(store.profiles().get(&id).unwrap().unwrap().name, "new");
    }

    #[test]
    fn rename_snapshot_reports_not_found_honestly() {
        let (store, _dir) = open_temp_store();
        let error = rename_snapshot(&store, "bae-missing", "x".to_owned()).unwrap_err();
        assert!(matches!(error, AppError::NotFound { .. }));
    }

    // ---- firmware archive ----

    fn firmware(path: &str) -> FirmwareRecord {
        FirmwareRecord {
            product: "Hyperpace mouse".to_owned(),
            version: "1.0a".to_owned(),
            cid: 102,
            mid: 1,
            path: path.to_owned(),
            sha256: "0".repeat(64),
            imported_at: 1_726_000_000,
        }
    }

    #[test]
    fn resolve_in_store_accepts_a_plain_relative_path() {
        let root = Path::new("/store");
        assert_eq!(
            resolve_in_store(root, "firmware/x.bin").unwrap(),
            root.join("firmware/x.bin")
        );
    }

    #[test]
    fn resolve_in_store_refuses_a_path_that_escapes_the_root() {
        let root = Path::new("/store");
        assert!(resolve_in_store(root, "../outside.bin").is_err());
        assert!(resolve_in_store(root, "firmware/../../outside.bin").is_err());
    }

    #[test]
    fn resolve_in_store_refuses_an_absolute_path() {
        let root = Path::new("/store");
        assert!(resolve_in_store(root, "/etc/passwd").is_err());
    }

    #[test]
    fn delete_firmware_removes_the_record_and_its_file() {
        let (store, dir) = open_temp_store();
        let full_path = dir.path().join("firmware/pkg.bin");
        std::fs::create_dir_all(full_path.parent().unwrap()).unwrap();
        std::fs::write(&full_path, b"package bytes").unwrap();
        let id = store
            .firmware()
            .create(&firmware("firmware/pkg.bin"))
            .unwrap();

        assert!(delete_firmware(&store, dir.path(), &id).unwrap());
        assert!(store.firmware().get(&id).unwrap().is_none());
        assert!(!full_path.exists());
    }

    #[test]
    fn delete_firmware_is_idempotent_for_a_missing_id() {
        let (store, dir) = open_temp_store();
        assert!(!delete_firmware(&store, dir.path(), "bae-missing").unwrap());
    }

    #[test]
    fn delete_firmware_still_removes_the_record_when_the_file_is_already_gone() {
        let (store, dir) = open_temp_store();
        let id = store
            .firmware()
            .create(&firmware("firmware/already-gone.bin"))
            .unwrap();
        assert!(delete_firmware(&store, dir.path(), &id).unwrap());
        assert!(store.firmware().get(&id).unwrap().is_none());
    }

    #[test]
    fn delete_firmware_refuses_a_record_whose_path_escapes_the_store() {
        let (store, dir) = open_temp_store();
        let id = store
            .firmware()
            .create(&firmware("../outside-the-store.bin"))
            .unwrap();
        let error = delete_firmware(&store, dir.path(), &id).unwrap_err();
        assert!(matches!(error, AppError::InvalidData(_)));
        // Refused before anything was touched: the index record is still there.
        assert!(store.firmware().get(&id).unwrap().is_some());
    }

    // ---- app settings ----

    #[test]
    fn reset_setting_deletes_the_key_and_returns_the_rest() {
        let (store, _dir) = open_temp_store();
        store
            .settings()
            .create(&AppSetting {
                key: "autostart".to_owned(),
                value: serde_json::json!(true),
            })
            .unwrap();
        store
            .settings()
            .create(&AppSetting {
                key: "low_battery_threshold_percent".to_owned(),
                value: serde_json::json!(5),
            })
            .unwrap();

        let response = reset_setting(&store, "low_battery_threshold_percent").unwrap();
        assert!(
            !response
                .settings
                .contains_key("low_battery_threshold_percent")
        );
        assert_eq!(response.settings["autostart"], serde_json::json!(true));
    }

    #[test]
    fn reset_setting_is_idempotent_for_a_key_never_written() {
        let (store, _dir) = open_temp_store();
        let response = reset_setting(&store, "never_written").unwrap();
        assert!(response.settings.is_empty());
    }

    // ---- whole-store backup ----

    #[test]
    fn build_backup_carries_every_collection_and_the_firmware_note() {
        let (store, _dir) = open_temp_store();
        store
            .macros()
            .create(&MacroRecord {
                name: "m".to_owned(),
                slot: None,
                events: vec![],
            })
            .unwrap();
        let backup = build_backup(&store).unwrap();
        assert_eq!(backup.macros.len(), 1);
        assert!(backup.firmware_note.contains("not included"));
    }

    #[test]
    fn export_then_import_round_trips_through_json() {
        let (store, _dir) = open_temp_store();
        store
            .macros()
            .create(&MacroRecord {
                name: "m".to_owned(),
                slot: None,
                events: vec![],
            })
            .unwrap();
        let backup = build_backup(&store).unwrap();
        let bytes = serde_json::to_vec(&backup).unwrap();
        let decoded: StoreBackup = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded.macros.len(), 1);
        assert_eq!(decoded.macros[0].name, "m");
    }

    #[test]
    fn merge_backup_adds_only_records_not_already_present() {
        let (store, _dir) = open_temp_store();
        let already_there = MacroRecord {
            name: "already there".to_owned(),
            slot: None,
            events: vec![],
        };
        store.macros().create(&already_there).unwrap();

        let backup = StoreBackup {
            exported_at: 1,
            firmware_note: FIRMWARE_NOTE.to_owned(),
            macros: vec![
                already_there.clone(),
                MacroRecord {
                    name: "new one".to_owned(),
                    slot: None,
                    events: vec![],
                },
            ],
            profiles: vec![],
            settings: vec![],
            firmware: vec![],
            events: vec![],
        };

        let response = merge_backup(&store, backup).unwrap();
        assert_eq!(response.macros_added, 1);
        assert_eq!(store.macros().list().unwrap().len(), 2);
    }

    #[test]
    fn merge_backup_never_adds_the_same_incoming_duplicate_twice() {
        let (store, _dir) = open_temp_store();
        let record = MacroRecord {
            name: "dup".to_owned(),
            slot: None,
            events: vec![],
        };
        let backup = StoreBackup {
            exported_at: 1,
            firmware_note: FIRMWARE_NOTE.to_owned(),
            macros: vec![record.clone(), record],
            profiles: vec![],
            settings: vec![],
            firmware: vec![],
            events: vec![],
        };
        let response = merge_backup(&store, backup).unwrap();
        assert_eq!(response.macros_added, 1);
        assert_eq!(store.macros().list().unwrap().len(), 1);
    }

    #[test]
    fn parse_backup_refuses_a_malformed_file() {
        let error = parse_backup(b"not json").unwrap_err();
        assert!(matches!(error, AppError::InvalidData(_)));
    }
}
