#!/usr/bin/env bash
# General Bots OS — bootable ISO built from the packaged Linux tree (#2110).
#
# One release carries Linux, Windows, macOS and this ISO. The image is a
# hybrid (BIOS + UEFI) El Torito ISO produced by `grub-mkrescue`/`xorriso`
# from the SAME tarball the release publishes:
#
#   /general-bots/        the unpacked `generalbots-<version>-linux-x86_64`
#                         tree (start.sh, generalbots, botserver, ui/, ...)
#   /general-bots/VERSION product version of the payload
#   /SHA256SUMS-linux     checksums shipped by the Linux packaging step
#   /README.txt           what the image is and how to use it
#   /boot/grub/grub.cfg   GRUB menu (BIOS + UEFI firmware)
#
# This is NOT the kiosk appliance: that image needs a Debian 13 rootfs and a
# systemd-repart/mkosi toolchain and lives in its own workflow
# (`desktop-os-iso.yml`, recipe `desktop/os/`). This ISO is the offline "carry
# the release on one medium" image; it boots GRUB on BIOS and UEFI firmware and
# carries the installer payload so a machine without network still has it.
#
# Usage:
#   scripts/build_iso.sh --payload dist/generalbots-7.0.35-linux-x86_64.tar.gz \
#                        --out dist/generalbots-os.iso --version 7.0.35
#   scripts/build_iso.sh --verify dist/generalbots-os.iso
#
# Options:
#   --payload PATH   tarball (.tar.gz) or unpacked tree placed on the ISO
#   --out FILE       output ISO (default: dist/generalbots-os.iso)
#   --label TEXT     ISO volume label (default: GENERALBOTS)
#   --version VER    product version written to /general-bots/VERSION
#   --kernel FILE    kernel copied to /boot/vmlinuz (optional; adds a boot entry)
#   --initrd FILE    initramfs copied to /boot/initrd.img (requires --kernel)
#   --work DIR       scratch directory (default: a fresh mktemp -d)
#   --keep           keep the scratch directory
#   --max-mb N       refuse an ISO larger than N MiB (default: 1900)
#   --verify ISO     verify an existing ISO instead of building one
#   --expect PATH    extra path that must exist inside the ISO (repeatable)
#   -h, --help       this help
#
# Tools (installed by the release job, no network use here):
#   sudo apt-get install -y grub-pc-bin grub-efi-amd64-bin grub-common \
#                           xorriso mtools
#
# Exit codes: 0 success, 1 build/verification failure, 2 usage error.
set -euo pipefail

REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

PAYLOAD=
OUT="dist/generalbots-os.iso"
LABEL="GENERALBOTS"
VERSION=
KERNEL=
INITRD=
WORK=
KEEP=0
MAX_MB=1900
VERIFY_ISO=
EXPECT=()

log() { printf '[build_iso] %s\n' "$*"; }
die() { printf '[build_iso] error: %s\n' "$*" >&2; exit 1; }

usage() {
  cat <<'USAGE'
Usage: scripts/build_iso.sh --payload PATH --out FILE [options]
       scripts/build_iso.sh --verify ISO [--expect PATH ...]

  --payload PATH   tarball (.tar.gz) or unpacked tree placed on the ISO
  --out FILE       output ISO (default: dist/generalbots-os.iso)
  --label TEXT     ISO volume label (default: GENERALBOTS)
  --version VER    product version written to /general-bots/VERSION
  --kernel FILE    kernel copied to /boot/vmlinuz (optional; adds a boot entry)
  --initrd FILE    initramfs copied to /boot/initrd.img (requires --kernel)
  --work DIR       scratch directory (default: a fresh mktemp -d)
  --keep           keep the scratch directory
  --max-mb N       refuse an ISO larger than N MiB (default: 1900)
  --verify ISO     verify an existing ISO instead of building one
  --expect PATH    extra path that must exist inside the ISO (repeatable)
  -h, --help       this help

Tools: grub-mkrescue (grub-common), xorriso, mtools:
  sudo apt-get install -y grub-pc-bin grub-efi-amd64-bin grub-common xorriso mtools
USAGE
}

need_value() {
  [ $# -ge 2 ] || { printf '[build_iso] error: %s needs a value\n' "$1" >&2; exit 2; }
}

while [ $# -gt 0 ]; do
  case "$1" in
    --payload) need_value "$@"; PAYLOAD="$2"; shift 2 ;;
    --out) need_value "$@"; OUT="$2"; shift 2 ;;
    --label) need_value "$@"; LABEL="$2"; shift 2 ;;
    --version) need_value "$@"; VERSION="$2"; shift 2 ;;
    --kernel) need_value "$@"; KERNEL="$2"; shift 2 ;;
    --initrd) need_value "$@"; INITRD="$2"; shift 2 ;;
    --work) need_value "$@"; WORK="$2"; shift 2 ;;
    --keep) KEEP=1; shift ;;
    --max-mb) need_value "$@"; MAX_MB="$2"; shift 2 ;;
    --verify) need_value "$@"; VERIFY_ISO="$2"; shift 2 ;;
    --expect) need_value "$@"; EXPECT+=("$2"); shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) printf '[build_iso] error: unknown option %s\n' "$1" >&2; usage >&2; exit 2 ;;
  esac
