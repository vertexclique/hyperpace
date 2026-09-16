# RPM package

`cargo tauri build --bundles rpm` builds the `.rpm`; nothing here is invoked
directly. This directory holds the inputs the Tauri config points at (see the
`bundle.linux.rpm` snippet in `../README.md`):

- the udev rule is placed at `/usr/lib/udev/rules.d/70-hyperpace.rules` via
  `rpm.files`, source `../udev/70-hyperpace.rules`
- `../udev/reload-udev-rules.sh` runs as both the `%post` and `%postun`
  scriptlet (`postInstallScript` / `postRemoveScript`), so a plugged-in
  device picks up (or drops) the rule without a replug
- the desktop entry is the bundler's own default template, driven by the
  shared `bundle.category` and `bundle.shortDescription` fields; no custom
  template is needed here

Install: `sudo rpm -i hyperpace-0.1.0-1.x86_64.rpm` (or `sudo dnf install
./hyperpace-*.rpm` to also pull `webkit2gtk4.1` and friends; package names
vary by distribution, unverified against a specific one here). See the repo
README for the udev permission step after install.
