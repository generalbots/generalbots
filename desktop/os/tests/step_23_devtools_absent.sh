#!/usr/bin/env bash
# One action: regression — devtools is absent from the live CLI surface (#2582).
set -uo pipefail
here="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=uat_common.sh
. "$here/uat_common.sh"

id="reg-devtools-absent"
uat_step_banner "$id — devtools absent from registry and status"
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
if grep -qw 'devtools' <<< "$list"; then
    uat_record "$id" FAIL "devtools still appears in botserver list --container"
    exit 1
fi

status="$(uat_botserver status devtools --container 2>&1)"
printf 'status: %s\n' "$status"
marker="$(printf '%s\n' "$status" | head -1 | cut -c1)"
if [ "$marker" = "*" ]; then
    uat_record "$id" FAIL "devtools reports installed"
    exit 1
fi
printf 'devtools: absent from registry; status marker=%s\n' "$marker"
uat_record "$id" PASS "devtools absent from list and not installed"
