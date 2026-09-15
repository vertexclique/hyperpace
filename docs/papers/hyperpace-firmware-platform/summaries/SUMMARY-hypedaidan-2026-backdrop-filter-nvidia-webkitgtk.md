# SUMMARY: hypedaidan-2026-backdrop-filter-nvidia-webkitgtk

## Claim
`backdrop-filter` does not render, along with other visual glitches, in a Tauri 2.9.5 app on Ubuntu 24.04 Wayland with NVIDIA drivers and webkit2gtk-4.1 2.50.4. In an earlier closed report, a maintainer noted that `var()` does not work inside `backdrop-filter` in WebKit, and called such failures "one of the many hardware specific issues webkitgtk has".

## Method
User reports with repro repositories, and maintainer triage.

## Result
14811 is open; 14040 was closed without a framework fix. Several other open issues involve backdrop blur with transparent windows.

## Evidence tier
3 (reports), 2 (maintainer comments).

## Performance
Not measured.

## Correctness
Specific to hardware and configuration; not reproduced here.

## Relevance to hyperpace
Do not make the design depend on blur or glass effects. If used, write literal values with the `-webkit-` prefix and provide a solid fallback.
