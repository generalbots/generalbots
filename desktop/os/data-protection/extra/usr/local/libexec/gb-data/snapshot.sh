#!/bin/bash
# Encrypted local snapshots of /var/lib/gbdesk with retention (#2470).
#
# Snapshots are read-only btrfs subvolumes on the LUKS2-encrypted data volume,
# so they are encrypted at rest. An optional GPG-encrypted tar archive is also
# written outside that volume (into $ARCHIVE_DIR) so a lost/damaged data volume
# still has an encrypted local backup with a documented restore path.
#
#   (default)  take a snapshot, apply retention, write encrypted archive
#   --list     list snapshots and encrypted archives
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "$SCRIPT_DIR/lib.sh"
load_config
skip_if_requested

usage() {
    cat <<EOF
Usage: $0 [--list|--now|--help]
  --now    take one snapshot now (default)
  --list   list snapshots + encrypted archives
Env: DRY_RUN=1, GB_DATA_PROTECTION_SKIP=1. Retention: keep newest
     \$SNAPSHOT_KEEP_DAILY daily snapshots + one per week for
     \$SNAPSHOT_KEEP_WEEKLY weeks; older ones are deleted.
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

iso_week() {
    local stamp="$1" d
    d="${stamp:0:4}-${stamp:4:2}-${stamp:6:2}"
    date -u -d "$d" +%G-W%V 2>/dev/null || printf 'unknown\n'
}

apply_retention() {
    local dir="$1" keep_daily="$2" keep_weekly="$3"
    if is_dry; then
        log "retention: keep newest $keep_daily daily + 1/week for $keep_weekly weeks; delete older"
        return 0
    fi
    [ -d "$dir" ] || return 0

    local -a all=()
    local f
    while IFS= read -r f; do
        all+=("$f")
    done < <(find "$dir" -maxdepth 1 -mindepth 1 -type d -name "${SNAPSHOT_PREFIX}-*" -printf '%f\n' | sort -r)
    [ "${#all[@]}" -gt 0 ] || return 0

    local idx=0 kept_weekly=0 week last_week="" snapshot meta
    for snapshot in "${all[@]}"; do
        meta="$dir/$snapshot.meta"
        if [ "$idx" -lt "$keep_daily" ]; then
            idx=$((idx + 1))
            continue
        fi
        week="$(iso_week "${snapshot#${SNAPSHOT_PREFIX}-}")"
        if [ "$week" != "unknown" ] && [ "$week" != "$last_week" ] && [ "$kept_weekly" -lt "$keep_weekly" ]; then
            last_week="$week"
            kept_weekly=$((kept_weekly + 1))
            continue
        fi
        log "pruning expired snapshot $snapshot (week=$week)"
        run btrfs subvolume delete "$dir/$snapshot"
        run rm -f "$meta"
    done
}

list_items() {
    log "encrypted archives in $ARCHIVE_DIR:"
    if is_dry; then
        log "  [dry-run] would list *.tar.gz.gpg in $ARCHIVE_DIR"
    else
        find "$ARCHIVE_DIR" -maxdepth 1 -type f -name '*.tar.gz.gpg' -printf '%f  %s bytes\n' 2>/dev/null \
            | sort || true
    fi
    log "btrfs snapshots on $MAPPER_NAME:"
    if is_dry; then
        log "  [dry-run] would mount $STAGE_MOUNT and list $SNAPSHOT_SUBDIR"
    else
        mount_toplevel
        find "$STAGE_MOUNT/$SNAPSHOT_SUBDIR" -maxdepth 1 -mindepth 1 -type d -printf '%f\n' 2>/dev/null | sort || true
        run umount "$STAGE_MOUNT"
    fi
}

take_snapshot() {
    local stamp name src dst archive
    stamp="$(date -u '+%Y%m%dT%H%M%SZ')"
    name="${SNAPSHOT_PREFIX}-${stamp}"
    src="$STAGE_MOUNT/$SUBVOLUME"
    dst="$STAGE_MOUNT/$SNAPSHOT_SUBDIR/$name"

    log "creating read-only snapshot $name"
    run btrfs subvolume snapshot -r "$src" "$dst" || die "btrfs snapshot failed"
    run_sh "printf '%s\n' 'snapshot=$name created=$(date -u '+%FT%TZ') source=$SUBVOLUME' > '$dst.meta'"

    apply_retention "$STAGE_MOUNT/$SNAPSHOT_SUBDIR" "$SNAPSHOT_KEEP_DAILY" "$SNAPSHOT_KEEP_WEEKLY"

    if [ "${SNAPSHOT_ENCRYPT_ARCHIVE:-1}" = "1" ]; then
        archive="$ARCHIVE_DIR/$name.tar.gz.gpg"
        log "writing GPG-encrypted archive $archive (AES256, key from private path)"
        run_sh "tar -C '$STAGE_MOUNT/$SUBVOLUME' -czf - . | gpg --batch --yes --symmetric --cipher-algo AES256 --passphrase-file '$ARCHIVE_KEY_FILE' --output '$archive'"
    else
        log "SNAPSHOT_ENCRYPT_ARCHIVE=0 — skipping encrypted archive"
    fi
}

main() {
    local mode="${1:---now}"
    case "$mode" in
        --now)  ;;
        --list) ;;
        -h|--help) usage; return 0 ;;
        *) die "unknown mode '$mode' (try --help)" ;;
    esac

    require_root
    require_cmds btrfs mount umount install find sort date tar gpg
    ensure_volume

    if [ "$mode" = "--list" ]; then
        list_items
        return 0
    fi

    mount_toplevel
    take_snapshot
    run umount "$STAGE_MOUNT"
    log "snapshot complete — encrypted at rest on the LUKS volume; archive in $ARCHIVE_DIR"
}

main "$@"
