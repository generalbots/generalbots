#!/usr/bin/env bash
# One action: print the live component registry of the binary under test.
# This is the CLI surface used by every absence/presence regression check.
set -uo pipefail
here="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=uat_common.sh
. "$here/uat_common.sh"

id="list"
uat_step_banner "$id — botserver list --container"
if ! uat_live_gate "$id"; then
    exit 3
fi

out="$(uat_component_list 2>&1)"
rc=$?
printf '%s\n' "$out"
if [ "$rc" -ne 0 ]; then
    uat_record "$id" FAIL "botserver list --container exited $rc"
    exit 1
fi
count="$(printf '%s\n' "$out" | wc -l)"
uat_record "$id" PASS "botserver list --container returned $count component lines"
