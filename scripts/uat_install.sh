#!/usr/bin/env bash
# #2582 install UAT dispatcher.
#
# One action per invocation: each `step` runs a single, fragmented step script,
# prints its observed state and stops. `plan` only prints the recommended order
# (no side effects). `report` aggregates recorded evidence; it can only say PASS
# when every step ran on an opted-in target host (UAT_TARGET_HOST=1).
# `--self-test` runs offline against synthetic stub evidence in /tmp/opencode and
# asserts the verdict logic; it never sets the target-host opt-in and never
# claims a real host PASS.
set -uo pipefail
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$script_dir/.." && pwd)"
tests_dir="$repo_root/desktop/os/tests"

step_file() {
    case "$1" in
        00|preflight) printf '%s\n' "$tests_dir/step_00_preflight.sh" ;;
        01|list) printf '%s\n' "$tests_dir/step_01_list.sh" ;;
        02|install) printf '%s\n' "$tests_dir/step_02_install.sh" ;;
        03|status) printf '%s\n' "$tests_dir/step_03_status.sh" ;;
        04|health) printf '%s\n' "$tests_dir/step_04_health.sh" ;;
        05|start) printf '%s\n' "$tests_dir/step_05_start.sh" ;;
        10|login-page) printf '%s\n' "$tests_dir/step_10_login_page.sh" ;;
        11|login-api) printf '%s\n' "$tests_dir/step_11_login_api.sh" ;;
        20|remote-desktop) printf '%s\n' "$tests_dir/step_20_remote_desktop.sh" ;;
        21|timeseries-paths) printf '%s\n' "$tests_dir/step_21_timeseries_paths.sh" ;;
        22|host-incus) printf '%s\n' "$tests_dir/step_22_host_incus.sh" ;;
        23|devtools-absent) printf '%s\n' "$tests_dir/step_23_devtools_absent.sh" ;;
        24|table-editor-absent) printf '%s\n' "$tests_dir/step_24_table_editor_absent.sh" ;;
        99|report) printf '%s\n' "$tests_dir/step_99_report.sh" ;;
        *) return 1 ;;
    esac
}

usage() {
    cat <<'EOF'
Usage: scripts/uat_install.sh <command> [args]

  help                 show this text
  steps                list the step ids and their one action
  plan                 print the recommended one-command-at-a-time order
  preflight            alias for `step 00`
  step <id> [args]     run ONE fragment and stop (see `steps`)
  report               aggregate evidence; PASS only with live results
  --self-test          offline verdict-logic assertions with a stub (no host)

Exit codes: 0 PASS, 1 FAIL, 2 usage/precondition error, 3 UNVERIFIED.
Nothing claims PASS unless UAT_TARGET_HOST=1, Incus and the botserver binary
under test are present. Credentials come from UAT_LOGIN_USER /
UAT_LOGIN_PASSWORD; no secret or internal address is stored in this harness.
`--self-test` uses a throwaway stub under /tmp/opencode; any PASS it prints is
synthetic proof of the verdict logic, NOT a result from a real target host.
EOF
}

list_steps() {
    cat <<'EOF'
00 preflight           target-host preconditions (Incus, binary, opt-in)
01 list                botserver list --container (live registry)
02 install <comp>      botserver install <comp> --container (one component)
03 status <comp>       botserver status <comp> --container (one component)
04 health <comp> <port> [path]   one health probe (HTTP path or TCP port)
05 start               botserver start --container (all installed components)
10 login-page          GET /login from the login UI
11 login-api           POST /api/auth/login with env credentials
20 remote-desktop      remote_desktop present, remote_terminal absent
21 timeseries-paths    generic timeseries paths, no influxdb directories
22 host-incus          host prep Incus-only (live + registrar source)
23 devtools-absent     devtools absent from CLI registry/status
24 table-editor-absent table_editor unmanaged and without a container
99 report              aggregate results into PASS / FAIL / UNVERIFIED
EOF
}

