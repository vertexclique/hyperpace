# SUMMARY: tauri-2026-notification-plugin

## Claim
tauri-plugin-notification 2.4.0 sends desktop notifications through notify-rust 4.11.7:
- **Linux:** zbus to the freedesktop notification service.
- **Windows:** tauri-winrt-notification.
- **macOS:** mac-notification-sys.

On desktop, permission always reports Granted.

On Windows, the AppUserModelID is set only when not running from `target/debug` or `target/release`. Plugin metadata says: "Only works for installed apps. Shows powershell name & icon in development."

On macOS, dev builds post as `com.apple.Terminal`; release builds post as the app identifier.

## Method
The docs, plugin `Cargo.toml`, `desktop.rs`, the workspace `Cargo.lock`, and the notify-rust README.

## Result
`show()` spawns an async task that calls `notification.show()` and then drops the handle.

## Evidence tier
1.

## Performance
Negligible per notification.

## Correctness
- **Proven:** the backends, the permission stub, and the identity logic.
- **Not verified:** why Windows requires an installed app (commonly a Start-menu shortcut that registers the AUMID; not sourced here).
- **Not verified:** whether macOS requires a signed bundle; only a third-party crate page claims it.
- **Not verified:** Linux behaviour without a notification daemon.
- GNOME 46+ drops these notifications (see giovannifranco1-2025).

## Relevance to hyperpace
- Send low-battery notifications from Rust, not JS.
- On Windows, toasts need the NSIS or MSI install to carry the app identity. A portable exe is likely to lose them (experiment).
- On GNOME, consider calling notify-rust directly and keeping the handle alive until the notification closes.
