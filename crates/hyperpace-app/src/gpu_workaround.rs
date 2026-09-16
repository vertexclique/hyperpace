//! `WebKitGTK` DMABUF renderer workaround for NVIDIA (Linux only).
//!
//! `docs/papers/hyperpace-firmware-platform/summaries/SUMMARY-tauri-2026-linux-graphics-docs.md`:
//! `WebKitGTK`'s DMABUF renderer produces a blank window on NVIDIA, and the documented fix is
//! `WEBKIT_DISABLE_DMABUF_RENDERER=1`, set before the webview exists. That summary is also explicit
//! that shipping the override unconditionally is the wrong fix (it silently degrades WebGL and
//! masks the renderer string on hardware that never had the bug), so this module detects the
//! failure condition instead of always setting the variable.
//!
//! # Why this re-executes the process instead of calling `std::env::set_var`
//!
//! This workspace forbids `unsafe_code`, and `std::env::set_var`/`remove_var` are `unsafe fn` (a
//! multi-threaded process reading the environment concurrently with a write is a data race on
//! some platforms' libc). There is no safe way to mutate this process's own environment after
//! start. Re-executing the same binary with the variable added to the child's environment
//! (`std::process::Command::env`, then `exec`, both safe) achieves the same effect: `WebKitGTK`'s
//! own subprocesses (the multi-process WebProcess/NetworkProcess) inherit environment from this
//! process however it started, so the variable only needs to be present before this process's
//! webview is created, which `exec` guarantees since it replaces the process image outright.
//!
//! # Detection
//!
//! Applied only when every one of these holds: running on Linux, a Wayland or X11 graphical
//! session is present (`XDG_SESSION_TYPE`, `WAYLAND_DISPLAY` or `DISPLAY`), and an NVIDIA GPU is
//! present (a `/sys/class/drm/card*/device/vendor` reading `0x10de`, or the `nvidia` kernel module
//! loaded per `/proc/modules`). Both checks are pure functions over paths and strings
//! (`nvidia_via_drm_sysfs`, `nvidia_module_loaded`, `graphical_session_present`, all private),
//! driven by real fixture files in tests rather than the machine this crate happens to build on.

use std::env;

/// The variable `WebKitGTK` reads. Set to `"1"` to disable the DMABUF renderer.
const WEBKIT_DISABLE_DMABUF_RENDERER: &str = "WEBKIT_DISABLE_DMABUF_RENDERER";

/// Environment-variable escape hatch: `auto` (default behaviour), `on`/`1`/`true` to force the
/// workaround on regardless of detection, `off`/`0`/`false` to force it off.
const OVERRIDE_ENV_VAR: &str = "HYPERPACE_DMABUF_WORKAROUND";

/// App-setting escape hatch, read through the existing `app_settings` command/store (see
/// `lib::read_gpu_workaround_setting`). Same three values as `OVERRIDE_ENV_VAR` (private); the
/// environment variable takes precedence when both are set.
pub const APP_SETTING_KEY: &str = "gpu_dmabuf_workaround";

/// The PCI vendor id sysfs reports for NVIDIA, as a `0x`-prefixed lowercase hex string.
const NVIDIA_VENDOR_ID: &str = "0x10de";

/// A resolved on/off/auto override, parsed from either escape hatch's raw string value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Override {
    Auto,
    On,
    Off,
}

/// Parses an override value (case-insensitive). Anything unrecognized is treated the same as
/// absent (`None`), which falls through to detection rather than erroring: a typo in an
/// environment variable should never crash app startup over a rendering workaround.
fn parse_override(raw: &str) -> Option<Override> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(Override::Auto),
        "on" | "1" | "true" => Some(Override::On),
        "off" | "0" | "false" => Some(Override::Off),
        _ => None,
    }
}

/// True if any DRM card device under `drm_class_dir` (real usage: `/sys/class/drm`, one
/// subdirectory per card, e.g. `card0`) reports vendor id `0x10de` (NVIDIA) in its
/// `device/vendor` file.
fn nvidia_via_drm_sysfs(drm_class_dir: &std::path::Path) -> bool {
    let Ok(entries) = std::fs::read_dir(drm_class_dir) else {
        return false;
    };
    entries.flatten().any(|entry| {
        let vendor_path = entry.path().join("device/vendor");
        std::fs::read_to_string(&vendor_path)
            .is_ok_and(|contents| contents.trim().eq_ignore_ascii_case(NVIDIA_VENDOR_ID))
    })
}

