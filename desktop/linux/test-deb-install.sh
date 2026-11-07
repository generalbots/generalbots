#!/usr/bin/env bash
# General Bots desktop — prove that a .deb resolves its dependencies on clean
# Debian 13 and Ubuntu 24.04 hosts.
#
#   ./test-deb-install.sh path/to/generalbots-linux-x86_64.deb
#
# With Docker available the package is installed inside clean `debian:13` and
# `ubuntu:24.04` containers, exactly as a user would:
#
#   docker run --rm -v "$PWD:/w" debian:13 \
#     bash -c 'apt-get update && apt-get install -y /w/<file>.deb'
#
# Without Docker (or with the daemon stopped) the script degrades to
# `dpkg-deb --info` plus a dependency-resolution check on the host: every
# `Depends:` alternative must name an installed package or one available from
# the configured apt repositories, and `apt-get --simulate` must resolve the
# whole install. That catches renames such as `libssl3` -> `libssl3t64` and
# `libgtk-3-0` -> `libgtk-3-0t64` without root. The container install remains
# the authoritative proof for both distributions; the fallback is a cheap
# guard for machines that cannot run Docker.
#
# Exits non-zero when the package is missing, is not a valid .deb, or has an
# unsatisfied dependency.
set -euo pipefail

usage() {
  echo "usage: $0 <path-to.deb>" >&2
}

DEB="${1:-}"
if [ -z "$DEB" ]; then
  usage
  exit 2
fi
if [ ! -f "$DEB" ]; then
  echo "error: package not found: $DEB" >&2
  exit 2
fi

DEB_DIR="$(cd "$(dirname "$DEB")" && pwd)"
DEB_NAME="$(basename "$DEB")"
DEB_PATH="$DEB_DIR/$DEB_NAME"

if ! PACKAGE="$(dpkg-deb -f "$DEB_PATH" Package 2>/dev/null)" || [ -z "$PACKAGE" ]; then
  echo "error: not a Debian package: $DEB_PATH" >&2
  exit 2
fi
DEB_ARCH="$(dpkg-deb -f "$DEB_PATH" Architecture)"

DOCKER_PLATFORM=""
case "$DEB_ARCH" in
  amd64) DOCKER_PLATFORM="linux/amd64" ;;
  arm64) DOCKER_PLATFORM="linux/arm64" ;;
esac

echo "package: $PACKAGE"
echo "arch:    $DEB_ARCH"
echo "file:    $DEB_PATH"

if command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
  docker_args=(run --rm)
  if [ -n "$DOCKER_PLATFORM" ]; then
    docker_args+=(--platform "$DOCKER_PLATFORM")
  fi
  for image in debian:13 ubuntu:24.04; do
    echo
    echo "==> installing in a clean $image container"
    docker "${docker_args[@]}" \
      -v "$DEB_DIR:/w:ro" \
      -e "DEB_PATH=/w/$DEB_NAME" \
      -e "DEB_PACKAGE=$PACKAGE" \
      "$image" \
      bash -c 'set -euo pipefail
        apt-get update
        apt-get install -y --no-install-recommends "$DEB_PATH"
        dpkg-query -W -f="\${Status}\n" "$DEB_PACKAGE" | grep -qx "install ok installed"
        . /etc/os-release
        echo "installed $DEB_PACKAGE in $PRETTY_NAME"'
  done
  echo
  echo "deb installs cleanly on debian:13 and ubuntu:24.04"
  exit 0
fi

echo
echo "docker is unavailable; falling back to dpkg-deb --info and a host"
echo "dependency-resolution check (no root required)."
echo "Run this on a machine with Docker for the authoritative Debian 13 and"
echo "Ubuntu 24.04 container installs."
echo
dpkg-deb --info "$DEB_PATH"
echo

depends="$(dpkg-deb -f "$DEB_PATH" Depends || true)"
if [ -z "$depends" ]; then
  echo "no Depends field: nothing to resolve"
  exit 0
fi

missing=()
IFS=',' read -r -a groups <<< "$depends"
for group in "${groups[@]}"; do
  group="$(echo "$group" | sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//')"
  [ -n "$group" ] || continue
  satisfied=0
  IFS='|' read -r -a alternatives <<< "$group"
  for alternative in "${alternatives[@]}"; do
    name="$(echo "$alternative" \
      | sed -E 's/^[[:space:]]+//; s/[[:space:]]*\([^)]*\)[[:space:]]*$//; s/[[:space:]]+$//; s/:.*$//')"
    [ -n "$name" ] || continue
    if dpkg -s "$name" >/dev/null 2>&1; then
      satisfied=1
      break
    fi
    if command -v apt-cache >/dev/null 2>&1 && apt-cache show "$name" >/dev/null 2>&1; then
      satisfied=1
      break
    fi
  done
  if [ "$satisfied" -ne 1 ]; then
    missing+=("$group")
  fi
done

if [ "${#missing[@]}" -gt 0 ]; then
  echo "unsatisfied dependencies:" >&2
  for group in "${missing[@]}"; do
    echo "  - $group" >&2
  done
  echo "the .deb would fail to install on this host" >&2
  exit 1
fi

if command -v apt-get >/dev/null 2>&1; then
  SIM_LOG="$(mktemp "${TMPDIR:-/tmp}/gb-deb-install.XXXXXX")"
  trap 'rm -f "$SIM_LOG"' EXIT
  echo
  echo "simulating apt install (version constraints included)"
  if ! apt-get -s install "$DEB_PATH" >"$SIM_LOG" 2>&1; then
    cat "$SIM_LOG" >&2
    echo "apt cannot resolve the package dependencies" >&2
    exit 1
  fi
  echo "apt simulated the install successfully"
fi

echo
echo "all dependencies in Depends resolve on this host"