done

require_tool() {
  command -v "$1" >/dev/null 2>&1 || die "$1 is required but not installed (apt: $2)"
}

iso_size() { stat -c %s "$1"; }

# ---------------------------------------------------------------------------
# Verification: El Torito boot record (BIOS + UEFI) and the payload on the ISO.
# The image is listed through xorriso, so no loop mount (and no root) is needed.
# ---------------------------------------------------------------------------
verify_iso() {
  local iso="$1"
  [ -f "$iso" ] || die "ISO not found: $iso"
  require_tool xorriso "xorriso"
  local bytes
  bytes=$(iso_size "$iso")
  [ "$bytes" -gt 1048576 ] || die "ISO is suspiciously small (${bytes} bytes): $iso"

  local eltorito
  eltorito=$(xorriso -indev "$iso" -report_el_torito plain 2>/dev/null) \
    || die "xorriso cannot read $iso"
  printf '%s\n' "$eltorito"
  grep -q 'El Torito' <<<"$eltorito" \
    || die "$iso has no El Torito boot record (not bootable)"
  grep -qiE 'El Torito boot img.*(BIOS|UEFI)' <<<"$eltorito" \
    || die "$iso has an El Torito record but no BIOS/UEFI boot image"

  # Hybrid images expose both firmware entries; assert UEFI explicitly so a
  # BIOS-only regression cannot pass silently.
  xorriso -indev "$iso" -report_el_torito as_mkisofs 2>/dev/null \
    | grep -q 'eltorito-alt-boot' \
    || die "$iso has no UEFI (El Torito alt boot) entry"

  local listing
  listing=$(xorriso -indev "$iso" -ls /general-bots 2>/dev/null) \
    || die "$iso has no /general-bots payload directory"
  printf '%s\n' "$listing"
  for required in start.sh generalbots VERSION; do
    grep -qF -- "$required" <<<"$listing" \
      || die "$iso payload is missing /general-bots/${required}"
  done
  xorriso -indev "$iso" -ls /boot/grub 2>/dev/null | grep -qF 'grub.cfg' \
    || die "$iso has no /boot/grub/grub.cfg"
  local extra dir
  for extra in "${EXPECT[@]:-}"; do
    [ -n "$extra" ] || continue
    dir="/$(dirname "$extra")"
    if [ "$dir" = "/." ]; then
      dir="/"
    fi
    xorriso -indev "$iso" -ls "$dir" 2>/dev/null \
      | grep -qF "$(basename "$extra")" \
      || die "$iso is missing expected path /${extra}"
  done

  log "verified ${iso} ($(( bytes / 1048576 )) MiB): El Torito BIOS+UEFI, payload present"
}

if [ -n "$VERIFY_ISO" ]; then
  verify_iso "$VERIFY_ISO"
  exit 0
fi

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------
[ -n "$PAYLOAD" ] || { printf '[build_iso] error: --payload is required to build an ISO\n' >&2; exit 2; }
require_tool grub-mkrescue "grub-common grub-pc-bin grub-efi-amd64-bin"
require_tool xorriso "xorriso"
require_tool mformat "mtools"

if [ -n "$INITRD" ] && [ -z "$KERNEL" ]; then
  die "--initrd requires --kernel"
fi
[ -z "$KERNEL" ] || [ -f "$KERNEL" ] || die "kernel not found: $KERNEL"
[ -z "$INITRD" ] || [ -f "$INITRD" ] || die "initramfs not found: $INITRD"

if [ -z "$WORK" ]; then
  WORK=$(mktemp -d "${TMPDIR:-/tmp}/gb-iso.XXXXXX")
  trap '[ "$KEEP" = 1 ] || rm -rf "$WORK"' EXIT
else
  mkdir -p "$WORK"
fi
STAGING="$WORK/iso"

log "staging in ${STAGING}"
rm -rf "$STAGING"
mkdir -p "$STAGING/general-bots" "$STAGING/boot/grub"

# --- payload -------------------------------------------------------------
SUMS_SOURCE=
if [ -d "$PAYLOAD" ]; then
  [ -x "$PAYLOAD/start.sh" ] || die "payload tree has no executable start.sh: $PAYLOAD"
  [ -x "$PAYLOAD/generalbots" ] || die "payload tree has no executable generalbots: $PAYLOAD"
  cp -a "$PAYLOAD/." "$STAGING/general-bots/"
  SUMS_SOURCE="$PAYLOAD/SHA256SUMS-linux"
