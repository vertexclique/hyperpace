URL: https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/linux/appimage/linuxdeploy.rs
Additional URLs: https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/linux/appimage/linuxdeploy-plugin-gtk.sh ; https://github.com/tauri-apps/tauri/tree/dev/.changes (appimage-respect-gdk-backend.md, fix-appimage-xdg-utils.md) ; https://github.com/tauri-apps/tauri/pull/15662 ; https://github.com/tauri-apps/tauri/pull/15786 ; https://github.com/tauri-apps/tauri/pull/15804
Retrieved: 2026-09-15 (tauri `dev` commit bca4ca58)
Source type: bundler source code, pending change files, PRs (primary)

## Directory listing, dev branch `crates/tauri-bundler/src/bundle/linux/appimage/`

```
linuxdeploy-plugin-gstreamer.sh  4.7K
linuxdeploy-plugin-gtk.sh  11.4K
linuxdeploy.rs  8.6K
mod.rs  574B
```

## Verbatim lines, linuxdeploy.rs (WebKitGTK helpers are copied into the AppDir; tooling downloaded at build time)

```
135:    "WebKitNetworkProcess",
136:    "WebKitWebProcess",
137:    "injected-bundle/libwebkit2gtkinjectedbundle.so",
141:      let source = source.join("webkit2gtk-4.1").join(file);
183:    "--plugin",
184:    "gtk",
226:    let data = download("https://raw.githubusercontent.com/tauri-apps/linuxdeploy-plugin-gtk/master/linuxdeploy-plugin-gtk.sh")?;
239:    let data = download(&format!("https://github.com/linuxdeploy/linuxdeploy-plugin-appimage/releases/download/continuous/linuxdeploy-plugin-appimage-{arch}.AppImage"));
243:        log::error!("Download of AppImage plugin failed. Using older built-in version instead.");
```

## Verbatim lines, linuxdeploy-plugin-gtk.sh (AppRun hook environment)

```
185:export GTK_DATA_PREFIX="$APPDIR"
186:export GTK_THEME="$APPIMAGE_GTK_THEME" # Custom themes are broken
187:export GDK_BACKEND="${GDK_BACKEND:-x11}" # Crash with Wayland backend on Wayland - We tested it without it and ended up with this: https://github.com/tauri-apps/tauri/issues/8541
188:export XDG_DATA_DIRS="$APPDIR/usr/share:/usr/share:$XDG_DATA_DIRS" # g_get_system_data_dirs() from GLib
```

## Verbatim, pending change files (dev, unreleased)

`appimage-respect-gdk-backend.md` (tauri-bundler patch:bug):
> Respect an explicitly configured `GDK_BACKEND` in AppImage GTK hooks while retaining `x11` as the default.

`fix-appimage-xdg-utils.md` (tauri-bundler, tauri-cli, @tauri-apps/cli patch:bug):
> On Linux, do not bundle xdg-open and xdg-utils in the AppImage anymore. This rarely worked and usually requires host system support anyway.

## PR states (gh search prs --repo tauri-apps/tauri "appimage", 2026-09-15; format: number state created closed title)

```
12491 open 2025-01-24 feat(bundler): Truly portable appimage (experimental)
15662 open 2026-07-06 feat(bundler): add AppImage library exclusion config
15786 merged 2026-07-27 2026-07-27 fix(bundler): respect configured GDK backend (fix: #15781)
15772 merged 2026-07-23 2026-07-23 fix(bundler/cef): change appimage layout, fixes gpu usage
15917 merged 2026-08-25 2026-08-25 fix(bundler): give quick-sharun list of all bins/libs in AppDir. detect tray-icon feature.
15804 merged 2026-08-01 2026-08-03 fix(bundler): stop bundling xdg-open and xdg-utils
14841 merged 2026-01-28 2026-01-28 refactor: split appimage bundler in multiple files to support new backends
14241 merged 2025-10-02 2025-10-06 fix(bundler): set APPIMAGE_EXTRACT_AND_RUN env var as well for linuxdeploy
```

## Verbatim, PR #15662 body (OPEN)

> Adds `bundle.linux.appimage.excludeLibraries`, allowing AppImage builds to pass library exclusion patterns through to linuxdeploy's `--exclude-library` option.
>
> This gives projects a supported way to avoid bundling known problematic system libraries, such as graphics or Wayland stack libraries.

```json
{
  "bundle": {
    "linux": {
      "appimage": {
        "excludeLibraries": ["libwayland-*.so*"]
      }
    }
  }
}
```

## Directory listing, `v3` branch `crates/tauri-bundler/src/bundle/linux/appimage/` (commit f44d1114)

```
linuxdeploy-plugin-gstreamer.sh  4.7K
linuxdeploy-plugin-gtk.sh  11.4K
linuxdeploy.rs  8.8K
mod.rs  1.0K
sharun_cef.rs  8.6K
```
