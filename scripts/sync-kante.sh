#!/usr/bin/env bash
# Copy Kante into this repository, unchanged (GUI rule): its QML module into the app and its
# colour source into the export crate; record the Kante commit with each.
# KANTE_DS: Kante checkout, default ../Kante next to this repository.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
src="${KANTE_DS:-$root/../Kante}"
dest="$root/crates/export/kante"

if [[ ! -f "$src/tokens/palette.json" ]]; then
    echo "sync-kante: $src/tokens/palette.json not found; set KANTE_DS" >&2
    exit 1
fi

qml="$root/crates/app/qml/Kante"
if [[ ! -f "$src/qml/Kante/qmldir" ]]; then
    echo "sync-kante: $src/qml/Kante not found; set KANTE_DS" >&2
    exit 1
fi
rm -rf "$qml"
cp -r "$src/qml/Kante" "$qml"
git -C "$src" rev-parse HEAD >"$qml/.kante-version"

mkdir -p "$dest"
cp "$src/tokens/palette.json" "$dest/palette.json"
git -C "$src" rev-parse HEAD >"$dest/.kante-version"

echo "sync-kante: QML module and palette.json from Kante $(cat "$dest/.kante-version")"
