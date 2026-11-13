#!/usr/bin/env bash
# Copy Kante's colour source into the export crate and record its commit.
# KANTE_DS: Kante checkout, default ../Kante next to this repository.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
src="${KANTE_DS:-$root/../Kante}"
dest="$root/crates/export/kante"

if [[ ! -f "$src/tokens/palette.json" ]]; then
    echo "sync-kante: $src/tokens/palette.json not found; set KANTE_DS" >&2
    exit 1
fi

mkdir -p "$dest"
cp "$src/tokens/palette.json" "$dest/palette.json"
git -C "$src" rev-parse HEAD >"$dest/.kante-version"

echo "sync-kante: palette.json from Kante $(cat "$dest/.kante-version")"
