#!/usr/bin/env bash
# Verify a mkosi-built General Bots OS image against the appliance manifest.
#
# #2440 — the release name `generalbots-os.iso` is reserved for an image that
# boots into generalbots-shell.service. A payload carrier (build_iso.sh) has
# GRUB and a tarball but no root filesystem, so it fails these checks.
#
# All assertions are read from appliance-manifest.tsv (one data file, no
# hard-coded marker lists). guestfish mounts the root partition read-only:
# libguestfs inspection (`guestfish -i`) must NOT be used here — it runs sgdisk,
# which rejects the El Torito hybrid GPT before anything is mounted.
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
IMAGE=
MANIFEST="$SCRIPT_DIR/appliance-manifest.tsv"
EVIDENCE=

log() { printf '[verify-os] %s\n' "$*"; }
die() { printf '[verify-os] error: %s\n' "$*" >&2; exit 1; }

usage() {
    cat <<'USAGE'
Usage: desktop/os/release/verify-image.sh --image FILE [options]

  --image FILE       mkosi disk/ISO image to verify (required)
  --manifest FILE    expected identity/markers (default: appliance-manifest.tsv)
  --evidence FILE    append one "CHECK <name> <status> <detail>" line per check
  -h, --help         show this help

Requires guestfish (libguestfs-tools). xorriso is optional: when available the
El Torito BIOS/UEFI boot records are asserted too.
USAGE
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --image) [ "$#" -ge 2 ] || die "--image needs a value"; IMAGE="$2"; shift 2 ;;
        --image=*) IMAGE="${1#*=}"; shift ;;
        --manifest) [ "$#" -ge 2 ] || die "--manifest needs a value"; MANIFEST="$2"; shift 2 ;;
        --manifest=*) MANIFEST="${1#*=}"; shift ;;
        --evidence) [ "$#" -ge 2 ] || die "--evidence needs a value"; EVIDENCE="$2"; shift 2 ;;
        --evidence=*) EVIDENCE="${1#*=}"; shift ;;
        -h|--help) usage; exit 0 ;;
        *) die "unknown option: $1" ;;
    esac
done

[ -n "$IMAGE" ] || { usage >&2; die "--image is required"; }
[ -f "$MANIFEST" ] || die "manifest not found: $MANIFEST"
[ -f "$IMAGE" ] || die "image not found: $IMAGE"
command -v guestfish >/dev/null 2>&1 \
    || die "guestfish is required but not installed (apt: libguestfs-tools)"

bytes=$(stat -c %s "$IMAGE")
[ "$bytes" -gt 1048576 ] \
    || die "image is suspiciously small (${bytes} bytes): $IMAGE"

# --- data access -----------------------------------------------------------
manifest_get() {
    awk -F '\t' -v key="$1" '$1 == key { print $2; exit }' "$MANIFEST"
}

manifest_all() {
    awk -F '\t' -v key="$1" '$1 == key { print $2 }' "$MANIFEST"
}

# --- result recording ------------------------------------------------------
checks=0
failed=0

record() {
    local status="$1" name="$2" detail="$3"
    checks=$((checks + 1))
    case "$status" in
        FAIL) failed=$((failed + 1)) ;;
    esac
    printf 'CHECK %s %s %s\n' "$name" "$status" "$detail"
    if [ -n "$EVIDENCE" ]; then
        printf '%s\t%s\t%s\n' "$name" "$status" "$detail" >> "$EVIDENCE"
    fi
}

pass() { record PASS "$1" "$2"; }
skip() { record SKIP "$1" "$2"; }
fail() { record FAIL "$1" "$2"; }

# --- guestfish helpers -----------------------------------------------------
gf_run() {
    guestfish --ro -a "$IMAGE" -- "$@" 2>&1
}

gf_root() {
    guestfish --ro -a "$IMAGE" -m "$root_dev:/" -- "$@" 2>&1
}

# --- structural boot layout ------------------------------------------------
# The first disk is /dev/sda to libguestfs. A bootable appliance has a GPT
# with at least a root and an EFI system partition (vfat), unlike the carrier
# ISO's single ISO 9660 filesystem.
parts="$(gf_run run : part-list /dev/sda 2>/dev/null || true)"
part_count=$(printf '%s\n' "$parts" | grep -cE '^/dev/sda[0-9]+' || true)
part_min=$(manifest_get boot.partitions_min)
[ -n "$part_min" ] || die "manifest has no boot.partitions_min entry"
if [ "$part_count" -ge "$part_min" ]; then
    pass boot.partitions "${part_count} partitions (>= ${part_min})"
