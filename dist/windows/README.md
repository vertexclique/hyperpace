# Windows installer (NSIS)

`cargo tauri build --bundles nsis` builds the installer; nothing here is
invoked directly.

Windows needs no udev-equivalent permission step: the device enumerates as a
standard USB HID collection (WebHID/hidapi on Windows do not need a signed
driver or admin rights to open it), so there is nothing device-related to
install beyond the app itself.

The installer does need one more thing, though, which neither `tauri.conf.json`
nor this directory previously called out: `crates/hyperpace-app/tauri.conf.json`
has no `bundle.windows` block at all, so once one is added it will have no
`nsis.webviewInstallMode` key, taking Tauri's default,
`DownloadBootstrapper { silent: true }` (confirmed by reading
`tauri-utils`' `WebviewInstallMode::default()`), which downloads and
silently runs the WebView2 bootstrapper during install if the WebView2
Runtime is not already present. That needs an internet connection at
install time; it costs nothing on a system that already has WebView2
(shipped with Windows 10 2004+ and Windows 11), but is a real requirement
on an older or stripped-down (e.g. LTSC) system. Switching to
`OfflineInstaller` or `FixedRuntime` to drop that requirement is a decision
for whoever owns `tauri.conf.json`, not made here.

`hyperpace-hooks.nsh` is meant to be wired in via
`bundle.windows.nsis.installerHooks` (see `../README.md`), the same way
`bundle.linux.deb`/`bundle.linux.rpm` already are in the live
`tauri.conf.json` today. As of this validation pass the `bundle.windows`
key is not present in that file yet (checked directly, not assumed), so
this hook file is not actually wired into any build until whoever owns
`tauri.conf.json` adds the snippet `../README.md` documents; that is
unchanged by this pass, which only fixed the hook file's own logic and this
README's accuracy. Its only hook, `NSIS_HOOK_POSTUNINSTALL`, offers to
remove `%USERPROFILE%\.hyperpace` (the macro library, profile snapshots and
settings, at the same path on every OS per the plan) since NSIS only removes
the install directory by default and would otherwise strand that data on an
uninstall; it skips that prompt during an in-place upgrade (`$UpdateMode`)
so updating the app never asks to delete the user's macros. Not
compile-checked here: `makensis` is not available in this sandbox; the
macro name and body follow the hook example in Tauri's own
`NsisConfig::installer_hooks` documentation, and the `$UpdateMode` guard and
the hook's insertion point were confirmed by reading `tauri-bundler`
3.0.0-alpha's own vendored `installer.nsi` template rather than assumed.

Header and sidebar bitmaps (`nsis.headerImage`, `nsis.sidebarImage`) and a
custom installer icon are skipped: no design assets exist in this repository
to place there, and NSIS's own defaults are used instead.
