# Packaging

Every packaging input lives here, one directory per target, each with its
own `README.md`. Builds go through Tauri's own bundler
(`cargo tauri build --bundles <target>` from `crates/hyperpace-app`, or
`--bundles all`); the one exception is `scripts/package-appimage.sh`
(`make package-appimage` from the repo root), which runs that same bundler
and then repairs the AppImage (see `appimage/README.md` and "Validation"
below for why it needs repairing). No package build needing root was run,
and none was installed: see "Validation" for exactly what was built and
what was actually launched.

| Directory | Target | Built by |
|---|---|---|
| `udev/` | the Linux permission rule, shared by deb, rpm, PKGBUILD and AppImage | n/a, installed directly |
| `deb/` | Debian/Ubuntu `.deb` | Tauri's bundler |
| `rpm/` | Fedora/openSUSE `.rpm` | Tauri's bundler |
| `appimage/` | portable `.AppImage` | Tauri's bundler |
| `arch/` | Arch `PKGBUILD` (`makepkg`) | not Tauri; no `pacman` target exists for it |
| `windows/` | NSIS installer | Tauri's bundler |
| `macos/` | `.app` / `.dmg` | Tauri's bundler |

## Wiring into the Tauri config

`crates/hyperpace-app/tauri.conf.json` is owned by the app crate, not by
this change. As of this validation pass (checked by reading the live file,
not assumed), the `linux.deb` and `linux.rpm` blocks below are already
present in it; the `windows.nsis.installerHooks` block is not, so
`dist/windows/hyperpace-hooks.nsh` exists but is not wired into any build
yet. Whoever owns that file adds the missing piece to its `bundle` object
(paths are relative to `tauri.conf.json` itself, i.e.
`crates/hyperpace-app/`):

```json
{
  "bundle": {
    "category": "Utility",
    "shortDescription": "Configure your mouse: buttons, DPI, lighting, macros and firmware",
    "linux": {
      "deb": {
        "files": { "/usr/lib/udev/rules.d/70-hyperpace.rules": "../../dist/udev/70-hyperpace.rules" },
        "postInstallScript": "../../dist/udev/reload-udev-rules.sh",
        "postRemoveScript": "../../dist/udev/reload-udev-rules.sh"
      },
      "rpm": {
        "files": { "/usr/lib/udev/rules.d/70-hyperpace.rules": "../../dist/udev/70-hyperpace.rules" },
        "postInstallScript": "../../dist/udev/reload-udev-rules.sh",
        "postRemoveScript": "../../dist/udev/reload-udev-rules.sh"
      }
    },
    "windows": {
      "nsis": {
        "installerHooks": "../../dist/windows/hyperpace-hooks.nsh"
      }
    }
  }
}
```

No `macos` block is needed (see `macos/README.md` for why) and no
`appimage` block is needed (see `appimage/README.md`): AppImage has no
`desktopTemplate` or maintainer-script hooks to point anywhere, and picks up
`category` / `shortDescription` the same way deb and rpm do.

`deb.files` and `rpm.files` map destination path to source path (verified
against `tauri-bundler`'s `copy_custom_files`, which takes the map key as
the package-relative install path and the value as the local file to copy);
getting this backwards fails the build looking for a source file at the
destination path, so it is called out here rather than left to be
discovered by trial and error.

Arch has no Tauri bundler target, hence a hand-written `PKGBUILD` in
`arch/` instead of a config snippet; see `arch/PKGBUILD` for exactly how it
builds and packages.

## The Linux permission step

Every Linux target ends up needing the same one-line install for a hidraw
node the logged-in user can use without root:

```sh
sudo install -Dm644 dist/udev/70-hyperpace.rules /etc/udev/rules.d/70-hyperpace.rules
sudo udevadm control --reload-rules
sudo udevadm trigger --subsystem-match=hidraw
```

The deb, rpm and Arch packages run the equivalent of the last two lines
automatically as a maintainer script (`udev/reload-udev-rules.sh` for deb
and rpm, inlined in `arch/hyperpace.install` for Arch) after installing the
rule file itself to `/usr/lib/udev/rules.d/`, so only the AppImage and a
manual build need the command above. See the repo `README.md` for the
user-facing version of this step.

## Validation

Four packages are built and real, from the current source tree, on this
machine (CachyOS/Arch, KDE Plasma, Wayland, an NVIDIA RTX 4000 Ada GPU on
the proprietary driver):

