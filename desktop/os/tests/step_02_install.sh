#!/usr/bin/env bash
# One action: install exactly ONE supported component in container mode.
# Usage: step_02_install.sh <component>
set -uo pipefail
here="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=uat_common.sh
. "$here/uat_common.sh"

comp="${1:-}"
if [ -z "$comp" ]; then
    printf 'usage: %s <component>\n' "$0" >&2
    exit 2
fi
id="install:$comp"
uat_step_banner "$id — botserver install $comp --container"
if ! uat_live_gate "$id"; then
    exit 3
fi

dir="$(uat_init_results)" || exit 2
log="$dir/install-$comp.log"
if uat_botserver install "$comp" --container 2>&1 | tee "$log"; then
    uat_record "$id" PASS "install command exited 0; log: $log"
else
    uat_record "$id" FAIL "install command failed; log: $log"
    exit 1
fi
