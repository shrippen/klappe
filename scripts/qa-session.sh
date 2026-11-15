#!/usr/bin/env bash
# One isolated session of Klappe in its own X server, for QA rounds. Klappe never sees the
# desktop, real files, sound or accounts:
#
#   Xvfb (free display) ─► D-Bus session + AT-SPI bus ─► klappe (xcb, no Wayland, own HOME and
#   XDG folders, a copy of the example project) ─► [--run CMD] ─► clean up
#
#   scripts/qa-session.sh start WORKDIR [--size 1440x900] [--language de|en] [--scale 1]
#                         [--open] [--run CMD | --prepare-only] [-- <klappe args>]
#   scripts/qa-session.sh stop WORKDIR
#
# start        starts the session in the background (one per WORKDIR, several in parallel,
#              each with its own display and buses) and writes into WORKDIR:
#                session.env   DISPLAY, bus addresses, process ids
#                qa            wrapper for scripts/qactl.py with this session and folder:
#                              WORKDIR/qa shot | click X Y | tree | ... | stop
#                shot-NNN.png, actions.log   (by qactl.py, kept after stop)
#                sandbox/      HOME, XDG folders, example copy, logs; nothing outside WORKDIR
# stop         ends the session and all its processes (WORKDIR/qa stop does the same)
# --size       screen and window size, the window fills it (default 1440x900)
# --language   app language and, with --open, the example (default de: demo/beispiel-de.klappe)
# --open       open a copy of the example project at once (sandbox/projects/); without it Klappe
#              starts on the start page with no project, as for a new user ("Beispiel ansehen"
#              makes its own copy in the session's XDG data folder)
# --scale      Qt scale factor (default 1: picture pixels equal click coordinates)
# --run CMD    stay in the foreground: run CMD with DISPLAY and the buses set, then stop
# --prepare-only  only prepare the folders and the wrapper, print WORKDIR: no X server, no Klappe
#
# Klappe binary: $KLAPPE_BIN, else $CARGO_TARGET_DIR/debug/klappe, else target/debug/klappe.
# Internal: --serve is what start runs in the background.
set -euo pipefail
# Paths given by the caller and --run hold from the caller's folder.
caller="$PWD"
repo="$(cd "$(dirname "$0")/.." && pwd)"

STOP_WAIT_DS=100
WINDOW_WAIT_DS=300
START_WAIT_DS=300
PAINT_WAIT_DS=100
ENV_FILE=session.env
PID_FILE=qa.pid

stop_session() {
    local dir="$1" pid
    pid="$(cat "$dir/$PID_FILE" 2>/dev/null || true)"
    if [ -z "$pid" ] || ! kill -0 "$pid" 2>/dev/null; then
        echo "qa-session: no running session in $dir" >&2
        return 0
    fi
    kill -TERM "$pid"
    for _ in $(seq "$STOP_WAIT_DS"); do
        kill -0 "$pid" 2>/dev/null || return 0
        sleep 0.1
    done
    echo "qa-session: session $pid did not stop" >&2
    return 1
}

command="${1:-}"
dir="${2:-}"
if [ -z "$dir" ] || { [ "$command" != start ] && [ "$command" != stop ]; }; then
    echo "usage: qa-session.sh start|stop WORKDIR (see the head of this script)" >&2
    exit 2
fi
dir="$(realpath -m "$dir")"
if [ "$command" = stop ]; then
    stop_session "$dir"
    exit $?
fi
shift 2

lang=de size="1440x900" scale=1 run="" mode=background open=0
orig_args=("$@")
while [ $# -gt 0 ]; do
    case "$1" in
        --size) size="$2"; shift 2 ;;
        --language) lang="$2"; shift 2 ;;
        --scale) scale="$2"; shift 2 ;;
        --open) open=1; shift ;;
        --run) run="$2"; mode=foreground; shift 2 ;;
        --serve) mode=foreground; shift ;;
        --prepare-only) mode=prepare; shift ;;
        --) shift; break ;;
        *) echo "qa-session: unknown option $1 (see the head of this script)" >&2; exit 2 ;;
    esac
done
case "$lang" in
    de|en) ;;
    *) echo "qa-session: --language de or en" >&2; exit 2 ;;
esac
if ! [[ "$size" =~ ^[0-9]+x[0-9]+$ ]]; then
    echo "qa-session: --size WIDTHxHEIGHT" >&2
    exit 2
fi

