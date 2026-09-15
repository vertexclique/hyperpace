# SUMMARY: mg-chao-2025-crash-after-all-windows-hidden

## Claim
Windows users of a Tauri app saw crashes after long idle with all windows hidden (a tao `flush_paint_messages` assertion). Updating tray-icon to 0.21.2 or later stopped the assertion for another app. A maintainer closed the issue, pointing at tray-icon PR 284.

## Method
Field reports, a debug panic trace, a downstream mitigation report, and the maintainer close.

## Result
Closed as fixed in tray-icon >= 0.21.2. The reporter mentioned other unexplained crash reports remained.

## Evidence tier
3 (reports), 2 (maintainer fix pointer).

## Performance
Not applicable.

## Correctness
The root cause is tied to tray-icon on Windows. Not reproduced by this stream.

## Relevance to hyperpace
Tauri 2.11 uses tray-icon 0.24, which includes the fix. A long-idle soak test on Windows, with the window hidden or destroyed, should still be part of verification.
