URL: https://crates.io/api/v1/crates/tauri (and the same endpoint for every crate listed below); https://registry.npmjs.org/@tauri-apps/cli (and the same endpoint for every npm package listed below)
Retrieved: 2026-09-15
Source type: package registry API (primary, machine-readable)

Method of capture: `curl` of the crates.io API and npm registry, printed with a small python3 one-liner (crate: max_stable_version, updated_at, newest three non-yanked versions as (num, created_at, rust_version); npm: dist-tags and publish time of `latest`). The lines below are the verbatim tool output.

## crates.io (format: max_stable_version updated_at [(num, created_at, rust_version) x3])

```
tauri: 2.11.5 2026-09-15T10:58:24.737189Z [('3.0.0-alpha.1', '2026-09-15', '1.95'), ('3.0.0-alpha.0', '2026-09-13', '1.95'), ('2.11.5', '2026-07-01', '1.77.2')]
tauri-build: 2.6.3 2026-09-13T01:51:27.050027Z [('3.0.0-alpha.0', '2026-09-13', '1.95'), ('1.5.7-edition2024.0', '2026-06-30', '1.85'), ('2.6.3', '2026-06-17', '1.77.2')]
tauri-bundler: 2.9.4 2026-09-12T20:05:36.673984Z [('3.0.0-alpha.0', '2026-09-12', '1.95'), ('2.9.4', '2026-06-28', '1.77.2'), ('2.9.3', '2026-06-17', '1.77.2')]
tauri-cli: 2.11.4 2026-09-15T11:02:49.971162Z [('3.0.0-alpha.1', '2026-09-15', '1.95'), ('3.0.0-alpha.0', '2026-09-13', '1.95'), ('2.11.4', '2026-06-28', '1.77.2')]
tauri-runtime-wry: 2.11.4 2026-09-15T10:59:05.478032Z [('3.0.0-alpha.1', '2026-09-15', '1.95'), ('3.0.0-alpha.0', '2026-09-13', '1.95'), ('2.11.4', '2026-06-30', '1.77.2')]
wry: 0.57.0 2026-09-08T03:54:05.166931Z [('0.57.0', '2026-09-08', '1.85'), ('0.56.1', '2026-08-13', '1.77'), ('0.56.0', '2026-07-30', '1.77')]
tao: 0.37.0 2026-08-21T18:29:39.389634Z [('0.37.0', '2026-08-21', '1.85'), ('0.36.0', '2026-07-29', '1.74'), ('0.35.3', '2026-05-23', '1.74')]
tray-icon: 0.25.0 2026-09-11T17:58:52.740447Z [('0.25.0', '2026-09-11', '1.90'), ('0.24.2', '2026-07-27', '1.73'), ('0.24.1', '2026-06-10', '1.73')]
tauri-plugin-notification: 2.4.0 2026-09-13T19:01:45.071738Z [('3.0.0-alpha.0', '2026-09-13', '1.77.2'), ('2.4.0', '2026-08-31', '1.77.2'), ('2.3.3', '2025-10-27', '1.77.2')]
tauri-plugin-autostart: 2.5.1 2026-09-13T18:58:58.002002Z [('3.0.0-alpha.0', '2026-09-13', '1.77.2'), ('2.5.1', '2025-10-27', '1.77.2'), ('2.5.0', '2025-06-25', '1.77.2')]
tauri-plugin-single-instance: 2.4.4 2026-09-13T19:02:43.199980Z [('3.0.0-alpha.0', '2026-09-13', '1.77.2'), ('2.4.4', '2026-08-31', '1.77.2'), ('2.4.3', '2026-07-13', '1.77.2')]
tauri-plugin-updater: 2.11.0 2026-09-13T19:03:20.392099Z [('3.0.0-alpha.0', '2026-09-13', '1.77.2'), ('2.11.0', '2026-08-31', '1.77.2'), ('2.10.1', '2026-04-04', '1.77.2')]
tauri-plugin-http: 2.6.0 2026-09-13T19:01:07.707332Z [('3.0.0-alpha.0', '2026-09-13', '1.77.2'), ('2.6.0', '2026-08-31', '1.77.2'), ('2.5.9', '2026-05-02', '1.77.2')]
tauri-plugin-store: 2.4.4 2026-09-13T19:02:57.739027Z [('3.0.0-alpha.0', '2026-09-13', '1.77.2'), ('2.4.4', '2026-07-18', '1.77.2'), ('2.4.3', '2026-05-02', '1.77.2')]
tauri-plugin-fs: 2.5.2 2026-09-13T18:59:59.957732Z [('3.0.0-alpha.0', '2026-09-13', '1.77.2'), ('2.5.2', '2026-08-31', '1.77.2'), ('2.5.1', '2026-05-02', '1.77.2')]
tauri-plugin-dialog: 2.7.3 2026-09-13T19:00:10.555696Z [('3.0.0-alpha.0', '2026-09-13', '1.77.2'), ('2.7.3', '2026-08-31', '1.77.2'), ('2.7.2', '2026-07-18', '1.77.2')]
tauri-plugin-opener: 2.5.5 2026-09-13T19:00:44.488856Z [('3.0.0-alpha.0', '2026-09-13', None), ('2.5.5', '2026-08-31', None), ('2.5.4', '2026-05-02', None)]
tauri-plugin-log: 2.9.1 2026-09-13T19:01:23.193444Z [('3.0.0-alpha.0', '2026-09-13', '1.77.2'), ('2.9.1', '2026-08-31', '1.77.2'), ('2.9.0', '2026-07-13', '1.77.2')]
tauri-plugin-window-state: 2.4.1 2026-09-13T19:03:51.638688Z [('3.0.0-alpha.0', '2026-09-13', '1.77.2'), ('2.4.1', '2025-10-27', '1.77.2'), ('2.4.0', '2025-07-20', '1.77.2')]
tauri-plugin-positioner: 2.3.4 2026-09-13T19:02:10.331213Z [('3.0.0-alpha.0', '2026-09-13', '1.77.2'), ('2.3.4', '2026-08-31', '1.77.2'), ('2.3.3', '2026-07-13', '1.77.2')]
webkit2gtk: 2.0.2 2025-12-16T11:13:04.461679Z [('2.0.2', '2025-12-16', None), ('2.0.1', '2023-10-26', None), ('2.0.0', '2023-10-03', None)]
tauri-runtime-cef: 3.0.0-alpha.1 [('3.0.0-alpha.1', '2026-09-15'), ('3.0.0-alpha.0', '2026-09-13')]
```

