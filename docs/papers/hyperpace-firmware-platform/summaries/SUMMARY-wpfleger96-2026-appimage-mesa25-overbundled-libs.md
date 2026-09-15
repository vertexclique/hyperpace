# SUMMARY: wpfleger96-2026-appimage-mesa25-overbundled-libs

## Claim
A default Tauri 2.11 AppImage built on ubuntu-24.04 starts headless on Ubuntu 26.04 (Mesa 25, GLib 2.88). The bundled `libwayland-client` makes `eglGetDisplay` fail with `EGL_BAD_PARAMETER`, so WebKitWebProcess aborts with no user-visible error.

Two further defects:
- AppRun exports an empty GStreamer plugin path.
- Bundled WebKit helpers can pick up system helpers of a different version.

## Method
LD_PRELOAD bisection of the bundled libraries on the reporter's machine.

## Result
- A maintainer said that when they update linuxdeploy to get the exclusion, "every time we try this, the appimages break even more".
- The maintainer pointed to the experimental bundler in PR 12491, which "requires building on arch".
- User workaround: `LD_PRELOAD` the host `libwayland-client`.

## Evidence tier
3 (report and bisection), 2 (maintainer replies).

## Performance
Not applicable (startup failure).

## Correctness
The EGL failure mechanism is well supported by the bisection and matches issue 15976. The reporter partly corrected the GStreamer claim. Coverage of distributions beyond Ubuntu 26.04 is by analogy.

## Relevance to hyperpace
CachyOS and Arch ship current Mesa. The default Tauri 2 AppImage is therefore at real risk of launching without a window on the operator's own distro, and must be tested early.

Mitigations to weigh:
- post-process the AppDir to drop the display-stack libraries;
- wait for PR 15662 (`excludeLibraries`);
- try the experimental sharun bundler;
- favour native packages on Linux.