print_plan() {
    cat <<'EOF'
# Preconditions: UAT_TARGET_HOST=1, incus on PATH, UAT_BOTSERVER=<binary under
# test>, UAT_LOGIN_USER/UAT_LOGIN_PASSWORD for step 11, UAT_SOURCE_ROOT for
# step 22. Results go to UAT_RESULTS_DIR (default /tmp/opencode/uat-install).
# Run ONE command at a time; read its output before choosing the next step.
# Excluded from installation: table_editor, webmail, news (out of scope #2582).
scripts/uat_install.sh preflight
scripts/uat_install.sh step 01
scripts/uat_install.sh step 02 vault
scripts/uat_install.sh step 02 tables
scripts/uat_install.sh step 02 cache
scripts/uat_install.sh step 02 drive
scripts/uat_install.sh step 02 llm
scripts/uat_install.sh step 02 email
scripts/uat_install.sh step 02 proxy
scripts/uat_install.sh step 02 directory
scripts/uat_install.sh step 02 alm
scripts/uat_install.sh step 02 alm-ci
scripts/uat_install.sh step 02 dns
scripts/uat_install.sh step 02 meet
scripts/uat_install.sh step 02 vector_db
scripts/uat_install.sh step 02 timeseries_db
scripts/uat_install.sh step 02 botmodels
scripts/uat_install.sh step 02 host
scripts/uat_install.sh step 02 observability
scripts/uat_install.sh step 02 remote_desktop
scripts/uat_install.sh step 05
scripts/uat_install.sh step 03 vault
scripts/uat_install.sh step 04 vault 8200
scripts/uat_install.sh step 03 tables
scripts/uat_install.sh step 04 tables 5432
scripts/uat_install.sh step 03 cache
scripts/uat_install.sh step 04 cache 6379
scripts/uat_install.sh step 03 drive
scripts/uat_install.sh step 04 drive 9100
scripts/uat_install.sh step 03 llm
scripts/uat_install.sh step 04 llm 8081
scripts/uat_install.sh step 03 email
scripts/uat_install.sh step 04 email 8025
scripts/uat_install.sh step 03 proxy
scripts/uat_install.sh step 04 proxy 80
scripts/uat_install.sh step 03 directory
scripts/uat_install.sh step 04 directory 8300
scripts/uat_install.sh step 03 alm
scripts/uat_install.sh step 04 alm 4747
scripts/uat_install.sh step 03 alm-ci
scripts/uat_install.sh step 03 dns
scripts/uat_install.sh step 04 dns 53
scripts/uat_install.sh step 03 meet
scripts/uat_install.sh step 04 meet 7880
scripts/uat_install.sh step 03 vector_db
scripts/uat_install.sh step 04 vector_db 6334
scripts/uat_install.sh step 03 timeseries_db
scripts/uat_install.sh step 04 timeseries_db 8086 /health
scripts/uat_install.sh step 03 botmodels
scripts/uat_install.sh step 04 botmodels 8085
scripts/uat_install.sh step 03 host
scripts/uat_install.sh step 03 observability
scripts/uat_install.sh step 04 observability 8686 /health
scripts/uat_install.sh step 03 remote_desktop
scripts/uat_install.sh step 04 remote_desktop 3389
scripts/uat_install.sh step 10
scripts/uat_install.sh step 11
scripts/uat_install.sh step 20
scripts/uat_install.sh step 21
scripts/uat_install.sh step 22
scripts/uat_install.sh step 23
scripts/uat_install.sh step 24
scripts/uat_install.sh report
EOF
}

# ---- #2582 offline self-test: assert verdict logic with synthetic evidence --
# Everything here runs in /tmp/opencode with a throwaway results directory. It
# clears every opt-in/credential variable, never invokes incus or botserver and
# never touches the real results directory. A PASS below means "the verdict
# logic behaved as asserted on stub rows" — never "a real host passed".
selftest_fail() {
    printf 'FAIL %s\n' "$1"
    SELFTEST_FAILURES=$((SELFTEST_FAILURES + 1))
}

selftest_run() {
    SELFTEST_OUT="$("$@" 2>&1)"
    SELFTEST_RC=$?
}

selftest_expect_code() {
    local desc="$1" want="$2"
    if [ "$SELFTEST_RC" -eq "$want" ]; then
        printf 'ok   %s (exit %s)\n' "$desc" "$want"
    else
        selftest_fail "$desc (expected exit $want, got $SELFTEST_RC)"
        printf '%s\n' "$SELFTEST_OUT" | sed 's/^/       | /'
    fi
}

selftest_expect_text() {
    local desc="$1" needle="$2"
    case "$SELFTEST_OUT" in
        *"$needle"*) printf 'ok   %s\n' "$desc" ;;
        *) selftest_fail "$desc (missing text: $needle)" ;;
    esac
}

# Build a stub results.tsv containing exactly the rows step_99_report.sh
# requires. The required ids are parsed from the report itself so the stub can
# never drift away from the verdict it is meant to satisfy.
selftest_full_evidence() {
    local report="$tests_dir/step_99_report.sh"
    local components health singles step comp entry
    components="$(sed -n 's/^components="\(.*\)"$/\1/p' "$report")"
    health="$(sed -n 's/^health="\(.*\)"$/\1/p' "$report")"
    singles="$(sed -n 's/^singles="\(.*\)"$/\1/p' "$report")"
    if [ -z "$components" ] || [ -z "$health" ] || [ -z "$singles" ]; then
        return 2
    fi
    for step in $singles; do
        printf '%s\tPASS\tselftest stub\n' "$step"
    done
    for comp in $components; do
        printf 'install:%s\tPASS\tselftest stub\n' "$comp"
        printf 'status:%s\tPASS\tselftest stub\n' "$comp"
    done
    for entry in $health; do
        printf 'health:%s\tPASS\tselftest stub\n' "${entry%%:*}"
    done
}

