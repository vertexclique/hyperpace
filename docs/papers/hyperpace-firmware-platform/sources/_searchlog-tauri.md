# Search log: stream "tauri"

Date: 2026-09-15. Scratch clones and downloads under the session scratchpad `research-bin/tauri/`. No binaries were executed, no packages installed, no USB or hidraw nodes touched.

## Registry and repository snapshots

| Angle | Command or target | Yield |
|---|---|---|
| Crate versions | crates.io API for tauri, tauri-build, tauri-bundler, tauri-cli, tauri-runtime-wry, wry, tao, tray-icon, 13 plugins, webkit2gtk, tauri-runtime-cef, sha2, minisign-verify, ed25519-dalek, auto-launch | Current 2.11.x line; 3.0.0-alpha.0/1 discovered (not in memory) |
| npm versions | registry.npmjs.org for @tauri-apps/cli, api, 3 plugins, create-tauri-app, svelte, kit, vite, adapter-static, vite-plugin-svelte | Current JS pins |
| Source clones | tauri `dev` (bca4ca58), tauri `v3` (f44d1114), plugins-workspace `v2` (0850317b), tauri-docs `v2` (a6b59b78), create-tauri-app `dev` (890e6208), tray-icon (aab34f8a), auto-launch (5a77b66c); crates.io tarball auto-launch 0.5.0; local cargo registry copy of libappindicator-sys 0.9.0 | Most tier 1 excerpts |
| Branch and tag listing | `git ls-remote` heads and tags | `v3` and `next` branches, tauri-v3.0.0-alpha.0/1 tags |
| Releases | `gh api` release bodies tauri-v3.0.0-alpha.0, alpha.1, v2.11.0, v2.10.0; tauri-action releases | MSRV 1.95, ksni tray, dates; action v1.0.0 |
| Benchmarks | raw tauri-recent-linux.json and tauri-data-linux.json; bench harness source | Binary size and peak memory, method (mprof) |
| Kernel | raw fs/binfmt_elf.c | ELF segments mapped with vm_mmap |

## Web searches (WebSearch)

1. "Tauri 3.0 alpha release announcement 2026": release pages; no blog post. Summarizer invented a "latest tauri-cef-v3.0.0-alpha.7 May 2026" claim that was not verified and not used.
2. WebFetch v2.tauri.app/blog: newest post is Board Elections 2026; no posts for 2.x minors or 3.0 (dead end for changelog).
3. WebFetch GitHub release tauri-v3.0.0-alpha.0: summarizer misdated it to 2024; corrected via `gh api` published_at.
4. "tauri WEBKIT_DISABLE_DMABUF_RENDERER blank window NVIDIA wayland issue": Linux graphics docs page, issue 9394, downstream issues, a third-party quirk plugin (not read).
5. "tauri AppImage Arch Linux EGL_BAD_PARAMETER OR Could not create default EGL display webkit": downstream Arch and Fedora reports, pointer to the portable AppImage PR.
6. "tauri tray app idle memory usage RAM hidden window webview benchmark MB": Medium posts (tier 4, not used), discussion 6308, issue 14088, discussion 11553.
7. "Tauri system tray icon GNOME not showing AppIndicator extension required Linux": tray-icon 177, tauri 13599, a linuxiac how-to (not fetched).
8. "tauri v2 AppImage file size MB bundled webkit2gtk AppImage too large": discussion 6918, two blogs (sizes quoted only from search summary, not verified).
9. "tauri-plugin-notification macOS notifications not showing bundle identifier signed app notify-rust": only a third-party plugin page claiming a signed bundle is required (unverified); an old core commit setting bundle id.
10. "webkitgtk 2.50 OR 2.52 regression tauri app crash blank Linux 2026": nothing version-specific; found phase-rs 8614 (idle CPU on NVIDIA).
11. "tauri tray app memory usage idle MB linux WebKitWebProcess hidden window measured": issue 5889, issue 7183, downstream memory-growth reports (not opened). No measured idle numbers for a hidden-window tray app found.
12. (scope extension) "rust include_bytes large file compile time memory usage issue rustc slow tens of megabytes": rust 65818, 103607, forum 59237.
13. (scope extension) "Apple code signature sealed resources bundle Contents/Resources CodeResources modification invalidates signature documentation": TN2206, Code Signing Tasks.
14. (scope extension) "AppImage runtime squashfs FUSE mount compression zstd appimagetool default comp docs.appimage.org": AppImage architecture docs, appimagetool 69.
15. (scope extension) "Authenticode Portable Executable signature format hash covers entire PE file sections excluded checksum certificate table Microsoft": Microsoft Learn page.

## WebFetch targets

