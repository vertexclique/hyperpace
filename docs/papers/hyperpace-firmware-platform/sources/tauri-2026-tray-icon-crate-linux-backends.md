URL: https://github.com/tauri-apps/tray-icon
Files read: src/lib.rs, Cargo.toml, CHANGELOG.md, src/platform_impl/gtk/mod.rs, src/platform_impl/ksni/mod.rs (shallow clone, commit aab34f8a6afc66e37c2ea35d87b7ef1f32146807, 2026-09-15); libappindicator-sys 0.9.0 src/lib.rs (crates.io source, local cargo registry copy); https://crates.io/crates/tray-icon
Retrieved: 2026-09-15
Source type: library source code and changelog (primary)

## Verbatim, src/lib.rs crate docs

```
//! # Platforms supported:
//!
//! - Windows
//! - macOS
//! - Linux and BSD (AppIndicator or KSNI)
//!
//! # Platform-specific notes:
//!
//! - On Windows and the Linux/BSD AppIndicator backend, an event loop must be running on the thread. The
//!   KSNI backend runs its D-Bus service on a worker thread and does not require a GTK event loop.
//! - When both the `libappindicator` and `ksni` features are enabled on Linux or BSD, tray-icon
//!   uses the KSNI backend and emits a Cargo warning.
//! - On macOS, an event loop must be running on the main thread so you also need to create the tray icon on the main thread.
//!
//! # Dependencies (Linux/BSD)
//!
//! The default Linux backend uses GTK, `libxdo`, and `libappindicator` or
//! `libayatana-appindicator`. The `ksni` backend does not require these system libraries unless a
//!  GTK backend is also enabled.
```

## Verbatim, TrayIconAttributes platform notes (src/lib.rs)

```
    /// Tray icon tooltip
    ///
    /// ## Platform-specific:
    ///
    /// - **Linux/BSD AppIndicator backend:** Unsupported.
    pub tooltip: Option<String>,

    /// Tray menu
    ///
    /// ## Platform-specific:
    ///
    /// - **Linux/BSD AppIndicator backend:** Once a menu is set, it cannot be removed.
    pub menu: Option<Box<dyn menu::ContextMenu>>,

    /// Tray icon
    ///
    /// ## Platform-specific:
    ///
    /// - **Linux/BSD AppIndicator backend:** Sometimes the icon won't be visible unless a menu is set.
    ///   Setting an empty [`Menu`](crate::menu::Menu) is enough.
    pub icon: Option<Icon>,

    /// Tray icon temp dir path. **Linux/BSD AppIndicator backend only**.
    pub temp_dir_path: Option<PathBuf>,
```
```
    /// Whether to show the tray menu on left click or not, default is `true`.
    ///
    /// ## Platform-specific:
    ///
    /// - **Linux:** Unsupported.
    pub menu_on_left_click: bool,
```
```
    /// A stable identity for the tray icon, as a UUID in `u128` form. **Windows only**.
    ///
    /// Windows remembers per-icon user settings (most importantly whether the
    /// icon is pinned to the taskbar or hidden in the overflow) keyed on the
    /// icon's identity. Without a GUID that identity is the executable path
    /// plus a per-process counter, so the setting is lost whenever the binary
    /// moves - for example every update of an installer that uses versioned
    /// directories. With a GUID, and an executable that is Authenticode-signed
    /// by the same publisher across versions, the setting survives.
```

## Verbatim, Cargo.toml features

```
[features]
default = ["muda-libxdo", "libappindicator"]
serde = ["dep:serde", "muda-serde"]
libappindicator = ["dep:dirs", "dep:libappindicator", "muda-gtk3"]
ksni = ["dep:ksni", "muda-snapshot"]
```
```
libappindicator = { version = "0.9", optional = true }
dirs = { version = "7", optional = true }
ksni = { version = "0.3.6", default-features = false, features = [
  "async-io",
  "blocking",
], optional = true }
```

## Verbatim, CHANGELOG.md

> ## [0.25.0]
>
> - [`c648db3`] ([#201] by [@dfaust]) Add a GTK-free `ksni` StatusNotifierItem backend for Linux and BSD.
> - [`c1990d6`] ([#361] by [@amrbashir]) Separate the Linux/BSD `libappindicator` and `ksni` tray backends from muda's GTK backend features, and expose muda features with a `muda-` prefix.
> - [`95fc3cc`] ([#366] by [@dylanh724]) Add `TrayIconBuilder::with_guid` / `TrayIconAttributes::guid` to register the icon with a stable `NOTIFYICONDATA.guidItem` on Windows, so the user's "always show in taskbar" setting survives the executable being replaced by an update.
>
> ## \[0.24.2]
>
> - [`962c9be`] ([#337] by [@Legend-Master]) Set `NOTIFYICONDATAW.cbSize` to `size_of::<NOTIFYICONDATAW>()`, this increased the max tooltip length from 64 to 128 characters.

## Verbatim, KSNI backend implements tooltip (src/platform_impl/ksni/mod.rs)

```
77:    pub fn set_tooltip<S: AsRef<str>>(&mut self, tooltip: Option<S>) -> crate::Result<()> {
83:        let _ = self.handle.update(move |tray| tray.tooltip = tooltip);
186:    fn tool_tip(&self) -> ksni::ToolTip {
190:            description: self.tooltip.clone(),
166:    fn activate(&mut self, x: i32, y: i32) {
170:    fn secondary_activate(&mut self, x: i32, y: i32) {
```

## Verbatim, AppIndicator backend writes a PNG per icon change (src/platform_impl/gtk/mod.rs)

```
60:            temp_icon_path(self.temp_dir_path.as_ref(), &self.id, self.counter)?;
63:            icon.inner.write_to_png(&icon_path)?;
138:    let icon_path = parent_path.join(format!("tray-icon-{}-{}.png", id.as_ref(), counter));
```

## Verbatim, libappindicator-sys 0.9.0 src/lib.rs: runtime dlopen and panic when missing

```rust
pub static LIB: Lazy<Library> = Lazy::new(|| {
  let libayatana = unsafe { Library::new("libayatana-appindicator3.so.1") };
  if let Ok(lib) = libayatana {
    return lib;
  }

  let libappindicator = unsafe { Library::new("libappindicator3.so.1") };
  if let Ok(lib) = libappindicator {
    return lib;
  }
...
  panic!(
    "Failed to load ayatana-appindicator3 or appindicator3 dynamic library\n{}\n{}",
    libayatana.unwrap_err(),
    libappindicator.unwrap_err()
  );
});
```

## Tauri 2 stable uses the AppIndicator-only line

tauri `dev` (2.11.5) `crates/tauri/Cargo.toml` line 92: `tray-icon = { version = "0.24", default-features = false, features = [` (0.24 predates the ksni backend added in 0.25.0).