elif [ -f "$PAYLOAD" ]; then
  case "$PAYLOAD" in
    *.tar.gz|*.tgz) : ;;
    *) die "payload must be a .tar.gz tarball or a directory: $PAYLOAD" ;;
  esac
  SUMS_SOURCE="$(dirname "$PAYLOAD")/SHA256SUMS-linux"
  if [ -f "$SUMS_SOURCE" ]; then
    # Fail closed on a tampered payload: the checksum file ships in the same
    # artifact directory as the tarball.
    asset=$(basename "$PAYLOAD")
    want=$(awk -v n="$asset" '$2 == n { print $1 }' "$SUMS_SOURCE" | head -n1)
    [ -n "$want" ] || die "SHA256SUMS-linux has no entry for ${asset}"
    echo "${want}  ${PAYLOAD}" | sha256sum -c - \
      || die "payload checksum verification failed for ${asset}"
  else
    log "no SHA256SUMS-linux next to the payload; skipping the tarball checksum"
  fi
  tar -xzf "$PAYLOAD" -C "$STAGING/general-bots" --strip-components=1
else
  die "payload not found: $PAYLOAD"
fi
[ -x "$STAGING/general-bots/start.sh" ] || die "staged payload has no start.sh"
[ -x "$STAGING/general-bots/generalbots" ] || die "staged payload has no generalbots"
[ -d "$STAGING/general-bots/ui/suite" ] || die "staged payload has no ui/suite"

if [ -f "$SUMS_SOURCE" ]; then
  cp "$SUMS_SOURCE" "$STAGING/SHA256SUMS-linux"
fi
printf '%s\n' "${VERSION:-unversioned}" > "$STAGING/general-bots/VERSION"

# --- optional kernel/initramfs -------------------------------------------
BOOT_ENTRY=
if [ -n "$KERNEL" ]; then
  install -m 0644 "$KERNEL" "$STAGING/boot/vmlinuz"
  BOOT_ENTRY="linux /boot/vmlinuz"
  if [ -n "$INITRD" ]; then
    install -m 0644 "$INITRD" "$STAGING/boot/initrd.img"
    BOOT_ENTRY="$BOOT_ENTRY
        initrd /boot/initrd.img"
  fi
fi

cat > "$STAGING/README.txt" <<EOF
General Bots ${VERSION:-unversioned} — bootable ISO
===================================================

This image boots GRUB on BIOS and UEFI firmware and carries the complete
General Bots Linux desktop payload, so a machine without network access can
install the suite from the same medium as the release download.

  /general-bots/        the unpacked generalbots-linux-x86_64 tree
                        (run ./start.sh inside it to install and start)
  /SHA256SUMS-linux     checksums of the packaged artifacts
  /boot/grub/grub.cfg   boot menu

Use it as a data medium: copy /general-bots to the target machine and run
./start.sh. This is not the kiosk appliance image (see desktop/os).

The ISO is unsigned. Verify it with the release SHA256SUMS-os.
EOF

{
  echo 'set timeout=15'
  echo 'set default=0'
  echo
  echo "menuentry 'General Bots ${VERSION:-unversioned} — payload on this medium' {"
  if [ -n "$BOOT_ENTRY" ]; then
    printf '    %s\n' "$BOOT_ENTRY"
  else
    echo '    echo "The General Bots installer payload is in /general-bots."'
    echo '    echo "This image has no kernel: boot a Linux system and copy it."'
    echo '    sleep 10'
  fi
  echo '}'
  echo
  echo "menuentry 'General Bots ${VERSION:-unversioned} — GRUB command line' {"
  echo '    commandline'
  echo '}'
} > "$STAGING/boot/grub/grub.cfg"

mkdir -p "$(dirname "$OUT")"
rm -f "$OUT"
log "building ${OUT} (label ${LABEL})"
grub-mkrescue -o "$OUT" "$STAGING" -- -volid "$LABEL" -joliet on -rockridge on

[ -s "$OUT" ] || die "grub-mkrescue produced no ISO"
bytes=$(iso_size "$OUT")
max_bytes=$(( MAX_MB * 1048576 ))
[ "$bytes" -le "$max_bytes" ] \
  || die "ISO is $(( bytes / 1048576 )) MiB, above the ${MAX_MB} MiB limit (GitHub caps release assets at 2 GB); trim the payload or raise --max-mb"
log "built ${OUT} ($(( bytes / 1048576 )) MiB, sha256 $(sha256sum "$OUT" | cut -d' ' -f1))"

# Never hand back an image the release cannot publish: verify what was built.
verify_iso "$OUT"
