URL: https://v2.tauri.app/develop/debug/linux-graphics/
Additional URLs: https://github.com/tauri-apps/tauri-docs/blob/v2/src/content/docs/develop/Debug/linux-graphics.md (commit a6b59b78) ; https://github.com/tauri-apps/tauri/issues/9394 ; https://bugs.webkit.org/show_bug.cgi?id=261874
Retrieved: 2026-09-15
Source type: official documentation (primary) plus GitHub issue search listing

## Verbatim, docs

> On Linux, Tauri renders through WebKitGTK. On some setups, most often NVIDIA GPUs, WebKitGTK and the graphics driver disagree and you get anything from a blank window to subtle rendering problems. This page collects the known symptoms and workarounds. See [tauri-apps/tauri#9394](https://github.com/tauri-apps/tauri/issues/9394) for the original reports.

> ## Common symptoms
>
> - The window opens but stays blank or white.
> - The window flickers, especially while resizing.
> - The app dies on resize with no useful error output.
> - Console shows `AcceleratedSurfaceDMABuf was unable to construct a complete framebuffer`.
> - Console shows `Gdk-Message: Error 71 (Protocol error) dispatching to Wayland display.`
>
> Most of these come from the WebKitGTK DMABUF renderer requesting buffer formats the NVIDIA driver does not provide.

> ## Workarounds
>
> Try these in order. The earlier ones keep hardware acceleration.
>
> 1. Make sure kernel mode setting is on. NVIDIA drivers older than 545 often need `nvidia_drm.modeset=1` as a kernel parameter.
> 2. Set `__NV_DISABLE_EXPLICIT_SYNC=1`. This often fixes the Wayland `Error 71` crash without a performance cost.
> 3. Set `WEBKIT_DISABLE_DMABUF_RENDERER=1`. Fixes the DMABUF framebuffer error and the `Error 71` crash, at the cost of the faster rendering path.
> 4. Set `WEBKIT_DISABLE_COMPOSITING_MODE=1`. Last resort for the silent crash on resize. This disables accelerated compositing entirely.

```rust
fn main() {
  // Workaround for WebKitGTK on NVIDIA, see tauri-apps/tauri#9394
  #[cfg(target_os = "linux")]
  std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
```

> Only ship an unconditional override like this if you have verified your app is affected. It disables a faster path for everyone, including users on working setups.

> - WebGL2 context creation succeeds even when the result is backed by a software rasterizer or a slow presentation path. There is no error to catch.
> - WebKitGTK masks the WebGL renderer string for fingerprinting protection. `WEBGL_debug_renderer_info` reports `Apple GPU` on every Linux machine, so you cannot check what is actually behind the context.

## Issue #9394 state (gh issue view, 2026-09-15)

```
[docs] Documenting Nvidia problems in Tauri OPEN 2024-04-06T22:54:16Z null
```

## Framework does not set these variables itself

`grep -rn 'WEBKIT_DISABLE'` over the tauri `dev` crates and plugins-workspace `v2` plugins returned no matches (2026-09-15).

## Search listing (gh search issues --repo tauri-apps/tauri "dmabuf" --sort created, 2026-09-15; format number state created comments title)

```
15936 open 2026-08-29 c=0 Blank window under Xvfb until WEBKIT_DISABLE_COMPOSITING_MODE=1, with no diagnostic
15165 open 2026-03-28 c=3 [bug] pnpm tauri dev fails to open any apps on Fedora 43
15050 closed 2026-03-06 c=2 [bug] Tauri 2 Blank Window on Fedora 43 + Sway (Wayland)
14924 open 2026-02-10 c=1 [Bug] Linux/Nvidia: Crash (GBM/Error 71) or Visual Artifacts (Ghosting/Black Corners) with Transparent Windows
14721 open 2026-01-01 c=1 [bug] SIGSEGV crash in libwebkit2gtk-4.1.so on Linux with NVIDIA GPU during navigation
14427 open 2025-11-06 c=3 [bug] Broken scrolling on Wayland and WSL
13493 closed 2025-05-22 c=6 [bug] Failed to create GBM buffer of size 2560x1440: Invalid argument
```
(Titles only; issue bodies were not opened except where cited in other source files.)
