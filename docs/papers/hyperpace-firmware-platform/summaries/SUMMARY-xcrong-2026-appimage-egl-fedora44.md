# SUMMARY: xcrong-2026-appimage-egl-fedora44

## Claim
A default AppImage built on ubuntu-22.04 aborts on Fedora 44 (GNOME Wayland, Mesa 26, AMD iGPU) with `EGL_BAD_PARAMETER`. The cause is ten bundled display-stack libraries: `libwayland-*`, `libxkbcommon`, `libxcb-randr/render/shm`, `libXau` and `libXdmcp`. Removing only those ten fixes startup.

## Method
The reporter extracted the AppImage, deleted the libraries and ran `./AppRun`. The same fix was confirmed in a non-Tauri linuxdeploy project.

## Result
The reporter attributes the problem to linuxdeploy's excludelist and asks for PR 15662 to be merged. `libEGL`, `libGL`, `libdrm` and `libc` were not bundled.

## Evidence tier
3.

## Performance
Not applicable.

## Correctness
The minimal fix was verified on one machine, with independent confirmation claimed in another project. The issue was open with no maintainer response at retrieval.

## Relevance to hyperpace
It gives a concrete, small exclusion list to test in a post-processing step for the hyperpace AppImage: on CachyOS with AMD, Intel and NVIDIA GPUs, under Wayland and X11.
