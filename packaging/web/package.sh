#!/usr/bin/env bash
# Build the browser version and zip it:  $DIST/raygun-web-<version>.zip
#
# Usage: packaging/web/package.sh [--skip-build]
#
# Needs: the wasm32-unknown-unknown target and the wasm-bindgen CLI at the version in Cargo.lock
# (see docs/web.md); `cargo xtask web` builds it. The zip holds a self-contained static site in raygun-web-<version>/ that works
# from any URL path and inside an <iframe>. Hosting notes: packaging/web/README.md.
set -euo pipefail
# shellcheck source=../env.sh
. "$(dirname "${BASH_SOURCE[0]}")/../env.sh"
HERE="$ROOT/packaging/web"

if [ "${1:-}" != "--skip-build" ]; then
  (cd "$ROOT" && cargo xtask web)
fi

SITE="$CARGO_TARGET_DIR/web"
[ -f "$SITE/index.html" ] || { echo "error: $SITE/index.html missing; run without --skip-build" >&2; exit 1; }
# Paths must be relative so the site works under any prefix.
if grep -Eq '(src|href)="/[^/]' "$SITE/index.html"; then
  echo "error: $SITE/index.html has root-absolute URLs; it would break when served from a sub-path" >&2
  exit 1
fi

NAME="raygun-web-$VERSION"
WORK="$CARGO_TARGET_DIR/web-package"
rm -rf "$WORK"
mkdir -p "$WORK/$NAME"
# Cross-compilation also writes host build scripts/proc-macros into target/web.
# Ship an explicit static-file list, never Cargo's caches or native binaries.
for file in index.html worker.js lightcraft_web.js lightcraft_web_bg.wasm raygun.svg; do
  [ -f "$SITE/$file" ] || { echo "error: $SITE/$file missing" >&2; exit 1; }
  cp "$SITE/$file" "$WORK/$NAME/"
  for suffix in gz br; do
    [ ! -f "$SITE/$file.$suffix" ] || cp "$SITE/$file.$suffix" "$WORK/$NAME/"
  done
done
if [ -f "$SITE/lightcraft_zh_hans.otf" ]; then
  [ -f "$SITE/OFL-noto-sans-cjk-sc.txt" ] || { echo "error: Chinese font licence missing" >&2; exit 1; }
  cp "$SITE/lightcraft_zh_hans.otf" "$SITE/OFL-noto-sans-cjk-sc.txt" "$WORK/$NAME/"
  for suffix in gz br; do
    [ ! -f "$SITE/lightcraft_zh_hans.otf.$suffix" ] || cp "$SITE/lightcraft_zh_hans.otf.$suffix" "$WORK/$NAME/"
  done
fi
# Sample server configs (MIME type, caching, compression); harmless where unused.
cp "$HERE/_headers" "$HERE/.htaccess" "$WORK/$NAME/"
cp "$HERE/README.md" "$WORK/$NAME/HOSTING.md"
copy_docs "$WORK/$NAME"
rm -f "$DIST/$NAME.zip"
(cd "$WORK" && zip -qr9 "$DIST/$NAME.zip" "$NAME")
echo "wrote $DIST/$NAME.zip"
ls -lh "$DIST/$NAME.zip"
