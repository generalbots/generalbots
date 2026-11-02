#!/usr/bin/env bash
# General Bots desktop — assembles the Linux installer tarball.
#
#   ./package.sh <version> <target-dir> <output-dir>
#
# <target-dir> holds the built `botapp` and `botserver` binaries (target/debug
# for a local run, target/release on CI). The same script produces the artifact
# that CI publishes, so a local download simulation and the release asset are
# byte-for-byte the same layout.
set -euo pipefail

VERSION="${1:?version is required (for example v6.7.0)}"
TARGET="${2:?target directory is required (for example target/release)}"
OUT="${3:?output directory is required (for example dist)}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PKG="generalbots-${VERSION}-linux-x86_64"
DEST="$OUT/$PKG"

[ -x "$TARGET/botapp" ] || { echo "missing $TARGET/botapp" >&2; exit 1; }
[ -x "$TARGET/botserver" ] || { echo "missing $TARGET/botserver" >&2; exit 1; }

rm -rf "$DEST"
mkdir -p "$DEST/ui" "$OUT"

install -m 0755 "$TARGET/botapp"    "$DEST/generalbots"
install -m 0755 "$TARGET/botserver" "$DEST/botserver"
cp -a "$ROOT/botui/ui/suite"  "$DEST/ui/suite"
cp -a "$ROOT/botui/ui/themes" "$DEST/ui/themes"
cp "$ROOT/botserver/.product.desktop-local" "$DEST/.product.desktop-local"
install -m 0755 "$ROOT/desktop/linux/start.sh"   "$DEST/start.sh"
install -m 0755 "$ROOT/desktop/linux/install.sh" "$DEST/install.sh"
cp "$ROOT/desktop/linux/README.txt" "$DEST/README.txt"
cp "$ROOT/botapp/icons/icon.png"    "$DEST/generalbots.png"

tar -czf "$OUT/${PKG}.tar.gz" -C "$OUT" "$PKG"
rm -rf "$DEST"
( cd "$OUT" && sha256sum "${PKG}.tar.gz" > SHA256SUMS-linux )

echo "built $OUT/${PKG}.tar.gz"
ls -lh "$OUT/${PKG}.tar.gz"
