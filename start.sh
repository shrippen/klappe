#!/bin/sh
# Builds and starts Klappe from this checkout.
#   ./start.sh            the app (start page)
#   ./start.sh app ARGS   the app, e.g. --open testdata/beispiel.klappe
#   ./start.sh tui ARGS   the terminal editor for scripts
#   ./start.sh cli ARGS   the command line, e.g. --help
# Release build: the first start takes a few minutes, later ones only rebuild what changed.
set -eu
cd "$(dirname "$0")"

part=${1:-app}
[ $# -gt 0 ] && shift

case $part in
    app) package=klappe-app ;;
    tui) package=klappe-tui ;;
    cli) package=klappe-cli ;;
    *) echo "usage: ./start.sh [app|tui|cli] [args…]" >&2; exit 64 ;;
esac

exec cargo run -q --release -p "$package" -- "$@"
