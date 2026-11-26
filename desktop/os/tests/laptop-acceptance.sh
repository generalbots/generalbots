#!/usr/bin/env bash
# Laptop/shell acceptance harness (#2471) — OFFLINE portion.
#
# Deterministic, read-only, no secrets, no network, no cargo, no server restart.
# It runs the three checks that can be proven from the checked-out tree —
#   (1) QEMU smoke invocation shape in the image workflow / build-image.sh,
#   (2) kiosk shell, tray, power-menu and lock marker presence in the recipe,
#   (3) disk (image) / RAM (smoke) budget assertions,
# plus gate wiring (PR paths + release-blocking dependency), then reports every
# runtime scenario and every physical hardware-matrix case as SKIP with a
# reason. SKIP is never reported as PASS: real hardware, a booted ISO and
# screenshots/serial logs are required for those, and are out of scope offline.
#
# Usage:  bash desktop/os/tests/laptop-acceptance.sh
# Exit:   0 when no check FAILed (SKIPs are informational in offline mode);
#         1 on any FAIL, or on any SKIP when GB_LAPTOP_REQUIRE_HARDWARE=1.
# Results (TSV) are written under UAT_RESULTS_DIR (default /tmp/opencode/uat-laptop).
set -uo pipefail

here="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$here/../../.." && pwd)"

workflow="$repo_root/.github/workflows/desktop-os-iso.yml"
matrix="$here/hardware-matrix.psv"
mkosi_conf="$repo_root/desktop/os/mkosi.conf"
build_image="$repo_root/desktop/os/build-image.sh"
readme="$repo_root/desktop/os/README.md"
finalize="$repo_root/desktop/os/mkosi.finalize.chroot"
shell_unit="$repo_root/desktop/os/mkosi.extra/usr/lib/systemd/system/generalbots-shell.service"
lock_unit="$repo_root/desktop/os/mkosi.extra/usr/lib/systemd/system/generalbots-lock-monitor.service"
tray_js="$repo_root/botui/ui/suite/desktop-inline/08_os_tray.js"
power_js="$repo_root/botui/ui/suite/js/power-menu.js"
lock_js="$repo_root/botui/ui/suite/js/lock-screen.js"
qemu_profile="$here/qemu-laptop-profile.sh"
shell_uat="$here/shell-uat.sh"

results_dir="${UAT_RESULTS_DIR:-/tmp/opencode/uat-laptop}"
results="$results_dir/results.tsv"
strict=0
[ "${GB_LAPTOP_REQUIRE_HARDWARE:-0}" = "1" ] && strict=1

mkdir -p -- "$results_dir" || exit 1
chmod 700 -- "$results_dir" 2>/dev/null || true
: > "$results" || exit 1

pass_count=0
fail_count=0
skip_count=0

# record <case-id> <PASS|FAIL|SKIP> <reason> — one row, tabs/newlines flattened.
record() {
    local id="$1" status="$2" reason="$3" flat
    flat="$(printf '%s' "$reason" | tr '\t' ' ' | tr '\n' ' ')"
    printf '%s\t%s\t%s\n' "$id" "$status" "$flat" >> "$results"
    printf '%-24s %-5s %s\n' "$id" "$status" "$flat"
    case "$status" in
        PASS) pass_count=$((pass_count + 1)) ;;
        FAIL) fail_count=$((fail_count + 1)) ;;
        SKIP) skip_count=$((skip_count + 1)) ;;
    esac
}

# require_token <file> <fixed-string> <label> — appends to the global $missing.
missing=""
require_token() {
    local file="$1" token="$2" label="$3"
    if [ ! -f "$file" ]; then
        missing="$missing [$label:missing-file]"
    elif ! grep -Fq -- "$token" "$file"; then
        missing="$missing [$label:missing-token]"
    fi
}

qemu_tokens=(
    '-acpitable file='
    'acpi-battery'
    'mac80211_hwsim'
    '-device virtio-net-pci'
    '-device intel-hda'
    '-device hda-duplex'
    '-device virtio-gpu-pci,max_outputs=2'
    'second head'
    '-net none'
    'if=pflash'
    'OVMF_VARS'
    '-serial "file:'
)

