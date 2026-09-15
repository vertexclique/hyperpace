# SUMMARY: tauri-2026-registry-version-snapshot

## Claim
On 2026-09-15 the stable Tauri line is 2.11: tauri 2.11.5 (2026-07-01), tauri-build 2.6.3, tauri-cli and @tauri-apps/cli 2.11.4, @tauri-apps/api 2.11.1. A 3.0 alpha line started 2026-09-13 (tauri 3.0.0-alpha.1 on 2026-09-15).

Current plugins:
- notification 2.4.0
- autostart 2.5.1
- single-instance 2.4.4
- updater 2.11.0
- http 2.6.0
- store 2.4.4
- fs 2.5.2
- dialog 2.7.3
- log 2.9.1
- window-state 2.4.1
- positioner 2.3.4

Frontend: svelte 5.57.0, @sveltejs/kit 2.70.3, vite 8.3.0, adapter-static 3.0.10, vite-plugin-svelte 7.3.0, create-tauri-app 4.7.4.

## Method
- Direct queries of the crates.io API and the npm registry.
- `rust-version` lines read from shallow git clones of the `dev` and `v3` branches.

## Result
Declared MSRV:
- 1.77.2 on the published 2.11.5.
- The `dev` branch carries an unreleased "Set MSRV to 1.90." change for the next 2.x minor.
- `v3` declares 1.95.
- tray-icon 0.25.0 declares 1.90.

Tauri 2.11 depends on tray-icon 0.24; v3 depends on 0.25.

## Evidence tier
1 (registry metadata and source manifests).

## Performance
Not applicable.

## Correctness
- Published versions and dates are authoritative.
- `rust_version` is declared by the crate author; this stream did not confirm it in CI.
- The 1.90 bump is pending and could change before release.
- npm and crates versions are numbered independently (api 2.11.1 vs crate 2.11.5), which is normal for Tauri.

## Relevance to hyperpace
rustc 1.98.0 on the dev machine satisfies every line. The 2.x CLI is not installed. It can come from cargo (`tauri-cli` 2.11.4) or from the npm devDependency (`@tauri-apps/cli` 2.11.4); which one is an operator decision. Pin tauri and plugin minor versions together.
