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

## Built and checked here

`cargo tauri build --bundles rpm` produced
`target/release/bundle/rpm/Hyperpace-0.1.0-1.x86_64.rpm`, 12,119,380 bytes
(11.56 MiB). Not installed (needs root, never used here). This machine
(Arch/CachyOS) has no `rpm`/`rpm2cpio` to check it with the normal tools, so
the file was decoded by hand instead: the RPM lead and header parsed to
find the payload offset, then the payload (zstd-compressed cpio) decoded
and listed directly. Its four files land at the same paths the deb does:
`usr/bin/hyperpace`, `usr/lib/udev/rules.d/70-hyperpace.rules`,
`usr/share/applications/Hyperpace.desktop`,
`usr/share/icons/hicolor/256x256/apps/hyperpace.png`.
