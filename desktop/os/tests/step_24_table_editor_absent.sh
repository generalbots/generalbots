#!/usr/bin/env bash
# One action: regression — table_editor is out of scope and unmanaged (#2582).
# It must be absent from the CLI registry and have no Incus container.
set -uo pipefail
here="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=uat_common.sh
. "$here/uat_common.sh"

id="reg-table-editor-absent"
uat_step_banner "$id — table_editor absent (out of scope, pending deletion)"
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
if grep -qw 'table_editor' <<< "$list"; then
    uat_record "$id" FAIL "table_editor still appears in botserver list --container"
    exit 1
fi

containers="$(incus list --format=csv 2>/dev/null)"
name="$(uat_tenant)-table_editor"
if grep -q "$name" <<< "$containers"; then
    uat_record "$id" FAIL "unexpected Incus container $name exists"
    exit 1
fi
printf 'table_editor: absent from registry; no %s container\n' "$name"
uat_record "$id" PASS "table_editor not managed and not installed"