selftest() {
    local dir="${UAT_SELFTEST_DIR:-/tmp/opencode/uat-install-selftest}"
    local report="$tests_dir/step_99_report.sh"
    local results="$dir/results.tsv"
    SELFTEST_FAILURES=0

    printf '=== SELF-TEST (synthetic stub; NOT a real host run) ===\n'
    printf 'All verdicts below are asserted from stub rows only. No live target is contacted.\n'

    case "$dir" in
        /tmp/opencode/*) ;;
        *)
            printf 'refusing self-test dir outside /tmp/opencode: %s\n' "$dir" >&2
            return 2
            ;;
    esac

    rm -rf -- "$dir"
    if ! mkdir -p "$dir" || ! chmod 700 "$dir"; then
        printf 'cannot create self-test dir %s\n' "$dir" >&2
        return 2
    fi

    # Force offline: remove every opt-in, credential and binary override.
    unset UAT_TARGET_HOST UAT_BOTSERVER UAT_LOGIN_USER UAT_LOGIN_PASSWORD UAT_SOURCE_ROOT
    export UAT_RESULTS_DIR="$dir"

    # 1. Live gate: without the opt-in a live step records UNVERIFIED (never
    #    PASS) and exits 3.
    selftest_run env -u UAT_TARGET_HOST bash "$tests_dir/step_01_list.sh"
    selftest_expect_code "live step exits 3 offline" 3
    selftest_expect_text "live step prints UNVERIFIED" "UNVERIFIED"
    if awk -F '\t' '$1 == "list" && $2 == "UNVERIFIED" { f = 1 } END { exit !f }' "$results"; then
        printf 'ok   offline gate recorded UNVERIFIED row (not PASS)\n'
    else
        selftest_fail "offline gate did not record an UNVERIFIED row for 'list'"
    fi

    # 2. Report with no results at all: UNVERIFIED offline.
    rm -f "$results"
    selftest_run bash "$report"
    selftest_expect_code "report with no evidence exits 3" 3
    selftest_expect_text "report says UNVERIFIED: no results recorded" \
        "UNVERIFIED: no results recorded"

    # 3. One bad row forces FAIL even when all other evidence is present.
    selftest_full_evidence > "$results" || { selftest_fail "cannot build full stub evidence"; return 2; }
    printf 'health:vault\tFAIL\tselftest injected failure\n' >> "$results"
    selftest_run bash "$report"
    selftest_expect_code "report with a bad step exits 1" 1
    selftest_expect_text "report says verdict: FAIL" "verdict: FAIL"
    selftest_expect_text "report names the failed step" "health:vault"

    # 4. Full evidence is the only way to PASS.
    selftest_full_evidence > "$results" || { selftest_fail "cannot build full stub evidence"; return 2; }
    selftest_run bash "$report"
    selftest_expect_code "report with full stub evidence exits 0" 0
    selftest_expect_text "report says verdict: PASS" "verdict: PASS"

    # 5. One missing required row -> UNVERIFIED, not PASS.
    selftest_full_evidence | awk -F '\t' '$1 != "status:vault"' > "$results"
    selftest_run bash "$report"
    selftest_expect_code "report with a missing row exits 3" 3
    selftest_expect_text "report names the missing evidence" "status:vault"

    # 6. A non-PASS row is not evidence (the PASS status is required).
    selftest_full_evidence | awk -F '\t' -v OFS='\t' \
        '$1 == "health:cache" { $2 = "UNVERIFIED" } { print }' > "$results"
    selftest_run bash "$report"
    selftest_expect_code "non-PASS row keeps verdict UNVERIFIED" 3
    selftest_expect_text "report names the non-PASS evidence" "health:cache"

    rm -rf -- "$dir"

    if [ "$SELFTEST_FAILURES" -eq 0 ]; then
        printf '=== SELF-TEST OK: all verdict assertions held on stub evidence ===\n'
        printf 'This is NOT a real host PASS. No live target was contacted.\n'
        return 0
    fi
    printf '=== SELF-TEST FAILED: %s assertion(s) did not hold ===\n' "$SELFTEST_FAILURES" >&2
    return 1
}

case "${1:-help}" in
    help|-h|--help)
        usage
        ;;
    --self-test|selftest)
        selftest
        ;;
    steps|--steps)
        list_steps
        ;;
    plan|--plan)
        print_plan
        ;;
    preflight)
        exec "$tests_dir/step_00_preflight.sh"
        ;;
    report)
        exec "$tests_dir/step_99_report.sh"
        ;;
    step)
        id="${2:-}"
        if [ -z "$id" ]; then
            printf 'missing step id\n' >&2
            usage >&2
            exit 2
        fi
        shift 2
        file="$(step_file "$id")" || {
            printf 'unknown step id: %s\n' "$id" >&2
            list_steps >&2
            exit 2
        }
        exec "$file" "$@"
        ;;
    *)
        printf 'unknown command: %s\n' "$1" >&2
        usage >&2
        exit 2
        ;;
esac
