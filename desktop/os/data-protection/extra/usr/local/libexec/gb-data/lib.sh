#!/bin/bash
# Shared helpers for the General Bots OS user-data protection scripts (#2470).
#
# SECURITY CONTRACT (read before editing):
#   * No key, passphrase or recovery key is ever stored in this repository.
#     Every secret is generated at RUNTIME from /dev/urandom into a 0600 file
#     under a private, root-owned path (see data-protection.conf).
#   * DRY_RUN=1 prints every action and touches NO disk, key, mount or service.
#     GB_DATA_PROTECTION_SKIP=1 makes a script exit 0 immediately (used by
#     non-LUKS CI hosts that only want to validate wiring).
#   * Every destructive/mutating command goes through run() / run_sh() so the
#     dry-run contract above cannot be broken by a stray direct call.
#   * Secrets are never echoed to stdout, the journal or issue comments; only
#     their private paths are logged.

set -uo pipefail

GB_DP_TAG="gb-data-protection"
GB_DP_CONFIG="${GB_DP_CONFIG:-/etc/generalbots/data-protection.conf}"
DRY_RUN="${DRY_RUN:-0}"
SKIP="${GB_DATA_PROTECTION_SKIP:-0}"

is_dry() { [ "$DRY_RUN" = "1" ]; }

# Logs go to stderr so functions that print a value on stdout (e.g.
# resolve_data_device) stay machine-readable via command substitution.
log()  { printf '%s[%s] %s\n' "$(date -u '+%FT%TZ')" "$GB_DP_TAG" "$*" >&2; }
warn() { printf '%s[%s] WARN: %s\n' "$(date -u '+%FT%TZ')" "$GB_DP_TAG" "$*" >&2; }
die()  { printf '%s[%s] ERROR: %s\n' "$(date -u '+%FT%TZ')" "$GB_DP_TAG" "$*" >&2; exit 1; }

# run <command> [args...] — execute, or only print, a mutating command.
run() {
    if is_dry; then
        printf '  [dry-run]'
        printf ' %q' "$@"
        printf '\n'
        return 0
    fi
    "$@"
}

# run_sh '<shell pipeline>' — same contract for pipelines/redirection.
run_sh() {
    if is_dry; then
        printf '  [dry-run] sh -c %q\n' "$1"
        return 0
    fi
    sh -c "$1"
}

skip_if_requested() {
    if [ "$SKIP" = "1" ]; then
        log "GB_DATA_PROTECTION_SKIP=1 — skipping $0 without touching the disk"
        exit 0
    fi
}

require_root() {
    if is_dry; then
        log "dry-run: skipping root check"
        return 0
    fi
    [ "$(id -u)" -eq 0 ] || die "must run as root (this script reformats disks and writes private keys)"
}

# Load defaults, then optional root-owned config overrides.
load_config() {
    DEVICE="${DEVICE:-}"
    DATA_FILE="${DATA_FILE:-/var/lib/gbdesk-private/data.img}"
    DATA_FILE_SIZE_MB="${DATA_FILE_SIZE_MB:-8192}"
    MAPPER_NAME="${MAPPER_NAME:-gbdata}"
    MOUNT_POINT="${MOUNT_POINT:-/var/lib/gbdesk}"
    STAGE_MOUNT="${STAGE_MOUNT:-/mnt/gbdata-stage}"
    SUBVOLUME="${SUBVOLUME:-@data}"
    SNAPSHOT_SUBDIR="${SNAPSHOT_SUBDIR:-.gb-snapshots}"
    KEY_DIR="${KEY_DIR:-/etc/generalbots/credentials}"
    MAIN_KEY_FILE="${MAIN_KEY_FILE:-$KEY_DIR/luks-main.bin}"
    RECOVERY_KEY_FILE="${RECOVERY_KEY_FILE:-$KEY_DIR/luks-recovery.txt}"
    ARCHIVE_KEY_FILE="${ARCHIVE_KEY_FILE:-$KEY_DIR/snapshot-archive.bin}"
    EXPORT_KEY_FILE="${EXPORT_KEY_FILE:-$KEY_DIR/export-passphrase.txt}"
    INIT_FLAG="${INIT_FLAG:-/etc/generalbots/data-protection.initialized}"
    PRIVATE_DIR="${PRIVATE_DIR:-/var/lib/gbdesk-private}"
    ARCHIVE_DIR="${ARCHIVE_DIR:-$PRIVATE_DIR/snapshots}"
    EXPORT_DIR="${EXPORT_DIR:-$PRIVATE_DIR/exports}"
    SNAPSHOT_KEEP_DAILY="${SNAPSHOT_KEEP_DAILY:-7}"
    SNAPSHOT_KEEP_WEEKLY="${SNAPSHOT_KEEP_WEEKLY:-4}"
    SNAPSHOT_PREFIX="${SNAPSHOT_PREFIX:-gbdesk}"
    # Partitions that reset/restore must NEVER touch (rescue is masked in this
    # appliance, so the rescue partition is the only recovery path).
    PROTECTED_DEVICES="${PROTECTED_DEVICES:-/dev/disk/by-partlabel/gbrescue /dev/disk/by-partlabel/gb-rescue /dev/disk/by-partlabel/EFI /dev/disk/by-partlabel/root}"

    if [ -f "$GB_DP_CONFIG" ]; then
        # shellcheck disable=SC1090
        . "$GB_DP_CONFIG"
    elif [ ! -f "$GB_DP_CONFIG" ] && [ "${GB_DP_ALLOW_MISSING_CONFIG:-0}" != "1" ]; then
        warn "config $GB_DP_CONFIG not found — using built-in defaults"
    fi
}

