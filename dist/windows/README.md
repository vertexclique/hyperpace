# Windows installer (NSIS)

`cargo tauri build --bundles nsis` builds the installer; nothing here is
invoked directly.

Windows needs no udev-equivalent permission step: the device enumerates as a
standard USB HID collection (WebHID/hidapi on Windows do not need a signed
driver or admin rights to open it), so there is nothing to install beyond
the app itself.

`hyperpace-hooks.nsh` is wired in via `bundle.windows.nsis.installerHooks`
(see `../README.md`). Its only hook, `NSIS_HOOK_POSTUNINSTALL`, offers to
remove `%USERPROFILE%\.hyperpace` (the macro library, profile snapshots and
settings, at the same path on every OS per the plan) since NSIS only removes
the install directory by default and would otherwise strand that data on an
uninstall. Not compile-checked here: `makensis` is not available in this
sandbox; the syntax follows the hook example in Tauri's own
`NsisConfig::installer_hooks` documentation exactly.

Header and sidebar bitmaps (`nsis.headerImage`, `nsis.sidebarImage`) and a
custom installer icon are skipped: no design assets exist in this repository
to place there, and NSIS's own defaults are used instead.
