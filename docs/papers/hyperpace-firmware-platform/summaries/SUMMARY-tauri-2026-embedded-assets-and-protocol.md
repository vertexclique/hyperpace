# SUMMARY: tauri-2026-embedded-assets-and-protocol

## Claim
- `frontendDist` files are brotli-compressed at compile time (quality 9 for release, 2 for debug) and embedded into the application binary through `generate_context!`.
- They are served from `tauri://localhost` on macOS and Linux, and from `http(s)://tauri.localhost` on Windows and Android.
- CSP is enforced only when configured. Tauri then injects hashes and nonces for bundled scripts and styles.

## Method
Read:
- tauri-codegen `embedded_assets.rs`, tauri-utils `assets.rs` and `manager/mod.rs`;
- the Cargo feature list;
- the CSP and App Size docs.

## Result
- `compression` is a default feature.
- At runtime, `Assets::get` decompresses the whole asset into a fresh `Vec` on every request.
- 2.11.3 made the `tauri://` protocol handlers load asynchronously.
- Size levers: the release profile (`lto`, `codegen-units = 1`, `opt-level = "s"`, `panic = "abort"`, `strip`) and `build.removeUnusedCommands` (tauri 2.4+).

## Evidence tier
1.

## Performance
No decompression timings were found. Full decompression per request is fine for a small Svelte bundle but costly for large assets.

## Correctness
Proven from source for the 2.11.5 tree. The frontend is embedded; the webview engine is not (WebView2 on Windows, system webkit2gtk-4.1 on Linux, WKWebView on macOS).

## Relevance to hyperpace
- The "one binary with every asset embedded" goal holds for UI assets on all three OSes.
- The binary still needs the platform webview at runtime; only an AppImage bundles it.
- Use a strict CSP, since no remote origins are needed.
- Do not route firmware blobs through `frontendDist`: they would be exposed to the webview and decompressed in full on every request.
