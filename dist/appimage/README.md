# AppImage

`cargo tauri build --bundles appimage` builds the `.AppImage`; nothing here
is invoked directly. There are no format-specific inputs to add: the AppImage
target always uses the bundler's built-in desktop-entry template (it has no
`desktopTemplate` option, unlike deb and rpm), driven by the same shared
`bundle.category` and `bundle.shortDescription` fields documented in
`../README.md`.

An AppImage has no package manager and no maintainer scripts, so it cannot
install `../udev/70-hyperpace.rules` itself: run the install command in the
repo README once, before the first launch. Without it, opening the real
hidraw node fails with a permission error; the plan calls for
`hyperpace-device` to surface the exact install command in that error rather
than a bare "permission denied", which is not implemented as of this writing
(`crates/hyperpace-device/src/hid.rs` propagates the raw hidapi error today).

Mark the file executable and run it directly: `chmod +x Hyperpace.AppImage &&
./Hyperpace.AppImage`. No install step, no root needed to run it.
