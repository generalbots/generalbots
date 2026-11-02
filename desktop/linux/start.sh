#!/usr/bin/env bash
# General Bots desktop — launcher.
#
# Runs the desktop shell (Chat + Drive) next to the botserver sidecar that it
# starts. Binaries and the suite stay read-only inside the install directory;
# every mutable file (stack, logs, workspace) lives under ~/.local/share.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# A truncated download extracts a partial tree (usually without `generalbots`).
# Fail with the recovery command instead of "No such file or directory" from exec.
missing=""
for required in generalbots botserver ui/suite/desktop.html .product.desktop-local; do
  [ -e "$HERE/$required" ] || missing="$missing $required"
done
if [ -n "$missing" ]; then
  cat >&2 <<MSG
This copy of General Bots is incomplete: missing$missing

The download was interrupted or truncated. Re-download and verify it:

  curl -fL -o generalbots-linux-x86_64.tar.gz \\
    https://github.com/generalbots/generalbots/releases/latest/download/generalbots-linux-x86_64.tar.gz
  sha256sum generalbots-linux-x86_64.tar.gz   # compare with SHA256SUMS-linux
  tar -xzf generalbots-linux-x86_64.tar.gz
MSG
  exit 1
fi

STACK="${BOTSERVER_STACK_PATH:-${XDG_DATA_HOME:-$HOME/.local/share}/generalbots/stack}"
mkdir -p "$STACK"

# The desktop profile (Chat + Drive, single-user sign-in, local taskbar search)
# is read from the stack. Seed it on first run so a fresh download behaves like
# a finished installation.
if [ ! -f "$STACK/.product.desktop-local" ] && [ -f "$HERE/.product.desktop-local" ]; then
  cp "$HERE/.product.desktop-local" "$STACK/.product.desktop-local"
fi

export BOTSERVER_STACK_PATH="$STACK"
export BOTUI_PATH="${BOTUI_PATH:-$HERE/ui/suite}"

# Where the provisioner looks for component archives and caches the ones it
# downloads. Keep it in the user's data directory: the install directory may be
# read-only and is replaced on upgrade, while this survives reinstalls.
if [ -n "${BOTSERVER_INSTALLERS_PATH:-}" ]; then
  INSTALLERS="$BOTSERVER_INSTALLERS_PATH"
else
  INSTALLERS="$(dirname "$STACK")/installers"
fi
mkdir -p "$INSTALLERS"

# Components vendored with the download (MinIO, whose upstream binaries are
# gone) are seeded into the cache once; the provisioner then downloads whatever
# is still missing into the same directory.
if [ -d "$HERE/installers" ]; then
  for archive in "$HERE/installers"/*; do
    [ -f "$archive" ] || continue
    name="$(basename "$archive")"
    [ -e "$INSTALLERS/$name" ] || cp "$archive" "$INSTALLERS/$name"
  done
fi
export BOTSERVER_INSTALLERS_PATH="$INSTALLERS"

# Missing components are fetched on the first run and reused afterwards.
# Air-gapped installations set GB_DESKTOP_OFFLINE=1 to refuse any download.
if [ "${GB_DESKTOP_OFFLINE:-0}" != "1" ]; then
  export GB_DESKTOP_ALLOW_DOWNLOAD=1
fi

# Headless provisioning: the setup wizard spawns exactly this child. Exposed
# here so the same component order can be driven from a terminal or a CI test
# without a display.
if [ "${1:-}" = "--provision" ]; then
  shift
  cd "$STACK"
  exec "$HERE/botserver" --provision-stack "$@"
fi

exec "$HERE/generalbots" "$@"