shell_uat_tokens=(
    'scenario_shell_visible()'
    'scenario_chat_roundtrip()'
    'scenario_tray_indicators_present()'
    'scenario_power_menu_opens()'
    'scenario_lock_requires_credential_test()'
    'scenario_app_launch()'
    'assert_app_launch()'
    'xdotool mousemove'
    'xdotool click'
    'xdotool type'
    'xdotool key'
    'import -window root'
    'scrot'
)

# --- Static check 1: QEMU smoke invocation shape -----------------------------
check_smoke_shape() {
    missing=""
    require_token "$workflow" 'mkosi --output-dir /out vm' workflow/qemu-vm
    require_token "$workflow" 'timeout 480' workflow/timeout
    require_token "$workflow" 'Started General Bots kiosk shell' workflow/marker
    require_token "$workflow" 'vm-smoke.log' workflow/log-artifact
    require_token "$workflow" 'REQUIRED boot smoke' workflow/required-gate
    require_token "$build_image" '--firmware=uefi-secure-boot' build-image/ovmf
    require_token "$build_image" '--firmware-variables=microsoft-mok' build-image/mok
    require_token "$build_image" 'smoke requires --mode release' build-image/signed-only
    if [ -n "$missing" ]; then
        printf 'smoke gating tokens absent:%s' "$missing"
        return 1
    fi
    printf 'mkosi vm smoke is timed, logged, marker-asserted and release-blocking; build-image --smoke uses OVMF Secure Boot'
}

# --- Static check 2: shell / kiosk marker presence ----------------------------
check_shell_markers() {
    missing=""
    require_token "$shell_unit" 'Description=General Bots kiosk shell' shell/description
    require_token "$shell_unit" 'ExecStart=/usr/local/bin/gb-session' shell/exec
    require_token "$shell_unit" 'Restart=always' shell/respawn
    require_token "$lock_unit" 'logind lock/resume bridge' shell/lock-bridge
    require_token "$finalize" 'systemctl mask getty@tty1.service' shell/no-getty
    require_token "$finalize" 'systemctl mask rescue.service emergency.service' shell/no-rescue
    require_token "$tray_js" 'id: "trayBattery"' tray/battery
    require_token "$tray_js" 'id: "trayNetworkStatus"' tray/network
    require_token "$tray_js" 'id: "trayPower"' tray/power
    require_token "$power_js" 'power-menu-confirm-' shell/power-menu
    require_token "$lock_js" 'gbLockPassword' shell/lock-credential
    if [ -n "$missing" ]; then
        printf 'kiosk/shell markers absent:%s' "$missing"
        return 1
    fi
    printf 'kiosk unit, respawn, lock bridge, no-getty/no-rescue, 3 tray indicators, power menu and credential lock markers present'
}

# --- Static check 3: disk / RAM budget assertions -----------------------------
check_budgets() {
    local size smoke_ram
    missing=""
    size="$(awk -F= '/^OutputSize=/{gsub(/[Gg]/,"",$2); print $2; exit}' "$mkosi_conf")"
    smoke_ram="$(awk 'match($0, /-m [0-9]+G/){print substr($0, RSTART+3, RLENGTH-4); exit}' "$readme")"
    if ! [[ "$size" =~ ^[0-9]+$ ]]; then
        printf 'mkosi.conf OutputSize is not a GiB integer (got %s)' "${size:-<empty>}"
        return 1
    fi
    if ! [[ "$smoke_ram" =~ ^[0-9]+$ ]]; then
        printf 'README QEMU smoke RAM floor (-m NG) not documented (got %s)' "${smoke_ram:-<empty>}"
        return 1
    fi
    if [ "$size" -lt "$smoke_ram" ]; then
        printf 'image budget %s GiB is below smoke RAM floor %s GiB' "$size" "$smoke_ram"
        return 1
    fi
    if [ "$size" -gt 64 ]; then
        printf 'image budget %s GiB exceeds the 64 GiB target-medium ceiling' "$size"
        return 1
    fi
    require_token "$readme" 'OutputSize=16G' docs/disk-budget
    if [ -n "$missing" ]; then
        printf 'budget documentation out of sync:%s' "$missing"
        return 1
    fi
    printf 'image budget %s GiB >= smoke RAM floor %s GiB, both documented; ceiling 64 GiB respected' "$size" "$smoke_ram"
}

