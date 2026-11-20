#!/bin/bash
# Static configuration check for the WebKitGTK bwrap sandbox (#2500).
#
# Checks the mkosi recipe sources and, optionally, an already-extracted image
# tree (pass its root as $1). This is deliberately STATIC: it proves that the
# packages and the "fail loudly" settings are configured. It does NOT prove
# that a running WebKitWebProcess actually lives in a bwrap namespace, nor that
# a file:// read of the stack is refused. Those need a booted image (QEMU smoke
# + /proc/<pid>/status and WebKit sandbox logging) and are tracked in #2500.
set -euo pipefail

here="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
conf="$here/mkosi.conf"
finalize="$here/mkosi.finalize.chroot"
unit="$here/mkosi.extra/usr/lib/systemd/system/generalbots-shell.service"
image_tree="${1:-}"

failed=0

pass() { printf 'PASS  %s\n' "$1"; }
fail() { printf 'FAIL  %s\n' "$1" >&2; failed=$((failed + 1)); }

# grep a required literal line in a text file
expect_line() {
    local label="$1" file="$2" pattern="$3"
    if [ -f "$file" ] && grep -Eq -- "$pattern" "$file"; then
        pass "$label"
    else
        fail "$label (missing in ${file#$here/})"
    fi
}

# assert a namespace allow-list contains a given token
expect_namespace() {
    local label="$1" token="$2" line
    line="$(grep -E '^RestrictNamespaces=' "$unit" 2>/dev/null || true)"
    line="${line#RestrictNamespaces=}"
    case " $line " in
        *" $token "*) pass "$label" ;;
        *) fail "$label (RestrictNamespaces=$line)" ;;
    esac
}

printf 'WebKitGTK sandbox configuration check (#2500) — static only\n'
printf 'This script does not prove runtime sandboxing; see the header.\n\n'

expect_line 'bubblewrap pinned in Packages=' "$conf" '^[[:space:]]*bubblewrap[[:space:]]*$'
expect_line 'libseccomp2 pinned in Packages=' "$conf" '^[[:space:]]*libseccomp2[[:space:]]*$'
expect_line 'shell unit forces WebKit sandbox' "$unit" '^Environment=WEBKIT_FORCE_SANDBOX=1$'
expect_line 'finalize exports WEBKIT_FORCE_SANDBOX to the session' \
    "$finalize" '^WEBKIT_FORCE_SANDBOX=1$'
expect_line 'finalize writes the /etc/environment.d drop-in' \
    "$finalize" '^cat >/etc/environment\.d/50-general-bots\.conf '
expect_line 'shell unit allows mount syscalls for bwrap' \
    "$unit" '^SystemCallFilter=@system-service @mount$'
if grep -Eq '^RestrictNamespaces=yes$' "$unit" 2>/dev/null; then
    fail 'shell unit does not blanket-block namespaces'
else
    pass 'shell unit does not blanket-block namespaces'
fi
for ns in user mnt ipc pid net; do
    expect_namespace "shell unit allows $ns namespace for bwrap" "$ns"
done

# Optional: check an already-extracted image tree. This stays static — a
# mounted/copied rootfs still cannot show a live web process namespace.
if [ -n "$image_tree" ]; then
    if [ ! -d "$image_tree" ]; then
        fail "image tree is not a directory: $image_tree"
    else
        printf '\nImage tree: %s\n' "$image_tree"
        if [ -x "$image_tree/usr/bin/bwrap" ]; then
            pass 'bwrap binary installed in image tree'
        else
            fail 'bwrap binary missing or not executable in image tree'
        fi
        expect_line 'shell unit in image forces WebKit sandbox' \
            "$image_tree/usr/lib/systemd/system/generalbots-shell.service" \
            '^Environment=WEBKIT_FORCE_SANDBOX=1$'
        expect_line 'environment.d drop-in in image forces WebKit sandbox' \
            "$image_tree/etc/environment.d/50-general-bots.conf" \
            '^WEBKIT_FORCE_SANDBOX=1$'
    fi
fi

printf '\n'
if [ "$failed" -ne 0 ]; then
    printf '%d static check(s) failed.\n' "$failed" >&2
    exit 1
fi
printf 'All static checks passed. Runtime sandboxing remains unverified.\n'
