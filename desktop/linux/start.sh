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

# The desktop profile (Chat + Drive, no login screen, local taskbar search) is
# read from the stack. Seed it on first run so a fresh download behaves like a
# finished installation.
if [ ! -f "$STACK/.product.desktop-local" ] && [ -f "$HERE/.product.desktop-local" ]; then
  cp "$HERE/.product.desktop-local" "$STACK/.product.desktop-local"
fi

export BOTSERVER_STACK_PATH="$STACK"
export BOTSERVER_INSTALLERS_PATH="${BOTSERVER_INSTALLERS_PATH:-$HERE/installers}"
export BOTUI_PATH="${BOTUI_PATH:-$HERE/ui/suite}"

# Services (PostgreSQL, Valkey, MinIO, Qdrant, Vault, Zitadel, llama.cpp ...) are
# installed on the first wizard run. This download ships no bundled installers,
# so the provisioner must be allowed to fetch them. The offline bundle ships
# `installers/` next to this script and therefore never enables downloads.
if [ ! -d "$BOTSERVER_INSTALLERS_PATH" ] || [ -z "$(ls -A "$BOTSERVER_INSTALLERS_PATH" 2>/dev/null)" ]; then
  export GB_DESKTOP_ALLOW_DOWNLOAD=1
fi

exec "$HERE/generalbots" "$@"
