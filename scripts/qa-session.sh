#!/usr/bin/env bash
# One separate session of Klappe in its own X server, for QA rounds. Klappe, the buses and the
# AT-SPI registry run in bwrap: the root is read-only, /home, /run, /tmp, /mnt and /media are
# empty, only the sandbox folder, Klappe's binary and QML and the session's X socket are bound in, and
# there is no network. They still run with your UID, with an environment of their own (env -i and
# a short list), a bus of their own and no desktop service (no portal). Needs bwrap and user
# namespaces; without them start stops with a message (no fallback):
#
#   Xvfb (free display) ─► [bwrap: D-Bus session + AT-SPI bus ─► klappe (xcb, no Wayland, own HOME
#   and XDG folders, a copy of the example project)] ─► [--run CMD] ─► clean up
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

# Waits in tenths of a second. The outer wait of `start` is longer than everything the
# background session waits for in turn, so a failed start is the session's own failure.
STOP_WAIT_DS=100
XVFB_WAIT_DS=50
BUS_WAIT_DS=50
WINDOW_WAIT_DS=300
PAINT_WAIT_DS=100
START_MARGIN_DS=100
START_WAIT_DS=$((XVFB_WAIT_DS + 2 * BUS_WAIT_DS + WINDOW_WAIT_DS + PAINT_WAIT_DS + START_MARGIN_DS))
# Test hook: a short wait for the session to come up (qa-session.test.sh).
START_WAIT_DS="${QA_START_WAIT_DS:-$START_WAIT_DS}"
ENV_FILE=session.env
PID_FILE=qa.pid
self="$repo/scripts/qa-session.sh"
# Test hook: another bwrap (qa-session.test.sh).
BWRAP="${QA_BWRAP:-bwrap}"
# Kante's QML module: the binary finds it through KLAPPE_QML_DIR; it lives in the repository,
# which the jail does not show, so the jail binds it in.
qml_dir="${QA_QML_DIR:-$repo/crates/app/qml}"

# Prints the pid in WORKDIR/qa.pid if it is a background session of this work folder, that is
# /proc/PID/cmdline reads "... qa-session.sh start WORKDIR --serve ..." (or "... --run CMD");
# otherwise discards the
# file and prints nothing. A pid is never signalled on the file's word alone.
session_pid() {
    local dir="$1" pid args i
    pid="$(cat "$dir/$PID_FILE" 2>/dev/null || true)"
    if [[ "$pid" =~ ^[0-9]+$ ]] && mapfile -d '' args 2> /dev/null < "/proc/$pid/cmdline"; then
        for i in "${!args[@]}"; do
            if [ "${args[$i]}" = "$self" ] && [ "${args[$((i + 1))]:-}" = start ] \
                && [ "${args[$((i + 2))]:-}" = "$dir" ] \
                && { [ "${args[$((i + 3))]:-}" = --serve ] || [[ " ${args[*]:$((i + 3))} " == *" --run "* ]]; }; then
                echo "$pid"
                return 0
            fi
        done
    fi
    rm -f "$dir/$PID_FILE"
}

