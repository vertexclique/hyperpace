# SUMMARY: tauri-2026-v3-alpha-release-notes

## Claim
Tauri 3.0.0-alpha.0 was published 2026-09-13, and alpha.1 on 2026-09-15. The alpha:
- sets MSRV 1.95;
- selects the webview runtime at build time (`tauri_runtime_wry` or `tauri_runtime_cef`);
- gates GTK behind `gtk3`/`gtk4` features;
- removes the `tauri::tao`/`tauri::wry` re-exports;
- switches the Linux tray to the ksni StatusNotifierItem backend by default, "dropping the libayatana-appindicator system dependency".

## Method
- GitHub release bodies, fetched through the API; dates come from `published_at`.
- `.changes` files and Cargo manifests on the `v3` branch.

## Result
- The v3 wry runtime still pins `webkit2gtk = "=2.0"` (webkit2gtk-4.1, GTK3), so Linux rendering with wry is unchanged.
- The CEF runtime selects GTK4.
- The updater's `v1Compatible` artifacts are slated for removal in v3.
- A maintainer said in July 2026: "we'll be switching out appindicator for ksni soon anyway".

## Evidence tier
1 (release notes and source).

## Performance
The ACL embedded in the binary shrinks, because commands are stored as lists. CEF bundles Chromium; its size and memory cost were not measured here.

## Correctness
- **Proven:** dates, MSRV, the breaking-change list, and ksni as the default on the v3 branch.
- **Not proven:** the timeline to v3 stable.
- **Not proven:** whether tauri-action v1 builds v3 alphas. Its v1 release notes drop support for "unstable v2 (alpha, beta, rc)"; how it handles v3 prereleases is unverified.
- A WebFetch summarizer misdated alpha.0 to 2024; the API date is authoritative.

## Relevance to hyperpace
A project starting now must choose between:
- **2.11 stable:** mature plugins, but an AppIndicator tray with no tooltip on Linux.
- **3.0 alpha:** a ksni tray with tooltip support and no libayatana runtime dependency, but breaking churn and alpha plugins.

This is an operator decision. A migration later is likely either way.
