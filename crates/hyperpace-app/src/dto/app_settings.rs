//! App-level (not device) settings, backed by [`hyperpace_store::AppSetting`].

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// An `app_settings` request: read every setting, or write one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "camelCase")]
pub enum AppSettingsRequest {
    /// Read every app setting.
    Get,
    /// Write one setting by key, creating it if it does not exist yet.
    Set {
        /// Setting name, for example `"autostart"` or `"low_battery_threshold_percent"`.
        key: String,
        /// The new value.
        value: serde_json::Value,
    },
}

/// Every app setting, keyed by name, as it stands after the request completed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettingsResponse {
    /// Every stored app setting.
    pub settings: BTreeMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn get_request_tags_without_extra_fields() {
        let json = serde_json::to_value(AppSettingsRequest::Get).unwrap();
        assert_eq!(json["action"], "get");
    }

    #[test]
    fn set_request_round_trips_through_json() {
        let request = AppSettingsRequest::Set {
            key: "autostart".to_owned(),
            value: serde_json::json!(true),
        };
        let json = serde_json::to_string(&request).unwrap();
        let back: AppSettingsRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, request);
    }
}
