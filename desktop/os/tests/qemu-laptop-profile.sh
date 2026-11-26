#!/usr/bin/env bash
# Laptop QEMU profile builder (#2471) — OFFLINE, read-only, no downloads.
#
# Emits a reusable QEMU command line for a laptop-shaped VM: fake ACPI battery,
# simulated Wi-Fi (guest mac80211_hwsim + virtio network device), audio, a
# second head, OVMF Secure Boot variables and screenshot/serial-log paths.
# It NEVER executes QEMU and NEVER reports a hardware PASS. `--check` is a
# static assertion of the emitted tokens plus `bash -n` on this file.
#
# Usage:
#   bash desktop/os/tests/qemu-laptop-profile.sh --print      # emit default profile
#   bash desktop/os/tests/qemu-laptop-profile.sh --net-none   # air-gapped variant
#   bash desktop/os/tests/qemu-laptop-profile.sh --check      # static self-check
#
# Operator-supplied assets (no download is performed here): GB_OVMF_CODE,
# GB_OVMF_VARS, GB_LAPTOP_BATTERY_SSDT, GB_LAPTOP_IMAGE, GB_LAPTOP_SERIAL_LOG,
# GB_LAPTOP_SCREENSHOT. All default to UAT_RESULTS_DIR under /tmp/opencode.
set -uo pipefail

self="${BASH_SOURCE[0]}"
results_dir="${UAT_RESULTS_DIR:-/tmp/opencode/uat-laptop}"

ovmf_code="${GB_OVMF_CODE:-/usr/share/OVMF/OVMF_CODE.secboot.fd}"
ovmf_vars="${GB_OVMF_VARS:-$results_dir/OVMF_VARS.fd}"
battery_ssdt="${GB_LAPTOP_BATTERY_SSDT:-$results_dir/battery-ssdt.aml}"
disk_image="${GB_LAPTOP_IMAGE:-$results_dir/generalbots-os.raw}"
serial_log="${GB_LAPTOP_SERIAL_LOG:-$results_dir/qemu-laptop-serial.log}"
screenshot="${GB_LAPTOP_SCREENSHOT:-$results_dir/qemu-laptop.png}"

# Guest setup hints are instructions, not results. They are emitted next to the
# command so a later VM run knows what to load; nothing here claims a PASS.
battery_guest_hint="guest: expose BAT0 via an ACPI battery SSDT (-acpitable) or load a documented xvfb battery shim"
battery_device_note="-device acpi-battery is not an upstream QEMU device; use -acpitable file=$battery_ssdt instead"
wifi_guest_hint="guest: modprobe mac80211_hwsim to simulate Wi-Fi; the host side is a virtio-net-pci device"
head_hint="guest: virtio-gpu,max_outputs=2 exposes a second head; configure both outputs in the shell"
audio_hint="guest: select the intel-hda / hda-duplex sink in the shell mixer"

QEMU_LAPTOP_CMD=()

build_qemu_laptop_command() {
    local net_none=0
    if [ "${1:-}" = "--net-none" ]; then net_none=1; fi
    QEMU_LAPTOP_CMD=(
        qemu-system-x86_64
        -enable-kvm
        -machine q35
        -cpu host
        -m 4G
        -smp 2
        -drive "if=pflash,format=raw,readonly=on,file=$ovmf_code"
        -drive "if=pflash,format=raw,file=$ovmf_vars"
        -drive "file=$disk_image,format=raw"
        -acpitable "file=$battery_ssdt"
        -display gtk,gl=on
        -device virtio-gpu-pci,max_outputs=2
        -audiodev pa,id=gb-audio
        -device intel-hda,audiodev=gb-audio
        -device hda-duplex,audiodev=gb-audio
        -serial "file:$serial_log"
        -monitor stdio
    )
    if [ "$net_none" -eq 1 ]; then
        QEMU_LAPTOP_CMD+=( -net none )
        wifi_guest_hint="guest: modprobe mac80211_hwsim for Wi-Fi simulation; host networking disabled (-net none)"
    else
        QEMU_LAPTOP_CMD+=(
            -netdev "user,id=gbnet0"
            -device virtio-net-pci,netdev=gbnet0
        )
    fi
}

emit_profile() {
    build_qemu_laptop_command "${1:-}"
    printf 'QEMU_LAPTOP_CMD=%s\n' "${QEMU_LAPTOP_CMD[*]}"
    printf 'GUEST_BATTERY_HINT=%s\n' "$battery_guest_hint"
    printf 'GUEST_BATTERY_NOTE=%s\n' "$battery_device_note"
    printf 'GUEST_WIFI_HINT=%s\n' "$wifi_guest_hint"
    printf 'GUEST_HEAD_HINT=%s\n' "$head_hint"
    printf 'GUEST_AUDIO_HINT=%s\n' "$audio_hint"
    printf 'SCREENSHOT=%s\n' "$screenshot"
    printf 'SERIAL_LOG=%s\n' "$serial_log"
}

warn_if_assets_missing() {
    local asset
    for asset in "$ovmf_code" "$ovmf_vars" "$battery_ssdt" "$disk_image"; do
        if [ ! -e "$asset" ]; then
            printf 'note: asset not present: %s (supply via env before running QEMU)\n' "$asset" >&2
        fi
    done
}

required_tokens=(
    '-acpitable file='
    'acpi-battery'
    'battery'
    'mac80211_hwsim'
    '-device virtio-net-pci'
    'netdev user,id=gbnet0'
    '-device intel-hda'
    '-device hda-duplex'
    '-device virtio-gpu-pci,max_outputs=2'
    'second head'
    'if=pflash'
    'OVMF_VARS'
    '-serial file:'
    'SCREENSHOT='
    'SERIAL_LOG='
)

check_mode() {
    local emitted airgap airgap_cmd tok missing=0
    emitted="$(emit_profile)"
    airgap="$(emit_profile --net-none)"
    airgap_cmd="$(printf '%s\n' "$airgap" | grep -m1 '^QEMU_LAPTOP_CMD=' || true)"
    for tok in "${required_tokens[@]}"; do
        if [[ "$emitted" != *"$tok"* ]]; then
            printf 'MISS emitted token: %s\n' "$tok"
            missing=$((missing + 1))
        fi
    done
    if [[ "$airgap_cmd" != *"-net none"* ]]; then
        printf 'MISS: --net-none variant does not emit -net none\n'
        missing=$((missing + 1))
    fi
    if [[ "$airgap_cmd" == *"virtio-net-pci"* ]]; then
        printf 'MISS: --net-none variant still emits virtio-net-pci\n'
        missing=$((missing + 1))
    fi
    if ! bash -n "$self"; then
        printf 'MISS: bash -n %s failed\n' "$self"
        missing=$((missing + 1))
    fi
    if [ "$missing" -ne 0 ]; then
        printf 'qemu-laptop-profile --check: FAIL (%d problem(s))\n' "$missing"
        return 1
    fi
    printf 'qemu-laptop-profile --check: PASS (static tokens + bash -n; no QEMU run, no hardware PASS)\n'
}

usage() {
    printf 'Usage: %s [--check|--print|--net-none|--help]\n' "$self"
}

case "${1:-}" in
    --check)
        check_mode
        ;;
    --print|'')
        warn_if_assets_missing
        emit_profile
        ;;
    --net-none)
        warn_if_assets_missing
        emit_profile --net-none
        ;;
    --help|-h)
        usage
        ;;
    *)
        printf 'unknown option: %s\n' "$1" >&2
        usage >&2
        exit 2
        ;;
esac
