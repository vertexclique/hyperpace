#!/usr/bin/env bash
# Repo gate entrypoint. Runs the stack-adaptive vertexia gate for the Rust
# workspace, then the frontend checks, which that gate does not detect on its
# own. Installed by vertexia init as scripts/gate.sh and extended here because a
# broken interface would otherwise pass a green gate.
set -uo pipefail
DIR="${1:-.}"
status=0

if command -v vertexia >/dev/null 2>&1; then
  vertexia gate "$DIR" || status=1
else
  echo "vertexia not on PATH. Install it (bash <vertexia>/install.sh) or run the per-stack gate under <vertexia>/presets/." >&2
  exit 127
fi

if [ -f "$DIR/ui/package.json" ]; then
  echo "> frontend (npm run build, npm run check)"
  npm --prefix "$DIR/ui" run build >/dev/null 2>&1 || { echo "x frontend build failed" >&2; status=1; }
  npm --prefix "$DIR/ui" run check || { echo "x frontend type check failed" >&2; status=1; }
fi

if [ "$status" -eq 0 ]; then
  echo "ok gate green (rust and frontend)"
else
  echo "x gate RED. Fix the above before pushing." >&2
fi
exit "$status"
