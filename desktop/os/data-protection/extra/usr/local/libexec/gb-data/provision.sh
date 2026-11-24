#!/bin/bash
# First-boot provisioning + every-boot unlock of the encrypted user-data volume
# (issue #2470). Runs as root from gb-data-provision.service.
#
#   --check   print resolved configuration and command availability (no writes)
#   --boot    idempotent: provision on first boot, otherwise unlock + mount
#
# Real LUKS2 formatting, TPM sealing, wrong-passphrase and no-TPM verification
# require a VM or hardware (see the package README header in mkosi fragment).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "$SCRIPT_DIR/lib.sh"
load_config
skip_if_requested

usage() {
    cat <<EOF
Usage: $0 [--check|--boot|--help]
  --check  validate config + tool availability, print plan (no disk writes)
  --boot   provision on first boot, then unlock and mount (default)
Env: DRY_RUN=1 prints actions without touching disks; GB_DATA_PROTECTION_SKIP=1 exits 0.
EOF
}

check() {
    log "config: $GB_DP_CONFIG ($([ -f "$GB_DP_CONFIG" ] && echo present || echo missing))"
    log "device: DEVICE='${DEVICE:-<loopback>}' DATA_FILE=$DATA_FILE (${DATA_FILE_SIZE_MB}MB)"
    log "mapper: $MAPPER_NAME -> mount $MOUNT_POINT (subvol=$SUBVOLUME)"
    log "private keys: $MAIN_KEY_FILE / $RECOVERY_KEY_FILE / $ARCHIVE_KEY_FILE (all runtime-generated, 0600)"
    log "snapshots: $SNAPSHOT_SUBDIR on the LUKS volume, keep $SNAPSHOT_KEEP_DAILY daily / $SNAPSHOT_KEEP_WEEKLY weekly"
    log "protected partitions: $PROTECTED_DEVICES"
    local missing="" c
    for c in cryptsetup mkfs.btrfs btrfs mount umount mountpoint rsync install truncate losetup base64 head mktemp; do
        command -v "$c" >/dev/null 2>&1 || missing="$missing $c"
    done
    if [ -n "$missing" ]; then
        warn "missing commands on this host:$missing"
    else
        log "all required commands present"
    fi
    if tpm_available; then log "TPM 2.0: present (will enroll PCR 7)"; else log "TPM 2.0: absent (root-owned key-file unlock)"; fi
    log "dry-run=$DRY_RUN skip=$SKIP — no disk writes performed by --check"
}

unlock() {
    local dev="$1"
    if mapping_exists; then
        log "mapping /dev/mapper/$MAPPER_NAME already open"
        return 0
    fi
    log "opening LUKS2 mapping /dev/mapper/$MAPPER_NAME on $dev"
    if tpm_available; then
        # systemd-cryptenroll places a TPM2 token in the header; --token-only
        # lets cryptsetup use it without reading an on-disk key. Fall back to
        # the root-owned key file if the token cannot be used.
        if ! run cryptsetup open --type luks --token-only "$dev" "$MAPPER_NAME" \
            && ! run cryptsetup open --type luks "$dev" "$MAPPER_NAME" --key-file "$MAIN_KEY_FILE"; then
            die "cannot unlock $dev with TPM token or key file (wrong passphrase?)"
        fi
    else
        run cryptsetup open --type luks "$dev" "$MAPPER_NAME" --key-file "$MAIN_KEY_FILE" \
            || die "cannot unlock $dev with key file (wrong passphrase?)"
    fi
    if ! mapping_exists && ! is_dry; then
        die "mapping /dev/mapper/$MAPPER_NAME did not appear"
    fi
}