app="${KLAPPE_BIN:-${CARGO_TARGET_DIR:-$repo/target}/debug/klappe}"
app="$(realpath -m "$app")"
if [ "$mode" != prepare ] && [ ! -x "$app" ]; then
    echo "qa-session: $app not found; cargo build -p klappe-app first" >&2
    exit 2
fi
if [ "$mode" = background ] && [ -e "$dir/$PID_FILE" ] && kill -0 "$(cat "$dir/$PID_FILE")" 2>/dev/null; then
    echo "qa-session: a session already runs in $dir (stop it first)" >&2
    exit 2
fi

# Everything the session touches lies under the work folder, never under ~/.config and the like.
sandbox="$dir/sandbox"
mkdir -p "$sandbox/home" "$sandbox/config" "$sandbox/cache" "$sandbox/data" \
    "$sandbox/runtime" "$sandbox/projects"
chmod 700 "$sandbox/runtime"

# The wrapper: qactl.py with this session and work folder; "stop" ends the session.
cat > "$dir/qa" <<WRAPPER
#!/usr/bin/env bash
if [ "\${1:-}" = stop ]; then
    exec "$repo/scripts/qa-session.sh" stop "$dir"
fi
exec python3 "$repo/scripts/qactl.py" --session "$dir" --work "$dir" "\$@"
WRAPPER
chmod +x "$dir/qa"

# With --open a fresh copy of the example; what a session changes never reaches demo/.
open_args=()
if [ "$open" = 1 ]; then
    project="$sandbox/projects/beispiel-$lang.klappe"
    rm -rf "$project"
    cp -r "$repo/demo/beispiel-$lang.klappe" "$project"
    open_args=(--open "$project")
fi
if [ "$mode" = prepare ]; then
    echo "$dir"
    exit 0
fi

if [ "$mode" = background ]; then
    rm -f "$dir/$ENV_FILE"
    setsid "$0" start "$dir" --serve "${orig_args[@]}" > "$dir/session.log" 2>&1 < /dev/null &
    for _ in $(seq "$START_WAIT_DS"); do
        if [ -s "$dir/$ENV_FILE" ]; then
            echo "$dir"
            exit 0
        fi
        sleep 0.1
    done
    echo "qa-session: no session after 30 s (see $dir/session.log)" >&2
    exit 1
fi

echo $$ > "$dir/$PID_FILE"
xvfb_pid="" app_pid="" bus_pids=()
cleanup() {
    # Klappe, each bus and the registry run in process groups of their own: stop all of it.
    [ -n "$app_pid" ] && kill -- "-$app_pid" 2>/dev/null || true
    for pid in "${bus_pids[@]}"; do
        kill -- "-$pid" 2>/dev/null || true
    done
    [ -n "$xvfb_pid" ] && kill "$xvfb_pid" 2>/dev/null || true
    wait 2>/dev/null || true
    rm -f "$dir/$PID_FILE" "$dir/$ENV_FILE"
}
trap cleanup EXIT
trap 'exit 143' INT TERM

# Xvfb picks a free display itself and reports it (-displayfd): parallel sessions never clash.
Xvfb -displayfd 3 -screen 0 "${size}x24" -nolisten tcp 3> "$sandbox/display" \
    > "$sandbox/xvfb.log" 2>&1 &
xvfb_pid=$!
for _ in $(seq 50); do
    [ -s "$sandbox/display" ] && break
    sleep 0.1
done
if [ ! -s "$sandbox/display" ]; then
    echo "qa-session: Xvfb did not start (see $sandbox/xvfb.log)" >&2
    exit 1
fi
display="$(head -1 "$sandbox/display")"

# The environment is cleaned before anything starts, the buses too: a bus started with the
# desktop's environment let a portal attach to the real Wayland session. HOME is inside the
# work folder (file dialogs start there); PULSE_SERVER points nowhere (no sound).
clean=(env -u WAYLAND_DISPLAY -u XDG_SESSION_TYPE -u DBUS_SESSION_BUS_ADDRESS
    QT_QPA_PLATFORM=xcb DISPLAY=":$display" HOME="$sandbox/home" XDG_RUNTIME_DIR="$sandbox/runtime"
    XDG_CONFIG_HOME="$sandbox/config" XDG_CACHE_HOME="$sandbox/cache" XDG_DATA_HOME="$sandbox/data"
    QT_AUTO_SCREEN_SCALE_FACTOR=0 QT_SCALE_FACTOR="$scale" QT_FORCE_STDERR_LOGGING=1
    PULSE_SERVER=unix:/nonexistent)
