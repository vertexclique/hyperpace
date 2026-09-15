# SUMMARY: appimage-2026-architecture-spec-compression

## Claim
- A type 2 AppImage is a runtime plus a SquashFS image. The runtime mounts the image read-only with FUSE before running `AppRun`.
- The spec allows an embedded `.sha256_sig` signature section and `.upd_info` update information (zsync).
- The newer appimagetool standardizes on zstd squashfs compression; its bundled mksquashfs supports only zstd.

## Method
The AppImage docs, the spec draft, and an appimagetool issue with a reply from the AppImage founder.

## Result
linuxdeploy-plugin-appimage exposes `LDAI_COMP` and refers to AppImageKit.

## Evidence tier
1 (docs, spec), 2 (issue).

## Performance
Files inside the image are decompressed on access through FUSE. The compression ratio for firmware blobs is unknown; vendor images may already be compressed or encrypted.

## Correctness
- Which appimagetool, and therefore which compressor, the Tauri 2 pipeline actually uses was not determined.
- Whether the runtime verifies `.sha256_sig` at launch is not stated in what was read; assume it does not.

## Relevance to hyperpace
Firmware in an AppImage, whether embedded or shipped as resources, stays inside one file and is compressed with the rest. AppImage signing gives hyperpace no runtime guarantee, so verify firmware in-app. Measure AppImage size with and without the blobs.
