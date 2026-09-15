# SUMMARY: phase-rs-2026-appimage-idle-cpu-nvidia

## Claim
On one Fedora 44 machine, a Tauri AppImage's WebKitWebProcess used this much of one core at idle:

| GPU | Bundled Ubuntu WebKitGTK 2.52.6 | Host Fedora WebKitGTK 2.52.5 |
|---|---|---|
| NVIDIA (driver 595.91.07) | 98.7% | 35.5% |
| AMD iGPU | about 23% | about 23% |

The cost did not change with window size or when the window was minimized.

## Method
A 2x2 controlled comparison on one dual-GPU machine: same app build and window, 11+ runs, and stack sampling (100% in WebCore in the bad cell). The app ran under XWayland because AppRun forces `GDK_BACKEND=x11`.

## Result
The defect lies in the combination of Ubuntu's WebKitGTK build and the NVIDIA driver.

## Evidence tier
3.

## Performance
Numbers above. The 23% baseline reflects that app's rendering-heavy screen, not Tauri.

## Correctness
A single machine and app. Well controlled, but the issue is open and not confirmed upstream.

## Relevance to hyperpace
Strong evidence that a webview kept alive while hidden or minimized can burn CPU on NVIDIA, which a background battery app cannot afford. It supports destroying the webview when the window closes, and testing the AppImage on NVIDIA specifically.