/// True if the `nvidia` kernel module is loaded, per the first (module name) column of
/// `proc_modules` (real usage: `/proc/modules`).
fn nvidia_module_loaded(proc_modules: &std::path::Path) -> bool {
    std::fs::read_to_string(proc_modules).is_ok_and(|contents| {
        contents
            .lines()
            .any(|line| line.split_whitespace().next() == Some("nvidia"))
    })
}

/// True if either sysfs or the loaded-module list shows an NVIDIA GPU.
fn nvidia_present(drm_class_dir: &std::path::Path, proc_modules: &std::path::Path) -> bool {
    nvidia_via_drm_sysfs(drm_class_dir) || nvidia_module_loaded(proc_modules)
}

/// True if this process is running inside a Wayland or X11 graphical session, per the standard
/// session-identifying environment variables.
fn graphical_session_present(
    session_type: Option<&str>,
    wayland_display: Option<&str>,
    display: Option<&str>,
) -> bool {
    let session_is_graphical = session_type.is_some_and(|value| {
        value.eq_ignore_ascii_case("wayland") || value.eq_ignore_ascii_case("x11")
    });
    session_is_graphical || wayland_display.is_some() || display.is_some()
}

/// The failure condition itself: a Wayland/X11 session with an NVIDIA GPU present.
fn detect(
    drm_class_dir: &std::path::Path,
    proc_modules: &std::path::Path,
    session_type: Option<&str>,
    wayland_display: Option<&str>,
    display: Option<&str>,
) -> bool {
    graphical_session_present(session_type, wayland_display, display)
        && nvidia_present(drm_class_dir, proc_modules)
}

