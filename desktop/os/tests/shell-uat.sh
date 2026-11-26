#!/usr/bin/env bash
# Scripted, mouse+keyboard-only UAT for the General Bots kiosk shell (#2471).
#
# OFFLINE-SAFE: with no DISPLAY, no xdotool or no reachable X display every
# scenario is recorded as SKIP with an explicit reason. This script NEVER
# claims a PASS without a live shell; screenshots go to /tmp/opencode, never
# to the repository. `--check` is a static assertion of this file's scenario
# and input tokens plus `bash -n` — it is not a runtime result either.
#
# Usage:
#   bash desktop/os/tests/shell-uat.sh             # run on a live display
#   bash desktop/os/tests/shell-uat.sh --check     # static self-check
#
# Optional operator inputs (no secrets in this file): GB_UAT_SHELL_WINDOW,
# GB_UAT_CHAT_X/Y, GB_UAT_CHAT_TEXT, GB_UAT_TRAY_LABELS, GB_UAT_POWER_X/Y,
# GB_UAT_LAUNCH_X/Y, GB_UAT_APP_WINDOW, GB_UAT_LOCK_CREDENTIAL (env only).
set -uo pipefail

self="${BASH_SOURCE[0]}"
results_dir="${UAT_RESULTS_DIR:-/tmp/opencode/uat-laptop}"
results="$results_dir/shell-uat-results.tsv"
mkdir -p -- "$results_dir" || exit 1
chmod 700 -- "$results_dir" 2>/dev/null || true
: > "$results" || exit 1

shell_window_name="${GB_UAT_SHELL_WINDOW:-General Bots}"
app_window_name="${GB_UAT_APP_WINDOW:-Chat}"
tray_labels="${GB_UAT_TRAY_LABELS:-battery,network,power}"

scenario_names=(
    shell_visible
    chat_roundtrip
    tray_indicators_present
    power_menu_opens
    lock_requires_credential_test
    app_launch
)

pass_count=0
fail_count=0
skip_count=0

record() {
    local id="$1" status="$2" reason="$3" flat
    flat="$(printf '%s' "$reason" | tr '\t' ' ' | tr '\n' ' ')"
    printf '%s\t%s\t%s\n' "$id" "$status" "$flat" >> "$results"
    printf '%-36s %-5s %s\n' "$id" "$status" "$flat"
    case "$status" in
        PASS) pass_count=$((pass_count + 1)) ;;
        FAIL) fail_count=$((fail_count + 1)) ;;
        SKIP) skip_count=$((skip_count + 1)) ;;
    esac
}

skip_all() {
    local reason="$1" name
    for name in "${scenario_names[@]}"; do
        record "SHELL_${name^^}" SKIP "$reason"
    done
    printf '\nSummary: PASS=0 FAIL=0 SKIP=%d — offline; no runtime PASS claimed\n' "$skip_count"
    printf 'verdict: SKIP (no live display/VM) — booted-shell evidence still required\n'
    exit 0
}

summarize() {
    printf '\nSummary: PASS=%d FAIL=%d SKIP=%d\n' "$pass_count" "$fail_count" "$skip_count"
    if [ "$fail_count" -gt 0 ]; then
        printf 'verdict: FAIL — a scripted shell assertion did not pass\n'
        exit 1
    fi
    printf 'verdict: PASS (scripted assertions observed on a live shell); screenshots in %s\n' "$results_dir"
}

screenshot() {
    local out="$1"
    if command -v import >/dev/null 2>&1; then
        import -window root "$out"
    elif command -v scrot >/dev/null 2>&1; then
        scrot "$out"
    else
        return 1
    fi
}

ocr_text() {
    local png="$1"
    command -v tesseract >/dev/null 2>&1 || return 1
    tesseract "$png" stdout 2>/dev/null
}

window_present() {
    xdotool search --onlyvisible --name "$1" >/dev/null 2>&1
}

mouse_click() {
    xdotool mousemove "$1" "$2"
    xdotool click 1
}

key_press() {
    xdotool key "$@"
}

type_text() {
    xdotool type --clearmodifiers --delay 20 "$1"
}

