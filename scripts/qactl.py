#!/usr/bin/env python3
"""Drives Klappe in a QA session (scripts/qa-session.sh): by AT-SPI (role and name) and by
picture and input (coordinates). Never against the desktop.

    scripts/qa-session.sh start WORKDIR            then: WORKDIR/qa <command>  (WORKDIR/qa stop)
    scripts/qa-session.sh start WORKDIR --run "scripts/qactl.py tree"

WORKDIR/qa is a wrapper for `qactl.py --session WORKDIR --work WORKDIR`.

    qactl.py tree [--all]                       visible widgets: role 'name' [x,y w×h]
    qactl.py find [--role R] [--contains] NAME  matching widgets
    qactl.py press [--role R] [--contains] NAME first matching widget's action (press, toggle)
    qactl.py text [--role R] [--contains] NAME  its text (Text interface, else name)
    qactl.py wait [--role R] [--contains] NAME [--timeout S]
    qactl.py layout [--json]                    cut-off, overlapping, nameless widgets (W3); exit 1 if any
    qactl.py focus                              the widget with keyboard focus (role, name, place)

For testers (picture and input only, coordinates are picture pixels):

    qactl.py shot                               picture of the session, numbered in the work folder
    qactl.py click X Y [--button N] [--double]
    qactl.py drag X1 Y1 X2 Y2
    qactl.py key COMBO                          e.g. ctrl+o, Return, shift+Tab
    qactl.py type TEXT
    qactl.py hover X Y

--session DIR (env QA_SESSION) is the session's folder (DISPLAY and buses from DIR/session.env;
inside --run they are set already). --work DIR (env QA_WORK, default the session folder) gets
the pictures (shot-001.png, ...) and actions.log: time, command, arguments, last picture number.
"""
import argparse
import enum
import json
import os
import re
import shlex
import subprocess
import sys
import time
import warnings
from datetime import datetime
from pathlib import Path

import gi

warnings.filterwarnings("ignore", category=DeprecationWarning)
gi.require_version("Atspi", "2.0")
from gi.repository import Atspi  # noqa: E402

# Klappe's process on the private bus (the binary is called "klappe").
APP_NAME = "klappe"
WAIT_S = 10.0
POLL_S = 0.2
# Actions that activate a widget, in order of preference (Qt names them like this).
ACTIONS = ("press", "toggle", "click", "activate", "showmenu")
# Widgets a user acts on or reads: these must not be cut off or lie on top of each other.
INTERACTIVE = {
    "button", "push button", "toggle button", "check box", "radio button", "combo box", "slider",
    "spin button", "text", "entry", "page tab", "menu item", "list item", "link",
}
READABLE = INTERACTIVE | {"label", "static", "heading"}
# Roles that scroll their content: what lies outside them is scrolled away, not cut off.
SCROLLING = {"scroll pane", "viewport", "list", "tree", "table", "page tab list"}
SCROLL_BAR = "scroll bar"
# Pixels two widgets may share before they count as overlapping (antialiased borders).
TOLERANCE_PX = 2


def lookup():
    """Klappe's accessible application, None while it has not registered yet."""
    desktop = Atspi.get_desktop(0)
    for i in range(desktop.get_child_count()):
        child = desktop.get_child_at_index(i)
        if child is not None and (child.get_name() or "").lower() == APP_NAME:
            return child
    return None


def app(timeout_s=WAIT_S):
    """Klappe's accessible application; waits up to `timeout_s` for it, then exits."""
    deadline = time.monotonic() + timeout_s
    while (found := lookup()) is None:
        if time.monotonic() > deadline:
            sys.exit(f"qactl: no {APP_NAME} on the accessibility bus; "
                     "run inside scripts/qa-session.sh or give --session")
        time.sleep(POLL_S)
    return found


def showing(node):
    states = node.get_state_set()
    return states.contains(Atspi.StateType.SHOWING) and states.contains(Atspi.StateType.VISIBLE)


def walk(node, depth=0, everything=False):
    """(depth, node) of `node` and its descendants; hidden subtrees are skipped unless
    `everything`."""
    if node is None:
        return
    if not everything and depth > 0 and not showing(node):
        return
    yield depth, node
    for i in range(node.get_child_count()):
        yield from walk(node.get_child_at_index(i), depth + 1, everything)


def extents(node):
    """(x, y, w, h) on screen."""
    r = node.get_extents(Atspi.CoordType.SCREEN)
    return r.x, r.y, r.width, r.height


