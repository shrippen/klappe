#!/usr/bin/env bash
# Test of scripts/qa-session.sh and scripts/qactl.py: the data folders, two parallel sessions,
# a picture, the AT-SPI tree, a file dialog that opens in the session and not on the desktop,
# no process left. Needs a built klappe ($KLAPPE_BIN, $CARGO_TARGET_DIR/debug/klappe or
# target/debug/klappe); the live part is skipped without it.
#
#   scripts/qa-session.test.sh
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
real_display="${DISPLAY:-}"
base="${TMPDIR:-$HOME/.cache/claude}"
mkdir -p "$base"
work="$(mktemp -d -p "$base" qa-session-test.XXXXXX)"
session="$root/scripts/qa-session.sh"
sessions=()
cleanup() {
    for s in "${sessions[@]}"; do
        "$session" stop "$s" > /dev/null 2>&1 || true
    done
    rm -rf "$work"
}
trap cleanup EXIT

DIALOG_WAIT_S=15
PAINT_SIZE=1440x900

fail() {
    echo "FAIL: $*"
    exit 1
}

# Windows of an X server (ids, sorted).
windows() {
    DISPLAY="$1" xdotool search --onlyvisible --name . 2> /dev/null | sort || true
}

# --- Folders: --prepare-only starts nothing and writes only below the work folder.
fakehome="$work/fakehome"
mkdir "$fakehome"
prepared="$(HOME="$fakehome" XDG_CONFIG_HOME= KLAPPE_BIN=/bin/true "$session" start "$work/prep" --prepare-only)"
[ "$prepared" = "$work/prep" ] || fail "--prepare-only printed '$prepared'"
[ -z "$(ls -A "$fakehome")" ] || fail "something was written to HOME: $(ls -A "$fakehome")"
for sub in home config cache data runtime projects; do
    [ -d "$prepared/sandbox/$sub" ] || fail "no sandbox/$sub"
done
[ -z "$(ls -A "$prepared/sandbox/projects")" ] || fail "the start page mode copied a project"
opened_dir="$(HOME="$fakehome" KLAPPE_BIN=/bin/true "$session" start "$work/prep-open" --open --language en --prepare-only)"
copy="$opened_dir/sandbox/projects/beispiel-en.klappe"
[ -d "$copy" ] && [ ! -L "$copy" ] || fail "--open: the example is not a copy"
[ -z "$(ls -A "$fakehome")" ] || fail "--open wrote to HOME"
[ -x "$prepared/qa" ] || fail "no executable qa wrapper"
echo "ok: folders"

# --- Live part.
app="${KLAPPE_BIN:-${CARGO_TARGET_DIR:-$root/target}/debug/klappe}"
if [ ! -x "$app" ]; then
    echo "SKIP: live part, $app not built"
    echo "qa-session.test: ok (folders only)"
    exit 0
fi
export KLAPPE_BIN="$app"
marker="$work/marker"
touch "$marker"
desktop_before="$(windows "$real_display")"

a="$work/a"
b="$work/b"
sessions+=("$a" "$b")
"$session" start "$a" > /dev/null
"$session" start "$b" --open --language en --size 1280x800 > /dev/null
disp_a="$(. "$a/session.env" && echo "$DISPLAY")"
disp_b="$(. "$b/session.env" && echo "$DISPLAY")"
[ "$disp_a" != "$disp_b" ] || fail "both sessions use $disp_a"
[ "$disp_a" != "$real_display" ] || fail "session A uses the real display"
echo "ok: two sessions ($disp_a, $disp_b)"

# A picture of each, in its size.
shot_a="$("$a/qa" shot)"
shot_b="$("$b/qa" shot)"
[ "$(identify -format %wx%h "$shot_a")" = 1440x900 ] || fail "picture A has the wrong size: $shot_a"
[ "$(identify -format %wx%h "$shot_b")" = 1280x800 ] || fail "picture B has the wrong size: $shot_b"
[ "$(magick "$shot_a" -format %k info:)" -gt 1 ] || fail "picture A is one colour (not painted)"
echo "ok: pictures $shot_a"

# AT-SPI: elements of each session.
"$a/qa" tree | grep -q '^  frame ' || fail "tree A has no window"
"$b/qa" tree | grep -q '^  frame ' || fail "tree B has no window"
# A: start page; B (--open): the project is open, the start page is not there.
"$a/qa" find --role button --contains "Projekt öffnen" | grep -q . || fail "A is not on the start page"
pid_b="$(. "$b/session.env" && echo "$QA_APP_PID")"
tr '\0' '\n' < "/proc/$pid_b/cmdline" | grep -qx "$b/sandbox/projects/beispiel-en.klappe" \
    || fail "B was not started with --open on its copy"
