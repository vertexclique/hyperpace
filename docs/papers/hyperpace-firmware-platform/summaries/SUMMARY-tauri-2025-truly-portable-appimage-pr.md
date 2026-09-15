# SUMMARY: tauri-2025-truly-portable-appimage-pr

## Claim
A maintainer-authored experimental AppImage bundler (sharun, quick-sharun, uruntime) aims to produce AppImages that isolate graphics drivers from bundled libraries. It has been open since 2025-01-24, and the maintainer says it requires building on Arch.

## Method
A PR thread with hands-on testing by the sharun tooling author and by a tester on AMD and NVIDIA.

## Result
- Testing in May 2025 found NVIDIA hardware-acceleration issues on Wayland when built on Ubuntu 24.04, due to older Mesa DRI loading. Forcing XWayland worked around it.
- sharun later gained a GBM hook.
- The tester went from "not working at all on AMD" to working on both GPUs.
- The PR also carries plans for zsync update support.
- Related merged work: the bundler split into backends (14841) and the quick-sharun file-list fix (15917).

## Evidence tier
2.

## Performance
Not measured in the thread.

## Correctness
Unmerged. How to enable it from a released CLI was not verified. Results are from 2025 test builds.

## Relevance to hyperpace
It is the most promising fix for AppImages on rolling distros, but experimental. An Arch build container fits the CachyOS workflow. Worth an early spike, with deb and rpm as a fallback Linux channel.