| Package | Command | Artifact | Size |
|---|---|---|---|
| `.deb` | `cargo tauri build --bundles deb` | `target/release/bundle/deb/Hyperpace_0.1.0_amd64.deb` | 18,358,286 bytes (17.51 MiB) |
| `.rpm` | `cargo tauri build --bundles rpm` | `target/release/bundle/rpm/Hyperpace-0.1.0-1.x86_64.rpm` | 12,119,380 bytes (11.56 MiB) |
| `.AppImage` | `scripts/package-appimage.sh` (`make package-appimage`) | `target/release/bundle/appimage/Hyperpace_0.1.0_amd64.AppImage` | 119,478,776 bytes (113.94 MiB) |
| Arch `.pkg.tar.zst` | `cd dist/arch && makepkg -f --nodeps` | `dist/arch/hyperpace-0.1.0-1-x86_64.pkg.tar.zst` | 12,546,890 bytes (11.97 MiB) |

`--nodeps` on the Arch build skips only pacman's own dependency check
(`cargo`/`npm` are rustup/nvm managed on this machine, not pacman packages,
so the check would otherwise refuse a build that in fact succeeds); it
installs nothing and needs no root, so it stays inside the same bounds as
everything else here. The runtime `depends` themselves
(`webkit2gtk-4.1`, `gtk3`, `libayatana-appindicator`) were separately
confirmed present in this machine's own `pacman` database, as before.

None of the four was installed system-wide (`dpkg -i` / `rpm -i` /
`makepkg -i` / a real `pacman -U` all need root, never used here). Only the
AppImage needs no install step, so it is the only one actually launched:

- **Unpatched** (the Tauri bundler's own, unpruned output): aborts with no
  window. `Failed to create GBM buffer of size 960x640: Invalid argument`,
  twice, then the process just sits there; nothing crashes, nothing ever
  paints. This is WebKitGTK's DMABUF renderer failing against this
  machine's NVIDIA driver over Wayland, not literally the `EGL_BAD_PARAMETER`
  the two AppImage research summaries name (that one is an AMD/Intel Mesa
  bug, from the bundled Wayland/X client libraries colliding with newer
  host ones). Confirmed fresh, twice, with and without those libraries
  pruned: the prune alone does not fix this machine's failure.
- **The actual fix is in the app, not the package.**
  `crates/hyperpace-app/src/gpu_workaround.rs` detects a Linux Wayland or
  X11 session with an NVIDIA GPU present (DRM sysfs vendor id, or the
  `nvidia` kernel module) at startup, before the webview exists, and
  re-execs itself with `WEBKIT_DISABLE_DMABUF_RENDERER=1`
  (`docs/papers/hyperpace-firmware-platform/summaries/SUMMARY-tauri-2026-linux-graphics-docs.md`
  is explicit that shipping that variable unconditionally is the wrong
  fix, since it silently degrades WebGL and masks the renderer string on
  hardware that never had the bug, hence the detection instead of a bare
  `env::set_var` at startup). Because the fix lives in the binary, every
  package format carries it, not only the AppImage.
- **Patched and rebuilt** (library prune plus the binary above, i.e. what
  `scripts/package-appimage.sh` now produces): launches clean. The startup
  log shows the detection firing, the re-exec, and then no GBM errors at
  all; a screenshot of the live window (taken this pass, not reused from an
  older run) shows the real UI rendering, not a blank surface.

The nine bundled libraries the AppImage prune removes
(`libwayland-client.so.0`, `libwayland-cursor.so.0`, `libwayland-egl.so.1`,
`libwayland-server.so.0`, `libxkbcommon.so.0`, `libxcb-render.so.0`,
`libxcb-shm.so.0`, `libXau.so.6`, `libXdmcp.so.6`) were confirmed by diffing
an unpruned AppDir's `usr/lib` against a pruned one, not copied from the
research summaries' prose: the two reports name up to ten, including
`libxcb-randr`, which this app's own dependency closure never actually
pulls in, hence nine here.

`deb`/`rpm`/the Arch package were checked the way they can be checked
without installing:

- `.deb`: `ar t` lists exactly `debian-binary`, `control.tar.gz`,
  `data.tar.gz`; `control.tar.gz`'s `control` file and `data.tar.gz`'s file
  list were extracted and read directly (not assumed), confirming the udev
  rule, desktop entry and icon land where `tauri.conf.json`'s `deb.files`
  says. One real, harmless defect found and left as a documented gap (not
  fixed, out of this pass's scope): `Depends` lists
  `libwebkit2gtk-4.1-0`/`libgtk-3-0` twice each, because `tauri-bundler`
  auto-detects them via `ldd` and then also appends `tauri.conf.json`'s own
  manually configured `deb.depends` list for the same two libraries. `dpkg`
  tolerates a duplicate `Depends` entry, so this does not break the
  package; whoever owns `tauri.conf.json` next should drop the manual
  `depends` array now that auto-detection already covers it.
- `.rpm`: no `rpm`/`rpm2cpio` exists on this Arch machine, so the payload
  was decoded by hand (the RPM lead and header, then the zstd-compressed
  cpio payload) and its file list read directly, confirming the same four
  files land at the same paths as the deb.
- Arch: a real `makepkg -f --nodeps` build (not just `--printsrcinfo`,
  network was available this pass), `bsdtar tf` and `bsdtar xOf .PKGINFO`
  read directly, confirming the package name, version, dependencies and
  file list match the `PKGBUILD`. No `LICENSE-MIT`/`LICENSE-APACHE` files
  exist at the repo root, so `usr/share/licenses/hyperpace/` in the built
  package is empty; the `PKGBUILD`'s own `|| true` already tolerates their
  absence rather than failing the build, so this is a pre-existing,
  harmless gap, not a new defect.

Everything below this point in the file predates this pass and was checked
the way it can be checked without running an installer, a package build, or
touching a real device:

- `dist/udev/70-hyperpace.rules`: `udevadm verify` passes; its vendor and
  product ids (`3554`, `fb16`, `fb14`) were cross-checked against
  `hyperpace-device`'s own `VENDOR_ID`/`PRODUCT_ID_RECEIVER`/
  `PRODUCT_ID_WIRED` constants, not just read as asserted.
- `dist/udev/reload-udev-rules.sh`, `dist/arch/PKGBUILD`,
  `dist/arch/hyperpace.install`: `bash -n` (both are read and executed as
  shell) passes.
- `dist/arch/hyperpace.desktop`: `desktop-file-validate` passes; its
  `Categories=Utility;` was checked against `AppCategory::Utility`'s
  freedesktop mapping in `tauri-bundler`'s `category.rs`, which is the same
  source `bundle.category` in `tauri.conf.json` feeds, so the two categories
  cannot drift.
- `dist/arch/PKGBUILD`: `makepkg --printsrcinfo` parses cleanly (run here,
  output reviewed; a full `makepkg` build was not run, see below). Its
  `pkgver()` reads the workspace root `Cargo.toml` and produced `0.1.0`,
  matching `[workspace.package].version`. `depends` (`webkit2gtk-4.1`,
  `gtk3`, `libayatana-appindicator`) matched this machine's own `pacman`
  database (the machine the plan names as the primary Arch target), and
  `libayatana-appindicator` was confirmed as a genuine runtime need (not a
  copy-paste): `tray-icon`'s default Linux backend (`libappindicator`
  feature, on by default) `dlopen`s `libappindicator-sys` at runtime rather
  than link-time, so it never shows up as a build-time link error if
  missing, only as a broken tray icon.
- `dist/windows/hyperpace-hooks.nsh`: not compile-checked, `makensis` is not
  available in this sandbox; see `windows/README.md` for what was instead
  confirmed by reading `tauri-bundler`'s vendored `installer.nsi` template,
  including a real bug this pass fixed (the uninstall-data prompt was firing
  during every in-place upgrade, not just a real uninstall) and a
  documentation gap it closed (the installer's default `webviewInstallMode`
  needs internet on first install on a system without WebView2 already
  present, which the README did not mention).
- `dist/macos/README.md`: its three factual claims (`LSApplicationCategoryType`
  and `NSHumanReadableCopyright` come from the shared `bundle.category` and
  `bundle.copyright` fields; codesigning, and therefore hardened runtime,
  is skipped entirely when no signing identity is configured) were each
  confirmed against `tauri-bundler`'s `macos/app.rs` and `macos/sign.rs`
  rather than taken on faith; no change was needed.
- The Tauri bundle config snippet in this file: its `linux.deb`/`linux.rpm`
  portion is already present in the live `crates/hyperpace-app/tauri.conf.json`
  (confirmed by reading that file directly); its `windows.nsis.installerHooks`
  portion is not (also confirmed directly, see `windows/README.md`), so
  `hyperpace-hooks.nsh` is validated but not yet wired into any Windows
  build. No package was built either way (out of ownership for the missing
  piece, and a full build needs network for `cargo fetch` / `npm ci` plus
  tens of minutes regardless). The `deb.files`/`rpm.files` direction and the
  `installerHooks` hook name were confirmed by reading `tauri-bundler`
  3.0.0-alpha's own source rather than assumed, and separately, reading
  `tauri-cli`'s own `bundle.rs` confirmed that `--bundles <target>` fully
  replaces `tauri.conf.json`'s `bundle.targets` list rather than being
  filtered by it, so the `cargo tauri build --bundles nsis` /
  `--bundles dmg` commands documented in `windows/README.md` and
  `macos/README.md` do not need "nsis"/"dmg" added to `targets` first.

`dpkg -i`, `rpm -i`, `makepkg -i`/`pacman -U` and every installer were
deliberately not run: all of them need root, never used here (see
"Validation" above for the real `makepkg -f --nodeps` build that stops
short of installing). Nothing in this pass ran on Windows or macOS; the
NSIS and macOS bundler claims above are validated by reading the vendored
Rust source of the exact bundler version this project pins, not by
producing or running a Windows or macOS artifact, which still needs a
Windows machine and a Mac respectively.
