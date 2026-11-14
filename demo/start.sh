#!/bin/sh
# Klappe with the Studio Weber example project: own settings and folders, nothing of yours.
#   demo/start.sh [de|en]
# With SHOT_DIR and SHOT_PLAN (demo/tools/screenshots.py in shrippen.github.io) it takes the
# shots of demo/shots.json offscreen and quits. An internal tool: neither this script nor
# demo/world.json is part of a release (demo/README.md).
set -eu
cd "$(dirname "$0")/.."

lang=${1:-${DEMO_LANG:-de}}
case $lang in
    de) folder=nachtwache.klappe ;;
    en) folder=nightwatch.klappe ;;
    *) echo "usage: demo/start.sh [de|en]" >&2; exit 64 ;;
esac

cargo build -q -p klappe-app
bin=${KLAPPE_BIN:-${CARGO_TARGET_DIR:-target}/debug/klappe}

# Settings, cache and projects live in one folder; each start begins with a fresh copy of
# the example, so what a shot changes never stays. The start page shows the project's path,
# so the folder is neutral (not under your home) and a shot never shows a real path.
tmp=${TMPDIR:-/tmp}
home=${KLAPPE_DEMO_HOME:-${tmp%/}/klappe-demo}
rm -rf "$home/config" "$home/cache" "$home/data" "$home/projects"
mkdir -p "$home/config/shrippen" "$home/cache" "$home/data" "$home/projects"
cp -r "demo/beispiel-$lang.klappe" "$home/projects/$folder"
project="$home/projects/$folder"
export XDG_CONFIG_HOME="$home/config" XDG_CACHE_HOME="$home/cache" XDG_DATA_HOME="$home/data"

# The start page lists the example as the project opened last (a QSettings value).
printf '[recent]\nprojects=[\\"%s\\"]\n' "$project" > "$home/config/shrippen/Klappe.conf"

if [ -z "${SHOT_PLAN:-}" ]; then
    exec "$bin" --language "$lang"
fi

# One run per shot of the plan: {name, open, time, shot} -> a PNG named <name>.
export QT_QPA_PLATFORM=${QT_QPA_PLATFORM:-offscreen} QT_FORCE_STDERR_LOGGING=1
python3 -c '
import json, os
for s in json.loads(os.environ["SHOT_PLAN"]):
    print("|".join([s["name"], "1" if s.get("open") else "", str(s.get("time", "")), s.get("shot", "")]))
' | while IFS='|' read -r name open time shot; do
    set -- --language "$lang" --screenshot "$SHOT_DIR/$name.png"
    if [ -n "$open" ]; then
        set -- "$@" --open "$project"
    fi
    if [ -n "$time" ]; then
        set -- "$@" --time "$time"
    fi
    if [ -n "$shot" ]; then
        set -- "$@" --shot "$shot"
    fi
    "$bin" "$@"
done
