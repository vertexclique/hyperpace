URL: https://v2.tauri.app/plugin/single-instance/
Additional URLs: https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/single-instance/src/platform_impl (linux.rs, windows.rs, macos.rs) ; https://github.com/tauri-apps/plugins-workspace/issues/3587 ; https://github.com/tauri-apps/plugins-workspace/issues/3542
Retrieved: 2026-09-15 (plugins-workspace `v2` commit 0850317b, plugin version 2.4.4)
Source type: official plugin docs and source (primary); GitHub issues (tier 3)

## Verbatim, docs

> The Single Instance plugin must be the first one to be registered to work well. This assures that it runs before other plugins can interfere.

> The plugin `init()` method takes a closure that is invoked when a new app instance was started, but closed by the plugin.

> On Linux the Single Instance plugin uses DBus to ensure that there will be only one instance running. It does so by publishing a service to DBus when the first instance starts running.
> Then, the following instances will try to publish the same service and, if it is already published, they will send a request to the service to notify the first instance, and exit right away.
>
> Despite this working pretty well when your app is bundled as a deb or rpm package or an AppImage, it won't work as intended for snap or flatpak packages by default because these packages run in a constrained sandboxed environment, where most of the communication to DBus services will be blocked if not explicitly declared on the packaging manifest.

> Because this Plugin currently does not have JavaScript APIs you do not have to configure [capabilities](/security/capabilities/) to use it.

## Verbatim, source: Linux (linux.rs)

```
13:use zbus::{blocking::Connection, interface, names::WellKnownName};
22:#[interface(name = "org.SingleInstance.DBus")]
37:let mut dbus_name = dbus_id.unwrap_or_else(|| app.config().identifier.clone());
38:dbus_name.push_str(".SingleInstance");
56:match zbus::blocking::connection::Builder::session()
69:Err(zbus::Error::NameTaken) => {
```
Cargo.toml: `semver = ["dep:semver"]` (optional feature appending a semver-compat suffix to the D-Bus name).

## Verbatim, source: Windows (windows.rs) and macOS (macos.rs)

```
windows.rs:19:        Threading::{CreateMutexW, ReleaseMutex},
windows.rs:70:                unsafe { CreateMutexW(std::ptr::null(), true.into(), mutex_name.as_ptr()) };
windows.rs:74:                    let hwnd = FindWindowW(class_name.as_ptr(), window_name.as_ptr());
windows.rs:91:                        SendMessageW(hwnd, WM_COPYDATA, 0, &cds as *const _ as _);
macos.rs:23:            let socket = socket_path(app.config(), app.package_info());
macos.rs:26:            match notify_singleton(&socket) {
macos.rs:35:                            listen_for_other_instances(socket, app.clone(), cb);
```

## Verbatim, issue #3587 (OPEN, 2026-09-14) "single-instance (Windows): second instance survives when FindWindowW misses the startup window"

> On Windows, a second instance survives `tauri-plugin-single-instance` when it starts during the window between the first instance's `CreateMutexW` and its `create_event_target_window`. The exit path is nested inside `if !hwnd.is_null()` and there is no `else`, so a `FindWindowW` miss falls through and the second process boots completely.
>
> Version: `2.4.2`. The same code is present on `v2` HEAD (`2.4.4`)

## Issue #3542 title (OPEN, 2026-08-18)

"single-instance: fail-closed ownership and pre-IPC secondary activation on Windows, Linux, and macOS"