# Private buses for Klappe and the --run command (qactl talks to Klappe over AT-SPI): a
# session bus, the accessibility bus beside it and the AT-SPI registry on it. Not by
# activation: at-spi-bus-launcher starts the registry through systemd where dbus-broker is
# used, which the clean environment must not reach. Qt and qactl find it by AT_SPI_BUS_ADDRESS.
A11Y_CONF=/usr/share/defaults/at-spi2/accessibility.conf
REGISTRY=/usr/lib/at-spi2-registryd
bus() {
    local name="$1"
    shift
    setsid "${clean[@]}" dbus-daemon "$@" --nofork --print-address=3 3> "$sandbox/$name" \
        > "$sandbox/$name.log" 2>&1 &
    bus_pids+=("$!")
    for _ in $(seq 50); do
        [ -s "$sandbox/$name" ] && return
        sleep 0.1
    done
    echo "qa-session: bus $name did not start (see $sandbox/$name.log)" >&2
    exit 1
}
# The session bus has no service directories: nothing is started by activation, so no portal
# (a file dialog is Qt's own) and no service of the desktop's systemd.
cat > "$sandbox/session.conf" <<CONF
<busconfig>
  <type>session</type>
  <listen>unix:dir=$sandbox/runtime</listen>
  <auth>EXTERNAL</auth>
  <policy context="default">
    <allow send_destination="*" eavesdrop="true"/>
    <allow eavesdrop="true"/>
    <allow own="*"/>
  </policy>
</busconfig>
CONF
bus session --config-file="$sandbox/session.conf"
bus a11y --config-file="$A11Y_CONF" --address="unix:dir=$sandbox/runtime"
clean+=(DBUS_SESSION_BUS_ADDRESS="$(head -1 "$sandbox/session")"
    AT_SPI_BUS_ADDRESS="$(head -1 "$sandbox/a11y")" QT_LINUX_ACCESSIBILITY_ALWAYS_ON=1)
setsid "${clean[@]}" "$REGISTRY" > "$sandbox/registry.log" 2>&1 &
bus_pids+=("$!")

setsid "${clean[@]}" "$app" --language "$lang" "${open_args[@]}" "$@" > "$sandbox/klappe.log" 2>&1 &
app_pid=$!

# Wait for the window (xdotool). The X server is Klappe's alone: any window is its own.
window=""
for _ in $(seq "$WINDOW_WAIT_DS"); do
    window="$(DISPLAY=":$display" xdotool search --onlyvisible --name . 2>/dev/null | head -1 || true)"
    [ -n "$window" ] && break
    if ! kill -0 "$app_pid" 2>/dev/null; then
        echo "qa-session: Klappe ended before its window appeared (see $sandbox/klappe.log)" >&2
        exit 1
    fi
    sleep 0.1
done
if [ -z "$window" ]; then
    echo "qa-session: no window after 30 s (see $sandbox/klappe.log)" >&2
    exit 1
fi
# The first frame: a screen that is still black has not been painted yet.
for _ in $(seq "$PAINT_WAIT_DS"); do
    average="$(DISPLAY=":$display" import -window root -resize 1x1 txt:- 2> /dev/null | tail -1)"
    case "$average" in *"#000000"*) sleep 0.1 ;; *) break ;; esac
done
# No window manager: fill the screen and take the focus ourselves.
DISPLAY=":$display" xdotool windowmove "$window" 0 0 windowsize "$window" "${size%x*}" "${size#*x}" \
    windowfocus "$window" 2> /dev/null || true

# Everything a later call needs, as `export KEY=VALUE` lines (scripts/qactl.py reads them).
{
    for pair in "DISPLAY=:$display" "DBUS_SESSION_BUS_ADDRESS=$(head -1 "$sandbox/session")" \
        "AT_SPI_BUS_ADDRESS=$(head -1 "$sandbox/a11y")" "XDG_RUNTIME_DIR=$sandbox/runtime" \
        "QA_WINDOW=$window" "QA_APP_PID=$app_pid" "QA_SESSION=$dir" "QA_WORK=$dir"; do
        printf 'export %s=%q\n' "${pair%%=*}" "${pair#*=}"
    done
} > "$dir/$ENV_FILE.tmp"
mv "$dir/$ENV_FILE.tmp" "$dir/$ENV_FILE"
echo "qa-session: DISPLAY=:$display window $window dir $dir" >&2

if [ -n "$run" ]; then
    status=0
    (cd "$caller" && "${clean[@]}" QA_WINDOW="$window" QA_SESSION="$dir" QA_WORK="$dir" bash -c "$run") || status=$?
    exit "$status"
fi
wait "$app_pid" || true
