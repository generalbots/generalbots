#!/usr/bin/env bash
# Deterministic release packaging for the General Bots OS appliance (#2440).
#
# This is the single entry point that may produce the artifact named
# `generalbots-os.iso`. It either
#
#   * builds the real kiosk image with mkosi, reusing desktop/os/mkosi.conf
#     (no source build, no CDN, no secrets), or
#   * re-blesses an already built image with --image FILE.
#
# In both cases the image is verified against appliance-manifest.tsv BEFORE it
# is copied to its final name; a failing check leaves no generalbots-os.iso
# behind. SHA256SUMS-os and a deterministic generalbots-os.manifest.txt are
# written next to the published artifact.
#
# Preflight is fail-safe in the CI sense: if mkosi/its inputs (build mode) or
# the image input (publish mode) are unavailable, the script prints an explicit
# SKIP reason and exits 0 — it never silently publishes a payload carrier.
# A present image that fails verification is a hard failure (exit 1).
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd)"
MANIFEST="$SCRIPT_DIR/appliance-manifest.tsv"
VERIFIER="$SCRIPT_DIR/verify-image.sh"
OUT_DIR="dist"
IMAGE=
MKOSI="${GB_OS_MKOSI:-mkosi}"
RELEASE="${GB_OS_RELEASE_TAG:-unpinned}"

log() { printf '[release-os] %s\n' "$*"; }
die() { printf '[release-os] error: %s\n' "$*" >&2; exit 1; }
skip() { printf '[release-os] SKIP: %s\n' "$*"; exit 0; }

usage() {
    cat <<'USAGE'
Usage: desktop/os/release/build-release-image.sh [options]

  --source-dir DIR   mkosi recipe directory (default: desktop/os)
  --image FILE       verify/publish an existing image instead of building it
  --manifest FILE    expected identity/markers (default: appliance-manifest.tsv)
  --out-dir DIR      where to publish (default: dist)
  --mkosi PATH       mkosi executable (default: $GB_OS_MKOSI or mkosi)
  --release TAG      pinned product tag recorded in the manifest
  -h, --help         show this help

Outputs (only on success):
  OUT_DIR/generalbots-os.iso
  OUT_DIR/SHA256SUMS-os
  OUT_DIR/generalbots-os.manifest.txt

Exit codes: 0 success or explicit SKIP, 1 verification/build failure, 2 usage.
USAGE
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --source-dir) [ "$#" -ge 2 ] || die "--source-dir needs a value"; SOURCE_DIR="$2"; shift 2 ;;
        --source-dir=*) SOURCE_DIR="${1#*=}"; shift ;;
        --image) [ "$#" -ge 2 ] || die "--image needs a value"; IMAGE="$2"; shift 2 ;;
        --image=*) IMAGE="${1#*=}"; shift ;;
        --manifest) [ "$#" -ge 2 ] || die "--manifest needs a value"; MANIFEST="$2"; shift 2 ;;
        --manifest=*) MANIFEST="${1#*=}"; shift ;;
        --out-dir) [ "$#" -ge 2 ] || die "--out-dir needs a value"; OUT_DIR="$2"; shift 2 ;;
        --out-dir=*) OUT_DIR="${1#*=}"; shift ;;
        --mkosi) [ "$#" -ge 2 ] || die "--mkosi needs a value"; MKOSI="$2"; shift 2 ;;
        --mkosi=*) MKOSI="${1#*=}"; shift ;;
        --release) [ "$#" -ge 2 ] || die "--release needs a value"; RELEASE="$2"; shift 2 ;;
        --release=*) RELEASE="${1#*=}"; shift ;;
        -h|--help) usage; exit 0 ;;
        *) printf '[release-os] error: unknown option %s\n' "$1" >&2; usage >&2; exit 2 ;;
    esac
done

[ -f "$MANIFEST" ] || die "appliance manifest not found: $MANIFEST"
[ -f "$VERIFIER" ] || die "image verifier not found: $VERIFIER"