def describe(node):
    x, y, w, h = extents(node)
    return f"{node.get_role_name()} '{node.get_name() or ''}' [{x},{y} {w}×{h}]"


def children(node):
    return [c for c in (node.get_child_at_index(i) for i in range(node.get_child_count()))
            if c is not None]


def separator_only(node):
    """A list entry that only holds a separator and nameless parts (the drawer's dividers)."""
    parts = children(node)
    roles = {c.get_role_name() for c in parts}
    return "separator" in roles and not any((c.get_name() or "").strip() for c in parts)


def scrolls(node):
    """A widget that scrolls its content: a scrolling role or a scroll bar of its own."""
    return node.get_role_name() in SCROLLING or any(
        c.get_role_name() == SCROLL_BAR for c in children(node))


def matches(node, args):
    if args.role and node.get_role_name() != args.role:
        return False
    name = node.get_name() or ""
    if args.contains:
        return args.name.lower() in name.lower()
    return name == args.name


def find(args):
    return [n for _, n in walk(app()) if matches(n, args)]


def first(args):
    found = find(args)
    if not found:
        sys.exit(f"qactl: no widget '{args.name}'" + (f" ({args.role})" if args.role else ""))
    return found[0]


def text_of(node):
    try:
        count = Atspi.Text.get_character_count(node)
        return Atspi.Text.get_text(node, 0, count)
    except Exception:  # noqa: BLE001 (no Text interface: GLib raises a plain error)
        return node.get_name() or ""


def press(node):
    """Runs the widget's activating action; Qt lists them by name."""
    # get_name is deprecated since AT-SPI 2.10; 2.62 no longer exposes it.
    names = [Atspi.Action.get_action_name(node, i).lower()
             for i in range(Atspi.Action.get_n_actions(node))]
    for wanted in ACTIONS:
        if wanted in names:
            return Atspi.Action.do_action(node, names.index(wanted))
    sys.exit(f"qactl: {describe(node)} has no action to press (has {names})")


def inside(inner, outer, axes="xy"):
    """Whether `inner` lies in `outer` along `axes` ("x": only left and right)."""
    ix, iy, iw, ih = inner
    ox, oy, ow, oh = outer
    across = ix >= ox - TOLERANCE_PX and ix + iw <= ox + ow + TOLERANCE_PX
    down = iy >= oy - TOLERANCE_PX and iy + ih <= oy + oh + TOLERANCE_PX
    return across and (down or axes == "x")


def overlap(a, b):
    ax, ay, aw, ah = a
    bx, by, bw, bh = b
    w = min(ax + aw, bx + bw) - max(ax, bx)
    h = min(ay + ah, by + bh) - max(ay, by)
    return w > TOLERANCE_PX and h > TOLERANCE_PX


def layout_findings(root):
    """Per window: readable widgets cut off by the window edge, interactive ones on top of each
    other, interactive ones without a name. Inside a scrolling area only the sides count: what
    runs off below is scrolled, not cut (Qt marks widgets scrolled out of view as not showing).
    Each finding is a dict with "kind", "widget" and maybe "other"."""
    findings = []
    for window in (w for w in children(root) if showing(w)):
        frame = extents(window)
        placed = []

        def visit(node, scrolled):
            role = node.get_role_name()
            box = extents(node)
            sized = box[2] > 0 and box[3] > 0
            # A nameless label shows nothing that could be cut (spacers, canvases).
            named = bool((node.get_name() or "").strip())
            if sized and named and role in READABLE \
                    and not inside(box, frame, "x" if scrolled else "xy"):
                findings.append({"kind": "outside window", "widget": describe(node)})
            if sized and role in INTERACTIVE:
                if not named and not separator_only(node):
                    findings.append({"kind": "no name", "widget": describe(node)})
                placed.append((node, box))
            for child in children(node):
                if showing(child):
                    visit(child, scrolled or scrolls(node))

        visit(window, False)

        # Interactive widgets on top of each other, unless one holds the other.
        for i, (a, abox) in enumerate(placed):
            for b, bbox in placed[i + 1:]:
                if inside(abox, bbox) or inside(bbox, abox):
                    continue
                if overlap(abox, bbox):
                    findings.append({"kind": "overlap", "widget": describe(a),
                                     "other": describe(b)})
    return findings


# ---- Session, pictures and input (testers) ----

