#!/usr/bin/env bash
# One action: verify target-host preconditions for the #2582 install UAT.
# Prints host facts only (no addresses, tokens or credentials) and stops.
set -uo pipefail
here="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=uat_common.sh
. "$here/uat_common.sh"

id="preflight"
uat_step_banner "$id — target-host preconditions"
if ! uat_live_gate "$id"; then
    exit 3
fi

printf 'kernel:    %s\n' "$(uname -sr)"
printf 'incus:     %s\n' "$(incus version 2>/dev/null | tr '\n' ' ')"
bin="$(uat_botserver_bin)"
printf 'botserver: %s\n' "$bin"
printf 'version:   %s\n' "$("$bin" --version 2>&1 | head -1)"
printf 'tenant:    %s\n' "$(uat_tenant)"
printf 'stack:     %s (container mode uses /opt/gbo inside each container)\n' \
    "${UAT_STACK_PATH:-/opt/gbo}"

dir="$(uat_init_results)" || exit 2
printf 'results:   %s\n' "$dir"
uat_record "$id" PASS \
    "Incus CLI and botserver binary present; UAT_TARGET_HOST=1; results dir writable"
