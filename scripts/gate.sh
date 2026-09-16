#!/usr/bin/env bash
# Repo gate entrypoint. Delegates to the stack-adaptive vertexia gate. Installed
# by vertexia init as scripts/gate.sh.
set -uo pipefail
if command -v vertexia >/dev/null 2>&1; then
  exec vertexia gate "${1:-.}"
fi
echo "vertexia not on PATH. Install it (bash <vertexia>/install.sh) or run the per-stack gate under <vertexia>/presets/." >&2
exit 127
