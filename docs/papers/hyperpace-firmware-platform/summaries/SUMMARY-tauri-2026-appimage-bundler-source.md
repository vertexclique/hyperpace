# SUMMARY: tauri-2026-appimage-bundler-source

## Claim
Tauri 2's AppImage bundler:
- drives linuxdeploy with the GTK plugin;
- copies the WebKitGTK helper processes and the injected bundle into the AppDir;
- downloads linuxdeploy plugins and the AppImage output plugin at build time;
- has a GTK hook that exports `GDK_BACKEND=x11` unless the user set one. The fix that respects a user-set value is pending release.

## Method
Read `linuxdeploy.rs`, `linuxdeploy-plugin-gtk.sh`, the pending `.changes`, and PR states.

## Result
- `xdg-open`/`xdg-utils` are no longer bundled (pending release).
- The `excludeLibraries` config (PR 15662) is open.
- The `v3` branch adds `sharun_cef.rs`; `dev` has no sharun backend file.

## Evidence tier
1.

## Performance
Forcing X11 means AppImage builds run under XWayland on Wayland sessions, where fractional scaling and input behave differently (not measured).

## Correctness
File-level facts are proven; the runtime effects of the X11 default are inferred.

## Relevance to hyperpace
- AppImage builds need network access during bundling.
- Plan a post-bundle step (extract, prune display-stack libraries, repack) unless PR 15662 lands.
- Decide whether the X11 default is acceptable for the UI.
