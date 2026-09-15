# SUMMARY: tauri-2026-linux-graphics-docs

## Claim
Blank windows, flicker, resize crashes, `AcceleratedSurfaceDMABuf` errors and Wayland `Error 71` mostly come from the WebKitGTK DMABUF renderer on NVIDIA.

Workarounds, in order:
1. kernel modesetting;
2. `__NV_DISABLE_EXPLICIT_SYNC=1`;
3. `WEBKIT_DISABLE_DMABUF_RENDERER=1`;
4. as a last resort, `WEBKIT_DISABLE_COMPOSITING_MODE=1`.

Do not ship an unconditional override. WebGL can silently fall back to slow paths, and the renderer string is masked.

## Method
The official docs page and an issue search.

## Result
- Tracking issue 9394 is still open.
- Neither tauri nor the plugins set these variables themselves.
- Recent open issues include a blank window under Xvfb, an NVIDIA SIGSEGV in libwebkit2gtk, and broken scrolling on Wayland.

## Evidence tier
1 (docs), 3 (issue listing).

## Performance
Disabling DMABUF or compositing trades rendering speed for stability; no numbers given.

## Correctness
The guidance is empirical and not tied to specific versions. No regression specific to WebKitGTK 2.50 or 2.52 was found.

## Relevance to hyperpace
- The UI is simple (no WebGL).
- Detect NVIDIA plus Wayland and apply `WEBKIT_DISABLE_DMABUF_RENDERER` only there, or expose a setting or env override. It must be set before the webview exists.
- CI screenshot tests under Xvfb may need `WEBKIT_DISABLE_COMPOSITING_MODE=1` (issue 15936).
