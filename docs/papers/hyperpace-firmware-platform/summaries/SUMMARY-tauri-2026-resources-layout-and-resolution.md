# SUMMARY: tauri-2026-resources-layout-and-resolution

## Claim
`bundle.resources` files are copied into a platform resource directory, resolved at runtime by `app.path().resolve(.., BaseDirectory::Resource)`:

| Package | Resource directory |
|---|---|
| Windows | directory of the main executable (NSIS copies files into `$INSTDIR`) |
| Linux deb | `/usr/lib/<product_name>` |
| AppImage | `${APPDIR}/usr/lib/<name>` |
| macOS | `Contents/Resources` inside the `.app` |

Tauri has no built-in integrity check for resources. The only built-in signature check found is the updater's.

## Method
The resources docs, `tauri-utils/src/platform.rs`, the deb, macOS and NSIS bundler sources, `assets.rs`, and a grep of the docs for integrity terms.

## Result
- Linux resolution tries `../lib/<package_info.name>` next to the executable, then `$APPDIR`, then `/usr/lib/<name>`.
- The docs comment says `${exe_name}`, the code uses `package_info.name`, and the deb bundler uses `product_name()`.
- The v3 alpha resolves unbundled resources from source paths.

## Evidence tier
1.

## Performance
Resources are plain files read on demand. Beyond what the package format applies (squashfs in AppImage), there is no decompression.

## Correctness
Proven from source for 2.11.5. The name mismatch between docs, resolver and deb bundler was not tested at runtime, and may be harmless when the names coincide.

## Relevance to hyperpace
Which option keeps "one binary":
- **`include_bytes!`:** keeps firmware inside the executable on every OS (raw exe, raw ELF, AppImage, `.app`).
- **Resources:** keep a single file only for an AppImage (inside the image) and a single bundle only for macOS (inside the `.app`, sealed by the code signature).
- **Resources break it** for a portable Windows exe or a bare Linux ELF, which then need files beside or under them.

Either way, integrity checking must be implemented by hyperpace.
