#!/bin/bash
# Restore paths for General Bots user data (#2470). DESTRUCTIVE — requires
# --confirm after exactly the action you intend. The rescue partition is never
# touched; assert_not_protected() guards the device before any write.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "$SCRIPT_DIR/lib.sh"
load_config
skip_if_requested

usage() {
    cat <<EOF
Usage:
  $0 --list
  $0 --snapshot NAME --confirm        rollback @data to a btrfs snapshot
  $0 --archive FILE.tar.gz.gpg --confirm   restore an encrypted archive
  $0 --help
Restore is destructive: stop the desktop stack first and boot from the rescue
partition when rolling back the live data volume. Never run on protected disks.
EOF
}

ensure_volume() {
    if ! mapping_exists && ! is_dry; then
        die "LUKS mapping /dev/mapper/$MAPPER_NAME not open — run provision.sh --boot first"
    fi
}

mount_toplevel() {
    run install -d -m 0755 "$STAGE_MOUNT" || die "create $STAGE_MOUNT failed"
    run mount "/dev/mapper/$MAPPER_NAME" "$STAGE_MOUNT" || die "top-level mount failed"
}

rollback_snapshot() {
    local name="$1" src old stamp
    stamp="$(date -u '+%Y%m%dT%H%M%SZ')"
    src="$STAGE_MOUNT/$SNAPSHOT_SUBDIR/$name"
    old="${SUBVOLUME}.old.${stamp}"

    if ! is_dry && [ ! -d "$src" ]; then
        die "snapshot '$name' not found in $SNAPSHOT_SUBDIR"
    fi

    log "safety snapshot of current $SUBVOLUME before rollback"
    run btrfs subvolume snapshot -r "$STAGE_MOUNT/$SUBVOLUME" "$STAGE_MOUNT/${SUBVOLUME}.pre-restore.${stamp}"
    log "rolling $SUBVOLUME back to snapshot $name"
    if ! is_dry; then
        run mv "$STAGE_MOUNT/$SUBVOLUME" "$STAGE_MOUNT/$old" || die "rename current subvolume failed"
    else
        log "  [dry-run] would rename $SUBVOLUME -> $old"
    fi
    run btrfs subvolume snapshot "$src" "$STAGE_MOUNT/$SUBVOLUME" || die "restore snapshot failed"
    log "restored; old data kept at $old — verify then delete it manually"
    run_sh "printf '%s\n' 'restored_from=$name at=\$(date -u '+%FT%TZ')' > '$STAGE_MOUNT/RESTORE_INFO.txt'"
}

restore_archive() {
    local archive="$1"
    if ! is_dry && [ ! -f "$archive" ]; then
        die "archive '$archive' not found"
    fi
    log "decrypting $archive with private key $ARCHIVE_KEY_FILE into $SUBVOLUME"
    run_sh "gpg --batch --yes --decrypt --passphrase-file '$ARCHIVE_KEY_FILE' '$archive' | tar -C '$STAGE_MOUNT/$SUBVOLUME' -xzf -"
    log "archive restored — restart the desktop stack to pick up the data"
}

main() {
    local action="" name="" archive="" confirm=0
    while [ "$#" -gt 0 ]; do
        case "$1" in
            --list)      action="list" ;;
            --snapshot)  action="snapshot"; name="${2:-}"; shift ;;
            --archive)   action="archive"; archive="${2:-}"; shift ;;
            --confirm)   confirm=1 ;;
            -h|--help)   usage; return 0 ;;
            *) die "unknown argument '$1' (try --help)" ;;
        esac
        shift
    done

    case "$action" in
        list)    ;;
        snapshot) [ -n "$name" ] || die "--snapshot needs a NAME" ;;
        archive)  [ -n "$archive" ] || die "--archive needs a FILE" ;;
        "") die "no action given (try --help)" ;;
    esac

    require_root
    require_cmds btrfs mount umount install find sort date tar gpg
    ensure_volume

    # The device we operate on must never be a rescue/system partition.
    local dev="/dev/mapper/$MAPPER_NAME"
    assert_not_protected "$dev"

    if [ "$action" = "list" ]; then
        /bin/bash "$SCRIPT_DIR/snapshot.sh" --list
        return 0
    fi
    if [ "$confirm" -ne 1 ]; then
        die "refusing destructive restore without --confirm"
    fi

    mount_toplevel
    if [ "$action" = "snapshot" ]; then
        rollback_snapshot "$name"
    else
        restore_archive "$archive"
    fi
    run umount "$STAGE_MOUNT"
}

main "$@"
