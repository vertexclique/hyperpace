//! Route 1 of the plan's three firmware acquisition routes ("watch for publication").
//!
//! `hyperpace_firmware::watch` is pure: it only evaluates bytes a caller already fetched. This
//! module owns the one thing that crate deliberately does not: the actual HTTP fetch, done with a
//! `reqwest::blocking::Client` so it fits the same synchronous shape every other command uses
//! (`crate::blocking::blocking`), the app-setting gate that lets the operator turn the route off,
//! and the background schedule that runs it at most once per app start plus a slow periodic check
//! afterward (`docs/plans/hyperpace.md`'s firmware paragraph: "never a busy loop").
//!
//! Never downloads or installs anything: every fetch here targets a small vendor config file or a
//! directory's fallback page, capped by `MAX_RESPONSE_BYTES`, and a `.bin` link a config file
//! names is only ever quoted as evidence, never followed.

use std::io::Read;
use std::time::Duration;

use hyperpace_firmware::watch::{
    CONFIG_TARGETS, DIRECTORY_TARGETS, WatchReport, evaluate_config, evaluate_directory,
};
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::blocking::blocking;
use crate::commands::firmware::unix_now;
use crate::dto::FirmwareWatchReportDto;
use crate::error::{AppError, to_command_result};
use crate::state::AppState;

/// `app_settings` key that turns this route off. Absent (the first-run default) means enabled:
/// the plan ships all three acquisition routes enabled, and this setting is the operator's
/// explicit opt-out, not an opt-in.
pub const FIRMWARE_WATCH_ENABLED_KEY: &str = "firmware_watch_enabled";

/// Maximum bytes read from any one response before this checker gives up on it. Every target here
/// is a small vendor config file or a directory fallback page; the largest archived config is
/// under 460 KB, so a response ballooning past this bound is itself unusual, and holding it in
/// memory to evaluate would not be proportionate to what this checker is for.
const MAX_RESPONSE_BYTES: u64 = 4 * 1024 * 1024;
/// Per-request timeout. This is a background side-channel check; nothing here is worth blocking a
/// command or the periodic thread on a stalled vendor host for.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
/// How long the background watcher sleeps between automatic checks after the one it runs at app
/// start. Slow by design (`docs/plans/hyperpace.md`: "at most, once per app start plus a slow
/// periodic check; never a busy loop").
const AUTO_CHECK_INTERVAL: Duration = Duration::from_hours(24);

/// Explicit, on-demand firmware watch check: the Firmware screen's "Check for firmware
/// publication" action. Fetches the two vendor config files and the firmware directory paths
/// `FIRMWARE-VERDICT.md` names and reports what changed; never downloads or installs anything.
///
/// # Errors
///
/// Returns an error message when the operator has turned the route off
/// ([`FIRMWARE_WATCH_ENABLED_KEY`]), or when the HTTP client itself could not be built.
#[tauri::command]
pub async fn firmware_watch_check(app: AppHandle) -> Result<FirmwareWatchReportDto, String> {
    to_command_result(
        blocking(move || {
            let state = app.state::<AppState>();
            if !watch_enabled(&state) {
                return Err(AppError::FirmwareWatchDisabled);
            }
            let client = build_client()?;
            Ok(FirmwareWatchReportDto::from_report(
                run_check(&client),
                unix_now(),
            ))
        })
        .await,
    )
}

/// Whether the operator has left the watch route enabled. Reads the store fresh on every call
/// (at most a few times a day) rather than caching, so toggling the setting off in `app_settings`
/// takes effect on the very next check with no other synchronization needed.
fn watch_enabled(state: &AppState) -> bool {
    state
        .store
        .settings()
        .list()
        .unwrap_or_default()
        .into_iter()
        .find(|(_, setting)| setting.key == FIRMWARE_WATCH_ENABLED_KEY)
        .and_then(|(_, setting)| setting.value.as_bool())
        .unwrap_or(true)
}

/// Build the blocking HTTP client every check in this module shares.
fn build_client() -> Result<reqwest::blocking::Client, AppError> {
    Ok(reqwest::blocking::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()?)
}

/// Fetch and evaluate every config and directory target the verdict names. Never fails outright on
/// one target's own fetch error: that target's failure is recorded in the report instead, so one
/// slow or unreachable path never blanks out findings on the others.
fn run_check(client: &reqwest::blocking::Client) -> WatchReport {
    let mut report = WatchReport::default();
    for target in CONFIG_TARGETS {
        match fetch_bounded(client, target.url) {
            Ok((_status, body)) => report.configs.push(evaluate_config(target, &body)),
            Err(reason) => report.fetch_errors.push((target.url.to_owned(), reason)),
        }
    }
    for target in DIRECTORY_TARGETS {
        match fetch_bounded(client, target.url) {
            Ok((status, body)) => report
                .directories
                .push(evaluate_directory(target, status, &body)),
            Err(reason) => report.fetch_errors.push((target.url.to_owned(), reason)),
        }
    }
    report
}

