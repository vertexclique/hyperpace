# SUMMARY: tauri-2026-system-tray-guide-and-tray-api

## Claim
- Enable the `tray-icon` feature and build with `TrayIconBuilder` (icon, menu, `on_menu_event`, `on_tray_icon_event`).
- Update at runtime with `TrayIcon::set_icon`, `set_tooltip`, `set_title` and `set_menu`.
- On Linux: tray mouse events are not emitted, tooltips are unsupported, menu-on-left-click is unsupported, and a menu cannot be removed once set.

## Method
The official guide plus the `crates/tauri/src/tray/mod.rs` doc comments (2.11.5 tree).

## Result
- **Windows:** `set_title` is unsupported.
- **Linux:** the title shows only next to an icon and "may not be shown in all visualizations".
- `set_icon` is dispatched to the main thread.

## Evidence tier
1.

## Performance
Each icon update crosses to the main thread. On the Linux AppIndicator backend it also writes a PNG file (see the tray-icon crate summary).

## Correctness
These limits describe Tauri 2 (tray-icon 0.24, AppIndicator). The ksni backend in tray-icon 0.25 (Tauri v3) implements tooltip and activate callbacks. Whether Tauri v3 surfaces click events on Linux was not verified.

## Relevance to hyperpace
Battery display on Tauri 2, per OS:
- **Windows:** tooltip text (up to 128 chars) plus a rendered icon.
- **macOS:** `set_title` shows text in the menu bar.
- **Linux:** there is no tooltip or click. Render the percentage into the icon image, update a disabled menu item ("Battery 73%"), and optionally use `set_title`.

On Linux the "open window" action must be a menu item, since a left click cannot be caught.
