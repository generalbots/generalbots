#!/usr/bin/env bash
# One action: report the install status of ONE component in container mode.
# Usage: step_03_status.sh <component>
# The leading marker is locale-independent: '*' installed, 'x' not installed
# (botcore/src/package_manager/cli/cli_run.rs, status branch).
set -uo pipefail
here="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=uat_common.sh
. "$here/uat_common.sh"

comp="${1:-}"
if [ -z "$comp" ]; then
    printf 'usage: %s <component>\n' "$0" >&2
    exit 2
fi
id="status:$comp"
uat_step_banner "$id — botserver status $comp --container"
if ! uat_live_gate "$id"; then
    exit 3
fi

out="$(uat_botserver status "$comp" --container 2>&1)"
rc=$?
printf '%s\n' "$out"
marker="$(printf '%s\n' "$out" | head -1 | cut -c1)"
if [ "$rc" -eq 0 ] && [ "$marker" = "*" ]; then
    uat_record "$id" PASS "status reports installed"
else
    uat_record "$id" FAIL "status does not report installed (marker='$marker', rc=$rc)"
    exit 1
fi
