#!/usr/bin/env bash
# One action: regression — remote_desktop is exposed and remote_terminal is not.
# Checks the live CLI registry and the live Incus container list (#2582).
set -uo pipefail
here="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=uat_common.sh
. "$here/uat_common.sh"

id="reg-remote-desktop"
uat_step_banner "$id — remote_desktop present, remote_terminal absent"
if ! uat_live_gate "$id"; then
    exit 3
fi

list="$(uat_component_list 2>&1)"
rc=$?
printf 'registry:\n%s\n' "$list"
if [ "$rc" -ne 0 ]; then
    uat_record "$id" FAIL "botserver list --container exited $rc"
    exit 1
fi

tenant="$(uat_tenant)"
containers="$(incus list --format=csv 2>/dev/null)"
new_ok=0
old_ok=0
container_ok=0
grep -qw 'remote_desktop' <<< "$list" && new_ok=1
if grep -qw 'remote_terminal' <<< "$list"; then
    old_ok=0
else
    old_ok=1
fi
grep -q "${tenant}-remote_desktop" <<< "$containers" && container_ok=1
printf 'key=remote_desktop:%s legacy=absent:%s container=%s-remote_desktop:%s\n' \
    "$new_ok" "$old_ok" "$tenant" "$container_ok"
if [ "$new_ok" -eq 1 ] && [ "$old_ok" -eq 1 ] && [ "$container_ok" -eq 1 ]; then
    uat_record "$id" PASS "remote_desktop registered and started; no remote_terminal key"
else
    uat_record "$id" FAIL "remote_desktop=$new_ok legacy_absent=$old_ok container=$container_ok"
    exit 1
fi
