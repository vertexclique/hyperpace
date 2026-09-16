//! Development mode: run the app against the simulator, loaded with a real device's settings,
//! alongside an ordinary instance.
//!
//! Screen work has to be judged on a populated screen, and the real mouse cannot be relied on for
//! that: it sleeps, it is in use, and nothing in development should write to it. Setting
//! `HYPERPACE_SIMULATOR` makes this process:
//!
//! - connect to the simulator at startup instead of the real device,
//! - load the simulator's flash from `HYPERPACE_SIM_SHADOW` when set (a 16 KiB settings dump, as
//!   the `dump_settings` example writes) and report the production model's identity, so every
//!   screen shows exactly what that device held,
//! - use `HYPERPACE_STORE_ROOT` as its data directory when set, since the embedded store holds an
//!   exclusive lock and an ordinary instance may already have the default one open,
//! - skip the single-instance guard and title its window "Hyperpace Simulator", so it runs next to
//!   an ordinary instance and can be told apart from it,
//! - open on the screen named by `HYPERPACE_START_SCREEN` (for example `lighting`), so a screen
//!   can be captured without clicking to it.
//!
//! None of this is reachable without the environment variable, and nothing in it touches hardware.

use std::path::PathBuf;

/// The variable that turns development mode on.
const SIMULATOR_VAR: &str = "HYPERPACE_SIMULATOR";
/// A settings dump to load into the simulator's flash.
const SHADOW_VAR: &str = "HYPERPACE_SIM_SHADOW";
/// A data directory to use instead of the default.
const STORE_ROOT_VAR: &str = "HYPERPACE_STORE_ROOT";
/// The screen the window opens on.
const START_SCREEN_VAR: &str = "HYPERPACE_START_SCREEN";

/// Whether this process runs in development mode.
#[must_use]
pub fn simulator_mode() -> bool {
    std::env::var_os(SIMULATOR_VAR).is_some()
}

/// The settings dump to load into the simulator, if one is configured and readable. An unreadable
/// file is logged and treated as absent, so the simulator still starts.
#[must_use]
pub fn simulator_shadow() -> Option<Vec<u8>> {
    let path = std::env::var_os(SHADOW_VAR)?;
    match std::fs::read(&path) {
        Ok(bytes) => Some(bytes),
        Err(error) => {
            tracing::warn!(%error, "could not read the simulator settings dump; starting erased");
            None
        }
    }
}

/// The data directory to use instead of the default, when one is configured.
#[must_use]
pub fn store_root_override() -> Option<PathBuf> {
    std::env::var_os(STORE_ROOT_VAR).map(PathBuf::from)
}

/// A script run before the page loads that names the screen development mode opens on, or an empty
/// script. The page reads it once at startup and ignores a name it does not know. The name is
/// restricted to lowercase letters, so nothing from the environment can inject script.
#[must_use]
pub fn start_screen_script() -> String {
    if !simulator_mode() {
        return String::new();
    }
    std::env::var(START_SCREEN_VAR)
        .ok()
        .filter(|screen| !screen.is_empty() && screen.chars().all(|c| c.is_ascii_lowercase()))
        .map_or_else(String::new, |screen| {
            format!("window.__HYPERPACE_START_SCREEN__ = '{screen}';")
        })
}

/// The main window's title.
#[must_use]
pub fn window_title() -> &'static str {
    if simulator_mode() {
        "Hyperpace Simulator"
    } else {
        "Hyperpace"
    }
}
