#!/usr/bin/env bash
# General Bots desktop — per-user installer (no root required).
#
#   ./install.sh                 installs into ~/.local/opt/general-bots
#   ./install.sh /some/prefix    installs into /some/prefix/general-bots
#
# Binaries and the suite are copied under the prefix; the launcher and the
# desktop entry point at it. Runtime state (stack, logs, workspace) is kept in
# ~/.local/share/generalbots and survives reinstalls and upgrades.
set -euo pipefail

# Resolve the symlink so a launcher installed elsewhere still finds the real
# files (BASH_SOURCE[0] is the link path when invoked through one).
SELF="$(readlink -f "${BASH_SOURCE[0]}")"
HERE="$(cd "$(dirname "$SELF")" && pwd)"

missing=""
for required in generalbots botserver ui/suite/desktop.html .product.desktop-local; do
  [ -e "$HERE/$required" ] || missing="$missing $required"
done
if [ -n "$missing" ]; then
  cat >&2 <<MSG
This copy of General Bots is incomplete: missing$missing

The download was interrupted or truncated. Re-download the tarball from
https://github.com/generalbots/generalbots/releases/latest and check it with
sha256sum against SHA256SUMS-linux before installing.
MSG
  exit 1
fi

PREFIX="${1:-$HOME/.local/opt}"
TARGET="$PREFIX/general-bots"
BIN_DIR="$HOME/.local/bin"
APP_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"

mkdir -p "$PREFIX" "$BIN_DIR" "$APP_DIR"

echo "Installing to $TARGET"
rm -rf "$TARGET"
mkdir -p "$TARGET"
cp -a "$HERE/." "$TARGET/"

chmod +x "$TARGET/generalbots" "$TARGET/botserver" "$TARGET/start.sh"
ln -sfn "$TARGET/start.sh" "$BIN_DIR/general-bots"

cat > "$APP_DIR/general-bots.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=General Bots
Comment=General Bots desktop (Chat + Drive)
Exec=$TARGET/start.sh
Icon=$TARGET/generalbots.png
Terminal=false
Categories=Office;Network;
StartupWMClass=general-bots
DESKTOP
chmod +x "$APP_DIR/general-bots.desktop"

echo
echo "Installed."
echo "  Run:        general-bots            (or $TARGET/start.sh)"
echo "  Launcher:   $APP_DIR/general-bots.desktop"
echo "  State:      ${XDG_DATA_HOME:-$HOME/.local/share}/generalbots"
echo
echo "The first run opens the setup wizard and installs the services it needs."
