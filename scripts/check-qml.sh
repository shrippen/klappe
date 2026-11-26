#!/usr/bin/env bash
# Runs the app's QML tests (crates/app/tests/qml) offscreen. Needs qmltestrunner
# (qt6-declarative) and Kirigami. Without qmltestrunner it skips with a note locally, but
# fails in CI (CI set), so a missing package cannot turn the step green.
# An argument names another folder of tests (`make perf`).
set -euo pipefail
cd "$(dirname "$0")/.."

RUNNER=""
for c in /usr/lib/qt6/bin/qmltestrunner qmltestrunner-qt6 qmltestrunner; do
    if command -v "$c" >/dev/null 2>&1; then
        RUNNER=$(command -v "$c")
        break
    fi
done
if [ -z "$RUNNER" ]; then
    if [ -n "${CI:-}" ]; then
        echo "check-qml: qmltestrunner not found" >&2
        exit 1
    fi
    echo "check-qml: qmltestrunner not found, skipped"
    exit 0
fi

# The tests read the catalogue (crates/services/i18n/*.json) with XMLHttpRequest.
export QML_XHR_ALLOW_FILE_READ=1
export QT_QPA_PLATFORM="${QT_QPA_PLATFORM:-offscreen}"
export QT_FORCE_STDERR_LOGGING=1
# Qt 6.12's incremental garbage collector crashes the runner now and then (SIGSEGV in
# GCStateMachine during qWait; tst_modulebar.qml alone: 7 of 20 runs). 0 collects in one go.
export QV4_GC_TIMELIMIT="${QV4_GC_TIMELIMIT:-0}"
# Tests that keep settings (the working mode) write to a folder of their own, not the user's.
CONFIG=$(mktemp -d)
trap 'rm -rf "$CONFIG"' EXIT
export XDG_CONFIG_HOME="$CONFIG"
"$RUNNER" -import crates/app/qml -import crates/app/qml/Kante -input "${1:-crates/app/tests/qml}"
