#!/usr/bin/env bash
# Build the app, run it in development mode against the simulator loaded with a real device's
# settings dump, open one screen, and capture that window to a PNG.
#
# Runs beside any ordinary Hyperpace instance: development mode skips the single-instance guard,
# uses its own data directory, and titles its window "Hyperpace Simulator". Only the instance this
# script started (tracked by PID) is ever stopped. KDE Plasma only: raising the window uses a KWin
# script and capturing uses spectacle.
#
# Usage: scripts/dev-screenshot.sh <screen> <output.png> [settings dump]
#   screen: buttons | performance | macros | lighting | firmware | settings | data
set -euo pipefail
SCREEN="$1"
OUT="$2"
SHADOW="${3:-${HYPERPACE_SIM_SHADOW:-}}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
STATE="${XDG_RUNTIME_DIR:-/tmp}/hyperpace-dev"
mkdir -p "$STATE"

# One build and one capture at a time, even when several people or agents run this at once. The
# app started below must not inherit this descriptor (it is closed for it with `9>&-`): an instance
# still holding the lock would block every later run for as long as it stays open.
exec 9>"$STATE/lock"
flock 9

(cd "$ROOT/ui" && npm run build >"$STATE/ui-build.log" 2>&1) || { tail -20 "$STATE/ui-build.log"; exit 1; }
(cd "$ROOT" && cargo build -p hyperpace-app >"$STATE/cargo-build.log" 2>&1) || { grep -E "^error" -A 8 "$STATE/cargo-build.log" | head -40; exit 1; }

if [[ -f "$STATE/pid" ]] && kill -0 "$(cat "$STATE/pid")" 2>/dev/null; then
    kill "$(cat "$STATE/pid")"
    sleep 1
fi

HYPERPACE_SIMULATOR=1 \
HYPERPACE_SIM_SHADOW="$SHADOW" \
HYPERPACE_STORE_ROOT="$STATE/store" \
HYPERPACE_START_SCREEN="$SCREEN" \
    "$ROOT/target/debug/hyperpace" >"$STATE/app.log" 2>&1 9>&- &
echo $! >"$STATE/pid"

sleep 8
cat >"$STATE/raise.js" <<'JS'
var wins = workspace.windowList();
for (var i = 0; i < wins.length; i++) {
    if (wins[i].caption === "Hyperpace Simulator") {
        workspace.activeWindow = wins[i];
        workspace.raiseWindow(wins[i]);
    }
}
JS
ID=$(qdbus6 org.kde.KWin /Scripting org.kde.kwin.Scripting.loadScript "$STATE/raise.js")
qdbus6 org.kde.KWin "/Scripting/Script$ID" org.kde.kwin.Script.run >/dev/null
qdbus6 org.kde.KWin /Scripting org.kde.kwin.Scripting.unloadScript "$STATE/raise.js" >/dev/null 2>&1 || true
sleep 1
spectacle -b -n -a -o "$OUT"
echo "captured $SCREEN -> $OUT"
