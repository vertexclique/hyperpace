//! The five record types this crate stores, and the trait that ties each to a collection.
//!
//! None of these types carry protocol knowledge: `hyperpace-store` never depends on
//! `hyperpace-protocol` (see the crate boundary table in
//! `docs/architecture/api-contract.md`), so a field whose shape genuinely belongs to the
//! protocol layer (a settings shadow, a macro's device-side encoding) is kept as opaque JSON
//! here and interpreted by whichever crate does know that shape.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// A type that [`crate::Collection`] can store one document per value of.
///
/// [`Collection`](crate::Collection) persists every record as a single JSON document under one
/// `data` field (see the crate docs for why), so the only thing a record type needs to supply
/// is the name of its collection in the embedded schema.
pub trait Record: Serialize + DeserializeOwned {
    /// GraphQL collection name this record type is stored under.
    const COLLECTION: &'static str;
}

/// A saved macro: a named sequence of key and delay events, optionally bound to a device
/// macro slot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacroRecord {
    /// Display name shown in the macro library and editor.
    pub name: String,
    /// Device macro slot this record is bound to, when it has been assigned one.
    pub slot: Option<u8>,
    /// The recorded events, in playback order.
    pub events: Vec<MacroEventRecord>,
}

impl Record for MacroRecord {
    const COLLECTION: &'static str = "Macro";
}

/// One event in a [`MacroRecord`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacroEventRecord {
    /// `true` for a key press, `false` for a release.
    pub press: bool,
    /// Event kind, named after `hyperpace_protocol`'s `MacroEventKind` variants (`"Key"`,
    /// `"Media"`, and so on). Kept as a string, not that enum, so this crate never depends on
    /// `hyperpace-protocol`; the caller maps between the two.
    pub kind: String,
    /// Key code or media code for this event; its meaning depends on `kind`.
    pub value: u16,
    /// Delay before the next event, in milliseconds.
    pub delay_ms: u16,
}

/// A snapshot of a device profile's settings, captured for backup or restore.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfileSnapshot {
    /// Display name for this snapshot.
    pub name: String,
    /// Device profile index this snapshot was captured from, or restores to.
    pub index: u8,
    /// Unix seconds when the snapshot was captured.
    pub captured_at: i64,
    /// The device settings shadow at capture time. Its shape belongs to
    /// `hyperpace_protocol::settings::Settings`, which this crate cannot depend on, so it is
    /// stored opaque and interpreted by the caller.
    pub settings: serde_json::Value,
}

impl Record for ProfileSnapshot {
    const COLLECTION: &'static str = "Profile";
}

/// One app-level (not device) preference, such as a UI or tray setting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppSetting {
    /// Setting name, for example `"autostart"` or `"low_battery_threshold_percent"`.
    pub key: String,
    /// The setting's value. Kept as JSON so this crate does not need to know the app's set of
    /// setting types; the caller decides what shape each key's value takes.
    pub value: serde_json::Value,
}

impl Record for AppSetting {
    const COLLECTION: &'static str = "Setting";
}

/// One entry in the local firmware archive index.
///
/// This is metadata only. The package bytes themselves live in a file named by [`Self::path`]
/// and are streamed by `hyperpace-firmware`, never buffered whole (see
/// `docs/architecture/api-contract.md`), so the archive index this crate stores stays
/// kilobytes even as packages grow.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FirmwareRecord {
    /// Product name as printed in the package header. Carries no vendor or model branding,
    /// matching every other product-facing string in this app.
    pub product: String,
    /// Firmware version, formatted `"{major}.{minor:02x}"` to match how the device reports it.
    pub version: String,
    /// Component id this image targets.
    pub cid: u8,
    /// Model id this image targets.
    pub mid: u8,
    /// Path to the package file, relative to the store root.
    pub path: String,
    /// sha256 of the package bytes, hex encoded.
    pub sha256: String,
    /// Unix seconds when this package was imported into the archive.
    pub imported_at: i64,
}

impl Record for FirmwareRecord {
    const COLLECTION: &'static str = "Firmware";
}

/// One entry in the app and device event log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventRecord {
    /// Unix seconds when the event happened.
    pub at: i64,
    /// Event kind, for example `"connected"`, `"disconnected"`, `"battery_low"`, or
    /// `"firmware_installed"`.
    pub kind: String,
    /// One plain sentence describing what happened, safe to show a person: no internal names,
    /// no stack traces (the honesty fence in `AGENTS.md` applies to every stored string a UI
    /// might display).
    pub message: String,
    /// Structured detail for troubleshooting, opaque to this crate.
    pub detail: Option<serde_json::Value>,
}

impl Record for EventRecord {
    const COLLECTION: &'static str = "Event";
}

#[cfg(test)]
mod tests {
    // Tests may assert: the doctrine bans panics on production paths, not in tests.
    #![allow(clippy::unwrap_used)]

    use super::*;

    /// Every record type round trips through JSON with no loss: this is the property the
    /// embedded store's `data: JSON` field relies on for every collection (see `collection.rs`
    /// for the integration tests that exercise the same property through a live store).
    #[test]
    fn every_record_type_round_trips_through_json() {
        let macro_record = MacroRecord {
            name: "burst fire".to_string(),
            slot: Some(2),
            events: vec![
                MacroEventRecord {
                    press: true,
                    kind: "Key".to_string(),
                    value: 4,
                    delay_ms: 12,
                },
                MacroEventRecord {
                    press: false,
                    kind: "Key".to_string(),
                    value: 4,
                    delay_ms: 8,
                },
            ],
        };
        assert_round_trips(&macro_record);

        let profile = ProfileSnapshot {
            name: "fps".to_string(),
            index: 1,
            captured_at: 1_726_000_000,
            settings: serde_json::json!({"polling_hz": 1000, "dpi_stages": [800, 1600]}),
        };
        assert_round_trips(&profile);

        let setting = AppSetting {
            key: "autostart".to_string(),
            value: serde_json::json!(true),
        };
        assert_round_trips(&setting);

        let firmware = FirmwareRecord {
            product: "Hyperpace mouse".to_string(),
            version: "1.0a".to_string(),
            cid: 102,
            mid: 1,
            path: "firmware/1.0a.bin".to_string(),
            sha256: "0".repeat(64),
            imported_at: 1_726_000_000,
        };
        assert_round_trips(&firmware);

        let event = EventRecord {
            at: 1_726_000_000,
            kind: "connected".to_string(),
            message: "device connected".to_string(),
            detail: Some(serde_json::json!({"link": "wireless"})),
        };
        assert_round_trips(&event);
    }

    #[test]
    fn optional_fields_round_trip_when_absent() {
        let macro_record = MacroRecord {
            name: "no slot yet".to_string(),
            slot: None,
            events: vec![],
        };
        assert_round_trips(&macro_record);

        let event = EventRecord {
            at: 0,
            kind: "offline".to_string(),
            message: "device went offline".to_string(),
            detail: None,
        };
        assert_round_trips(&event);
    }

    fn assert_round_trips<T>(value: &T)
    where
        T: Record + PartialEq + std::fmt::Debug,
    {
        let encoded = serde_json::to_value(value).unwrap();
        let decoded: T = serde_json::from_value(encoded).unwrap();
        assert_eq!(&decoded, value);
    }
}
