# SUMMARY: tauri-2026-windows-installer-webview2

## Claim
Windows bundles are either MSI (WiX v3, buildable only on Windows) or an NSIS setup.exe. NSIS can be cross-built from Linux with `cargo-xwin`, LLVM/LLD and NSIS, but this "is not tested as much", and signing needs an external tool.

WebView2 install modes and the size they add:

| Mode | Added size |
|---|---|
| Download bootstrapper (default) | 0 MB |
| Embedded bootstrapper | ~1.8 MB |
| Offline installer | ~127 MB |
| Fixed runtime | ~180 MB |

WebView2 ships with Windows 10 (April 2018+) and Windows 11. The macOS `.app` and `.dmg` bundling code compiles only on a macOS host.

## Method
The official docs, and the `cfg` gates in `tauri-bundler/src/bundle.rs`.

## Result
`minimumWebview2Version` can force a runtime update for newer features.

## Evidence tier
1.

## Performance
Installer size deltas as listed.

## Correctness
The sizes are docs estimates. The macOS host restriction is proven for the Tauri bundler; third-party cross toolchains for macOS were not investigated.

## Relevance to hyperpace
- The Linux dev machine can produce Linux bundles and, with effort, an unsigned NSIS installer.
- MSI and all macOS artifacts need Windows and macOS CI runners.
- The default bootstrapper keeps the installer small.
- A raw exe also works on Windows 10/11 when WebView2 is present, except that bundle resources and notification identity need the installer.
