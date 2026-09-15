# SUMMARY: tauri-2026-http-store-dialog-plugins

## Claim
- **http plugin:** a reqwest re-export (default features `rustls-tls`, `http2`, `charset`, `system-proxy`, `cookies`) with URL allow and deny scopes for JS callers.
- **store plugin:** persists `serde_json::Value` maps to a JSON file. It saves on graceful exit, or after a 100 ms debounce when autosave is on.
- **dialog plugin:** uses rfd 0.16 with GTK3 by default, and has an `xdg-portal` feature.

## Method
The docs and plugin manifests, plus one closed store issue.

## Result
A store deadlock when saving during `RunEvent::Exit` was reported in April 2026 and closed (fix version not verified).

## Evidence tier
1 (docs, manifests), 3 (issue).

## Performance
Not measured.

## Correctness
Plugin scope rules apply to JS only; Rust callers use reqwest directly.

## Relevance to hyperpace
- **Firmware downloads:** these run in Rust, so plain `reqwest` with rustls, streamed to a temp file with a running SHA-256, is the least code. The http plugin adds nothing unless the webview fetches.
- **Settings:** the store plugin, or a small serde struct written atomically. The plugin brings debounce and JS access.
- **File import and export** (profiles or firmware): the dialog plugin, preferring `xdg-portal` on Wayland (to verify).
