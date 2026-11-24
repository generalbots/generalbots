#!/bin/bash
# "Reset this PC" (#2470): wipe ALL user data and re-arm the first-run wizard,
# while the rescue partition / firmware boot path is left completely untouched.
# This is the missing recovery path in an appliance whose rescue/emergency
# shells are masked (mkosi.finalize.chroot).
#
# Destructive: requires the exact token --confirm RESET-THIS-PC.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "$SCRIPT_DIR/lib.sh"
load_config
skip_if_requested

usage() {
    cat <<EOF
Usage: $0 --confirm RESET-THIS-PC [--reboot]
Wipes: LUKS keyslots on the data volume, all snapshots, encrypted archives and
exports, the provisioning flag and the desktop stack state.
Preserves: the rescue/system partitions, EFI boot entries, the LUKS container
device/header and the rescue key material path (new keys are generated).
Afterwards provision.sh --boot re-initialises an empty encrypted volume and the
normal first-run wizard repopulates /var/lib/gbdesk.
Env: DRY_RUN=1 (no disk writes), GB_DATA_PROTECTION_SKIP=1 (exit 0).
EOF
}

detach_data_loops() {
    if is_dry; then
        log "  [dry-run] would detach any loop devices backed by $DATA_FILE"
        return 0
    fi
    [ -n "$DEVICE" ] && return 0
    local loops l
    loops="$(losetup -j "$DATA_FILE" 2>/dev/null | cut -d: -f1)" || return 0
    [ -n "$loops" ] || return 0
    for l in $loops; do
        run losetup -d "$l"
    done
}

clear_private_dir() {
    local dir="$1"
    if is_dry; then
        log "  [dry-run] would empty $dir"
        return 0
    fi
    [ -d "$dir" ] || return 0
    run find "$dir" -mindepth 1 -delete
}

main() {
    local token="" reboot=0
    while [ "$#" -gt 0 ]; do
        case "$1" in
            --confirm) token="${2:-}"; shift ;;
            --reboot)  reboot=1 ;;
            -h|--help) usage; return 0 ;;
            *) die "unknown argument '$1' (try --help)" ;;
        esac
        shift
    done

    if [ "$token" != "RESET-THIS-PC" ]; then
        die "refusing reset without '--confirm RESET-THIS-PC'"
    fi

    require_root
    require_cmds cryptsetup mountpoint umount losetup find install systemctl rm

    local dev
    detach_data_loops
    dev="$(resolve_data_device)" || die "resolve_data_device failed"
    [ -n "$dev" ] || die "empty data device"

    # Hard safety gate: never let a reset reach a rescue/system partition.
    assert_not_protected "$dev"
    log "protected partitions (UNTOUCHED): $PROTECTED_DEVICES"
    log "target data volume to wipe: $dev"

    log "stopping desktop stack and snapshot timer"
    run systemctl stop generalbots-shell.service || true
    run systemctl stop gb-snapshot.timer || true
    run systemctl stop gb-snapshot.service || true

    log "unmounting user data and closing LUKS mapping"
    if mounted_here "$MOUNT_POINT"; then run umount "$MOUNT_POINT" || true; fi
    if mounted_here "$STAGE_MOUNT"; then run umount "$STAGE_MOUNT" || true; fi
    if mapping_exists; then run cryptsetup close "$MAPPER_NAME" || true; fi

    log "destroying all LUKS keyslots on $dev (irreversible data wipe)"
    run cryptsetup luksErase "$dev" || die "luksErase failed"
    detach_data_loops

    log "clearing snapshots, encrypted archives, exports and provisioning flag"
    clear_private_dir "$ARCHIVE_DIR"
    clear_private_dir "$EXPORT_DIR"
    run rm -f "$INIT_FLAG"
    clear_private_dir "$PRIVATE_DIR"

    log "re-initialising a fresh encrypted volume; new runtime keys will be generated"
    if is_dry; then
        log "  [dry-run] would call provision.sh --boot"
    else
        "$SCRIPT_DIR/provision.sh" --boot || die "re-provision after reset failed"
    fi

    log "RESET COMPLETE — rescue partition preserved; wizard will repopulate $MOUNT_POINT"
    if [ "$reboot" -eq 1 ]; then
        log "rebooting"
        run systemctl reboot
    else
        log "reboot (or restart generalbots-shell.service) to enter the wizard"
    fi
}

main "$@"
