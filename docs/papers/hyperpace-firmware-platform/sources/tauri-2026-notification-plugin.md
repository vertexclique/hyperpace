URL: https://v2.tauri.app/plugin/notification/
Additional URLs: https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/notification/Cargo.toml ; https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/notification/src/desktop.rs ; https://github.com/tauri-apps/plugins-workspace/blob/v2/Cargo.lock ; https://github.com/hoodie/notify-rust/blob/main/README.md
Retrieved: 2026-09-15 (plugins-workspace `v2` commit 0850317b5c85092cbf4ea9caf4ef3a9c771fcf27, 2026-09-10; plugin version in tree 2.4.0)
Source type: official plugin documentation and source (primary); upstream crate README (primary for that crate)

## Verbatim, docs: send flow

> Follow these steps to send a notification:
>
> 1. Check if permission is granted
> 2. Request permission if not granted
> 3. Send the notification

```rust
        use tauri_plugin_notification::NotificationExt;
        app.notification()
            .builder()
            .title("Tauri")
            .body("Tauri is awesome")
            .show()
            .unwrap();
```

## Verbatim, plugin Cargo.toml platform metadata and dependencies

```
[package.metadata.platforms.support]
windows = { level = "full", notes = "Only works for installed apps. Shows powershell name & icon in development." }
linux = { level = "full", notes = "" }
macos = { level = "full", notes = "" }
```
```
[target."cfg(windows)".dependencies]
win7-notifications = { version = "0.4.5", optional = true }
windows-version = { version = "0.1", optional = true }

[target."cfg(any(target_os = \"macos\", windows, target_os = \"linux\", target_os = \"dragonfly\", target_os = \"freebsd\", target_os = \"openbsd\", target_os = \"netbsd\"))".dependencies]
notify-rust = "4.11"
```

## Verbatim, desktop.rs: permission is always granted on desktop

```rust
    pub fn request_permission(&self) -> crate::Result<PermissionState> {
        Ok(PermissionState::Granted)
    }

    pub fn permission_state(&self) -> crate::Result<PermissionState> {
        Ok(PermissionState::Granted)
    }
```

## Verbatim, desktop.rs: Windows app identity and macOS identity

```rust
            #[cfg(windows)]
            {
                let exe = tauri::utils::platform::current_exe()?;
                let exe_dir = exe.parent().expect("failed to get exe directory");
                let curr_dir = exe_dir.display().to_string();
                // set the notification's System.AppUserModel.ID only when running the installed app
                if !(curr_dir.ends_with(format!("{SEP}target{SEP}debug").as_str())
                    || curr_dir.ends_with(format!("{SEP}target{SEP}release").as_str()))
                {
                    notification.app_id(&self.identifier);
                }
            }
            #[cfg(target_os = "macos")]
            {
                let _ = notify_rust::set_application(if tauri::is_dev() {
                    "com.apple.Terminal"
                } else {
                    &self.identifier
                });
            }

            tauri::async_runtime::spawn(async move {
                let _ = notification.show();
            });
```

## Verbatim, plugins-workspace Cargo.lock: notify-rust backends

```
name = "notify-rust"
version = "4.11.7"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "6442248665a5aa2514e794af3b39661a8e73033b1cc5e59899e1276117ee4400"
dependencies = [
 "futures-lite",
 "log",
 "mac-notification-sys",
 "serde",
 "tauri-winrt-notification",
 "zbus",
]
```

## Verbatim, notify-rust README

> The main audience of this library are Linux/BSD based desktop environments that follow the XDG specification (see [gnome dev docs](...) or [libnotify docs](...)). These include KDE, Gnome, XFCE, LXDC, Mate (and probably also most others that I haven't personally tested).

> ## macOS support
>
> This library shines on linux and bsd, which is its original target platform.
> Lately it gained support for macOS thanks to [mac-notification-sys](https://crates.io/crates/mac-notification-sys).
> However this only includes a small subset of the current functionality, since [`NSNotification`](https://developer.apple.com/reference/foundation/nsnotification)s don't have as many features.

> ## Windows support
>
> Similar to macOS we support windows via the help of [winrt-notification](https://crates.io/crates/winrt-notification).

## Verbatim, Windows installer docs (https://v2.tauri.app/distribute/windows-installer/)

> Additionally, to use the Notification API in Windows 7, you need to enable the `windows7-compat` Cargo feature:

## Not found

A primary statement that macOS notifications require a signed app bundle was not found. WebSearch returned only a third-party crate page (lib.rs/crates/tauri-plugin-notifications, not the official plugin) claiming "On macOS you need a signed app package, otherwise notifications don't work"; not verified. mac-notification-sys README (master branch, fetched via GitHub API) shows `get_bundle_identifier_or_default` / `set_application` usage and no signing statement.
