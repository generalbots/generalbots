#!/usr/bin/env bash
# Stage the darwin botserver sidecar for Tauri's `bundle.externalBin`.
#
# Tauri resolves `binaries/botserver` to
# `binaries/botserver-<target-triple>`, so a macOS build needs a Mach-O
# executable with the Apple target triple in its name. This script keeps the
# naming and architecture check in one place for CI and local packaging.
#
# Usage:
#   desktop/macos/stage-sidecar.sh [target-triple]
#
# The default target triple is the Rust host (aarch64-apple-darwin on the
# GitHub macOS runner). Build botserver first:
#   cargo build -p botserver --release --no-default-features --features desktop-local

set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
host_triple="$(rustc -vV | sed -n 's/^host: //p')"
triple="${1:-$host_triple}"

case "$triple" in
  aarch64-apple-darwin | x86_64-apple-darwin) ;;
  *)
    echo "error: not a macOS target triple: $triple" >&2
    exit 64
    ;;
esac

source_bin="$root/target/release/botserver"
dest_dir="$root/botapp/binaries"
dest="$dest_dir/botserver-$triple"

if [ ! -x "$source_bin" ]; then
  echo "error: $source_bin is missing; build botserver for $triple first" >&2
  exit 66
fi

if ! file "$source_bin" | grep -q 'Mach-O 64-bit executable'; then
  echo "error: $source_bin is not a Mach-O executable; refusing to bundle it" >&2
  file "$source_bin" >&2
  exit 65
fi

if [ "$triple" = "aarch64-apple-darwin" ] && ! file "$source_bin" | grep -q 'arm64'; then
  echo "error: $source_bin is not built for arm64 ($triple)" >&2
  file "$source_bin" >&2
  exit 65
fi

if [ "$triple" = "x86_64-apple-darwin" ] && ! file "$source_bin" | grep -q 'x86_64'; then
  echo "error: $source_bin is not built for x86_64 ($triple)" >&2
  file "$source_bin" >&2
  exit 65
fi

mkdir -p "$dest_dir" "$root/botserver/botserver-installers"
cp "$source_bin" "$dest"
chmod +x "$dest"

# The macOS bundle ships the sidecar and the UI only. Linux service archives
# must never leak into the .dmg; keep the resource directory empty so the
# first-run wizard downloads the stack for a Linux VM or remote Incus host.
if find "$root/botserver/botserver-installers" -maxdepth 1 -type f -print -quit | grep -q .; then
  echo "warning: botserver/botserver-installers contains files; the macOS dmg should not bundle Linux stack components" >&2
fi

echo "staged $dest"
file "$dest"