ENV_FILE = "session.env"
PID_FILE = "qa.pid"
SESSION_SCRIPT = "qa-session.sh"
ENV_KEYS = ("DISPLAY", "DBUS_SESSION_BUS_ADDRESS", "AT_SPI_BUS_ADDRESS", "XDG_RUNTIME_DIR")
ACTIONS_LOG = "actions.log"
SHOT_PATTERN = re.compile(r"shot-(\d+)\.png$")
SHOT_NAME = "shot-{:03d}.png"
# xdotool is slow to deliver motion to Qt when events arrive back to back.
STEP_S = 0.03
DRAG_STEPS = 8
DRAG_SETTLE_S = 0.15
DOUBLE_CLICK_MS = 80
TYPE_DELAY_MS = 20
# What reads the screen only and is not an action: not logged.
READ_ONLY = {"tree", "find", "text", "wait", "layout", "focus"}


class Outcome(enum.Enum):
    DONE = enum.auto()
    FAILED = enum.auto()


def start_time(pid):
    """Field 22 of /proc/PID/stat (after the command name, which may hold spaces)."""
    stat = Path(f"/proc/{pid}/stat").read_text()
    return stat.rsplit(")", 1)[1].split()[19]


def session_runs(folder):
    """Whether WORKDIR/qa.pid ("PID STARTTIME", written by the session) is a running qa-session.sh
    that was started at that time, as scripts/qa-session.sh checks."""
    try:
        pid, started = (folder / PID_FILE).read_text().split()
        words = Path(f"/proc/{pid}/cmdline").read_bytes().decode(errors="replace").split("\0")
        return start_time(pid) == started and any(
            Path(word).name == SESSION_SCRIPT for word in words)
    except (OSError, ValueError, IndexError):
        return False


def load_session(folder):
    """Sets DISPLAY and the bus addresses of the session in `folder` (its session.env). Without
    a valid session nothing runs: the caller's own DISPLAY and buses are the desktop."""
    if not folder:
        sys.exit("qactl: no session; use WORKDIR/qa or --session WORKDIR "
                 "(scripts/qa-session.sh start WORKDIR)")
    path = Path(folder) / ENV_FILE
    if not path.is_file():
        sys.exit(f"qactl: no {path}; start the session with scripts/qa-session.sh start")
    found = {}
    for line in path.read_text().splitlines():
        key, _, value = shlex.split(line)[-1].partition("=")
        if key in ENV_KEYS:
            found[key] = value
    missing = [key for key in ENV_KEYS if key not in found]
    if missing:
        sys.exit(f"qactl: {path} lacks {', '.join(missing)}")
    if not session_runs(Path(folder)):
        sys.exit(f"qactl: no session runs in {folder} (stopped? scripts/qa-session.sh start)")
    os.environ.update(found)
    os.environ.pop("WAYLAND_DISPLAY", None)


def last_shot(work):
    """Number of the newest picture in `work`, 0 if none."""
    numbers = [int(m.group(1)) for p in work.iterdir() if (m := SHOT_PATTERN.match(p.name))]
    return max(numbers, default=0)


def log_action(work, args, outcome=Outcome.DONE):
    """One line in actions.log: time, command, its options as typed, last picture number, and
    FAILED if the action did not work."""
    command_at = sys.argv.index(args.command, 1)
    typed = shlex.join(sys.argv[command_at + 1:])
    stamp = datetime.now().isoformat(timespec="seconds")
    fields = [stamp, args.command, typed, f"shot={last_shot(work):03d}"]
    with open(work / ACTIONS_LOG, "a") as log:
        log.write("\t".join(fields + (["FAILED"] if outcome is Outcome.FAILED else [])) + "\n")


def xdotool(*words):
    result = subprocess.run(["xdotool", *map(str, words)], capture_output=True, text=True)
    if result.returncode != 0:
        sys.exit(f"qactl: xdotool {' '.join(map(str, words))}: {result.stderr.strip()}")


def shot(work):
    path = work / SHOT_NAME.format(last_shot(work) + 1)
    result = subprocess.run(["import", "-window", "root", str(path)], capture_output=True, text=True)
    if result.returncode != 0:
        sys.exit(f"qactl: import: {result.stderr.strip()}")
    print(path)


def click(x, y, button, double):
    xdotool("mousemove", x, y)
    time.sleep(STEP_S)
    repeat = ["--repeat", 2, "--delay", DOUBLE_CLICK_MS] if double else []
    xdotool("click", *repeat, button)


