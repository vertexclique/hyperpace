URL: https://github.com/tauri-apps/tauri/issues/15665
Retrieved: 2026-09-15 (via `gh issue view 15665 --repo tauri-apps/tauri`)
Source type: GitHub issue with bisection by the reporter and maintainer replies (tier 3 report, tier 2 maintainer comments); state OPEN; opened 2026-07-07

Note: em-dash characters in the original text are replaced with " - " per project rule.

Title: "AppImages from default bundler settings fail on Mesa 25+ distros (over-bundled libwayland/glib/gstreamer, stale GST_PLUGIN_SYSTEM_PATH in AppRun)"

## Verbatim, body (wpfleger96)

> I hit this with a tauri 2.11 app ([block/buzz](https://github.com/block/buzz)) shipped as an AppImage built with the default bundler settings (no `bundle.linux` config at all, built on ubuntu-24.04 GitHub runners). On newer distros - Ubuntu 26.04 with Mesa 25 and GLib 2.88 in my case - the AppImage starts but the window never appears: WebKitWebProcess aborts and the app keeps running headless with no error surfaced to the user.

> **1. `libwayland-client` gets bundled, which breaks newer Mesa.** linuxdeploy's dependency follower sweeps `libwayland-client.so.0` (and the glib family that pins it) into `usr/lib` - its excludelist doesn't contain it, though the pkg2appimage community excludelist does, with a comment that bundling it breaks newer Mesa. With the host's Mesa 25 loaded against the bundled libwayland-client 1.22, `eglGetDisplay(EGL_DEFAULT_DISPLAY)` fails with `EGL_BAD_PARAMETER` under a Wayland session, WebKitWebProcess prints "Could not create default EGL display: EGL_BAD_PARAMETER. Aborting..." and dies. I confirmed by LD_PRELOAD-bisecting the bundled libs one at a time: every lib that reproduced the EGL failure links `libwayland-client`, and removing the wayland/glib infra libs fixes it.

> **2. AppRun exports `GST_PLUGIN_SYSTEM_PATH_1_0=$APPDIR/usr/lib/gstreamer-1.0` unconditionally, but with the default `bundleMediaFramework: false` that directory is never created.** ... so WebKit's media setup fails ("GStreamer element appsink not found"), a NULL signal connect follows (GLib-GObject-CRITICAL), and the render process dies.

> **3. (latent) The bundled WebKit helper binaries only have `RUNPATH=$ORIGIN`.** Any launch path that doesn't go through AppRun's `cd $APPDIR/usr` makes the bundled `libwebkit2gtk` (2.44 in my build) spawn the *system* WebKit helpers (2.52 on my machine) - the version mismatch kills WebKitNetworkProcess with SIGBUS.

## Verbatim, maintainer replies (FabianLars, 2026-07-07)

> 1) We'd just have to update linuxdeploy to get the new exclude this but every time we try this, the appimages break even more 🙃

> oh and you may want to try the new appimage bundler from https://github.com/tauri-apps/tauri/pull/12491 if you want to get actually working appimages - that requires building on arch though :/

## Verbatim, reporter correction (wpfleger96, 2026-07-07)

> On (2), `GST_PLUGIN_SYSTEM_PATH_1_0`: it's the AppImageKit `AppRun` binary the bundler installs unconditionally, not a hook or the gstreamer plugin.

## Verbatim, workaround from another user (su-huldra, 2026-07-09)

> An easy workaround is to rewrite the LD_PRELOAD variable when starting the appimage
>
> ```bash
> ldconfig -p | grep wayland-client
> # Take the output path:  /usr/lib/x86_64-linux-gnu/libwayland-client.so.0 then:
> LD_PRELOAD=/usr/lib/x86_64-linux-gnu/libwayland-client.so.0 ./TauriApp.AppImage
> ```