- doc.rust-lang.org include_bytes: docs text.
- AppImageSpec draft.md: type 2 requirements, `.sha256_sig`, `.upd_info`.
- users.rust-lang.org thread 59237: `.rodata` sharing answer.
- Apple TN2206: sealed resources, Mach-O append prohibition.
- learn.microsoft.com understanding-pe-signatures: Authenticode exclusions.
- docs.appimage.org architecture: SquashFS plus FUSE.
- raw mac-notification-sys README on `main`: 404 (dead end); retried via `gh api .../readme` on `master`: no signing statement.

## GitHub searches (gh CLI)

| Query | Yield |
|---|---|
| prs tauri "appimage" | 12491 portable AppImage (open), 15662 excludeLibraries (open), 15917, 15786, 15804, 14841 |
| issues tauri "EGL_BAD_PARAMETER" (first attempt failed: wrong JSON field `comments`; retried with `commentsCount`) | 15976, 15665, older Fedora issues |
| issues tauri "EGL AppImage" | no results |
| issues tauri "appimage arch" | no results |
| issues tauri "appimage size" | two 2021-2022 items, not relevant |
| issues tauri "dmabuf" sorted by created | recent Linux rendering issues list |
| issues tauri "webkitgtk 2.50" | nothing version-specific |
| issues tauri "tray gnome"; tray-icon "gnome" | tray-icon 177, 336, 85 |
| issues plugins-workspace "notification windows" | 3512, 2150, 2902 |
| issues plugins-workspace "notification macos permission" | no results |
| issues plugins-workspace "notification not showing" | no results |
| prs plugins-workspace "notification" sorted | 3588 (GNOME), 3541, 3540, 3546 (autostart) |
| issues tauri "hidden window cpu usage" | no results |
| issues tauri "idle cpu linux webkit" | no results |
| issues tauri "backdrop-filter" | 14811, 14040, 12804, 15512, 6876 |
| issues plugins-workspace "autostart appimage" | no results |
| issues plugins-workspace "autostart linux" | no results |
| issues plugins-workspace "autostart" sorted | 3166 Flatpak autostart, unrelated others |
| issues plugins-workspace "single-instance" sorted | 3587 Windows race, 3542, 3394 store deadlock |
| issues plugins-workspace "updater appimage" | no results |
| issues tauri "tray set_icon linux" | no results |
| issues tauri "ipc channel leak" | no results |
| issues plugins-workspace "notification powershell" | no results |
| issues plugins-workspace "delta update" | 2672 zsync delta request |
| issues tauri "delta updates" | no results |
| issues tauri "differential update" | no results |
| GraphQL discussions 6308, 11553, 6918 | tray-without-webview answer, process layout, AppImage size |
| Issue and PR reads | 9394, 13599, 14088, 15665, 15976, 8541, 5889, 7183, 14811, 14040, 12491 (+comments), 14841, 15917, 15662; plugins-workspace 2566, 3512, 3540, 3541, 3588, 3546, 1922, 3166, 3587, 3394; tray-icon 177, 336; rust 65818, 103607; phase-rs 8614; appimagetool 69 |

## Local greps

- tauri-docs for `include_bytes`: only window-menu and an old blog post (no guidance on large blobs).
- tauri-docs for `checksum|sha256|integrity`: no runtime integrity feature.
- tauri-docs cross-compile grep: first attempt failed on a fish glob (`--include=*.mdx`), retried with quoting; only Windows NSIS cross-compile documented; macOS host restriction confirmed from bundler source instead.
- tauri and plugins source for `WEBKIT_DISABLE`: no matches (framework sets none).

## Adversarial verification and stopping

Headline claims checked against open issues: AppImage on new Mesa (15665, 15976 open), portable AppImage status (12491 open), tray Linux limits (177, 336, 13599), notifications on GNOME (2566 open) and Windows (3512 open), single-instance race (3587 open), autostart Flatpak (3166 open), benchmark validity (5889).

Stopping: for the original scope, the last eight differently angled searches in a row ("hidden window cpu usage", "idle cpu linux webkit", "updater appimage", "tray set_icon linux", "ipc channel leak", "notification powershell", "delta updates", "differential update") added nothing new, which meets the five-in-a-row rule. For the firmware-blob scope extension the rule was not fully met: Windows PE and macOS Mach-O demand paging, the effective compressor of Tauri AppImages, and macOS runtime re-validation frequency remain unsourced and are labelled unverified in the summaries.

## Not found after searching

- Measured idle RAM or CPU of a Tauri tray app with a hidden vs destroyed webview (searches 6 and 11, issue searches "hidden window cpu usage", "idle cpu linux webkit").
- A primary source that macOS notifications from tauri-plugin-notification require a signed bundle (search 9, mac-notification-sys README).
- Any Tauri built-in delta update mechanism (delta and differential searches).
- Any Tauri built-in integrity check for bundled resources (docs grep).
