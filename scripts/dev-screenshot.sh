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
# The app embeds the built interface at compile time, and cargo does not watch ui/build, so a
# UI-only change leaves the binary holding the previous interface and the capture shows stale
# screens. Touching the crate's build script forces the embed to happen again.
touch "$ROOT/crates/hyperpace-app/build.rs"
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

# Raise our own window, then capture the active window and CHECK IT IS OURS before accepting it.
# Capturing "the active window" on a desktop somebody is using is not safe on its own: when the
# raise does not take, the capture is of their window instead, which has happened. The compositor
# reports our window's size, and a capture that does not match it is rejected and retried. The
# window's own contents are what is compared against; nothing else on screen is kept.
cat >"$STATE/raise.js" <<'JS'
var wins = workspace.windowList();
for (var i = 0; i < wins.length; i++) {
    var w = wins[i];
    if (w.caption === "Hyperpace Simulator") {
        workspace.activeWindow = w;
        workspace.raiseWindow(w);
        var g = w.frameGeometry;
        var mine = workspace.activeWindow && workspace.activeWindow.caption === "Hyperpace Simulator";
        console.log("HYPERSHOT " + Math.round(g.width) + " " + Math.round(g.height) + " " +
                    (mine ? "mine" : "other"));
    }
}
JS

if command -v magick >/dev/null; then MAGICK=magick; else MAGICK=convert; fi
DEADLINE=$((SECONDS + 180))
CAPTURED=0
while (( SECONDS < DEADLINE )); do
    sleep 3
    SINCE=$(date -u "+%Y-%m-%d %H:%M:%S")
    ID=$(qdbus6 org.kde.KWin /Scripting org.kde.kwin.Scripting.loadScript "$STATE/raise.js")
    qdbus6 org.kde.KWin "/Scripting/Script$ID" org.kde.kwin.Script.run >/dev/null
    qdbus6 org.kde.KWin /Scripting org.kde.kwin.Scripting.unloadScript "$STATE/raise.js" >/dev/null 2>&1 || true
    sleep 1

    LINE=$(journalctl --user --since "$SINCE" --no-pager 2>/dev/null | grep -o "HYPERSHOT .*" | tail -1)
    [[ -n "$LINE" ]] || continue
    read -r _ WW WH WHOSE <<<"$LINE"
    # The raise did not take: something else still has focus, so a capture now would be its window.
    # The compositor decides the match, because the operator's own instance is titled "Hyperpace"
    # and a text comparison here would accept it.
    [[ "$WHOSE" == "mine" ]] || continue

    spectacle -b -n -a -o "$OUT" 2>/dev/null || continue
    [[ -s "$OUT" ]] || continue

    # The capture must match the window the compositor just reported, at whichever scale this
    # screen uses (1 or 1.5 here, so the ratio is checked rather than assumed).
    read -r CW CH < <($MAGICK identify -format "%w %h" "$OUT")
    if ! awk -v cw="$CW" -v ch="$CH" -v ww="$WW" -v wh="$WH" \
        'BEGIN { rx = cw / ww; ry = ch / wh; exit !(rx > 0.9 && rx < 3.1 && (rx - ry) < 0.05 && (ry - rx) < 0.05) }'; then
        continue
    fi

    # The interface is dark: a capture whose content is mostly white has not painted yet.
    MEAN=$($MAGICK "$OUT" -alpha off -gravity center -crop 50%x50%+0+0 +repage -colorspace Gray -format "%[fx:mean]" info: 2>/dev/null || echo 1)
    if awk -v m="$MEAN" 'BEGIN { exit !(m < 0.5) }'; then
        CAPTURED=1
        break
    fi
done
if (( CAPTURED == 0 )); then
    rm -f "$OUT"
    echo "could not capture the window within the deadline (never rendered, or never came to the front); see $STATE/app.log" >&2
    exit 1
fi
echo "captured $SCREEN -> $OUT"
