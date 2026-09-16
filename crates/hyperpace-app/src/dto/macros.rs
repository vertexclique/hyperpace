//! Macro library shapes, mirroring `hyperpace_store`'s macro records.

use hyperpace_store::{MacroEventRecord, MacroRecord};
use serde::{Deserialize, Serialize};

/// Mirrors [`MacroEventRecord`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MacroEventDto {
    /// `true` for a key press, `false` for a release.
    pub press: bool,
    /// Event kind, named after `hyperpace_protocol::MacroEventKind`'s variants.
    pub kind: String,
    /// Key code or media code for this event; its meaning depends on `kind`.
    pub value: u16,
    /// Delay before the next event, in milliseconds.
    pub delay_ms: u16,
}

impl From<MacroEventRecord> for MacroEventDto {
    fn from(event: MacroEventRecord) -> Self {
        Self {
            press: event.press,
            kind: event.kind,
            value: event.value,
            delay_ms: event.delay_ms,
        }
    }
}

impl From<MacroEventDto> for MacroEventRecord {
    fn from(event: MacroEventDto) -> Self {
        Self {
            press: event.press,
            kind: event.kind,
            value: event.value,
            delay_ms: event.delay_ms,
        }
    }
}

/// A saved macro, mirroring [`MacroRecord`] plus its store document id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MacroRecordDto {
    /// Store document id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Device macro slot this record is bound to, when it has been assigned one.
    pub slot: Option<u8>,
    /// The recorded events, in playback order.
    pub events: Vec<MacroEventDto>,
}

impl MacroRecordDto {
    /// Pair a store id with the record it names.
    #[must_use]
    pub fn from_record(id: String, record: MacroRecord) -> Self {
        Self {
            id,
            name: record.name,
            slot: record.slot,
            events: record.events.into_iter().map(Into::into).collect(),
        }
    }
}

/// A `save_macro` request: an existing document id to update, or `None` to create a new one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveMacroRequest {
    /// The document id to update, or `None` to create a new macro.
    #[serde(default)]
    pub id: Option<String>,
    /// Display name.
    pub name: String,
    /// Device macro slot to bind this macro to, and write it to when a device is connected.
    #[serde(default)]
    pub slot: Option<u8>,
    /// The recorded events, in playback order.
    pub events: Vec<MacroEventDto>,
}

impl SaveMacroRequest {
    /// The store record this request describes.
    #[must_use]
    pub fn to_record(&self) -> MacroRecord {
        MacroRecord {
            name: self.name.clone(),
            slot: self.slot,
            events: self.events.iter().cloned().map(Into::into).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn macro_record_dto_round_trips_through_json() {
        let record = MacroRecordDto {
            id: "bae-1".to_owned(),
            name: "burst".to_owned(),
            slot: Some(2),
            events: vec![MacroEventDto {
                press: true,
                kind: "Key".to_owned(),
                value: 4,
                delay_ms: 10,
            }],
        };
        let json = serde_json::to_string(&record).unwrap();
        let back: MacroRecordDto = serde_json::from_str(&json).unwrap();
        assert_eq!(back, record);
    }

    #[test]
    fn save_macro_request_converts_to_a_store_record() {
        let request = SaveMacroRequest {
            id: None,
            name: "burst".to_owned(),
            slot: Some(1),
            events: vec![],
        };
        let record = request.to_record();
        assert_eq!(record.name, "burst");
        assert_eq!(record.slot, Some(1));
    }
}
