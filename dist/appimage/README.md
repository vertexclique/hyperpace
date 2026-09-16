# AppImage

`cargo tauri build --bundles appimage` builds the raw `.AppImage`, but do
not stop there: run `scripts/package-appimage.sh` (`make package-appimage`
from the repo root) instead, which builds deb/rpm/appimage and then repairs
the AppImage. There are no format-specific config inputs to add beyond that
repair step: the AppImage target always uses the bundler's built-in
desktop-entry template (it has no `desktopTemplate` option, unlike deb and
rpm), driven by the same shared `bundle.category` and `bundle.shortDescription`
fields documented in `../README.md`.

## Why it needs repairing

The default AppImage bundles its own copies of the Wayland/X11 client
libraries WebKitGTK links against. On a host whose own display stack is
newer than the ones the bundler linked against (current Mesa, or a
proprietary NVIDIA driver, both real on this repo's own dev machine), that
produces a window that never appears, confirmed here by actually launching
both the default and the repaired build:

- **Default build, launched as-is**: `Failed to create GBM buffer of size
  960x640: Invalid argument`, twice, then nothing ever paints. On this
  machine (NVIDIA, Wayland) that is WebKitGTK's DMABUF renderer failing
  against the proprietary driver
  (`../../docs/papers/hyperpace-firmware-platform/summaries/SUMMARY-tauri-2026-linux-graphics-docs.md`);
  on an AMD or Intel Mesa host it can instead be the bundled Wayland/X
  client libraries colliding with newer host ones, aborting with
  `EGL_BAD_PARAMETER`
  (`../../docs/papers/hyperpace-firmware-platform/summaries/SUMMARY-wpfleger96-2026-appimage-mesa25-overbundled-libs.md`,
  `SUMMARY-xcrong-2026-appimage-egl-fedora44.md`). Confirmed on this
  machine that the library prune alone does not fix the NVIDIA case; the
  two are separate bugs sharing one symptom.
- **`scripts/package-appimage.sh`'s output, launched**: clean startup log,
  no GBM errors, a real window with the app's UI rendered (verified with a
  screenshot, not just a quiet log). Two things make this work together:
  the script deletes the nine bundled libraries
  (`libwayland-client.so.0`, `libwayland-cursor.so.0`, `libwayland-egl.so.1`,
  `libwayland-server.so.0`, `libxkbcommon.so.0`, `libxcb-render.so.0`,
  `libxcb-shm.so.0`, `libXau.so.6`, `libXdmcp.so.6`) that cause the Mesa
  case and repacks (the exact list confirmed by diffing an unpruned AppDir
  against a pruned one, not copied from the research prose); and the app
  binary itself now carries `crates/hyperpace-app/src/gpu_workaround.rs`,
  which detects an NVIDIA GPU on a Wayland/X11 session at startup and
  re-execs with `WEBKIT_DISABLE_DMABUF_RENDERER=1` before the webview is
  created, fixing the NVIDIA case. The binary fix ships in every package
  format, not only the AppImage; the library prune is AppImage-specific,
  since only the AppImage bundles its own copies of those libraries.

Built and verified on this machine: `Hyperpace_0.1.0_amd64.AppImage`,
119,478,776 bytes (113.94 MiB). The unpruned build the bundler itself
produces is kept alongside it as `Hyperpace_0.1.0_amd64.default-unpatched.AppImage`
(119,749,112 bytes, 114.20 MiB) for comparison; it is not shippable, and the
script never overwrites it once present, so the two stay comparable across
reruns.

An AppImage has no package manager and no maintainer scripts, so it cannot
install `../udev/70-hyperpace.rules` itself: run the install command in the
repo README once, before the first launch. Without it, opening the real
hidraw node fails with a permission error; the plan calls for
`hyperpace-device` to surface the exact install command in that error rather
than a bare "permission denied", which is not implemented as of this writing
(`crates/hyperpace-device/src/hid.rs` propagates the raw hidapi error today).

Mark the file executable and run it directly: `chmod +x Hyperpace_0.1.0_amd64.AppImage &&
./Hyperpace_0.1.0_amd64.AppImage`. No install step, no root needed to run it.