# --- preflight: explicit SKIP when inputs are unavailable ------------------
build_dir=
stage_dir=
cleanup() {
    [ -z "$build_dir" ] || rm -rf -- "$build_dir"
    [ -z "$stage_dir" ] || rm -rf -- "$stage_dir"
}
trap cleanup EXIT

if [ -n "$IMAGE" ]; then
    [ -f "$IMAGE" ] || skip "image input unavailable: ${IMAGE}"
    [ -s "$IMAGE" ] || skip "image input is empty: ${IMAGE}"
    log "publish mode: verifying existing image ${IMAGE}"
else
    command -v "$MKOSI" >/dev/null 2>&1 \
        || skip "mkosi is unavailable (${MKOSI}); no source build is performed here"
    for required in mkosi.conf mkosi.finalize.chroot; do
        [ -f "$SOURCE_DIR/$required" ] \
            || skip "mkosi recipe input unavailable: ${SOURCE_DIR}/${required}"
    done
    payload="$SOURCE_DIR/mkosi.extra/opt/general-bots"
    for required in start.sh generalbots ui/suite; do
        [ -e "$payload/$required" ] \
            || skip "image payload input unavailable: ${payload}/${required}"
    done
    log "build mode: mkosi recipe ${SOURCE_DIR}, payload ${payload}"
fi

# --- build (optional) ------------------------------------------------------
if [ -z "$IMAGE" ]; then
    build_dir="$(mktemp -d "${TMPDIR:-/tmp}/gb-os-build.XXXXXX")"
    log "building the kiosk image into ${build_dir}"
    (
        cd -- "$SOURCE_DIR"
        "$MKOSI" --output-dir "$build_dir" build
    )
    IMAGE="$(find "$build_dir" -maxdepth 1 -type f \
        \( -name 'generalbots-os.raw' -o -name 'generalbots-os.iso' -o -name 'generalbots-os*.raw' \) \
        -print | LC_ALL=C sort | head -n1)"
    [ -n "$IMAGE" ] || die "mkosi produced no generalbots-os image in ${build_dir}"
    log "mkosi output: ${IMAGE}"
fi

# --- verify before naming --------------------------------------------------
stage_dir="$(mktemp -d "${TMPDIR:-/tmp}/gb-os-release.XXXXXX")"
log "verifying ${IMAGE} against ${MANIFEST}"
bash "$VERIFIER" --image "$IMAGE" --manifest "$MANIFEST" \
    --evidence "$stage_dir/evidence.tsv"

# --- deterministic manifest ------------------------------------------------
sha256="$(sha256sum "$IMAGE" | cut -d' ' -f1)"
size_bytes="$(stat -c %s "$IMAGE")"
{
    printf '# General Bots OS release manifest (#2440)\n'
    printf 'schema=gb-os-release/1\n'
    printf 'artifact=generalbots-os.iso\n'
    printf 'sha256=%s\n' "$sha256"
    printf 'size_bytes=%s\n' "$size_bytes"
    printf 'pin=%s\n' "$RELEASE"
    printf 'verification=PASS\n'
    printf 'image_source=%s\n' "$(basename "$IMAGE")"
    while IFS=$'\t' read -r name status detail; do
        printf 'check.%s=%s\n' "$name" "$status"
    done < "$stage_dir/evidence.tsv"
} > "$stage_dir/generalbots-os.manifest.txt"

# The final name is created only after the image passed every check.
cp -- "$IMAGE" "$stage_dir/generalbots-os.iso"
(
    cd -- "$stage_dir"
    sha256sum generalbots-os.iso > SHA256SUMS-os
    sha256sum -c SHA256SUMS-os
)

mkdir -p -- "$OUT_DIR"
mv -f -- "$stage_dir/generalbots-os.iso" "$OUT_DIR/generalbots-os.iso"
mv -f -- "$stage_dir/SHA256SUMS-os" "$OUT_DIR/SHA256SUMS-os"
mv -f -- "$stage_dir/generalbots-os.manifest.txt" "$OUT_DIR/generalbots-os.manifest.txt"
log "published ${OUT_DIR}/generalbots-os.iso (sha256 ${sha256})"
cat "$OUT_DIR/generalbots-os.manifest.txt"
