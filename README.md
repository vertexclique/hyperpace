# Hyperpace

Hyperpace is a native desktop app that configures your mouse over its USB
HID configuration channel: buttons, DPI stages and colors, polling rate,
lighting, macros, profiles, the receiver, and firmware. It runs in the
background with a tray icon showing battery, and the window can be closed
without exiting. No cloud account, no vendor branding anywhere in the app.

Built from five Rust crates (a protocol codec, the device layer, a firmware
engine, local storage, and a Tauri shell) plus a SvelteKit UI. See
`docs/architecture/api-contract.md` for the crate boundaries and
`docs/plans/hyperpace.md` for the full design.

## Supported hardware

One mouse: USB vendor id `0x3554`, product id `0xFB16` over the 2.4 GHz
receiver or `0xFB14` wired. No other device is recognized.

## Supported systems

| OS | Format | Status |
|---|---|---|
| Linux | `.deb`, `.rpm`, AppImage | primary channel |
| Arch Linux | `PKGBUILD` | first-class; this is what the app is developed and run on |
| Windows | NSIS installer | built, unsigned (SmartScreen will warn) |
| macOS | `.app` / `.dmg` | built, unsigned (Gatekeeper will warn) |

Every build has been run only against the bundled device simulator. No
release here has been verified against real hardware except by the operator
running the app read-only on their own machine; see `docs/plans/hyperpace.md`
for why (the mouse this project configures is the operator's own daily
device, and this project never opens it directly).

## Install

Packaging inputs for every target live in `dist/`; see `dist/README.md` for
how each one is built and what it contains.

**Debian / Ubuntu**
```sh
sudo apt install ./hyperpace_<version>_amd64.deb
```

**Fedora / openSUSE**
```sh
sudo dnf install ./hyperpace-<version>-1.x86_64.rpm
```

**Arch Linux**
```sh
cd dist/arch && makepkg -si
```

**AppImage** (any Linux, no install)
```sh
chmod +x Hyperpace-<version>.AppImage
./Hyperpace-<version>.AppImage
```

**Windows**: run the `.exe` installer and follow the prompts.

**macOS**: open the `.dmg` and drag Hyperpace to Applications. Unsigned
builds need an explicit right-click, Open the first time to clear Gatekeeper.

### Linux permission step

The device's hidraw node is root-only by default. Every Linux package above
installs a udev rule that grants the logged-in user access without root or a
setuid helper; the deb, rpm and Arch packages apply it automatically on
install. Building from source or running the AppImage needs one manual step,
once:

```sh
sudo install -Dm644 dist/udev/70-hyperpace.rules /etc/udev/rules.d/70-hyperpace.rules
sudo udevadm control --reload-rules
sudo udevadm trigger --subsystem-match=hidraw
```

Unplug and replug the mouse (or the receiver) afterwards if the app was
already running when you installed the rule.

### Blank window on NVIDIA (Wayland or X11)

WebKitGTK's DMABUF renderer can produce a completely blank window on an
NVIDIA GPU (see
`docs/papers/hyperpace-firmware-platform/summaries/SUMMARY-tauri-2026-linux-graphics-docs.md`).
Hyperpace detects this itself: at startup, before the window is created, it
checks for a Linux Wayland or X11 session with an NVIDIA GPU present (the
DRM device vendor id under `/sys/class/drm`, or the `nvidia` kernel module
loaded) and, only then, sets `WEBKIT_DISABLE_DMABUF_RENDERER=1` for itself.
The decision is logged once at startup (`RUST_LOG=info`, the default) so it
is visible which case applied.

If you already export `WEBKIT_DISABLE_DMABUF_RENDERER` yourself, Hyperpace
leaves it alone. Two escape hatches force the behaviour either way,
independent of detection:

- Environment variable `HYPERPACE_DMABUF_WORKAROUND=on` (always apply) or
  `=off` (never apply); `=auto` or unset restores detection.
- App setting `gpu_dmabuf_workaround`, same three values (`on` / `off` /
  `auto`), read through the existing `app_settings` command/store. The
  environment variable wins when both are set.

## Firmware

**No firmware package is bundled**, and the in-app firmware screen says so
plainly: "No firmware available for this device yet." This is not a missing
feature so much as a missing input: no genuine, vendor-built firmware image
for this specific mouse has been found anywhere it was searched (documented
in `docs/research/firmware-update-spec.md`), so there is nothing to ship.

The firmware engine itself is fully built: the update procedure, the
identity and checksum guards, the battery and do-not-unplug checks, and the
streamed flash path are all implemented against the byte-exact spec in that
document. It has never been run against real hardware, and nothing in the
app or its documentation claims otherwise. Three ways to add a genuine
package are built in (a drafted vendor request, a watcher for packages
appearing where the vendor's own tools look, and an import path that only
accepts a package whose identity markers match this hardware); until one of
them produces a real image, install and rollback stay built but unusable,
and the archive stays empty.

### Asking the vendor

`docs/firmware-request.md` has a ready-to-send message for the vendor's
support channel, asking for the mouse and receiver firmware packages (every
version, not only the latest) for this exact hardware, and explains why a
raw payload or a rebuilt image will not work. The Firmware screen has a
copyable version of the same message. This is the only route that can
plausibly return the version history rollback needs.

The watcher (the second route) checks the vendor's own configuration files
and firmware directory paths for a change and reports what it finds; it
never downloads or installs anything on its own, and a changed file is
evidence to look at, not proof that firmware is available. It runs once at
app start plus a slow periodic check in the background, and can be turned
off in app preferences.

## Development

```sh
cargo build --workspace       # Rust crates
npm --prefix ui ci && npm --prefix ui run build   # UI
cargo test                    # everything runs against the simulator
```

See `Makefile` for the full set of `make` targets (`setup`, `build`, `test`,
`gate`, `fmt`, `lint`).
