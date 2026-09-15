# SUMMARY: tauri-2026-single-instance-plugin

## Claim
tauri-plugin-single-instance 2.4.4 must be registered first.
- **Linux:** owns the session D-Bus name `{identifier}.SingleInstance` via zbus, and forwards args to the first instance.
- **Windows:** a named mutex plus `FindWindowW` and `WM_COPYDATA`.
- **macOS:** a Unix socket.
- Snap and Flatpak need manifest permissions.

## Method
The docs and the platform source files.

## Result
- **Open issue 3587:** on Windows, a second instance survives if it starts between the first instance creating its mutex and creating its message window.
- **Open issue 3542:** proposes fail-closed ownership on all three OSes.

## Evidence tier
1 (source, docs), 3 (issues).

## Performance
Not applicable.

## Correctness
The Windows race is a code-reading report, not reproduced here; the window is small (startup only). Linux depends on a session bus being present.

## Relevance to hyperpace
Two instances would both open the mouse's HID interface and could interleave commands, which matters most during a firmware flash.
- Use the plugin for UX: focus or recreate the window on a second launch.
- Add a device-level exclusive guard in Rust as a second line of defence, for example an advisory lock file under the runtime dir. The guard design belongs to the device stream.