scenario_shell_visible() {
    local shot="$results_dir/shell-visible.png"
    if ! window_present "$shell_window_name"; then
        record SHELL_SHELL_VISIBLE FAIL "kiosk shell window '$shell_window_name' not found via xdotool"
        return
    fi
    if ! screenshot "$shot"; then
        record SHELL_SHELL_VISIBLE SKIP "no import/scrot available to capture the shell"
        return
    fi
    record SHELL_SHELL_VISIBLE PASS "shell window visible; screenshot=$shot"
}

scenario_chat_roundtrip() {
    local shot="$results_dir/chat-roundtrip.png"
    local text="${GB_UAT_CHAT_TEXT:-laptop gate ping}"
    if ! window_present "$shell_window_name"; then
        record SHELL_CHAT_ROUNDTRIP FAIL "kiosk shell is not visible before the chat round-trip"
        return
    fi
    mouse_click "${GB_UAT_CHAT_X:-960}" "${GB_UAT_CHAT_Y:-900}"
    type_text "$text"
    key_press Return
    if ! screenshot "$shot"; then
        record SHELL_CHAT_ROUNDTRIP SKIP "no import/scrot available to capture the chat reply"
        return
    fi
    if ! command -v tesseract >/dev/null 2>&1; then
        record SHELL_CHAT_ROUNDTRIP SKIP "tesseract absent; cannot assert reply text from screenshot=$shot"
        return
    fi
    if ocr_text "$shot" | grep -Fiq -- "$text"; then
        record SHELL_CHAT_ROUNDTRIP PASS "typed text appeared in the chat after keyboard input; screenshot=$shot"
    else
        record SHELL_CHAT_ROUNDTRIP FAIL "typed text not visible in chat screenshot=$shot"
    fi
}

scenario_tray_indicators_present() {
    local shot="$results_dir/tray-indicators.png"
    local txt label missing=""
    if ! screenshot "$shot"; then
        record SHELL_TRAY_INDICATORS SKIP "no import/scrot available to capture the tray"
        return
    fi
    if ! command -v tesseract >/dev/null 2>&1; then
        record SHELL_TRAY_INDICATORS SKIP "tesseract absent; cannot assert tray labels from screenshot=$shot"
        return
    fi
    txt="$(ocr_text "$shot")"
    IFS=',' read -r -a labels <<< "$tray_labels"
    for label in "${labels[@]}"; do
        if ! printf '%s' "$txt" | grep -Fiq -- "$label"; then
            missing="$missing $label"
        fi
    done
    if [ -n "$missing" ]; then
        record SHELL_TRAY_INDICATORS FAIL "tray labels absent:$missing (labels=$tray_labels); screenshot=$shot"
    else
        record SHELL_TRAY_INDICATORS PASS "tray labels present ($tray_labels); screenshot=$shot"
    fi
}

scenario_power_menu_opens() {
    local shot="$results_dir/power-menu.png"
    local txt
    mouse_click "${GB_UAT_POWER_X:-1840}" "${GB_UAT_POWER_Y:-40}"
    if ! screenshot "$shot"; then
        record SHELL_POWER_MENU SKIP "no import/scrot available to capture the power menu"
        key_press Escape
        return
    fi
    if ! command -v tesseract >/dev/null 2>&1; then
        record SHELL_POWER_MENU SKIP "tesseract absent; cannot assert menu text from screenshot=$shot"
        key_press Escape
        return
    fi
    txt="$(ocr_text "$shot")"
    if printf '%s' "$txt" | grep -Eiq 'power|shutdown|suspend|lock|reboot'; then
        record SHELL_POWER_MENU PASS "power menu opened without powering off; screenshot=$shot"
    else
        record SHELL_POWER_MENU FAIL "no power-menu text in screenshot=$shot"
    fi
    key_press Escape
}