[ -z "$("$b/qa" find --role button --contains "Open project")" ] || fail "B shows the start page"
"$a/qa" find --role button --contains Datei | grep -q Datei || fail "find A: no Datei button"
echo "ok: tree and find"

# A file dialog in session A: a new window there, none on the desktop, none in session B.
before_a="$(windows "$disp_a")"
before_b="$(windows "$disp_b")"
"$a/qa" press --role button --contains "Projekt öffnen" > /dev/null
opened=""
for _ in $(seq $((DIALOG_WAIT_S * 5))); do
    after_a="$(windows "$disp_a")"
    if [ "$after_a" != "$before_a" ]; then
        opened=1
        break
    fi
    sleep 0.2
done
[ -n "$opened" ] || fail "no new window in session A after the open button"
[ "$(windows "$disp_b")" = "$before_b" ] || fail "session B got a window too"
"$a/qa" shot > /dev/null
echo "ok: the file dialog opened in session A"

# The desktop: no new X window (Xwayland) and no Klappe process with the desktop's display.
if [ -n "$real_display" ] && xdotool getdisplaygeometry > /dev/null 2>&1; then
    [ "$(windows "$real_display")" = "$desktop_before" ] || fail "a window appeared on $real_display"
    echo "ok: no new window on $real_display"
else
    echo "NOT CHECKABLE: no real X display to compare"
fi
for s in "$a" "$b"; do
    pid="$(. "$s/session.env" && echo "$QA_APP_PID")"
    env_of="$(tr '\0' '\n' < "/proc/$pid/environ")"
    grep -q '^WAYLAND_DISPLAY=' <<< "$env_of" && fail "Klappe in $s sees WAYLAND_DISPLAY"
    grep -q "^XDG_CONFIG_HOME=$s/sandbox/config$" <<< "$env_of" || fail "Klappe in $s has the wrong XDG_CONFIG_HOME"
    grep -q "^HOME=$s/sandbox/home$" <<< "$env_of" || fail "Klappe in $s has the wrong HOME"
    grep -q '^QT_QPA_PLATFORM=xcb$' <<< "$env_of" || fail "Klappe in $s is not on xcb"
done
echo "NOT CHECKABLE: native Wayland windows of the desktop (no tool for the compositor); Klappe has no WAYLAND_DISPLAY and cannot open one"
# Nothing of Klappe in the real settings or caches.
leaked="$(find "$HOME/.config" "$HOME/.local/share" "$HOME/.cache" -maxdepth 2 -newer "$marker" \
    \( -ipath '*shrippen*' -o -ipath '*klappe*' \) -not -path "$HOME/.cache/claude/*" 2> /dev/null || true)"
[ -z "$leaked" ] || fail "files in the real folders: $leaked"
echo "ok: nothing in the real config, cache or data folders"

# The example tile makes its copy in the session's data folder (close the dialog first).
"$a/qa" key Escape
sleep 1
# The tile has no accessible name, so it is clicked: the first tile on a start page without
# recent projects (picture pixels equal coordinates).
EXAMPLE_TILE_X=128
EXAMPLE_TILE_Y=190
"$a/qa" click "$EXAMPLE_TILE_X" "$EXAMPLE_TILE_Y"
for _ in $(seq $((DIALOG_WAIT_S * 5))); do
    made="$(find "$a/sandbox/data" -maxdepth 4 -name '*.klappe' 2> /dev/null | head -1)"
    [ -n "$made" ] && break
    sleep 0.2
done
[ -n "${made:-}" ] || fail "the example tile made no copy in $a/sandbox/data"
echo "ok: example copy $made"

# actions.log: time, command, arguments, picture number.
[ "$(grep -c "	shot	" "$a/actions.log")" -ge 2 ] || fail "actions.log lacks the shots"
grep -q "	press	.*	shot=001$" "$a/actions.log" || fail "actions.log lacks the press line"
echo "ok: actions.log"

# Stop: no process, no X server, pictures stay.
pid_a="$(. "$a/session.env" && echo "$QA_APP_PID")"
pid_b="$(. "$b/session.env" && echo "$QA_APP_PID")"
"$a/qa" stop
"$session" stop "$b"
for pid in "$pid_a" "$pid_b"; do
    ! kill -0 "$pid" 2> /dev/null || fail "Klappe $pid still runs"
done
left="$(ps -eo pid,args | grep -F "$work" | grep -v -e grep -e "$$" || true)"
[ -z "$left" ] || fail "processes left: $left"
[ -z "$(windows "$disp_a")" ] && ! DISPLAY="$disp_a" xdotool getdisplaygeometry 2> /dev/null \
    || fail "X server $disp_a still runs"
[ -f "$shot_a" ] || fail "the picture is gone after stop"
sessions=()
echo "ok: stopped, nothing left"
echo "qa-session.test: ok"
