# SUMMARY: ttntm-2023-appimage-size-discussion

## Claim
AppImages are much larger than other Tauri bundles because they carry WebKitGTK and its dependency closure. A maintainer says there is no meaningful way to shrink them. A small Vue SPA (647.9 kB web build) produced an 89.3 MB AppImage.

## Method
A discussion thread and one user's build output.

## Result
No accepted answer. The maintainer explains that an AppImage must bundle everything, unlike a deb, which pulls WebKitGTK from the system.

## Evidence tier
2 (maintainer), 3 (size data point).

## Performance
89.3 MB for one Tauri 1 era build. Web search summaries attribute about 76 MB to other Tauri 2 apps (not verified).

## Correctness
A single 2023 data point with Tauri 1; the Tauri 2 bundler and WebKitGTK 4.1 differ.

## Relevance to hyperpace
Expect roughly 70 to 90 MB for the AppImage before adding firmware; measure the hyperpace build. deb and rpm stay in the low MB range.
