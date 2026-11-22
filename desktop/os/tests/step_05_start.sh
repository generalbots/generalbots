#!/usr/bin/env bash
# One action: start all installed components in container mode.
# Does not restart the host, does not rebuild botserver and does not touch git.
set -uo pipefail
here="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=uat_common.sh
. "$here/uat_common.sh"

id="start"
uat_step_banner "$id — botserver start --container"
if ! uat_live_gate "$id"; then
    exit 3
fi

out="$(uat_botserver start --container 2>&1)"
rc=$?
printf '%s\n' "$out"
if [ "$rc" -eq 0 ]; then
    uat_record "$id" PASS "start command exited 0"
else
    uat_record "$id" FAIL "start command exited $rc"
    exit 1
fi