/// What [`resolve`] decided, and why (for the one info log this module ever emits per process).
enum Decision {
    Apply(&'static str),
    Skip(&'static str),
}

/// Resolves the override chain (environment variable, then the app setting, then detection) into
/// a final apply/skip decision. `Auto` (explicit or absent) at either level falls through to the
/// next one. `read_app_setting` is called at most once, and only when the environment variable
/// did not already settle it: the caller's implementation opens this app's local store to answer
/// it, and skipping that call whenever it is not needed avoids an extra store open on every
/// startup (see `lib::run`'s doc comment on `read_gpu_workaround_setting`).
fn resolve(read_app_setting: impl FnOnce() -> Option<String>) -> Decision {
    match env::var(OVERRIDE_ENV_VAR)
        .ok()
        .as_deref()
        .and_then(parse_override)
    {
        Some(Override::On) => return Decision::Apply("forced on by HYPERPACE_DMABUF_WORKAROUND"),
        Some(Override::Off) => return Decision::Skip("forced off by HYPERPACE_DMABUF_WORKAROUND"),
        Some(Override::Auto) | None => {}
    }
    match read_app_setting().as_deref().and_then(parse_override) {
        Some(Override::On) => {
            return Decision::Apply("forced on by the gpu_dmabuf_workaround app setting");
        }
        Some(Override::Off) => {
            return Decision::Skip("forced off by the gpu_dmabuf_workaround app setting");
        }
        Some(Override::Auto) | None => {}
    }
    if detect(
        std::path::Path::new("/sys/class/drm"),
        std::path::Path::new("/proc/modules"),
        env::var("XDG_SESSION_TYPE").ok().as_deref(),
        env::var("WAYLAND_DISPLAY").ok().as_deref(),
        env::var("DISPLAY").ok().as_deref(),
    ) {
        Decision::Apply(
            "Linux Wayland/X11 session with an NVIDIA GPU detected (DRM sysfs vendor id or the nvidia kernel module)",
        )
    } else {
        Decision::Skip("no NVIDIA GPU detected on this Linux graphical session")
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use std::os::unix::process::CommandExt;
    use std::process::Command;

    use super::{Decision, WEBKIT_DISABLE_DMABUF_RENDERER, resolve};

    /// Applies the workaround for this process if needed. Never fails the caller: any problem
    /// along the way (an unresolvable executable path, a failed `exec`) is logged and the app
    /// continues without the workaround rather than aborting startup over a rendering hint.
    ///
    /// `read_app_setting` reads the current value of the [`super::APP_SETTING_KEY`] app setting
    /// (see `lib::read_gpu_workaround_setting`); called at most once, and not at all when the
    /// variable is already set or the environment override already settles the decision.
    pub fn apply(read_app_setting: impl FnOnce() -> Option<String>) {
        if std::env::var_os(WEBKIT_DISABLE_DMABUF_RENDERER).is_some() {
            tracing::info!(
                var = WEBKIT_DISABLE_DMABUF_RENDERER,
                "already set; leaving the DMABUF renderer workaround as configured"
            );
            return;
        }

        match resolve(read_app_setting) {
            Decision::Skip(reason) => {
                tracing::info!(reason, "DMABUF renderer workaround not applied");
            }
            Decision::Apply(reason) => reexec_with_workaround(reason),
        }
    }

    /// Re-executes this binary with `WEBKIT_DISABLE_DMABUF_RENDERER=1` in its environment; see
    /// this module's doc comment for why a re-exec is used instead of `std::env::set_var`. Only
    /// returns on failure (`exec` never returns on success).
    fn reexec_with_workaround(reason: &str) {
        tracing::info!(
            reason,
            var = WEBKIT_DISABLE_DMABUF_RENDERER,
            "re-executing with WEBKIT_DISABLE_DMABUF_RENDERER=1 to work around WebKitGTK's \
             blank-window failure on NVIDIA (see docs/papers/hyperpace-firmware-platform/summaries/\
             SUMMARY-tauri-2026-linux-graphics-docs.md)"
        );

        let Ok(exe) = std::env::current_exe() else {
            tracing::warn!(
                "could not resolve the current executable path; continuing without the workaround. \
                 Set WEBKIT_DISABLE_DMABUF_RENDERER=1 by hand if the window is blank."
            );
            return;
        };

        let error = Command::new(exe)
            .args(std::env::args_os().skip(1))
            .env(WEBKIT_DISABLE_DMABUF_RENDERER, "1")
            .exec();
        // `exec` only returns when it failed to replace the process image.
        tracing::warn!(
            %error,
            "could not re-exec to apply the DMABUF renderer workaround; continuing without it. \
             Set WEBKIT_DISABLE_DMABUF_RENDERER=1 by hand if the window is blank."
        );
    }
}

#[cfg(not(target_os = "linux"))]
mod other {
    /// No-op outside Linux: WebKitGTK does not exist there, so there is nothing to detect or set.
    pub fn apply(_read_app_setting: impl FnOnce() -> Option<String>) {}
}

#[cfg(target_os = "linux")]
pub use linux::apply;
#[cfg(not(target_os = "linux"))]
pub use other::apply;

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use std::fs;

    use super::{
        Override, detect, graphical_session_present, nvidia_module_loaded, nvidia_present,
        nvidia_via_drm_sysfs, parse_override,
    };

    /// A scratch directory unique to one test, cleaned up on drop so parallel tests never share
    /// fixture files.
    struct FixtureDir(std::path::PathBuf);

    impl FixtureDir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "hyperpace-gpu-workaround-test-{name}-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }

        fn path(&self) -> &std::path::Path {
            &self.0
        }
    }

    impl Drop for FixtureDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn parse_override_recognizes_every_spelling() {
        assert_eq!(parse_override("auto"), Some(Override::Auto));
        assert_eq!(parse_override("AUTO"), Some(Override::Auto));
        assert_eq!(parse_override("on"), Some(Override::On));
        assert_eq!(parse_override("1"), Some(Override::On));
        assert_eq!(parse_override("true"), Some(Override::On));
        assert_eq!(parse_override("off"), Some(Override::Off));
        assert_eq!(parse_override("0"), Some(Override::Off));
        assert_eq!(parse_override("false"), Some(Override::Off));
        assert_eq!(parse_override("  On \n"), Some(Override::On));
        assert_eq!(parse_override("nonsense"), None);
        assert_eq!(parse_override(""), None);
    }

    #[test]
    fn nvidia_via_drm_sysfs_matches_a_card_with_vendor_0x10de() {
        let fixture = FixtureDir::new("drm-nvidia");
        let card = fixture.path().join("card0/device");
        fs::create_dir_all(&card).unwrap();
        fs::write(card.join("vendor"), "0x10de\n").unwrap();

        assert!(nvidia_via_drm_sysfs(fixture.path()));
    }

    #[test]
    fn nvidia_via_drm_sysfs_ignores_a_non_nvidia_card() {
        let fixture = FixtureDir::new("drm-amd");
        let card = fixture.path().join("card0/device");
        fs::create_dir_all(&card).unwrap();
        fs::write(card.join("vendor"), "0x1002\n").unwrap();

        assert!(!nvidia_via_drm_sysfs(fixture.path()));
    }

    #[test]
    fn nvidia_via_drm_sysfs_checks_every_card_not_just_the_first() {
        let fixture = FixtureDir::new("drm-multi");
        let intel = fixture.path().join("card0/device");
        let nvidia = fixture.path().join("card1/device");
        fs::create_dir_all(&intel).unwrap();
        fs::create_dir_all(&nvidia).unwrap();
        fs::write(intel.join("vendor"), "0x8086\n").unwrap();
        fs::write(nvidia.join("vendor"), "0x10de\n").unwrap();

        assert!(nvidia_via_drm_sysfs(fixture.path()));
    }

    #[test]
    fn nvidia_via_drm_sysfs_handles_a_missing_directory() {
        let missing = std::env::temp_dir().join("hyperpace-gpu-workaround-does-not-exist");
        assert!(!nvidia_via_drm_sysfs(&missing));
    }

    #[test]
    fn nvidia_module_loaded_reads_the_module_name_column() {
        let fixture = FixtureDir::new("proc-modules-nvidia");
        let modules = fixture.path().join("modules");
        fs::write(
            &modules,
            "nvidia 12345678 3 nvidia_uvm,nvidia_drm Live 0x0000000000000000\n\
             i915 987654 1 - Live 0x0000000000000000\n",
        )
        .unwrap();

        assert!(nvidia_module_loaded(&modules));
    }

    #[test]
    fn nvidia_module_loaded_is_false_without_nvidia() {
        let fixture = FixtureDir::new("proc-modules-none");
        let modules = fixture.path().join("modules");
        fs::write(&modules, "i915 987654 1 - Live 0x0000000000000000\n").unwrap();

        assert!(!nvidia_module_loaded(&modules));
    }

    #[test]
    fn nvidia_present_is_true_when_only_the_module_check_matches() {
        let empty_drm = FixtureDir::new("nvidia-present-drm-empty");
        let modules_fixture = FixtureDir::new("nvidia-present-modules");
        let modules = modules_fixture.path().join("modules");
        fs::write(&modules, "nvidia 1 0 - Live 0x0\n").unwrap();

        assert!(nvidia_present(empty_drm.path(), &modules));
    }

    #[test]
    fn graphical_session_present_recognizes_wayland_and_x11() {
        assert!(graphical_session_present(Some("wayland"), None, None));
        assert!(graphical_session_present(Some("x11"), None, None));
        assert!(graphical_session_present(None, Some(":0"), None));
        assert!(graphical_session_present(None, None, Some(":1")));
        assert!(!graphical_session_present(None, None, None));
        assert!(!graphical_session_present(Some("tty"), None, None));
    }

    #[test]
    fn detect_requires_both_a_graphical_session_and_nvidia() {
        let fixture = FixtureDir::new("detect-full");
        let card = fixture.path().join("card0/device");
        fs::create_dir_all(&card).unwrap();
        fs::write(card.join("vendor"), "0x10de\n").unwrap();
        let modules = fixture.path().join("modules");
        fs::write(&modules, "i915 1 0 - Live 0x0\n").unwrap();

        // NVIDIA present, no session: the operator's machine before a Wayland session starts.
        assert!(!detect(fixture.path(), &modules, None, None, None));
        // Wayland session, NVIDIA present: the reported failure condition.
        assert!(detect(
            fixture.path(),
            &modules,
            Some("wayland"),
            None,
            None
        ));

        let no_nvidia = FixtureDir::new("detect-no-nvidia");
        let empty_card = no_nvidia.path().join("card0/device");
        fs::create_dir_all(&empty_card).unwrap();
        fs::write(empty_card.join("vendor"), "0x1002\n").unwrap();
        let no_nvidia_modules = no_nvidia.path().join("modules");
        fs::write(&no_nvidia_modules, "i915 1 0 - Live 0x0\n").unwrap();

        // Wayland session, AMD GPU: not the failure condition.
        assert!(!detect(
            no_nvidia.path(),
            &no_nvidia_modules,
            Some("wayland"),
            None,
            None
        ));
    }
}