/// GET `url` and read its body bounded by [`MAX_RESPONSE_BYTES`]: returns the status and body on
/// success, or a plain-English reason this target could not be evaluated. The bound is enforced by
/// reading, not by trusting a declared `Content-Length` (which a server is free to omit or
/// misstate), so a runaway body can never cost more than one byte past the limit held in memory.
fn fetch_bounded(client: &reqwest::blocking::Client, url: &str) -> Result<(u16, Vec<u8>), String> {
    let response = client.get(url).send().map_err(|err| err.to_string())?;
    let status = response.status().as_u16();
    let body = read_bounded(response, MAX_RESPONSE_BYTES)?;
    Ok((status, body))
}

/// Read all of `reader` into memory, refusing loudly rather than silently truncating once more
/// than `limit` bytes have arrived.
fn read_bounded(mut reader: impl Read, limit: u64) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    reader
        .by_ref()
        .take(limit + 1)
        .read_to_end(&mut buf)
        .map_err(|err| err.to_string())?;
    if buf.len() as u64 > limit {
        return Err(format!(
            "response exceeded the {limit} byte bound this checker allows"
        ));
    }
    Ok(buf)
}

/// Send a notification through the OS notification center when a background check finds
/// something. Never claims firmware was found; states only what kind of thing changed and repeats
/// the evidence disclaimer, matching every finding's own wording.
fn notify_finding(app: &AppHandle, report: &WatchReport) {
    use std::fmt::Write as _;

    let notable_configs = report.configs.iter().filter(|f| f.is_notable()).count();
    let notable_dirs = report.directories.iter().filter(|f| f.is_notable()).count();
    let mut body = format!(
        "{notable_configs} config file(s) and {notable_dirs} directory path(s) changed since the \
         research recorded them."
    );
    if !report.fetch_errors.is_empty() {
        let _ = write!(
            body,
            " {} target(s) could not be reached.",
            report.fetch_errors.len()
        );
    }
    let _ = write!(body, " {}", hyperpace_firmware::watch::EVIDENCE_DISCLAIMER);
    let result = app
        .notification()
        .builder()
        .title("Hyperpace")
        .body(body)
        .show();
    if let Err(error) = result {
        tracing::warn!(%error, "could not show the firmware watch notification");
    }
}

/// Start the background task that runs one check now, then again every `AUTO_CHECK_INTERVAL`
/// for as long as the app runs. Never a busy loop: the only wait between iterations is one bounded
/// sleep, and each iteration re-reads [`FIRMWARE_WATCH_ENABLED_KEY`], so turning the setting off
/// takes effect on the next tick without a restart. Called once from `lib::run`'s `setup`.
pub fn spawn_periodic_check(app: AppHandle) {
    let spawned = std::thread::Builder::new()
        .name("hyperpace-firmware-watch".to_owned())
        .spawn(move || {
            loop {
                let Some(state) = app.try_state::<AppState>() else {
                    return; // state not managed yet/anymore; should not happen once setup has run
                };
                if watch_enabled(&state) {
                    match build_client() {
                        Ok(client) => {
                            let report = run_check(&client);
                            if report.has_findings() {
                                notify_finding(&app, &report);
                            }
                        }
                        Err(error) => {
                            tracing::warn!(%error, "firmware watch could not build its client");
                        }
                    }
                }
                std::thread::sleep(AUTO_CHECK_INTERVAL);
            }
        });
    if let Err(error) = spawned {
        tracing::error!(%error, "could not start the firmware watch background thread");
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn read_bounded_accepts_a_body_within_the_limit() {
        let body = b"a small config file".to_vec();
        let read = read_bounded(std::io::Cursor::new(body.clone()), 1024).unwrap();
        assert_eq!(read, body);
    }

    #[test]
    fn read_bounded_refuses_a_body_over_the_limit() {
        let body = vec![0u8; 10];
        let err = read_bounded(std::io::Cursor::new(body), 4).unwrap_err();
        assert!(err.contains("byte bound"));
    }

    #[test]
    fn read_bounded_accepts_a_body_exactly_at_the_limit() {
        let body = vec![1u8; 8];
        let read = read_bounded(std::io::Cursor::new(body.clone()), 8).unwrap();
        assert_eq!(read, body);
    }
}
