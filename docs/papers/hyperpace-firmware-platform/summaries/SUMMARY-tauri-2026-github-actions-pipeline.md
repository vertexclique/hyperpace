# SUMMARY: tauri-2026-github-actions-pipeline

## Claim
The official pipeline uses `tauri-apps/tauri-action@v1` with a matrix of:
- `macos-latest` (aarch64 and x86_64 targets);
- `ubuntu-22.04`, plus `ubuntu-22.04-arm` in public repos;
- `windows-latest`.

On Ubuntu it installs `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf xdg-utils`.

tauri-action 1.0.0 (2026-06-29):
- adds the version to `.app.tar.gz` names;
- points `latest.json` URLs at GitHub URLs;
- drops Tauri v1 and unstable v2 prereleases.

## Method
The docs workflow and the GitHub release notes.

## Result
tauri-action generates the static updater JSON for GitHub Releases.

## Evidence tier
1.

## Performance
Not applicable.

## Correctness
Whether v1 supports Tauri 3 alphas was not verified.

## Relevance to hyperpace
CI is the only practical route to macOS and signed Windows artifacts. As the maintainer suggested, consider a separate AppImage job (an Arch container for the experimental bundler, or a pruning step), apart from the deb and rpm job on ubuntu-22.04.
