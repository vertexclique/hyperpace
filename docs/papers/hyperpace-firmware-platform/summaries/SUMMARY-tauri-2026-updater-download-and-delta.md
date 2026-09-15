# SUMMARY: tauri-2026-updater-download-and-delta

## Claim
The Tauri updater downloads the complete update artifact into an in-memory `Vec<u8>`, verifies the minisign signature over the whole buffer, then installs. There is no delta or differential update support. The only related item is an open maintainer feature request for AppImage zsync deltas, tied to the unmerged experimental bundler.

## Method
Read `Update::download` and `verify_signature` in plugins-workspace `v2`, and searched issues for delta and differential updates.

## Result
- Tauri repo searches for "delta updates" and "differential update" returned nothing.
- linuxdeploy-plugin-appimage can embed update information and emit a `.zsync` file for external updaters (`LDAI_UPDATE_INFORMATION`).

## Evidence tier
1 (source), 2 (feature request).

## Performance
- Peak RAM during an update is at least the artifact size, plus install-time copies.
- Transfer size equals the full artifact on every release.

## Correctness
Proven for updater 2.11.0. zsync for Tauri AppImages is not available in the stable bundler.

## Relevance to hyperpace
Embedding all firmware (several MB to tens of MB) adds that size to every app update and to the updater's peak memory, even when no firmware changed.

Options to present:
- accept it;
- compress the blobs before embedding;
- split firmware into a separately versioned, signed package with its own offline cache.

The operator wants offline install and revert, so this needs an explicit decision.
