# SUMMARY: tauri-2026-autostart-plugin

## Claim
tauri-plugin-autostart 2.5.1 wraps auto-launch 0.5:
- **Linux:** writes `~/.config/autostart/{app_name}.desktop` with `Exec={path} {args}`. When running as an AppImage, the path is `$APPIMAGE`.
- **macOS:** uses a LaunchAgent by default, or AppleScript login items.
- Arguments passed at init are appended to the launch command.

## Method
The docs, plugin `lib.rs`, the auto-launch 0.5.0 crate source, the auto-launch 0.6.0 repo, PR 3546, and issues 3166 and 1922.

## Result
- auto-launch 0.6.0 (2026-01-10) adds a systemd user-service mode; the plugin upgrade PR is open.
- Flatpak autostart is broken: `Exec` points inside the sandbox.
- Apps needing admin rights reportedly do not autostart on Windows (issue 1922, open).

## Evidence tier
1 (source), 3 (issues).

## Performance
Not applicable.

## Correctness
- **Unverified:** the Windows mechanism (the registry Run key in auto-launch); that code was not read.
- **Inferred from code:** the app path is recorded only when `enable()` writes the file, so a moved or renamed AppImage leaves a stale entry until `enable()` runs again.
- **Inferred, not tested:** the `Exec` line is built without quoting, so paths with spaces may break.

## Relevance to hyperpace
- Pass a flag such as `--autostart` so a login launch starts hidden in the tray.
- When the user setting is on, call `enable()` on every launch to refresh the AppImage path.
- Keep AppImage file names free of spaces, or test quoting.
