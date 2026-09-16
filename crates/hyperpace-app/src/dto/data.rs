//! Data Management screen shapes: the overview strip, the paged event log, and profile snapshots.
//! Backed by `commands::data`, which reads and writes every [`hyperpace_store`] collection.

use hyperpace_store::ProfileSnapshot;
use serde::Serialize;

/// One collection's record count, the store's root directory, and its on-disk size, for the Data
/// screen's overview strip.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataOverviewDto {
    /// Saved macros.
    pub macros: u64,
    /// Saved profile snapshots.
    pub profiles: u64,
    /// Stored app settings.
    pub settings: u64,
    /// Archived firmware packages.
    pub firmware: u64,
    /// Device and app event log entries.
    pub events: u64,
    /// The store's root directory, as an absolute path.
    pub store_root: String,
    /// Total bytes the store occupies on disk, walked from `store_root`.
    pub store_size_bytes: u64,
}

/// One event log entry, mirroring [`hyperpace_store::EventRecord`] plus its store document id.
/// Carries no `detail` field: the Data screen shows only what a person reads at a glance (the
/// spec's own field list), not the structured troubleshooting payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventRecordDto {
    /// Store document id.
    pub id: String,
    /// Unix seconds when the event happened.
    pub at: i64,
    /// Event kind, for example `"connected"` or `"battery_low"`.
    pub kind: String,
    /// One plain sentence describing what happened.
    pub message: String,
}

/// One page of the event log, newest first, plus the log's total size so the Data screen can page
/// it without loading more than one page's worth into the UI at a time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListEventsResponse {
    /// This page's entries, newest first.
    pub events: Vec<EventRecordDto>,
    /// Total entries in the log, independent of this page's size.
    pub total: u64,
}

/// A saved profile snapshot, mirroring [`ProfileSnapshot`] plus its store document id. Never
/// carries the snapshot's stored configuration bytes: the Data screen lists and manages
/// snapshots by name and capture time, and restoring one is a dedicated command, not a client-side
/// decode of this DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSnapshotDto {
    /// Store document id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Device profile index this snapshot was captured from.
    pub index: u8,
    /// Unix seconds when this snapshot was captured.
    pub captured_at: i64,
}

impl ProfileSnapshotDto {
    /// Pair a store id with the record it names.
    #[must_use]
    pub fn from_record(id: String, record: ProfileSnapshot) -> Self {
        Self {
            id,
            name: record.name,
            index: record.index,
            captured_at: record.captured_at,
        }
    }
}

/// Result of `import_store`: how many records were actually added to each collection. A record
/// present in both the backup and the destination store already (by full field equality) is
/// skipped, not counted here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportStoreResponse {
    /// Macros added.
    pub macros_added: u64,
    /// Profile snapshots added.
    pub profiles_added: u64,
    /// App settings added.
    pub settings_added: u64,
    /// Firmware archive entries added.
    pub firmware_added: u64,
    /// Event log entries added.
    pub events_added: u64,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn profile_snapshot_dto_pairs_the_store_id() {
        let record = ProfileSnapshot {
            name: "fps".to_owned(),
            index: 2,
            captured_at: 1_726_000_000,
            settings: serde_json::json!([1, 2, 3]),
        };
        let dto = ProfileSnapshotDto::from_record("bae-1".to_owned(), record);
        assert_eq!(dto.id, "bae-1");
        assert_eq!(dto.name, "fps");
        assert_eq!(dto.index, 2);
    }

    #[test]
    fn data_overview_dto_serializes_camel_case() {
        let dto = DataOverviewDto {
            macros: 1,
            profiles: 2,
            settings: 3,
            firmware: 4,
            events: 5,
            store_root: "/home/x/.hyperpace".to_owned(),
            store_size_bytes: 6,
        };
        let json = serde_json::to_value(dto).unwrap();
        assert_eq!(json["storeRoot"], "/home/x/.hyperpace");
        assert_eq!(json["storeSizeBytes"], 6);
    }
}