# --- Static check 4: gate wiring (PR paths + release dependency) --------------
check_gate_wiring() {
    missing=""
    require_token "$workflow" 'pull_request:' workflow/pr-trigger
    require_token "$workflow" 'desktop/os/**' workflow/pr-path-os
    require_token "$workflow" 'botserver/crates/botsettings/**' workflow/pr-path-botsettings
    require_token "$workflow" 'laptop-acceptance-offline:' workflow/gate-job
    require_token "$workflow" 'bash desktop/os/tests/laptop-acceptance.sh' workflow/gate-command
    require_token "$workflow" 'needs: laptop-acceptance-offline' workflow/release-blocking
    if [ -n "$missing" ]; then
        printf 'acceptance gate wiring absent:%s' "$missing"
        return 1
    fi
    printf 'gate runs on PRs touching OS/shell/botsettings and blocks the release image job'
}

# --- Static check 5: scripted laptop QEMU profile ----------------------------
check_qemu_profile() {
    local out rc tok
    if [ ! -f "$qemu_profile" ]; then
        printf 'laptop QEMU profile script missing: %s' "$qemu_profile"
        return 1
    fi
    if ! bash -n "$qemu_profile" >/dev/null 2>&1; then
        printf 'laptop QEMU profile script is not bash -n clean'
        return 1
    fi
    out="$(bash "$qemu_profile" --check 2>&1)"
    rc=$?
    if [ "$rc" -ne 0 ] || ! grep -Fq -- 'qemu-laptop-profile --check: PASS' <<< "$out"; then
        printf 'laptop QEMU profile --check did not report PASS: %s' "$(printf '%s' "$out" | tail -n 2)"
        return 1
    fi
    missing=""
    for tok in "${qemu_tokens[@]}"; do
        if ! grep -Fq -- "$tok" "$qemu_profile"; then missing="$missing [$tok]"; fi
    done
    if [ -n "$missing" ]; then
        printf 'laptop QEMU profile device/option tokens absent:%s' "$missing"
        return 1
    fi
    printf 'laptop QEMU profile present, bash -n clean, --check PASS; %d device/option tokens verified' "${#qemu_tokens[@]}"
}

# --- Static check 6: scripted shell UAT (mouse+keyboard only) ----------------
check_shell_uat() {
    local out rc tok
    if [ ! -f "$shell_uat" ]; then
        printf 'scripted shell UAT script missing: %s' "$shell_uat"
        return 1
    fi
    if ! bash -n "$shell_uat" >/dev/null 2>&1; then
        printf 'scripted shell UAT script is not bash -n clean'
        return 1
    fi
    out="$(bash "$shell_uat" --check 2>&1)"
    rc=$?
    if [ "$rc" -ne 0 ] || ! grep -Fq -- 'shell-uat --check: PASS' <<< "$out"; then
        printf 'scripted shell UAT --check did not report PASS: %s' "$(printf '%s' "$out" | tail -n 2)"
        return 1
    fi
    missing=""
    for tok in "${shell_uat_tokens[@]}"; do
        if ! grep -Fq -- "$tok" "$shell_uat"; then missing="$missing [$tok]"; fi
    done
    if [ -n "$missing" ]; then
        printf 'scripted shell UAT scenario/input tokens absent:%s' "$missing"
        return 1
    fi
    printf 'scripted shell UAT present, bash -n clean, --check PASS; %d scenario/input tokens verified' "${#shell_uat_tokens[@]}"
}

run_static() {
    local id="$1" title="$2" fn="$3" reason rc
    reason="$("$fn")"
    rc=$?
    if [ "$rc" -eq 0 ]; then
        record "$id" PASS "$title: $reason"
    else
        record "$id" FAIL "$title: $reason"
    fi
}

