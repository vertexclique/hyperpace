#!/usr/bin/env bash
# Builds the Linux packages (deb, rpm, appimage) via Tauri's bundler, then repairs the AppImage.
#
# The default AppImage bundles its own copies of the Wayland/X11 client libraries WebKitGTK
# links against. On a host whose own display stack is newer (current Mesa, which is what this
# repo's dev machine runs), the bundled copies collide with the host's EGL/Wayland stack and the
# app aborts with no window ever shown:
# docs/papers/hyperpace-firmware-platform/summaries/SUMMARY-wpfleger96-2026-appimage-mesa25-overbundled-libs.md
# docs/papers/hyperpace-firmware-platform/summaries/SUMMARY-xcrong-2026-appimage-egl-fedora44.md
#
# The fix both reports converge on is deleting the bundled display-stack libraries from the
# AppDir and repacking. This script does that with the exact nine libraries linuxdeploy bundles
# for this app (confirmed by diffing an unpatched AppDir against a pruned one; the research
# papers name up to ten, including libxcb-randr, which this app's own dependency closure never
# pulls in, hence nine here, not ten). It is idempotent: re-running it skips any library already
# removed, and repacking always regenerates the final AppImage from the current AppDir.
#
# A second, separate display bug affects every Linux package this repo builds, not only the
# AppImage: WebKitGTK's DMABUF renderer produces a blank window on NVIDIA + Wayland/X11
# (docs/papers/hyperpace-firmware-platform/summaries/SUMMARY-tauri-2026-linux-graphics-docs.md).
# That one is fixed in the app binary itself (crates/hyperpace-app/src/gpu_workaround.rs detects
# the condition at startup and re-execs with WEBKIT_DISABLE_DMABUF_RENDERER=1), so every package
# format carries the fix automatically and no packaging-time step is needed for it here.
#
# Tauri's own AppImage step (linuxdeploy, plus its gtk plugin, plus a runtime-binary download)
# was observed flaky in this dev environment: it failed outright on 4 of 5 build attempts made
# while writing this script, always with the same terse `failed to run linuxdeploy` and no
# further detail, and always after deb and rpm had already bundled successfully. deb/rpm are
# built once; the appimage target alone is retried with a small bound (never unbounded retries)
# since a rebuild is cheap once the binary itself is already compiled.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."
REPO_ROOT="$(pwd)"
APPDIR_REL="target/release/bundle/appimage/Hyperpace.AppDir"
APPDIR="$REPO_ROOT/$APPDIR_REL"
DEFAULT_APPIMAGE_REL="target/release/bundle/appimage/Hyperpace_0.1.0_amd64.AppImage"
DEFAULT_APPIMAGE="$REPO_ROOT/$DEFAULT_APPIMAGE_REL"
UNPATCHED="$REPO_ROOT/target/release/bundle/appimage/Hyperpace_0.1.0_amd64.default-unpatched.AppImage"
MAX_APPIMAGE_ATTEMPTS=3

echo "==> cargo tauri build (targets from tauri.conf.json: deb, rpm, appimage)"
if ! ( cd crates/hyperpace-app && cargo tauri build ); then
	echo "==> full build failed (deb/rpm may still have bundled; see above); retrying the appimage target alone"
	attempt=2
	until ( cd crates/hyperpace-app && cargo tauri build --bundles appimage ); do
		attempt=$((attempt + 1))
		if [ "$attempt" -gt "$MAX_APPIMAGE_ATTEMPTS" ]; then
			echo "error: cargo tauri build --bundles appimage failed $MAX_APPIMAGE_ATTEMPTS times in a row;" \
				"giving up rather than shipping a stale or partial AppImage. deb/rpm above are unaffected." >&2
			exit 1
		fi
		echo "==> appimage bundling failed (attempt $((attempt - 1)) of $MAX_APPIMAGE_ATTEMPTS); retrying"
	done
fi

if [ ! -d "$APPDIR" ]; then
	echo "no AppImage AppDir at $APPDIR_REL; appimage is not in tauri.conf.json's bundle.targets, nothing to fix" >&2
	exit 0
fi

if [ -f "$DEFAULT_APPIMAGE" ] && [ ! -f "$UNPATCHED" ]; then
	echo "==> preserving the Tauri bundler's own (unpruned) output for comparison"
	cp "$DEFAULT_APPIMAGE" "$UNPATCHED"
fi

echo "==> pruning bundled display-stack libraries known to break on current Mesa hosts"
PRUNE_LIBS=(
	libwayland-client.so.0
	libwayland-cursor.so.0
	libwayland-egl.so.1
	libwayland-server.so.0
	libxkbcommon.so.0
	libxcb-render.so.0
	libxcb-shm.so.0
	libXau.so.6
	libXdmcp.so.6
)
for lib in "${PRUNE_LIBS[@]}"; do
	if [ -f "$APPDIR/usr/lib/$lib" ]; then
		rm -f "$APPDIR/usr/lib/$lib"
		echo "   removed $lib"
	fi
done

PLUGIN="$HOME/.cache/tauri/linuxdeploy-plugin-appimage.AppImage"
if [ ! -x "$PLUGIN" ]; then
	echo "error: $PLUGIN not found. cargo tauri build downloads it on its first AppImage build;" \
		"the build above should have fetched it." >&2
	exit 1
fi

echo "==> repacking the AppImage from the pruned AppDir"
ARCH=x86_64 OUTPUT="$DEFAULT_APPIMAGE" NO_STRIP=1 "$PLUGIN" --appdir "$APPDIR"

echo "==> done"
echo "    unpatched, for comparison only, do not ship: $UNPATCHED"
echo "    fixed, shippable:                            $DEFAULT_APPIMAGE"
echo
echo "Note: the AppImage still needs the udev rule installed once before first launch on any"
echo "machine (see dist/appimage/README.md and the repo README's udev step); it has no installer"
echo "to do that for itself."
