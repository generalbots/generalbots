#!/bin/bash
# "Export my data" (#2470): full encrypted archive of /var/lib/gbdesk.
# The passphrase is generated at runtime into a root-owned 0600 file and is
# never stored in the repository or written to logs.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
. "$SCRIPT_DIR/lib.sh"
load_config
skip_if_requested

usage() {
    cat <<EOF
Usage: $0 [--now|--list|--help]
  --now   create \$EXPORT_DIR/gbdesk-<UTC>.tar.gz.gpg (default)
  --list  list existing exports
Copy the .gpg file to external/Drive storage; keep the passphrase file
\$EXPORT_KEY_FILE (0600, root-owned) separate from the ciphertext.
EOF
}

main() {
    local mode="${1:---now}"
    case "$mode" in
        --now)   ;;
        --list)  ;;
        -h|--help) usage; return 0 ;;
        *) die "unknown mode '$mode' (try --help)" ;;
    esac

    require_root
    require_cmds tar gpg install find sort date

    if [ "$mode" = "--list" ]; then
        log "exports in $EXPORT_DIR:"
        if is_dry; then
            log "  [dry-run] would list *.tar.gz.gpg in $EXPORT_DIR"
        else
            find "$EXPORT_DIR" -maxdepth 1 -type f -name '*.tar.gz.gpg' -printf '%f  %s bytes\n' 2>/dev/null | sort || true
        fi
        return 0
    fi

    if ! mounted_here "$MOUNT_POINT" && ! is_dry; then
        die "$MOUNT_POINT is not mounted — run provision.sh --boot first"
    fi
    if ! is_dry && [ ! -f "$EXPORT_KEY_FILE" ]; then
        die "export passphrase file missing: $EXPORT_KEY_FILE (run provision.sh first)"
    fi
    is_dry && log "dry-run: assuming runtime passphrase file $EXPORT_KEY_FILE exists"

    run install -d -m 0700 -o root -g root "$EXPORT_DIR" || die "create $EXPORT_DIR failed"
    local out="$EXPORT_DIR/gbdesk-$(date -u '+%Y%m%dT%H%M%SZ').tar.gz.gpg"
    log "exporting $MOUNT_POINT -> $out (AES256, runtime passphrase from private path)"
    run_sh "tar -C '$MOUNT_POINT' -czf - . | gpg --batch --yes --symmetric --cipher-algo AES256 --passphrase-file '$EXPORT_KEY_FILE' --output '$out'"
    log "export ready: $out"
    log "passphrase file: $EXPORT_KEY_FILE (0600) — store it OUTSIDE the exported archive"
}

main "$@"
