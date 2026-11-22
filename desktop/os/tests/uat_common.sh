#!/usr/bin/env bash
# Shared helpers for the #2582 install UAT steps. Source this file; never run it
# directly. Every step records one result line under UAT_RESULTS_DIR.
#
# Live results (PASS/FAIL) are only possible when UAT_TARGET_HOST=1, `incus` is
# available and a botserver binary under test is supplied. Without that opt-in
# every live step records UNVERIFIED and exits 3 — no step can claim PASS
# offline. Credentials are read from the environment; none are stored here.

UAT_COMMON_VERSION=1

uat_step_banner() {
    printf '\n=== %s ===\n' "$1"
}

uat_results_dir() {
    printf '%s\n' "${UAT_RESULTS_DIR:-/tmp/opencode/uat-install}"
}

uat_init_results() {
    local dir
    dir="$(uat_results_dir)"
    if ! mkdir -p "$dir"; then
        return 2
    fi
    if ! chmod 700 "$dir"; then
        return 2
    fi
    printf '%s\n' "$dir"
}

# uat_record <step-id> <PASS|FAIL|UNVERIFIED|STATIC> <single-line-detail>
# Replaces any earlier row for the same step id and prints the row it stored.
uat_record() {
    local id="$1" status="$2" detail="$3" dir file tmp
    dir="$(uat_init_results)" || return 2
    file="$dir/results.tsv"
    tmp="$dir/.results.tsv.$$"
    : > "$tmp" || return 2
    if [ -f "$file" ]; then
        awk -F '\t' -v id="$id" '$1 != id' "$file" > "$tmp" || return 2
    fi
    detail="$(printf '%s' "$detail" | tr '\t' ' ' | tr '\n' ' ')"
    printf '%s\t%s\t%s\n' "$id" "$status" "$detail" >> "$tmp" || return 2
    mv -f "$tmp" "$file" || return 2
    printf '%s  %s  %s\n' "$status" "$id" "$detail"
}

uat_botserver_bin() {
    if [ -n "${UAT_BOTSERVER:-}" ] && [ -x "${UAT_BOTSERVER}" ]; then
        printf '%s\n' "${UAT_BOTSERVER}"
        return 0
    fi
    if command -v botserver >/dev/null 2>&1; then
        command -v botserver
        return 0
    fi
    return 1
}

uat_botserver() {
    local bin
    bin="$(uat_botserver_bin)" || return 1
    "$bin" "$@"
}

# The gate every live step must pass. Records UNVERIFIED and returns 3 when the
# environment is not an opted-in target host. It never returns 0 offline.
uat_live_gate() {
    local id="$1" reason=""
    if [ "${UAT_TARGET_HOST:-0}" != "1" ]; then
        reason="UAT_TARGET_HOST is not 1; offline/dev session cannot claim a live result"
    elif ! command -v incus >/dev/null 2>&1; then
        reason="incus CLI not found on PATH (Incus is the only container layer)"
    elif ! uat_botserver_bin >/dev/null 2>&1; then
        reason="botserver binary under test not found (set UAT_BOTSERVER)"
    fi
    if [ -n "$reason" ]; then
        uat_record "$id" UNVERIFIED "$reason"
        printf 'UNVERIFIED: %s\n' "$reason" >&2
        return 3
    fi
    return 0
}

# Live registry from the binary under test, container mode (one component list).
uat_component_list() {
    uat_botserver list --container
}

uat_tenant() {
    printf '%s\n' "${UAT_TENANT:-default}"
}

# HTTP status code only. No body, header or credential is ever printed.
uat_http_status() {
    local url="$1" method="${2:-GET}" code
    code="$(curl -sS -o /dev/null -w '%{http_code}' --connect-timeout 3 -m 15 \
        -X "$method" "$url" 2>/dev/null)" || return 1
    printf '%s\n' "$code"
}

# POST a JSON body supplied on stdin (never in argv) and print the status code.
uat_http_json_post_status() {
    local url="$1" body="$2" out="$3" code
    code="$(printf '%s' "$body" | curl -sS -o "$out" -w '%{http_code}' \
        --connect-timeout 3 -m 20 -X POST -H 'Content-Type: application/json' \
        --data-binary @- "$url" 2>/dev/null)" || return 1
    printf '%s\n' "$code"
}

# True when a step id has a PASS row in the accumulated results.
uat_has_pass() {
    local id="$1" results
    results="$(uat_results_dir)/results.tsv"
    [ -f "$results" ] || return 1
    awk -F '\t' -v id="$id" '$1 == id && $2 == "PASS" { found = 1 } END { exit !found }' \
        "$results"
}
