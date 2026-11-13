#!/usr/bin/env bash
# Runs the app's QML tests (crates/app/tests/qml) offscreen. Needs qmltestrunner
# (qt6-declarative) and Kirigami; skips with a note if qmltestrunner is missing.
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
    echo "check-qml: qmltestrunner not found, skipped"
    exit 0
fi

# The tests read the catalogue (i18n/*.json) with XMLHttpRequest.
export QML_XHR_ALLOW_FILE_READ=1
export QT_QPA_PLATFORM="${QT_QPA_PLATFORM:-offscreen}"
export QT_FORCE_STDERR_LOGGING=1
exec "$RUNNER" -import crates/app/qml -import crates/app/qml/Kante -input crates/app/tests/qml