printf 'Laptop/shell acceptance harness (#2471) — offline portion\n'
printf 'repo: %s\nresults: %s\nstrict(hardware-required): %s\n\n' "$repo_root" "$results" "$strict"
printf '%-24s %-5s %s\n' CASE STATUS REASON

run_static GATE_SMOKE_SHAPE 'QEMU smoke invocation shape' check_smoke_shape
run_static GATE_SHELL_MARKERS 'Shell/kiosk marker presence' check_shell_markers
run_static GATE_BUDGETS 'Disk/RAM budget assertions' check_budgets
run_static GATE_WIRING 'CI gate wiring' check_gate_wiring
run_static GATE_QEMU_PROFILE 'Scripted laptop QEMU profile' check_qemu_profile
run_static GATE_SHELL_UAT 'Scripted mouse+keyboard shell UAT' check_shell_uat

# --- Runtime scenarios: SKIP offline, with the precise missing evidence -------
while IFS='|' read -r id title reason; do
    case "$id" in ''|'#'*|id) continue ;; esac
    record "$id" SKIP "$title — $reason"
done <<'SCENARIOS'
SCENARIO_QEMU_PROFILE|Laptop QEMU profile (fake battery, mac80211_hwsim, audio, second head)|scripted profile harness PRESENT and statically checked; runtime-unverified — needs a booted VM with those devices and a serial log
SCENARIO_SHELL_VISIBLE|Shell visible in booted image|scripted assertion PRESENT in shell-uat.sh; runtime-unverified — needs a booted ISO and a screenshot
SCENARIO_CHAT_ROUNDTRIP|Chat message round-trip with mouse+keyboard|scripted assertion PRESENT in shell-uat.sh; runtime-unverified — needs a booted ISO, provisioned stack and a screenshot
SCENARIO_TRAY_INDICATORS|Tray indicators present and live|scripted assertion PRESENT in shell-uat.sh; runtime-unverified — needs a booted shell and a screenshot
SCENARIO_POWER_MENU|Power menu opens (no instant poweroff)|scripted assertion PRESENT in shell-uat.sh; runtime-unverified — needs a booted shell with input injection
SCENARIO_LOCK_CREDENTIAL|Lock requires the directory credential|scripted assertion PRESENT in shell-uat.sh; runtime-unverified — needs a booted shell and a credential supplied via env
SCENARIO_NO_TERMINAL|Every scenario done only with mouse+keyboard|scripted xdotool harness PRESENT; runtime-unverified — needs a real shell session with input injection
SCENARIO_APP_LAUNCH|At least one app launch asserted after scripted input|scripted app-launch assertion PRESENT in shell-uat.sh; runtime-unverified — needs a booted shell
SCENARIO_EVIDENCE|Serial log + screenshots on every OS-shell issue closure|screenshot capture scripted; runtime-unverified — needs a real closure with attached artifacts
SCENARIOS

# --- Physical hardware matrix: every case stays SKIP offline ------------------
if [ ! -f "$matrix" ]; then
    record MATRIX_DATA FAIL "hardware matrix data file missing: $matrix"
else
    while IFS='|' read -r case_id class cpu ram firmware gpu wifi ext display input model fw evidence status; do
        case "$case_id" in ''|'#'*|case_id) continue ;; esac
        record "$case_id" SKIP "$class ($model, $fw, ${ram} GiB, $gpu, $wifi, ext-display=$ext, $input): physical machine required; attach $evidence to the issue (matrix status=$status)"
    done < "$matrix"
fi

printf '\nSummary: PASS=%d FAIL=%d SKIP=%d (strict=%d)\n' "$pass_count" "$fail_count" "$skip_count" "$strict"
if [ "$fail_count" -gt 0 ]; then
    printf 'verdict: FAIL — offline gate blocks publication\n'
    exit 1
fi
if [ "$strict" -eq 1 ] && [ "$skip_count" -gt 0 ]; then
    printf 'verdict: FAIL — GB_LAPTOP_REQUIRE_HARDWARE=1 and %d case(s) lack hardware evidence\n' "$skip_count"
    exit 1
fi
printf 'verdict: PASS (offline) — runtime and hardware cases are SKIP, not PASS; real hardware evidence still required\n'
exit 0