require_cmds() {
    local missing=""
    local c
    for c in "$@"; do
        command -v "$c" >/dev/null 2>&1 || missing="$missing $c"
    done
    if [ -n "$missing" ]; then
        if is_dry; then
            warn "dry-run: commands not present on this host:$missing (the image installs them via 40-data-protection.conf)"
        else
            die "missing required commands:$missing"
        fi
    fi
}

# Generate a 256-bit secret into a private 0600 file. Never in the repo.
generate_secret() {
    local out="$1" bytes="${2:-32}" tmp
    if is_dry; then
        log "generate ${bytes}-byte secret -> $out (runtime, mode 0600, root-owned)"
        return 0
    fi
    tmp="$(mktemp "${TMPDIR:-/tmp}/gbdp.XXXXXXXX")" || die "mktemp failed"
    chmod 0600 "$tmp" || { rm -f "$tmp"; die "chmod $tmp failed"; }
    if ! LC_ALL=C head -c "$bytes" /dev/urandom | base64 | tr -d '\n' > "$tmp"; then
        rm -f "$tmp"
        die "secret generation failed"
    fi
    printf '\n' >> "$tmp"
    if ! install -m 0600 -o root -g root "$tmp" "$out"; then
        rm -f "$tmp"
        die "install secret to $out failed"
    fi
    rm -f "$tmp"
}

# True when a TPM 2.0 is present and systemd-cryptenroll can use it.
tpm_available() {
    if is_dry; then
        log "tpm probe (dry-run: assuming absent)"
        return 1
    fi
    command -v systemd-cryptenroll >/dev/null 2>&1 || return 1
    [ -e /dev/tpm0 ] || [ -e /dev/tpmrm0 ] || return 1
    return 0
}

# Refuse to operate on a protected (rescue/system) partition.
assert_not_protected() {
    local candidate="$1" p
    for p in $PROTECTED_DEVICES; do
        if [ "$candidate" = "$p" ]; then
            die "refusing to touch protected partition '$candidate' (rescue/system must survive reset)"
        fi
    done
    return 0
}

# Print the block device to encrypt: explicit DEVICE, else a loopback file.
# Echoes the resolved path on stdout; logs only non-secret details.
resolve_data_device() {
    if [ -n "$DEVICE" ]; then
        assert_not_protected "$DEVICE"
        [ -e "$DEVICE" ] || die "configured DEVICE '$DEVICE' does not exist"
        printf '%s\n' "$DEVICE"
        return 0
    fi
    if is_dry; then
        log "would create ${DATA_FILE_SIZE_MB}MB loopback file $DATA_FILE and attach it"
        printf '/dev/loopDRY\n'
        return 0
    fi
    install -d -m 0700 -o root -g root "$PRIVATE_DIR" || die "create $PRIVATE_DIR failed"
    if [ ! -f "$DATA_FILE" ]; then
        run truncate -s "${DATA_FILE_SIZE_MB}M" "$DATA_FILE" || die "create $DATA_FILE failed"
        run chmod 0600 "$DATA_FILE" || die "chmod $DATA_FILE failed"
    fi
    local loop
    loop="$(losetup -f --show "$DATA_FILE")" || die "losetup attach failed"
    printf '%s\n' "$loop"
}

mapping_exists() { [ -e "/dev/mapper/$MAPPER_NAME" ]; }

mounted_here() { mountpoint -q "$1" 2>/dev/null; }
