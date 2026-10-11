#!/usr/bin/env bash
# Regenerate every app icon file from assets/app-icon/raygun.svg.
#
# Needs: resvg (brew install resvg / cargo install resvg). The .ico/.icns are
# packed by `cargo xtask ico` / `cargo xtask icns`, so no iconutil is needed.
# The outputs are committed, so building and packaging never need these tools.
#
#   packaging/icons.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIR="$ROOT/assets/app-icon"
SVG="$DIR/raygun.svg"
ID="io.github.kao_offline.raygun"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

command -v resvg >/dev/null || { echo "error: resvg not found (brew install resvg / cargo install resvg)" >&2; exit 1; }

# The SVG is the full-bleed 512 tile. Windows and Linux use it as is, so the gun reads at
# 16-48 px. macOS icons follow Apple's grid: an 824/1024 body with a transparent margin, made by
# widening the viewBox (512 / 0.805 = 636, so 62 units each side).
MAC="$TMP/macos.svg"
sed 's/viewBox="0 0 512 512"/viewBox="-62 -62 636 636"/' "$SVG" >"$MAC"
grep -q 'viewBox="-62 -62 636 636"' "$MAC" || { echo "error: unexpected viewBox in $SVG" >&2; exit 1; }

render() { resvg -w "$2" -h "$2" "$1" "$3" </dev/null; }

render "$SVG" 1024 "$DIR/raygun-1024.png"
# Runtime window/Dock icon on macOS (embedded by apps/lightcraft/src/main.rs).
render "$MAC" 512 "$DIR/raygun-macos-512.png"

# Linux hicolor theme (also the runtime window icon on Windows and Linux: 256x256).
for s in 16 24 32 48 64 128 256 512; do
  mkdir -p "$DIR/hicolor/${s}x${s}/apps"
  render "$SVG" "$s" "$DIR/hicolor/${s}x${s}/apps/$ID.png"
done
mkdir -p "$DIR/hicolor/scalable/apps"
cp "$SVG" "$DIR/hicolor/scalable/apps/$ID.svg"

# Windows .ico.
ICO_PNGS=()
for s in 16 20 24 32 40 48 64 128 256; do
  render "$SVG" "$s" "$TMP/ico-$s.png"
  ICO_PNGS+=("$TMP/ico-$s.png")
done
(cd "$ROOT" && cargo run -q -p xtask -- ico "$DIR/raygun.ico" "${ICO_PNGS[@]}")

# macOS .icns (PNG-compressed elements; no iconutil needed).
(cd "$ROOT" && cargo run -q -p xtask -- icns "$DIR/raygun.icns" "$TMP/ico-128.png" "$TMP/ico-256.png" "$DIR/raygun-macos-512.png" <(render "$SVG" 1024 "$TMP/icns-1024.png" && echo "$TMP/icns-1024.png"))
echo "icons written to $DIR"