def drag(x1, y1, x2, y2):
    """Press at 1, move to 2 in steps (Qt needs the motion), release."""
    xdotool("mousemove", x1, y1)
    time.sleep(STEP_S)
    xdotool("mousedown", 1)
    time.sleep(DRAG_SETTLE_S)
    for step in range(1, DRAG_STEPS + 1):
        xdotool("mousemove", x1 + (x2 - x1) * step // DRAG_STEPS,
                y1 + (y2 - y1) * step // DRAG_STEPS)
        time.sleep(STEP_S)
    time.sleep(DRAG_SETTLE_S)
    xdotool("mouseup", 1)


def input_command(args, work):
    """Runs a picture or input command."""
    if args.command == "shot":
        shot(work)
    elif args.command == "click":
        click(args.x, args.y, args.button, args.double)
    elif args.command == "drag":
        drag(args.x1, args.y1, args.x2, args.y2)
    elif args.command == "key":
        xdotool("key", "--clearmodifiers", args.combo)
    elif args.command == "type":
        xdotool("type", "--delay", TYPE_DELAY_MS, "--", args.text)
    elif args.command == "hover":
        xdotool("mousemove", args.x, args.y)


# ---- AT-SPI commands ----


def atspi_command(args):
    if args.command == "tree":
        for depth, node in walk(app(), everything=args.all):
            print("  " * depth + describe(node))
    elif args.command == "find":
        for node in find(args):
            print(describe(node))
    elif args.command == "press":
        node = first(args)
        press(node)
        print(f"pressed {describe(node)}")
    elif args.command == "text":
        print(text_of(first(args)))
    elif args.command == "wait":
        wait(args)
    elif args.command == "focus":
        # The deepest focused widget: Qt marks the window and the item both.
        focused = [node for _, node in walk(app())
                   if node.get_state_set().contains(Atspi.StateType.FOCUSED)]
        print(describe(focused[-1]) if focused else "qactl: nothing has focus")
    elif args.command == "layout":
        layout(args)


def wait(args):
    deadline = time.monotonic() + args.timeout
    while not find(args):
        if time.monotonic() > deadline:
            sys.exit(f"qactl: '{args.name}' did not appear within {args.timeout} s")
        time.sleep(POLL_S)
    print(describe(find(args)[0]))


def layout(args):
    findings = layout_findings(app())
    if args.json:
        print(json.dumps(findings, ensure_ascii=False, indent=1))
    else:
        for f in findings:
            rest = f.get("other", "")
            print(f"{f['kind']}: {f['widget']}" + (f"  /  {rest}" if rest else ""))
    sys.exit(1 if findings else 0)


def parser():
    p = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    p.add_argument("--session", default=os.environ.get("QA_SESSION"))
    p.add_argument("--work", default=os.environ.get("QA_WORK"))
    sub = p.add_subparsers(dest="command", required=True)
    tree = sub.add_parser("tree")
    tree.add_argument("--all", action="store_true", help="hidden widgets too")
    for name in ("find", "press", "text", "wait"):
        q = sub.add_parser(name)
        q.add_argument("--role")
        q.add_argument("--contains", action="store_true", help="name contains NAME, any case")
        q.add_argument("name")
        if name == "wait":
            q.add_argument("--timeout", type=float, default=WAIT_S)
    sub.add_parser("focus")
    sub.add_parser("shot")
    sub.add_parser("layout").add_argument("--json", action="store_true")
    c = sub.add_parser("click")
    c.add_argument("x", type=int)
    c.add_argument("y", type=int)
    c.add_argument("--button", type=int, default=1)
    c.add_argument("--double", action="store_true")
    d = sub.add_parser("drag")
    for name in ("x1", "y1", "x2", "y2"):
        d.add_argument(name, type=int)
    h = sub.add_parser("hover")
    h.add_argument("x", type=int)
    h.add_argument("y", type=int)
    sub.add_parser("key").add_argument("combo")
    sub.add_parser("type").add_argument("text")
    return p


def act(args, work):
    if args.command == "press":
        atspi_command(args)
    else:
        input_command(args, work)


def main():
    args = parser().parse_args()
    load_session(args.session)
    work = Path(args.work or args.session)
    work.mkdir(parents=True, exist_ok=True)
    if args.command in READ_ONLY or args.command == "press":
        Atspi.init()
    if args.command in READ_ONLY:
        atspi_command(args)
        return
    try:
        act(args, work)
    except SystemExit as error:
        if error.code not in (None, 0):
            log_action(work, args, Outcome.FAILED)
        raise
    log_action(work, args)


if __name__ == "__main__":
    main()
