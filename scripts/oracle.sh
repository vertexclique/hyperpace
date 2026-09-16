#!/usr/bin/env bash
# Regenerate crates/hyperpace-protocol/tests/vectors/oracle.json: the differential oracle fixture
# (docs/plans/hyperpace.md phase 1 gate, section 6). Runs the vendor's own OLD and NEW driver
# JavaScript, sealed under bubblewrap, and records the exact bytes it sends for a broad matrix of
# operations. Never invokes node directly (house rule 6): every vendor-code execution happens
# inside the bwrap sandbox below.
#
# Usage: scripts/oracle.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HARNESS_DIR="$ROOT/scripts/oracle/harness"
VENDOR_DIR="$ROOT/firmware/archive/other-devices/_web-driver-snapshots/web-snapshot-2026-09-15/www.lofree.tech"
VECTORS_DIR="$ROOT/crates/hyperpace-protocol/tests/vectors"
OUT_FILE="$VECTORS_DIR/oracle.json"

if [ ! -f "$VENDOR_DIR/app.js" ] || [ ! -f "$VENDOR_DIR/home/index-BTVblIUr.js" ]; then
  echo "oracle.sh: expected vendor bundles under $VENDOR_DIR" >&2
  echo "           (firmware/archive is read-only evidence; if the snapshot moved, update this path, never the archive)" >&2
  exit 1
fi

if ! command -v bwrap >/dev/null 2>&1; then
  echo "oracle.sh: bubblewrap (bwrap) is required to run vendor JavaScript sealed; none found on PATH" >&2
  exit 1
fi

NODE_BIN="$(command -v node)"
if [ -z "$NODE_BIN" ]; then
  echo "oracle.sh: node not found on PATH" >&2
  exit 1
fi

WORK_OUT="$(mktemp -d)"
trap 'rm -rf "$WORK_OUT"' EXIT

echo "oracle.sh: running the vendor OLD (app.js) and NEW (home/index-BTVblIUr.js) protocol code sealed under bwrap..." >&2

bwrap \
  --unshare-all \
  --die-with-parent \
  --clearenv \
  --ro-bind /usr /usr \
  --ro-bind /lib64 /lib64 \
  --symlink usr/bin /bin \
  --symlink usr/lib /lib \
  --ro-bind "$HARNESS_DIR" /work \
  --ro-bind "$VENDOR_DIR" /vendor \
  --bind "$WORK_OUT" /out \
  --tmpfs /tmp \
  --chdir /work \
  "$NODE_BIN" /work/generate.mjs /out/oracle.json

mkdir -p "$VECTORS_DIR"
cp "$WORK_OUT/oracle.json" "$OUT_FILE"
echo "oracle.sh: wrote $OUT_FILE" >&2
python3 - "$OUT_FILE" <<'PY'
import json, sys
data = json.load(open(sys.argv[1]))
print(f"oracle.sh: {data['meta']['operation_count']} operations, bundle sha256:", file=sys.stderr)
for name, info in data["meta"]["bundles"].items():
    print(f"  {name}: {info['path']} {info['sha256']}", file=sys.stderr)
PY
