//! App-level (not device) settings, backed by [`hyperpace_store::AppSetting`].

use std::collections::BTreeMap;

use hyperpace_store::AppSetting;
use tauri::{AppHandle, Manager};

use crate::blocking::blocking;
use crate::dto::{AppSettingsRequest, AppSettingsResponse};
use crate::error::to_command_result;
use crate::state::{AppState, LOW_BATTERY_THRESHOLD_KEY};

/// Read every app setting, or write one and read all of them back.
///
/// # Errors
///
/// Returns an error message when the store cannot be reached.
#[tauri::command]
pub async fn app_settings(
    app: AppHandle,
    request: AppSettingsRequest,
) -> Result<AppSettingsResponse, String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();

            if let AppSettingsRequest::Set { key, value } = &request {
                let existing = state.store.settings().list()?;
                let record = AppSetting {
                    key: key.clone(),
                    value: value.clone(),
                };
                match existing.iter().find(|(_, setting)| &setting.key == key) {
                    Some((id, _)) => state.store.settings().update(id, &record)?,
                    None => {
                        state.store.settings().create(&record)?;
                    }
                }

                if key == LOW_BATTERY_THRESHOLD_KEY
                    && let Some(percent) = value
                        .as_u64()
                        .and_then(|percent| u8::try_from(percent).ok())
                {
                    state.set_low_battery_threshold(percent);
                }
            }

            let settings = state
                .store
                .settings()
                .list()?
                .into_iter()
                .map(|(_, setting)| (setting.key, setting.value))
                .collect::<BTreeMap<_, _>>();
            Ok(AppSettingsResponse { settings })
        })
        .await,
    )
}
