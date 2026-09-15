URL: https://docs.appimage.org/reference/architecture.html
Additional URLs: https://github.com/AppImage/AppImageSpec/blob/master/draft.md ; https://github.com/AppImage/appimagetool/issues/69
Retrieved: 2026-09-15 (docs and spec via WebFetch, which returned the quoted passages; issue via `gh issue view`)
Source type: AppImage project documentation and specification (primary); upstream issue with the AppImage founder's reply (tier 2)

## docs.appimage.org, Architecture

> An AppImage comprises "two parts: a _runtime_ and a _file system image_. For the current type 2, the file system in use is SquashFS."

> "The runtime, the executable part, tries to mount the file system image using FUSE."

> When mounting succeeds, "the AppDir is available in a temporary mountpoint, and can be used like a read-only directory."

> The runtime proceeds by "calling the AppDir's 'entrypoint' AppRun using the operating system facilities."

(The WebFetch tool wrapped the original sentences in quotes; the text inside quotes is the page text.)

## AppImageSpec draft, type 2 (as returned by WebFetch)

> **"MUST have appended to it a filesystem that the ELF part can mount"** and **"MUST, when executed, mount the AppImage and execute the executable file `AppRun`"**

> **"MAY embed a digital signature in the ELF section `.sha256_sig`. If this section exists then it MUST either be empty or contain a valid digital signature of the sha256"** of the AppImage

> **"MAY embed update information in the ELF section `.upd_info`"**

The fetch reported that the spec text does not name squashfs or FUSE for type 2 and states no compression requirement for type 2.

## appimagetool issue #69 (OPEN, 2024-10-07) "Remove the --comp option in appimagetool"

pkwong4321:
> I believe it's becuase the bundled mksquashfs did not enable any compression methods other than zstd, making --comp meaningless
> ref: ci/install-static-mksquashfs.sh
> `make -j"$jobs" GZIP_SUPPORT=0 XZ_SUPPORT=0 LZO_SUPPORT=0 LZ4_SUPPORT=0 ZSTD_SUPPORT=1 COMP_DEFAULT=zstd LDFLAGS=-static USE_PREBUILT_MANPAGES=y install`

probonopd (2024-10-07):
> Hi @pkwong4321, thanks for your comment. However, we are standardizing (no pun intended) on zstd.

## Not verified

Which appimagetool build (AppImageKit legacy or the newer AppImage/appimagetool) the `linuxdeploy-plugin-appimage` continuous release used by the Tauri 2 bundler invokes, and therefore the effective squashfs compressor of a Tauri AppImage, was not determined. The plugin README documents `LDAI_COMP` and points to the AppImageKit repository.