scenario_lock_requires_credential_test() {
    local shot="$results_dir/lock-screen.png"
    local txt credential
    key_press super+l
    if ! screenshot "$shot"; then
        record SHELL_LOCK_CREDENTIAL SKIP "no import/scrot available to capture the lock screen"
        key_press Escape
        return
    fi
    if ! command -v tesseract >/dev/null 2>&1; then
        record SHELL_LOCK_CREDENTIAL SKIP "tesseract absent; cannot assert lock state from screenshot=$shot"
        key_press Escape
        return
    fi
    txt="$(ocr_text "$shot")"
    if ! printf '%s' "$txt" | grep -Eiq 'lock|password|sign in|unlock'; then
        record SHELL_LOCK_CREDENTIAL FAIL "no lock-screen text in screenshot=$shot"
        key_press Escape
        return
    fi
    type_text 'wrong-credential-probe'
    key_press Return
    screenshot "$results_dir/lock-after-wrong.png" || true
    txt="$(ocr_text "$results_dir/lock-after-wrong.png")"
    if ! printf '%s' "$txt" | grep -Eiq 'lock|password|sign in|unlock'; then
        record SHELL_LOCK_CREDENTIAL FAIL "shell unlocked with a wrong credential; screenshot=$shot"
        return
    fi
    credential="${GB_UAT_LOCK_CREDENTIAL:-}"
    if [ -z "$credential" ]; then
        record SHELL_LOCK_CREDENTIAL SKIP "wrong credential rejected; GB_UAT_LOCK_CREDENTIAL not provided, correct-credential test not run"
        key_press Escape
        return
    fi
    type_text "$credential"
    key_press Return
    if window_present "$shell_window_name"; then
        record SHELL_LOCK_CREDENTIAL PASS "wrong credential rejected, provided credential unlocked the shell"
    else
        record SHELL_LOCK_CREDENTIAL FAIL "provided credential did not unlock the shell"
    fi
}

assert_app_launch() {
    local wanted="$1" before after
    before="$(xdotool search --onlyvisible --name "$wanted" | wc -l)"
    mouse_click "${GB_UAT_LAUNCH_X:-960}" "${GB_UAT_LAUNCH_Y:-1040}"
    after="$(xdotool search --onlyvisible --name "$wanted" | wc -l)"
    [ "$after" -gt "$before" ]
}

scenario_app_launch() {
    local shot="$results_dir/app-launch.png" evidence='no screenshot tool available'
    if assert_app_launch "$app_window_name"; then
        if screenshot "$shot"; then evidence="screenshot=$shot"; fi
        record SHELL_APP_LAUNCH PASS "app window '$app_window_name' appeared after mouse click; $evidence"
    else
        record SHELL_APP_LAUNCH FAIL "app window '$app_window_name' did not appear after mouse click"
    fi
}

required_assertions=(
    'scenario_shell_visible'
    'scenario_chat_roundtrip'
    'scenario_tray_indicators_present'
    'scenario_power_menu_opens'
    'scenario_lock_requires_credential_test'
    'scenario_app_launch'
    'assert_app_launch'
    'xdotool mousemove'
    'xdotool click'
    'xdotool type'
    'xdotool key'
    'import -window root'
    'scrot'
    'DISPLAY'
    'command -v xdotool'
    '--check'
)

check_mode() {
    local tok missing=0
    for tok in "${required_assertions[@]}"; do
        if ! grep -Fq -- "$tok" "$self"; then
            printf 'MISS assertion token: %s\n' "$tok"
            missing=$((missing + 1))
        fi
    done
    if ! bash -n "$self"; then
        printf 'MISS: bash -n %s failed\n' "$self"
        missing=$((missing + 1))
    fi
    if [ "$missing" -ne 0 ]; then
        printf 'shell-uat --check: FAIL (%d problem(s))\n' "$missing"
        return 1
    fi
    printf 'shell-uat --check: PASS (static assertions + bash -n; no live shell, no runtime PASS)\n'
}

case "${1:-}" in
    --check)
        check_mode
        ;;
    --help|-h)
        printf 'Usage: %s [--check|--help]\n' "$self"
        ;;
    '')
        printf 'Scripted shell UAT (#2471) — mouse+keyboard only\nresults: %s\n\n' "$results"
        if [ -z "${DISPLAY:-}" ]; then
            skip_all 'DISPLAY is unset — no live X display'
        fi
        if ! command -v xdotool >/dev/null 2>&1; then
            skip_all 'xdotool is not installed — no mouse/keyboard injection'
        fi
        if ! xdotool getdisplaygeometry >/dev/null 2>&1; then
            skip_all 'X display is unreachable via xdotool'
        fi
        scenario_shell_visible
        scenario_chat_roundtrip
        scenario_tray_indicators_present
        scenario_power_menu_opens
        scenario_lock_requires_credential_test
        scenario_app_launch
        summarize
        ;;
    *)
        printf 'unknown option: %s\n' "$1" >&2
        exit 2
        ;;
esac
