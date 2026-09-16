# Packaging

Every packaging input lives here, one directory per target, each with its
own `README.md`. Builds go through Tauri's own bundler
(`cargo tauri build --bundles <target>` from `crates/hyperpace-app`, or
`--bundles all`); nothing under `dist/` is a build script to run by hand,
and nothing here was built or installed by this change (no package build
needing root was run; see "Validation" below for what was actually checked).

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
this change, so the keys below are documented here rather than applied.
Whoever owns that file adds this to its `bundle` object (paths are relative
to `tauri.conf.json` itself, i.e. `crates/hyperpace-app/`):

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

Everything under `dist/` was checked the way it can be checked without
running an installer, a package build, or touching a real device:

- `dist/udev/70-hyperpace.rules`: `udevadm verify` passes.
- `dist/udev/reload-udev-rules.sh`, `dist/arch/PKGBUILD`,
  `dist/arch/hyperpace.install`: `bash -n` (both are read and executed as
  shell) passes.
- `dist/arch/hyperpace.desktop`: `desktop-file-validate` passes.
- `dist/windows/hyperpace-hooks.nsh`: not compile-checked, `makensis` is not
  available in this sandbox; see `windows/README.md`.
- The Tauri bundle config snippet above: not applied or built (out of
  ownership, and a full build needs network for `cargo fetch` / `npm ci`
  plus tens of minutes); its `deb.files`/`rpm.files` direction and the
  `installerHooks` hook names were confirmed by reading
  `tauri-bundler` 3.0.0-alpha's own source rather than assumed.
- The Arch dependency names (`webkit2gtk-4.1`, `gtk3`,
  `libayatana-appindicator`) matched this machine's own `pacman` database,
  which is the machine the plan names as the primary Arch target.

`makepkg`, `dpkg -i`, `rpm -i` and every installer were deliberately not
run: none of them can complete without root or a full network build, and
neither is part of this task.

## A conflict worth flagging

`Makefile`'s `clean` target runs `rm -rf target dist node_modules/.cache`
from the repo root, which deletes this directory (source-controlled
packaging input, not build output) along with the actual build output
directories. That line is outside this change's ownership (repo-root
`Makefile`), so it is reported rather than edited: whoever owns it should
either scope the removal to `crates/hyperpace-app/dist` (the Tauri frontend
build output, a different directory with the same name) and `ui/build`, or
rename one of the two `dist` directories so `make clean` cannot delete
committed packaging inputs.