stop_session() {
    local dir="$1" pid
    pid="$(session_pid "$dir")"
    if [ -z "$pid" ]; then
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

# A second start in the folder of a running session would make the first unreachable.
if [ -n "$(session_pid "$dir")" ]; then
    echo "qa-session: a session already runs in $dir (stop it first)" >&2
    exit 2
fi

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

if [ "$mode" != prepare ] && ! "$BWRAP" --unshare-user --unshare-pid --unshare-net --ro-bind / / true 2> /dev/null; then
    echo "qa-session: bwrap with user namespaces is required (the session runs Klappe in it); not found or not allowed: $BWRAP" >&2
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
    setsid "$self" start "$dir" --serve "${orig_args[@]}" > "$dir/session.log" 2>&1 < /dev/null &
    serve_pid=$!
    # Ending this call (timeout, TERM, Ctrl-C) ends the session it started.
    end_serve() {
        kill -TERM "$serve_pid" 2> /dev/null || true
        for _ in $(seq "$STOP_WAIT_DS"); do
            kill -0 "$serve_pid" 2> /dev/null || return 0
            sleep 0.1
        done
        kill -KILL "$serve_pid" 2> /dev/null || true
    }
    trap 'end_serve; exit 143' INT TERM
    for _ in $(seq "$START_WAIT_DS"); do
        if [ -s "$dir/$ENV_FILE" ]; then
            trap - INT TERM
            echo "$dir"
            exit 0
        fi
        sleep 0.1
    done
    echo "qa-session: no session in time (see $dir/session.log)" >&2
    end_serve
    exit 1
fi

echo $$ > "$dir/$PID_FILE"
# Klappe and the buses start in the session's home, not in the caller's folder (a file dialog
# would show it).
cd "$sandbox/home"
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
# The jail has no network namespace in common with the server: it reaches the path socket
# /tmp/.X11-unix/X<N>, not the abstract one.
Xvfb -displayfd 3 -screen 0 "${size}x24" -nolisten tcp 3> "$sandbox/display" \
    > "$sandbox/xvfb.log" 2>&1 &
xvfb_pid=$!
for _ in $(seq "$XVFB_WAIT_DS"); do
    [ -s "$sandbox/display" ] && break
    sleep 0.1
done
if [ ! -s "$sandbox/display" ]; then
    echo "qa-session: Xvfb did not start (see $sandbox/xvfb.log)" >&2
    exit 1
fi
display="$(head -1 "$sandbox/display")"

# Nothing of the caller's environment is passed on (SSH agent, desktop, session and D-Bus
# variables): only what is listed here. HOME is inside the work folder (file dialogs start
# there); PULSE_SERVER points nowhere (no sound).
case "$lang" in
    de) locale=de_DE.UTF-8 ;;
    *) locale=en_US.UTF-8 ;;
esac
clean=(env -i PATH="$PATH" LANG="$locale" LC_ALL="$locale"
    QT_QPA_PLATFORM=xcb DISPLAY=":$display" HOME="$sandbox/home" XDG_RUNTIME_DIR="$sandbox/runtime"
    XDG_CONFIG_HOME="$sandbox/config" XDG_CACHE_HOME="$sandbox/cache" XDG_DATA_HOME="$sandbox/data"
    KLAPPE_QML_DIR="$qml_dir"
    QT_AUTO_SCREEN_SCALE_FACTOR=0 QT_SCALE_FACTOR="$scale" QT_FORCE_STDERR_LOGGING=1
    PULSE_SERVER=unix:/nonexistent)
# The jail around Klappe, the buses and the registry. What is not bound is empty: the owner's
# home, /run (the desktop's sockets), /tmp but the session's own X socket, other mounts. The
# SysV IPC namespace stays shared: the X server's shared memory (MIT-SHM) needs it.
hidden=()
for folder in /home /mnt /media /srv; do
    [ -d "$folder" ] && hidden+=(--tmpfs "$folder")
done
jail=("$BWRAP" --unshare-user --unshare-pid --unshare-net --unshare-uts --die-with-parent
    --new-session --ro-bind / / "${hidden[@]}" --tmpfs "$HOME" --tmpfs /run --tmpfs /tmp
    --dev /dev --proc /proc
    --bind "$sandbox" "$sandbox" --ro-bind "$app" "$app" --ro-bind "$qml_dir" "$qml_dir"
    --bind "/tmp/.X11-unix/X$display" "/tmp/.X11-unix/X$display")
# Private buses for Klappe and the --run command (qactl talks to Klappe over AT-SPI): a
# session bus, the accessibility bus beside it and the AT-SPI registry on it. Not by
# activation: at-spi-bus-launcher starts the registry through systemd where dbus-broker is
# used, which the clean environment must not reach. Qt and qactl find it by AT_SPI_BUS_ADDRESS.
A11Y_CONF=/usr/share/defaults/at-spi2/accessibility.conf
REGISTRY=/usr/lib/at-spi2-registryd
bus() {
    local name="$1"
    shift
    setsid "${jail[@]}" "${clean[@]}" dbus-daemon "$@" --nofork --print-address=3 3> "$sandbox/$name" \
        > "$sandbox/$name.log" 2>&1 &
    bus_pids+=("$!")
    for _ in $(seq "$BUS_WAIT_DS"); do
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
setsid "${jail[@]}" "${clean[@]}" "$REGISTRY" > "$sandbox/registry.log" 2>&1 &
bus_pids+=("$!")

setsid "${jail[@]}" "${clean[@]}" "$app" --language "$lang" "${open_args[@]}" "$@" > "$sandbox/klappe.log" 2>&1 &
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
