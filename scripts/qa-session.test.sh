#!/usr/bin/env bash
# Test of scripts/qa-session.sh and scripts/qactl.py: the data folders, two parallel sessions,
# a picture, the AT-SPI tree, a file dialog that opens in the session and not on the desktop,
# no process left. Needs a built klappe ($KLAPPE_BIN, $CARGO_TARGET_DIR/debug/klappe or
# target/debug/klappe); the live part is skipped without it.
#
#   scripts/qa-session.test.sh
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
# The checks must see a caller in the repository: Klappe inherits the folder it is started from.
cd "$root"
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

# The pid of the klappe process below `$1` (bwrap runs it a level deeper than the session's pid).
klappe_pid() {
    local todo="$1" pid
    while [ -n "$todo" ]; do
        pid="${todo%% *}"
        todo="${todo#"$pid"}"
        todo="${todo# }"
        if [ "$(cat "/proc/$pid/comm" 2> /dev/null)" = klappe ]; then
            echo "$pid"
            return
        fi
        todo="$todo $(pgrep -P "$pid" | tr '\n' ' ')"
        todo="${todo# }"
    done
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

# --- qactl.py without a session never touches the caller's display or accessibility bus.
fakebin="$work/fakebin"
mkdir "$fakebin"
printf '#!/bin/sh\necho "$0 $*" >> "%s/calls"\n' "$work" > "$fakebin/xdotool"
cp "$fakebin/xdotool" "$fakebin/import"
chmod +x "$fakebin/xdotool" "$fakebin/import"
for command in "click 1 1" "shot" "key a" "tree"; do
    # shellcheck disable=SC2086
    if env -u QA_SESSION PATH="$fakebin:$PATH" DISPLAY=:987 QA_WORK="$work/qactl-work" \
        python3 "$root/scripts/qactl.py" $command > /dev/null 2>&1; then
        fail "qactl.py $command ran without a session"
    fi
done
[ ! -e "$work/calls" ] || fail "qactl.py without a session called: $(cat "$work/calls")"
[ ! -e "$work/qactl-work" ] || fail "qactl.py without a session made its work folder"
echo "ok: qactl.py needs a session"

# --- A start that does not come up in time leaves nothing running, and stop and start never
# end a process they did not start (a foreign pid in qa.pid).
hang="$work/hang.sh"
printf '#!/bin/sh\nexec sleep 987654\n' > "$hang"
chmod +x "$hang"
slow_start() {
    QA_START_WAIT_DS=20 KLAPPE_BIN="$hang" "$session" start "$1" 2>&1
}
if slow_start "$work/slow" > "$work/slow.out"; then
    fail "start succeeded although Klappe never showed a window"
fi
left="$(ps -eo args | grep -e "$work/slow" -e 'sleep 987654' | grep -v grep || true)"
[ -z "$left" ] || fail "a failed start left processes: $left"

mkdir "$work/foreign"
sleep 987655 &
foreign=$!
echo "$foreign" > "$work/foreign/qa.pid"
"$session" stop "$work/foreign" > /dev/null 2>&1 || true
kill -0 "$foreign" 2> /dev/null || fail "stop ended a foreign process"
[ ! -e "$work/foreign/qa.pid" ] || fail "stop kept the stale qa.pid"
echo "$foreign" > "$work/foreign/qa.pid"
slow_start "$work/foreign" > "$work/foreign.out" || true
! grep -q 'already runs' "$work/foreign.out" || fail "start took a foreign pid for a session"
kill -0 "$foreign" 2> /dev/null || fail "start ended a foreign process"
kill "$foreign"
echo "ok: failed start cleans up, foreign pids stay"

# A start that is interrupted (TERM, Ctrl-C) ends the session it started.
mkdir "$work/term"
KLAPPE_BIN="$hang" timeout -s TERM 1 "$session" start "$work/term" > /dev/null 2>&1 || true
left="$(ps -eo args | grep -e "$work/term" -e 'sleep 987654' | grep -v grep || true)"
[ -z "$left" ] || fail "an interrupted start left processes: $left"
echo "ok: interrupted start cleans up"

# qactl.py with a session folder whose session does not run (no qa.pid, or a foreign pid in it)
# stops before any X or AT-SPI call.
dead="$work/dead"
mkdir "$dead"
printf 'export %s\n' DISPLAY=:987 DBUS_SESSION_BUS_ADDRESS=unix:path=/nonexistent \
    AT_SPI_BUS_ADDRESS=unix:path=/nonexistent XDG_RUNTIME_DIR=/nonexistent > "$dead/session.env"
sleep 987656 &
foreign=$!
for pidfile in none "$foreign"; do
    rm -f "$work/calls" "$dead/qa.pid"
    [ "$pidfile" = none ] || echo "$pidfile" > "$dead/qa.pid"
    if PATH="$fakebin:$PATH" python3 "$root/scripts/qactl.py" --session "$dead" click 1 1 > /dev/null 2>&1; then
        fail "qactl.py acted on a session that does not run (qa.pid: $pidfile)"
    fi
    [ ! -e "$work/calls" ] || fail "qactl.py called $(cat "$work/calls") for a dead session"
done
kill "$foreign"
echo "ok: qactl.py needs a running session"

# --- Without bwrap, start says so and stops: no silent fallback.
if QA_BWRAP=/nonexistent/bwrap KLAPPE_BIN="$hang" "$session" start "$work/nobwrap" > "$work/nobwrap.out" 2>&1; then
    fail "start succeeded without bwrap"
fi
grep -q 'user namespaces' "$work/nobwrap.out" || fail "start without bwrap said: $(cat "$work/nobwrap.out")"
[ ! -e "$work/nobwrap/session.env" ] || fail "start without bwrap made a session"
echo "ok: no bwrap, no session"

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

# Variables of the desktop the session must not inherit.
export SSH_AUTH_SOCK=/nonexistent/agent KDE_FULL_SESSION=true XDG_CURRENT_DESKTOP=KDE
export XDG_SESSION_ID=1 XDG_CONFIG_DIRS=/etc/xdg

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
pid_b="$(klappe_pid "$(. "$b/session.env" && echo "$QA_APP_PID")")"
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
    pid="$(klappe_pid "$(. "$s/session.env" && echo "$QA_APP_PID")")"
    [ -n "$pid" ] || fail "no klappe process in $s"
    env_of="$(tr '\0' '\n' < "/proc/$pid/environ")"
    grep -q '^WAYLAND_DISPLAY=' <<< "$env_of" && fail "Klappe in $s sees WAYLAND_DISPLAY"
    grep -q "^XDG_CONFIG_HOME=$s/sandbox/config$" <<< "$env_of" || fail "Klappe in $s has the wrong XDG_CONFIG_HOME"
    grep -q "^HOME=$s/sandbox/home$" <<< "$env_of" || fail "Klappe in $s has the wrong HOME"
    grep -q '^QT_QPA_PLATFORM=xcb$' <<< "$env_of" || fail "Klappe in $s is not on xcb"
    inherited="$(grep -E '^(SSH_AUTH_SOCK|XDG_CONFIG_DIRS|KDE_|XDG_CURRENT_DESKTOP|XDG_SESSION_)' <<< "$env_of" || true)"
    [ -z "$inherited" ] || fail "Klappe in $s inherited: $inherited"
    [[ "$(readlink "/proc/$pid/cwd")" == "$s/sandbox/"* ]] \
        || fail "Klappe in $s runs in $(readlink "/proc/$pid/cwd"), not in its sandbox"
    # What Klappe sees of the owner's files, through its own root (/proc/PID/root).
    seen="/proc/$pid/root"
    for hidden in "$HOME/Hacking" "$HOME/.config" "$HOME/.ssh" "$HOME/.local" "$root/scripts" "$root/agent.md"; do
        [ ! -e "$seen$hidden" ] || fail "Klappe in $s sees $hidden"
    done
    [ -d "$seen$s/sandbox/config" ] || fail "Klappe in $s does not see its own sandbox"
    [ -z "$(ls -A "$seen/run/user" 2> /dev/null)" ] || fail "Klappe in $s sees /run/user"
    [ -z "$(ls -A "$seen/tmp" 2> /dev/null | grep -v '^\.X11-unix$')" ] || fail "Klappe in $s sees /tmp"
    [ ! -e "$seen$s/qa" ] || fail "Klappe in $s sees the work folder outside its sandbox"
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
grep -q "	press	--role button --contains 'Projekt öffnen'	shot=001$" "$a/actions.log" \
    || fail "actions.log lacks the press line with its options"
"$a/qa" press "No such widget" > /dev/null 2>&1 && fail "press of a missing widget succeeded"
grep -q "	press	'No such widget'	shot=[0-9]*	FAILED$" "$a/actions.log" \
    || fail "actions.log lacks the FAILED line"
echo "ok: actions.log"

# A second start in a running session's folder, of any kind, changes nothing.
pidfile_before="$(cat "$a/qa.pid")"
for kind in "--prepare-only" "--run true"; do
    # shellcheck disable=SC2086
    if "$session" start "$a" $kind > /dev/null 2>&1; then
        fail "start $kind ran in the folder of a running session"
    fi
    [ "$(cat "$a/qa.pid" 2> /dev/null)" = "$pidfile_before" ] || fail "start $kind changed qa.pid"
    "$a/qa" tree > /dev/null || fail "session A is gone after start $kind"
done
echo "ok: a second start is refused"

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
