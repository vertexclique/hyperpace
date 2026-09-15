URL: https://github.com/phase-rs/phase/issues/8614
Retrieved: 2026-09-15 (via `gh issue view 8614 --repo phase-rs/phase`)
Source type: GitHub issue in a downstream Tauri app, with a controlled 2x2 measurement (tier 3); state OPEN; opened 2026-09-06

Note: em-dash characters in the original text are replaced with " - " per project rule.

Title: "Linux AppImage burns a full CPU core at idle on NVIDIA - bundled WebKitGTK x NVIDIA driver interaction" (original uses an em-dash and a multiplication sign)

## Verbatim, body

> **Revised 2026-09-06.** The original body blamed the bundled WebKitGTK build alone. That was wrong: it is an *interaction* with the NVIDIA driver.

> The Linux AppImage pegs a full CPU core while idle on the home screen - **on NVIDIA**. It is an interaction between the WebKitGTK build bundled into the AppImage and the NVIDIA proprietary driver. **Neither factor alone explains it.** On the AMD integrated GPU of the same machine, the same AppImage costs ~23% and is fine.

> ## Environment
>
> - `Phase-Desktop-Linux-x86_64.appimage`, shell-v1.0.12; web app v0.75.0-v0.76.0
> - Nobara / Fedora 44, KWin Wayland (the app runs under XWayland - `AppRun` forces `GDK_BACKEND=x11`)
> - Two GPUs, which is what made the test possible: NVIDIA RTX 3090 (driver 595.91.07) and an AMD Raphael iGPU (Mesa)
> - Bundled WebKitGTK **2.52.6**; host WebKitGTK **2.52.5**

> Four cells, same machine, same app build, same AppDir harness, same 3840x2049 window, same web-app version, run back to back, each screenshot-verified rendering the same home screen:

| | **NVIDIA 595.91.07** | **AMD iGPU (Mesa)** |
|---|---|---|
| **Bundled Ubuntu WebKit 2.52.6** | **98.7%** | **22.6%** |
| **Host Fedora WebKit 2.52.5** | **35.5%** | **23.8%** |

> - **On AMD the WebKit build is irrelevant** - 22.6% vs 23.8%.
> - **On NVIDIA it is decisive** - 98.7% vs 35.5%.

> - `WebKitWebProcess` sits at 92-100% of one core indefinitely, across 11+ runs
> - **Flat from 700x500 to 3840x2049** - cost does not respond to viewport area, which is not how paint cost behaves
> - Unchanged when the window is minimized

(The app is a game client with a rendering-heavy home screen, so ~23% idle CPU on the good cells is app-specific and not a Tauri baseline.)