format_and_mount() {
    local mapper="/dev/mapper/$MAPPER_NAME"
    log "creating btrfs on $mapper (LUKS2 -> snapshots are encrypted at rest)"
    run mkfs.btrfs -f "$mapper" || die "mkfs.btrfs failed"
    run install -d -m 0755 "$STAGE_MOUNT" || die "create stage mount failed"
    run mount "$mapper" "$STAGE_MOUNT" || die "stage mount failed"
    run btrfs subvolume create "$STAGE_MOUNT/$SUBVOLUME" || die "create subvolume failed"
    run install -d -m 0700 "$STAGE_MOUNT/$SNAPSHOT_SUBDIR" || die "create snapshot dir failed"

    # Carry over anything already in /var/lib/gbdesk (created by mkosi finalize)
    # so provisioning state is not lost by putting the encrypted volume there.
    if [ -d "$MOUNT_POINT" ] && [ -n "$(ls -A "$MOUNT_POINT" 2>/dev/null)" ]; then
        log "preserving existing $MOUNT_POINT content into the encrypted volume"
        run_sh "rsync -a '$MOUNT_POINT'/ '$STAGE_MOUNT/$SUBVOLUME'/"
    fi

    run umount "$STAGE_MOUNT" || die "stage unmount failed"
    run install -d -m 0700 -o gbdesk -g gbdesk "$MOUNT_POINT" || die "create mount point failed"
    run mount -o "subvol=$SUBVOLUME,compress=zstd:1,space_cache=v2" "$mapper" "$MOUNT_POINT" \
        || die "mount encrypted user data at $MOUNT_POINT failed"
}

first_time_provision() {
    local dev="$1"
    log "first-boot LUKS2 provisioning of $dev"

    run install -d -m 0700 -o root -g root "$KEY_DIR" || die "create $KEY_DIR failed"
    run install -d -m 0700 -o root -g root "$PRIVATE_DIR" || die "create $PRIVATE_DIR failed"
    run install -d -m 0700 -o root -g root "$ARCHIVE_DIR" || die "create $ARCHIVE_DIR failed"
    run install -d -m 0700 -o root -g root "$EXPORT_DIR" || die "create $EXPORT_DIR failed"

    # All secrets are generated at runtime; none exists in the repo or image.
    generate_secret "$MAIN_KEY_FILE" 32
    generate_secret "$RECOVERY_KEY_FILE" 32
    generate_secret "$ARCHIVE_KEY_FILE" 32
    generate_secret "$EXPORT_KEY_FILE" 32
    log "recovery key written to private path: $RECOVERY_KEY_FILE (0600, root-owned)"

    run cryptsetup luksFormat --type luks2 --batch-mode --key-file "$MAIN_KEY_FILE" "$dev" \
        || die "luksFormat failed"
    run cryptsetup luksAddKey "$dev" "$RECOVERY_KEY_FILE" --key-file "$MAIN_KEY_FILE" \
        || die "luksAddKey (recovery slot) failed"

    if tpm_available; then
        log "enrolling TPM 2.0 (PCR 7) — no on-disk key needed with a valid token"
        if run systemd-cryptenroll --tpm2-device=auto --tpm2-pcrs=7 --key-file="$MAIN_KEY_FILE" "$dev"; then
            log "TPM2 token enrolled"
        else
            warn "TPM enrollment failed; keeping root-owned key-file unlock"
        fi
    else
        log "no TPM 2.0: unlock uses the root-owned key file (documented tradeoff — the key is 0600 in $KEY_DIR)"
    fi

    unlock "$dev"
    format_and_mount

    run_sh "printf 'initialized %s\n' \"\$(date -u '+%FT%TZ')\" > '$INIT_FLAG'"
    log "user-data volume ready at $MOUNT_POINT"
    log "IMPORTANT: export $RECOVERY_KEY_FILE to offline storage now and store it outside this device"
}

main() {
    local mode="${1:---boot}"
    case "$mode" in
        --check) check; return 0 ;;
        --boot)  ;;
        -h|--help) usage; return 0 ;;
        *) die "unknown mode '$mode' (try --help)" ;;
    esac

    require_root
    require_cmds cryptsetup mkfs.btrfs btrfs mount umount mountpoint rsync install truncate losetup base64 head mktemp
    local dev
    dev="$(resolve_data_device)" || die "resolve_data_device failed"
    [ -n "$dev" ] || die "empty data device"

    if [ -e "$INIT_FLAG" ]; then
        log "already provisioned — unlocking and mounting $dev"
        unlock "$dev"
        if ! mounted_here "$MOUNT_POINT"; then
            run install -d -m 0700 -o gbdesk -g gbdesk "$MOUNT_POINT" || die "create mount point failed"
            run mount -o "subvol=$SUBVOLUME,compress=zstd:1,space_cache=v2" "/dev/mapper/$MAPPER_NAME" "$MOUNT_POINT" \
                || die "mount encrypted user data at $MOUNT_POINT failed"
        fi
        log "user-data volume ready at $MOUNT_POINT"
    else
        first_time_provision "$dev"
    fi
}

main "$@"
