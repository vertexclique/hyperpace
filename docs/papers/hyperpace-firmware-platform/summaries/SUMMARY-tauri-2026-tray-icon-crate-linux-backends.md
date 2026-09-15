# SUMMARY: tauri-2026-tray-icon-crate-linux-backends

## Claim
tray-icon 0.25.0 (2026-09-11) adds a GTK-free ksni backend (StatusNotifierItem over D-Bus). It runs on a worker thread and implements tooltips. The default remains libappindicator.

The AppIndicator backend:
- has no tooltip;
- may hide the icon unless a menu is set;
- writes a PNG to a temp dir on every icon change;
- loads `libayatana-appindicator3.so.1` or `libappindicator3.so.1` with dlopen, and panics if both are missing.

## Method
Read tray-icon `lib.rs`, `Cargo.toml`, `CHANGELOG.md`, the gtk and ksni platform modules, and libappindicator-sys 0.9.0.

## Result
- **Windows:** 0.24.2 raised the tooltip limit from 64 to 128 characters.
- 0.25 adds a GUID, so "always show" survives updates when both versions are signed by the same publisher.

## Evidence tier
1.

## Performance
On Linux AppIndicator, each `set_icon` costs one PNG write plus a D-Bus update. Not measured; bounded by how often the app changes the icon.

## Correctness
- **Proven:** the panic path, in libappindicator-sys source.
- **Not tested:** whether a missing library crashes a Tauri 2 app at tray creation.
- **Not tested:** whether Tauri's AppImage bundler always includes the library. PR 15917 ("detect tray-icon feature") concerns the quick-sharun backend only.

## Relevance to hyperpace
On Tauri 2 Linux:
- the AppImage must carry libayatana-appindicator, or the host must have it;
- always set a menu;
- throttle battery icon changes, updating only when the displayed bucket changes.

ksni via v3 removes the runtime library and adds tooltips, but GNOME still needs an SNI host extension.
