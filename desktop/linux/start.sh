#!/usr/bin/env bash
# General Bots desktop — launcher.
#
# Runs the desktop shell (Chat + Drive) next to the botserver sidecar that it
# starts. Binaries and the suite stay read-only inside the install directory;
# every mutable file (stack, logs, workspace) lives under ~/.local/share.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
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

# Services (PostgreSQL, Valkey, MinIO, Qdrant, Vault, Zitadel, llama.cpp ...) are
# installed on the first wizard run. Two shapes are supported:
#   - offline bundle: `installers/` next to this script already holds the
#     archives; nothing is downloaded.
#   - slim download: no archives, so they are fetched once into the user's data
#     directory and reused by later runs (a reinstall never re-downloads).
INSTALLERS="${BOTSERVER_INSTALLERS_PATH:-$HERE/installers}"
if [ -d "$INSTALLERS" ] && [ -n "$(ls -A "$INSTALLERS" 2>/dev/null)" ]; then
  export BOTSERVER_INSTALLERS_PATH="$INSTALLERS"
else
  export BOTSERVER_INSTALLERS_PATH="$(dirname "$STACK")/installers"
  export GB_DESKTOP_ALLOW_DOWNLOAD=1
fi
mkdir -p "$BOTSERVER_INSTALLERS_PATH"

# Headless provisioning: the setup wizard spawns exactly this child. Exposed
# here so the same component order can be driven from a terminal or a CI test
# without a display.
if [ "${1:-}" = "--provision" ]; then
  shift
  cd "$STACK"
  exec "$HERE/botserver" --provision-stack "$@"
fi

exec "$HERE/generalbots" "$@"
