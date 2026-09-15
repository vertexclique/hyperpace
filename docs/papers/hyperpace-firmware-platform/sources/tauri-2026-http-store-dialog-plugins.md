URL: https://v2.tauri.app/plugin/http-client/
Additional URLs: https://v2.tauri.app/plugin/store/ ; https://v2.tauri.app/plugin/dialog/ ; https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/http/Cargo.toml ; https://github.com/tauri-apps/plugins-workspace/blob/v2/plugins/dialog/Cargo.toml ; https://github.com/tauri-apps/plugins-workspace/issues/3394
Retrieved: 2026-09-15 (tauri-docs `v2` commit a6b59b78; plugins-workspace `v2` commit 0850317b)
Source type: official plugin documentation and source (primary); GitHub issue (tier 3)

## Verbatim, HTTP plugin docs

> The HTTP plugin is available in both Rust as a [reqwest](https://docs.rs/reqwest/) re-export and JavaScript.

```json
          "allow": [{ "url": "https://*.tauri.app" }],
          "deny": [{ "url": "https://private.tauri.app" }]
```

> In Rust you can utilize the `reqwest` crate re-exported by the plugin. For more details refer to [reqwest docs](https://docs.rs/reqwest/).

```rust
use tauri_plugin_http::reqwest;
let res = reqwest::get("http://my.api.host/data.json").await;
```

## Verbatim, HTTP plugin Cargo.toml features

```
[features]
default = ["rustls-tls", "http2", "charset", "system-proxy", "cookies"]
multipart = ["reqwest/multipart"]
json = ["reqwest/json"]
stream = ["reqwest/stream"]
native-tls = ["reqwest/native-tls"]
...
rustls-tls = ["reqwest/rustls-tls"]
...
system-proxy = ["reqwest/system-proxy"]
```

## Verbatim, Store plugin docs

> This store will allow you to persist state to a file which can be saved and loaded on demand including between app restarts. Note that this process is asynchronous which will require handling it within your code. It can be used both in the webview or within Rust.

```
// You can manually save the store after making changes.
// Otherwise, it will save upon graceful exit
// And if you set `autoSave` to a number or left empty,
// it will save the changes to disk after a debounce delay, 100ms by default.
```
```
            let store = app.store("store.json")?;
            // Note that values must be serde_json::Value instances,
```

## Verbatim, Dialog plugin docs and Cargo.toml

> The file dialog APIs returns file system paths on Linux, Windows and macOS.

```
default = ["gtk3"]
xdg-portal = ["rfd/xdg-portal", "rfd/tokio", "rfd/wayland"]
gtk3 = ["rfd/gtk3"]
rfd = { version = "0.16", default-features = false, features = [
```

## Verbatim, store issue #3394 (CLOSED, opened 2026-04-17) "[tauri-plugin-store] Deadlock on RunEvent::Exit save"

> Visually, the main window got unresponsive when I tried to close the app, and I had to kill it.

> Just a tauri starter app + under 40 lines of code (repeatedly try to save something into store).
