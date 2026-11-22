#!/usr/bin/env bash
# One action: regression — host prep is Incus-only, live and in source (#2582).
# Live: Incus storage/profile/proxy device exist. Source: the host registrar
# invokes incus and contains no lxc/lxd host commands. The source half needs a
# repo checkout (UAT_SOURCE_ROOT) and is never evaluated offline.
set -uo pipefail
here="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=uat_common.sh
. "$here/uat_common.sh"

id="reg-host-incus"
uat_step_banner "$id — host component uses Incus only"
if ! uat_live_gate "$id"; then
    exit 3
fi

src_root="${UAT_SOURCE_ROOT:-$(cd -- "$here/../../.." && pwd)}"
registrar="$src_root/botserver/crates/botcorepkg/src/installer_regs/register_observability.rs"
if [ ! -f "$registrar" ]; then
    uat_record "$id" UNVERIFIED "registrar not found under UAT_SOURCE_ROOT"
    exit 3
fi

live_ok=0
if incus storage show default >/dev/null 2>&1 \
    && incus profile device show default 2>/dev/null | grep -q 'incus-sock'; then
    live_ok=1
fi
printf 'live incus storage=default profile-device=incus-sock: %s\n' "$live_ok"

source_ok=0
if grep -q 'incus admin init' "$registrar" \
    && grep -q '/var/lib/incus/unix.socket' "$registrar" \
    && ! grep -Eq '(^|[^[:alnum:]_])(lxc|lxd)([[:space:]]|$)' "$registrar"; then
    source_ok=1
fi
printf 'source incus-only: %s (%s)\n' "$source_ok" "${registrar#$src_root/}"

if [ "$live_ok" -eq 1 ] && [ "$source_ok" -eq 1 ]; then
    uat_record "$id" PASS "Incus storage/device present; registrar has no legacy runtime"
else
    uat_record "$id" FAIL "live=$live_ok source=$source_ok"
    exit 1
fi
