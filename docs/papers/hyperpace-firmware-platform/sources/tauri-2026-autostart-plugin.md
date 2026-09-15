URL: https://v2.tauri.app/plugin/autostart/
Additional URLs: https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/autostart/src/lib.rs ; https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/autostart/Cargo.toml ; https://crates.io/crates/auto-launch/0.5.0 (source tarball, src/linux.rs) ; https://github.com/zzzgydi/auto-launch (0.6.0, commit 5a77b66cfb745e2ead928fe37aedee828cb993ea) ; https://github.com/tauri-apps/plugins-workspace/pull/3546 ; https://github.com/tauri-apps/plugins-workspace/issues/3166 ; https://github.com/tauri-apps/plugins-workspace/issues/1922
Retrieved: 2026-09-15 (plugins-workspace `v2` commit 0850317b, plugin version 2.5.1)
Source type: official plugin docs and source (primary); upstream crate source (primary); GitHub issues and PR (tier 2/3)

## Verbatim, docs: init signature

```rust
app.handle().plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--flag1", "--flag2"]) /* arbitrary number of args to pass to your app */));
```

## Verbatim, docs: capability permissions

```json title="src-tauri/capabilities/default.json"
{
  "permissions": [
    ...,
    "autostart:allow-enable",
    "autostart:allow-disable",
    "autostart:allow-is-enabled"
  ]
}
```

## Verbatim, plugin src/lib.rs

```rust
//! Automatically launch your application at startup. Supports Windows, Mac (via AppleScript or Launch Agent), and Linux.
```
```rust
#[derive(Debug, Default, Copy, Clone)]
pub enum MacosLauncher {
    #[default]
    LaunchAgent,
    AppleScript,
}
```
```rust
                let current_exe = current_exe()?;

                #[cfg(windows)]
                builder.set_app_path(&current_exe.display().to_string());

                #[cfg(target_os = "macos")]
                {
                    builder.set_use_launch_agent(matches!(
                        self.macos_launcher,
                        MacosLauncher::LaunchAgent
                    ));
                    // on macOS, current_exe gives path to /Applications/Example.app/MacOS/Example
                    // but this results in seeing a Unix Executable in macOS login items
                    // It must be: /Applications/Example.app
```
```rust
                #[cfg(target_os = "linux")]
                if let Some(appimage) = app
                    .env()
                    .appimage
                    .and_then(|p| p.to_str().map(|s| s.to_string()))
                {
                    builder.set_app_path(&appimage);
                } else {
                    builder.set_app_path(&current_exe.display().to_string());
                }
```

## Verbatim, plugin Cargo.toml dependency

```
auto-launch = "0.5"
```

## Verbatim, auto-launch 0.5.0 src/linux.rs (the version the plugin pins)

```
29:    /// - failed to create dir `~/.config/autostart`
30:    /// - failed to create file `~/.config/autostart/{app_name}.desktop`
39:            Exec={} {}\n\
60:    /// - failed to remove file `~/.config/autostart/{app_name}.desktop`
76:        get_dir().join(format!("{}.desktop", self.app_name))
80:/// Get the autostart dir
82:    dirs::home_dir().unwrap().join(".config").join("autostart")
```

## Verbatim, auto-launch 0.6.0 src/linux.rs (not yet used by the plugin)

```
9:    /// - `launch_mode`: launch mode (XDG Autostart or systemd)
```
```rust
fn build_xdg_autostart_data(app_name: &str, app_path: &str, args: &[String]) -> String {
    format!(
        "[Desktop Entry]\n\
        Type=Application\n\
        Version=1.0\n\
        Name={}\n\
        Comment={} startup script\n\
        Exec={} {}\n\
        StartupNotify=false\n\
        Terminal=false",
```

crates.io versions: `[('0.6.0', '2026-01-10'), ('0.5.0', '2023-09-10'), ...]`

## Verbatim, PR #3546 "chore(autostart): update auto-launch to 0.6" (state OPEN, 2026-08-20)

> Fix #1922
>
> Waiting for https://github.com/zzzgydi/auto-launch/pull/35 and https://github.com/zzzgydi/auto-launch/pull/36

Issue #1922 title (OPEN, 2024-10-12): "auto start doesn't work on programs that requires admin rights"

## Verbatim, issue #3166 (OPEN, 2025-12-16) "Flatpak autostart breaks with Tauri autostart plugin (Exec should use flatpak app id)"

> Tauri generates an autostart entry similar to:
>
> ```ini
> Exec=/app/bin/my-app --tray
> ```
>
> This command **cannot be executed outside the Flatpak sandbox**, so the app does not start on login.
