# Debian package

`cargo tauri build --bundles deb` builds the `.deb`; nothing here is invoked
directly. This directory holds the inputs the Tauri config points at (see the
`bundle.linux.deb` snippet in `../README.md`):

- the udev rule is placed at `/usr/lib/udev/rules.d/70-hyperpace.rules` via
  `deb.files`, source `../udev/70-hyperpace.rules`
- `../udev/reload-udev-rules.sh` runs as both `postInstallScript` and
  `postRemoveScript`, so a plugged-in device picks up (or drops) the rule
  without a replug
- the desktop entry and its `Categories=` line are the bundler's own default
  template, driven by the shared `bundle.category` and `bundle.shortDescription`
  fields (also in `../README.md`); no custom template is needed here

Install: `sudo dpkg -i hyperpace_0.1.0_amd64.deb` (or `sudo apt install
./hyperpace_*.deb` to also pull `webkit2gtk-4.1` and friends). See the repo
README for the udev permission step after install.

## Built and checked here

`cargo tauri build --bundles deb` produced
`target/release/bundle/deb/Hyperpace_0.1.0_amd64.deb`, 18,358,286 bytes
(17.51 MiB). Not installed (needs root, never used here); checked instead
by extracting it (`ar t`/`ar x`, then `tar tzf`/`tar xzf` on the two
members) and reading the result directly: `debian-binary`,
`control.tar.gz`, `data.tar.gz` are the only three members, the udev rule
lands at `/usr/lib/udev/rules.d/70-hyperpace.rules`, and the binary,
desktop entry and icon land where `deb.files` and the bundler's own
defaults put them.

One real, harmless defect found this way and left as a documented gap, not
fixed here: the `control` file's `Depends` lists
`libwebkit2gtk-4.1-0`/`libgtk-3-0` twice each. `tauri-bundler` auto-detects
both via `ldd` against the built binary, then also appends
`tauri.conf.json`'s manually configured `deb.depends` for the same two
libraries. `dpkg` tolerates a duplicate `Depends` entry, so the package
still installs correctly; whoever owns `tauri.conf.json` next should drop
the manual `depends` array now that auto-detection already covers it,
rather than list both.
