//! Linux desktop integration for a binary that was not installed by a package.
//!
//! A Wayland compositor shows a window's icon by matching the window's app id (`hyperpace`) to a
//! desktop entry. The deb, rpm and Arch packages install that entry and the icon, so an installed
//! app needs nothing from here. A binary run from its build tree or from an `AppImage` has no entry,
//! and the window gets the compositor's generic placeholder instead of the Hyperpace mark.
//!
//! So at startup, when no entry for this app exists in any XDG data directory, this writes a
//! per-user one under `~/.local/share`, pointing at the running executable, plus the icon. It never
//! overwrites an entry that already exists, packaged or not, and a failure is logged and ignored:
//! a missing icon must never stop the app from starting.

use std::fs;
use std::path::{Path, PathBuf};

/// The window's app id, which is what the compositor looks the entry up by.
const APP_ID: &str = "hyperpace";

/// The app icon, the same image the packages install.
const ICON_PNG: &[u8] = include_bytes!("../icons/icon.png");

/// The entry's file name, matching the app id.
fn entry_file_name() -> String {
    format!("{APP_ID}.desktop")
}

/// The contents of the per-user entry for the executable at `exec`.
fn entry_contents(exec: &Path) -> String {
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=Hyperpace\n\
         Comment=Configure your mouse: buttons, DPI, lighting, macros and firmware\n\
         Exec=\"{}\"\n\
         Icon={APP_ID}\n\
         StartupWMClass={APP_ID}\n\
         Categories=Utility;\n\
         Terminal=false\n",
        exec.display()
    )
}

/// Every XDG data directory, user first, per the base directory specification.
fn data_dirs(home: &Path) -> Vec<PathBuf> {
    let user =
        std::env::var_os("XDG_DATA_HOME").map_or_else(|| home.join(".local/share"), PathBuf::from);
    let system = std::env::var_os("XDG_DATA_DIRS").map_or_else(
        || "/usr/local/share:/usr/share".into(),
        |dirs| dirs.to_string_lossy().into_owned(),
    );
    std::iter::once(user)
        .chain(
            system
                .split(':')
                .filter(|dir| !dir.is_empty())
                .map(PathBuf::from),
        )
        .collect()
}

/// Whether any data directory already holds an entry this compositor would match to the app, under
/// either the app id or the product name the packages use.
fn entry_exists(dirs: &[PathBuf]) -> bool {
    dirs.iter().any(|dir| {
        let applications = dir.join("applications");
        applications.join(entry_file_name()).exists()
            || applications.join("Hyperpace.desktop").exists()
    })
}

/// Install the per-user entry and icon if no entry exists yet. Linux only; a no-op elsewhere.
pub fn ensure_installed() {
    if !cfg!(target_os = "linux") {
        return;
    }
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return;
    };
    let dirs = data_dirs(&home);
    if entry_exists(&dirs) {
        return;
    }
    let Some(user_dir) = dirs.first() else {
        return;
    };
    if let Err(error) = install(user_dir) {
        tracing::warn!(%error, "could not install the desktop entry; the window may show a generic icon");
    }
}

/// Write the icon and the entry under `data_dir`.
fn install(data_dir: &Path) -> std::io::Result<()> {
    let exec = std::env::current_exe()?;
    let icon_dir = data_dir.join("icons/hicolor/512x512/apps");
    fs::create_dir_all(&icon_dir)?;
    fs::write(icon_dir.join(format!("{APP_ID}.png")), ICON_PNG)?;

    let applications = data_dir.join("applications");
    fs::create_dir_all(&applications)?;
    fs::write(applications.join(entry_file_name()), entry_contents(&exec))?;
    tracing::info!(path = %applications.display(), "installed the desktop entry for this user");
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn the_entry_matches_the_window_app_id_and_names_the_icon() {
        let contents = entry_contents(Path::new("/opt/hyperpace/hyperpace"));
        assert!(contents.contains("StartupWMClass=hyperpace\n"));
        assert!(contents.contains("Icon=hyperpace\n"));
        assert!(contents.contains("Exec=\"/opt/hyperpace/hyperpace\"\n"));
    }

    #[test]
    fn an_existing_packaged_entry_is_recognized_so_it_is_never_shadowed() {
        let root = std::env::temp_dir().join(format!("hyperpace-entry-{}", std::process::id()));
        let applications = root.join("applications");
        fs::create_dir_all(&applications).unwrap();
        assert!(!entry_exists(std::slice::from_ref(&root)));

        fs::write(applications.join("Hyperpace.desktop"), "").unwrap();
        assert!(entry_exists(std::slice::from_ref(&root)));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn install_writes_the_icon_and_the_entry() {
        let root = std::env::temp_dir().join(format!("hyperpace-install-{}", std::process::id()));
        install(&root).unwrap();
        assert!(
            root.join("icons/hicolor/512x512/apps/hyperpace.png")
                .exists()
        );
        assert!(entry_exists(std::slice::from_ref(&root)));
        fs::remove_dir_all(&root).unwrap();
    }
}