else
    fail boot.partitions "${part_count} partitions, expected >= ${part_min}"
fi

filesystems="$(gf_run run : list-filesystems 2>/dev/null || true)"
fs_marker=$(manifest_get boot.filesystem)
[ -n "$fs_marker" ] || die "manifest has no boot.filesystem entry"
if grep -qF -- "$fs_marker" <<<"$filesystems"; then
    pass boot.esp "${fs_marker} filesystem present"
else
    fail boot.esp "no ${fs_marker} (EFI system partition) in image"
fi

# --- root filesystem -------------------------------------------------------
root_label=$(manifest_get root.label)
[ -n "$root_label" ] || die "manifest has no root.label entry"
inspect_output="$(gf_run run : findfs-label "$root_label" 2>&1)" \
    || die "cannot find root label '${root_label}': $(tr '\n' ' ' <<<"$inspect_output" | cut -c1-160)"
root_dev="$inspect_output"
case "$root_dev" in
    /dev/sd*|/dev/vd*) pass root.mount "found ${root_label} on ${root_dev}" ;;
    *) die "unexpected root device for ${root_label}: ${root_dev}" ;;
esac

# --- in-image paths --------------------------------------------------------
while IFS= read -r path; do
    [ -n "$path" ] || continue
    answer="$(gf_root exists "$path" 2>&1)" \
        && [ "$answer" = "true" ] \
        && pass "path$(printf '%s' "$path" | tr '/ ' '__')" "$path" \
        || fail "path$(printf '%s' "$path" | tr '/ ' '__')" "missing ${path} (${answer:-no answer})"
done < <(manifest_all required.path)

# --- identity, shell unit and their markers --------------------------------
identity_file=$(manifest_get identity.file)
unit_file=$(manifest_get unit.file)
[ -n "$identity_file" ] && [ -n "$unit_file" ] \
    || die "manifest is missing identity.file or unit.file"

os_release="$(gf_root cat "$identity_file" 2>&1)" \
    || die "cannot read ${identity_file} from the image"
while IFS= read -r marker; do
    [ -n "$marker" ] || continue
    grep -qF -- "$marker" <<<"$os_release" \
        && pass "identity$(printf '%s' "$marker" | tr -cd '[:alnum:]' | cut -c1-24)" "$marker" \
        || fail "identity$(printf '%s' "$marker" | tr -cd '[:alnum:]' | cut -c1-24)" "missing '${marker}' in ${identity_file}"
done < <(manifest_all identity.marker)

unit="$(gf_root cat "$unit_file" 2>&1)" \
    || die "cannot read ${unit_file} from the image"
while IFS= read -r marker; do
    [ -n "$marker" ] || continue
    grep -qF -- "$marker" <<<"$unit" \
        && pass "unit$(printf '%s' "$marker" | tr -cd '[:alnum:]' | cut -c1-24)" "$marker" \
        || fail "unit$(printf '%s' "$marker" | tr -cd '[:alnum:]' | cut -c1-24)" "missing '${marker}' in ${unit_file}"
done < <(manifest_all unit.marker)

# --- El Torito boot records (optional tool) --------------------------------
if command -v xorriso >/dev/null 2>&1; then
    eltorito="$(xorriso -indev "$IMAGE" -report_el_torito plain 2>/dev/null || true)"
    while IFS= read -r marker; do
        [ -n "$marker" ] || continue
        grep -qiF -- "$marker" <<<"$eltorito" \
            && pass "boot$(printf '%s' "$marker" | tr -cd '[:alnum:]')" "$marker boot record" \
            || fail "boot$(printf '%s' "$marker" | tr -cd '[:alnum:]')" "no '${marker}' El Torito record"
    done < <(manifest_all boot.marker)
else
    skip boot.el-torito "xorriso not installed (GPT/vfat checks still apply)"
fi

printf 'SUMMARY checks=%s failed=%s\n' "$checks" "$failed"
[ "$failed" -eq 0 ] \
    || die "image is NOT a verified General Bots OS appliance (${failed} failed check(s))"
log "verified ${IMAGE} ($(( bytes / 1048576 )) MiB): kiosk shell, OS identity and boot layout present"