## crates.io, integrity-related crates (format: max_stable_version updated_at total_downloads)

```
sha2: 0.11.0 2026-03-25 942938203
minisign-verify: 0.2.5 2026-03-03 13881037
ed25519-dalek: 3.0.0 2026-07-06 211101037
auto-launch: [('0.6.0', '2026-01-10'), ('0.5.0', '2023-09-10'), ('0.4.0', '2022-09-10'), ('0.3.0', '2022-08-24'), ('0.2.0', '2022-01-15')]
```

## npm registry (format: dist-tags, publish time of `latest`)

```
@tauri-apps/cli: {'latest': '2.11.4', 'next': '3.0.0-alpha.1'} 2026-06-28T17:58:47.722Z
@tauri-apps/api: {'next': '3.0.0-alpha.0', 'latest': '2.11.1'} 2026-06-17T13:41:27.442Z
@tauri-apps/plugin-notification: {'latest': '2.4.0', 'next': '3.0.0-alpha.0'} 2026-08-31T11:20:55.239Z
@tauri-apps/plugin-autostart: {'latest': '2.5.1', 'next': '3.0.0-alpha.0'} 2025-10-27T10:32:14.759Z
@tauri-apps/plugin-updater: {'latest': '2.11.0', 'next': '3.0.0-alpha.0'} 2026-08-31T11:22:05.060Z
create-tauri-app: {'next': '2.0.0', 'latest': '4.7.4'} 2026-09-04T19:58:34.805Z
@sveltejs/kit: {'latest': '2.70.3', 'next': '3.0.0-next.27'} 2026-08-18T15:02:00.715Z
svelte: {'next': '5.0.0-next.272', 'latest': '5.57.0'} 2026-08-28T23:33:45.038Z
vite: {'alpha': '6.0.0-alpha.24', 'previous': '7.3.6', 'latest': '8.3.0', 'beta': '8.3.0-beta.1'} 2026-09-10T11:30:26.283Z
@sveltejs/adapter-static: {'latest': '3.0.10', 'next': '4.0.0-next.4'} 2025-10-02T14:22:08.571Z
@sveltejs/vite-plugin-svelte: {'beta': '2.0.0-beta.3', 'version-2': '2.5.3', 'version-4': '4.0.4', 'version-5': '5.1.1', 'next': '7.0.0-next.1', 'latest': '7.3.0'} 2026-08-08T05:02:42.306Z
```

## Workspace MSRV declarations in source (git clones, 2026-09-15)

- tauri `dev` branch, commit bca4ca58da02f182ef00ef1165e40e400a3cd8dd (2026-09-15), `Cargo.toml` line 46: `rust-version = "1.90"`; `crates/tauri/Cargo.toml` line 3: `version = "2.11.5"`.
- tauri `dev` branch pending change file `.changes/msrv-1.90.md`: "Set MSRV to 1.90." (bump type `minor:changes` for tauri, tauri-utils, tauri-runtime, tauri-runtime-wry, tauri-codegen, tauri-macros, tauri-plugin, tauri-build, tauri-cli, tauri-macos-sign, tauri-bundler, tauri-driver).
- tauri `v3` branch, commit f44d11142d641534ac0857490df9168550353de7 (2026-09-15), `Cargo.toml` line 49: `rust-version = "1.95"`.
- tauri `dev` branch `crates/tauri/Cargo.toml` line 92: `tray-icon = { version = "0.24", default-features = false, features = [`
- tauri `v3` branch `crates/tauri/Cargo.toml` line 100: `tray-icon = { version = "0.25", default-features = false, features = [`
