# SUMMARY: tauri-2026-linux-packaging-docs

## Claim
- **AppImage:** bundles all dependencies and grows "from the 2-6 MB range to 70+ MB".
- **Build base:** build on the oldest supported base (Ubuntu 22.04 or Debian 12), or hit glibc errors on older systems.
- **deb:** declares `libwebkit2gtk-4.1-0`, `libgtk-3-0`, and `libappindicator3-1` when a tray is used.
- **ARM:** AppImages cannot be cross-compiled.
- **Signing:** AppImage gpg signing is optional, and a failed signing does not fail the build by default.

## Method
The official AppImage, Debian, RPM, prerequisites, Linux signing and webview-versions pages.

## Result
Arch build dependencies: `webkit2gtk-4.1 base-devel curl wget file openssl appmenu-gtk-module libappindicator-gtk3 librsvg xdotool`.

## Evidence tier
1.

## Performance
The size figure (70+ MB) is the docs' own estimate; no measurement method is given.

## Correctness
The "build on oldest base" advice is about glibc. It conflicts with the Mesa and libwayland breakage on the newest distros documented in open issues 15665 and 15976: an old base maximizes glibc reach but bundles old display-stack libraries.

## Relevance to hyperpace
- Building the AppImage on CachyOS (rolling) would demand a very new glibc on target systems. Build in an Ubuntu 22.04 container or in CI instead.
- Also ship deb and rpm: they are small and use the system WebKitGTK.
- Consider an AUR PKGBUILD for Arch users (operator decision).
