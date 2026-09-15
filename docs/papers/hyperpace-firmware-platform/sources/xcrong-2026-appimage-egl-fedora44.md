URL: https://github.com/tauri-apps/tauri/issues/15976
Retrieved: 2026-09-15 (via `gh issue view 15976 --repo tauri-apps/tauri`)
Source type: GitHub issue (community root-cause analysis with a verified minimal fix); state OPEN; opened 2026-09-04; 0 comments at retrieval

Note: em-dash characters in the original text are replaced with " - " per project rule.

Title: "[bug] Default AppImage bundles libwayland/libxkbcommon/libxcb; aborts with EGL_BAD_PARAMETER on newer Mesa (Fedora 44)"

## Verbatim, body (xcrong)

> A default-config AppImage (`tauri build --bundles appimage`, no `bundle.linux` config) built on `ubuntu-22.04` aborts on Fedora 44 under a Wayland session before any window appears:
>
> ```
> Could not create default EGL display: EGL_BAD_PARAMETER. Aborting...
> ```
>
> Root cause is over-bundled display-stack libraries, not app code: I extracted the AppImage and found these in `usr/lib`, all shadowed ahead of the host copies via the `LD_LIBRARY_PATH` that `AppRun.wrapped` injects:
>
> ```
> libwayland-client.so.0  libwayland-cursor.so.0
> libwayland-egl.so.1     libwayland-server.so.0
> libxkbcommon.so.0
> libxcb-randr.so.0  libxcb-render.so.0  libxcb-shm.so.0
> libXau.so.6  libXdmcp.so.6
> ```
>
> The host's Mesa 26 ends up negotiating EGL against the Ubuntu 22.04-era bundled `libwayland-client`, and `eglGetDisplay` fails. Notably `libEGL/libGL/libdrm/libc` are *not* bundled (linuxdeploy's built-in excludelist already covers them) - the display-stack libs above are the gap.

> The same failure and the same fix were independently confirmed in a second, non-Tauri project that also ships via linuxdeploy: excluding `wayland/xkbcommon/xcb/Xau/Xdmcp` and letting the host provide them fixed startup on the same class of machine. So this points at linuxdeploy's missing excludelist entries rather than anything Tauri-specific.

> 2. Run the AppImage on Fedora 44, GNOME Wayland, Mesa 26, AMD Rembrandt iGPU.

> ### Verified minimal fix
>
> Extracted the AppImage, removed only the 10 libraries listed above, ran `./AppRun`: the app boots normally (backend starts listening, process stays alive instead of aborting within seconds). No need to strip glib/gstreamer or touch anything else - the 10 display-stack libs are sufficient, and the host versions (wayland 1.25, xkbcommon 1.13) are drop-in compatible.

> - Provide the supported escape hatch requested in #15665: review/merge #15662 (`bundle.linux.appimage.excludeLibraries` -> `--exclude-library`).

## Related search listing (gh search issues --repo tauri-apps/tauri "EGL_BAD_PARAMETER", 2026-09-15)

```
15976 open 2026-09-04 c=0 [bug] Default AppImage bundles libwayland/libxkbcommon/libxcb; aborts with EGL_BAD_PARAMETER on newer Mesa (Fedora 44)
15665 open 2026-07-07 c=4 AppImages from default bundler settings fail on Mesa 25+ distros (over-bundled libwayland/glib/gstreamer, stale GST_PLUGIN_SYSTEM_PATH in AppRun)
10749 closed 2024-08-23 c=10 [bug][v2][linux] fails to run on Fedora 40 tauri appimage built on GitHub Actions
11994 closed 2024-12-17 c=42 [bug] Tauri fails to build on Fedora 41 (Failed to get GBM device)
11988 closed 2024-12-17 c=3 [bug] AppImages built on Linux Mint do not work for Fedora with Tauri 2.0
```

Downstream projects reporting the same abort on Arch (web search results, titles only, not opened): gitbutlerapp/gitbutler#5282, readest/readest#190 ("for the AppImage version on Fedora and Arch"), atampy25/glacierkit#38 ("AppImage fails to launch on Arch Linux with Plasma desktop running under Wayland"), jub0t/Concat#31.
