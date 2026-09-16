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
