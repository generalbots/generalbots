#!/usr/bin/env bash
# One action: aggregate recorded step results into a verdict. Never invents PASS.
# Exit: 0 PASS (all live evidence present), 1 FAIL, 3 UNVERIFIED.
set -uo pipefail
here="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=uat_common.sh
. "$here/uat_common.sh"

id="report"
uat_step_banner "$id — evidence summary (#2582 install UAT)"
results="$(uat_results_dir)/results.tsv"
if [ ! -f "$results" ]; then
    printf 'UNVERIFIED: no results recorded at %s\n' "$results" >&2
    printf 'PASS requires live evidence on a target host; nothing can pass offline.\n' >&2
    exit 3
fi

printf 'step\tstatus\tdetail\n'
awk -F '\t' '{ printf "%s\t%s\t%s\n", $1, $2, $3 }' "$results" | sort

# Component registry from manager_construction.rs::all_components, excluding
# the removed devtools and the renamed remote_terminal.
components="vault tables cache drive llm email proxy directory alm alm-ci dns meet vector_db timeseries_db botmodels host observability remote_desktop"
health="vault:8200 tables:5432 cache:6379 drive:9100 llm:8081 email:8025 proxy:80 directory:8300 alm:4747 dns:53 meet:7880 vector_db:6334 timeseries_db:8086 botmodels:8085 observability:8686 remote_desktop:3389"
singles="preflight list start login-page login-api reg-remote-desktop reg-timeseries-paths reg-host-incus reg-devtools-absent"

if awk -F '\t' '$2 == "FAIL" { found = 1 } END { exit !found }' "$results"; then
    failed="$(awk -F '\t' '$2 == "FAIL" { print $1 }' "$results" | paste -sd ' ' -)"
    printf '\nverdict: FAIL\nfailed steps: %s\n' "$failed"
    exit 1
fi

missing=""
for step in $singles; do
    uat_has_pass "$step" || missing="$missing $step"
done
for comp in $components; do
    uat_has_pass "install:$comp" || missing="$missing install:$comp"
    uat_has_pass "status:$comp" || missing="$missing status:$comp"
done
for entry in $health; do
    comp="${entry%%:*}"
    uat_has_pass "health:$comp" || missing="$missing health:$comp"
done

if [ -n "$missing" ]; then
    printf '\nverdict: UNVERIFIED\nmissing live evidence:%s\n' "$missing"
    printf 'PASS requires UAT_TARGET_HOST=1, Incus, the binary under test and every step above.\n'
    exit 3
fi

printf '\nverdict: PASS\n'
printf 'All registry, install, status, health, login and regression steps have live PASS evidence.\n'
