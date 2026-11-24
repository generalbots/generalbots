#!/bin/bash
# Runtime GPU / VA-API profile detection for the General Bots OS appliance (#2481).
#
# Reads ONLY sysfs (plus uname and /dev/dri) to classify the host GPU, then:
#   * picks the matching profile from gpu-driver-profiles.psv (next to this file);
#   * installs only that profile's driver packages when a package manager (apt)
#     is available and the packages are not already installed (idempotent);
#   * writes the WebKitGTK/GBM/VA environment to an environment.d drop-in;
#   * falls back to an explicit software (llvmpipe/swrast) profile with a clear
#     log line when no usable GPU or driver is present.
#
# Modes:
#   (none)      apply: detect, install missing packages, write env drop-in.
#   --audit     report GPU / V4L2 / VA-API capability from sysfs ONLY — no changes.
#   --dry-run   report every action it WOULD take — writes nothing, installs nothing.
#   -h|--help   print usage.
#
# SKIP / idempotency contract (mirrors the gb-data scripts, #2470):
#   GB_GRAPHICS_SKIP=1  -> exit 0 immediately, no output beyond one log line.
#   DRY_RUN=1           -> same as --dry-run (useful for the boot service in CI).
#   Re-running in apply mode is safe: installed packages are detected with
#   dpkg-query and the env file is replaced only when its rendered content
#   actually differs (cmp), so a second run is a no-op.
#
# No secrets, no network calls, no CDN, no eval. All logs go to stderr so the
# audit report on stdout stays machine-readable.
set -euo pipefail

GB_GFX_TAG="gb-graphics"
PROFILES_FILE="${GB_GRAPHICS_PROFILES:-}"
ENV_FILE="${GB_GRAPHICS_ENV_FILE:-/etc/environment.d/60-gb-graphics.conf}"
DRY_RUN="${DRY_RUN:-0}"
SKIP="${GB_GRAPHICS_SKIP:-0}"

here="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
[ -n "$PROFILES_FILE" ] || PROFILES_FILE="$here/gpu-driver-profiles.psv"

log()  { printf '%s[%s] %s\n' "$(date -u '+%FT%TZ')" "$GB_GFX_TAG" "$*" >&2; }
warn() { printf '%s[%s] WARN: %s\n' "$(date -u '+%FT%TZ')" "$GB_GFX_TAG" "$*" >&2; }
die()  { printf '%s[%s] ERROR: %s\n' "$(date -u '+%FT%TZ')" "$GB_GFX_TAG" "$*" >&2; exit 1; }
is_dry() { [ "$DRY_RUN" = "1" ]; }

usage() {
    cat <<'EOF'
Usage: detect-graphics.sh [--audit | --dry-run | --help]

  (no args)  Detect GPU from sysfs, install the matching Mesa/VA-API driver
             packages (only when apt is available and they are missing), and
             write /etc/environment.d/60-gb-graphics.conf for WebKitGTK/GBM.
  --audit    Print GPU / V4L2 / VA-API capability from sysfs only. No changes.
  --dry-run  Print the exact plan (packages + environment). No changes.
  --help     Show this help.

Environment:
  GB_GRAPHICS_SKIP=1        exit immediately without touching the host.
  DRY_RUN=1                 equivalent to --dry-run.
  GB_GRAPHICS_PROFILES=PATH override the gpu-driver-profiles.psv data file.
  GB_GRAPHICS_ENV_FILE=PATH override the environment.d drop-in target.
EOF
}

skip_if_requested() {
    if [ "$SKIP" = "1" ]; then
        log "GB_GRAPHICS_SKIP=1 — skipping graphics detection, no changes made"
        exit 0
    fi
}

load_profiles() {
    [ -r "$PROFILES_FILE" ] || die "profiles data file not readable: $PROFILES_FILE"
}

# read_sysfs FILE -> first line of FILE (empty when unreadable).
read_sysfs() {
    local f="$1" v=""
    if [ -r "$f" ]; then
        IFS= read -r v < "$f" || true
    fi
    printf '%s' "$v"
}

# arch_name -> x86_64 / aarch64 / raw uname value.
arch_name() {
    case "$(uname -m)" in
        x86_64|amd64) printf 'x86_64' ;;
        aarch64|arm64) printf 'aarch64' ;;
        *) uname -m ;;
    esac
}

# sysfs evidence collectors. Each prints one token per line (unique).
# Classification uses ONLY DRM-card evidence: a generic PCI vendor of 0x8086
# (Intel chipset/IDE/USB) or 0x1002 on a non-graphics device is NOT a GPU.
# All PCI vendors are still reported in --audit for diagnostics.
collect_drm_vendors() {
    local f v
    for f in /sys/class/drm/card*/device/vendor; do
        [ -r "$f" ] || continue
        v="$(read_sysfs "$f")"
        v="${v#0x}"
        v="${v,,}"
        [ -n "$v" ] && printf '%s\n' "$v"
    done | sort -u
}

collect_pci_vendors() {
    local f v
    for f in /sys/bus/pci/devices/*/vendor; do
        [ -r "$f" ] || continue
        v="$(read_sysfs "$f")"
        v="${v#0x}"
        v="${v,,}"
        [ -n "$v" ] && printf '%s\n' "$v"
    done | sort -u
}

collect_drm_kmods() {
    local d t
    for d in /sys/class/drm/card*/device/driver; do
        [ -e "$d" ] || continue
        t="$(basename "$(readlink -f "$d" 2>/dev/null || true)")"
        if [ -n "$t" ] && [ "$t" != "/" ] && [ "$t" != "." ]; then
            printf '%s\n' "$t"
        fi
    done | sort -u
}

is_virtual_firmware() {
    local f p
    for f in /sys/class/dmi/id/product_name /sys/class/dmi/id/sys_vendor /sys/class/dmi/id/board_name; do
        [ -r "$f" ] || continue
        p="$(read_sysfs "$f")"
        case "$p" in
            *QEMU*|*qemu*|*VirtualBox*|*VMware*|*KVM*|*kvm*) return 0 ;;
        esac
    done
    return 1
}

count_render_nodes() {
    local n=0 r
    for r in /dev/dri/renderD*; do
        [ -e "$r" ] || continue
        n=$((n + 1))
    done
    printf '%s' "$n"
}

count_drm_cards() {
    local n=0 c
    for c in /sys/class/drm/card[0-9]*; do
        [ -d "$c" ] || continue
        n=$((n + 1))
    done
    printf '%s' "$n"
}

# token_in_list NEEDLE "comma,separated,tokens" -> 0 when present.
token_in_list() {
    local needle="$1" list="$2" tok
    [ -n "$needle" ] || return 1
    local IFS=','
    for tok in $list; do
        [ "$tok" = "$needle" ] && return 0
    done
    return 1
}

# probe_profile_row "ARCH|DRIVER" -> populate P_* globals, 0 on match.
probe_profile_row() {
    local want="$1"
    local p_arch p_driver p_vid p_kmod p_mesa p_va p_fw p_sw p_vadrv p_desc
    while IFS='|' read -r p_arch p_driver p_vid p_kmod p_mesa p_va p_fw p_sw p_vadrv p_desc; do
        case "$p_arch" in ''|\#*) continue ;; esac
        [ "$p_arch|$p_driver" = "$want" ] || continue
        PROFILE_KEY="$want"
        P_VID="$p_vid"; P_KMOD="$p_kmod"
        P_MESA="$p_mesa"; P_VA="$p_va"; P_FW="$p_fw"
        P_SW="$p_sw"; P_VADRV="$p_vadrv"; P_DESC="$p_desc"
        return 0
    done < "$PROFILES_FILE"
    return 1
}

# driver_class_from_list VENDORS KMODS VIRT -> print intel|amd|nvidia|virt|software.
driver_class_from_list() {
    local vendors="$1" kmods="$2" virt="$3"
    local vid kmod
    # Iterate data-file rows for this architecture in declaration order so the
    # first matching class wins (intel before amd before nvidia before virt).
    while IFS='|' read -r p_arch p_driver p_vid p_kmod p_mesa p_va p_fw p_sw p_vadrv p_desc; do
        case "$p_arch" in ''|\#*) continue ;; esac
        [ "$p_arch" = "$ARCH" ] || continue
        if [ "$p_driver" = "software" ]; then continue; fi
        for vid in $(printf '%s' "$p_vid" | tr ',' '\n'); do
            if token_in_list "$vid" "$vendors"; then printf '%s' "$p_driver"; return 0; fi
        done
        for kmod in $(printf '%s' "$p_kmod" | tr ',' '\n'); do
            if token_in_list "$kmod" "$kmods"; then printf '%s' "$p_driver"; return 0; fi
        done
    done < "$PROFILES_FILE"
    if [ "$virt" = "1" ]; then printf 'virt'; return 0; fi
    printf 'software'
}

detect() {
    ARCH="$(arch_name)"
    PCI_VENDORS="$(collect_pci_vendors | tr '\n' ',' | sed 's/,$//')"
    VENDORS="$(collect_drm_vendors | tr '\n' ',' | sed 's/,$//')"
    KMODS="$(collect_drm_kmods | tr '\n' ',' | sed 's/,$//')"
    if is_virtual_firmware; then VIRT=1; else VIRT=0; fi
    CARDS="$(count_drm_cards)"
    RENDER_NODES="$(count_render_nodes)"
    V4L2="$(find_video4linux)"

    if [ "$CARDS" -eq 0 ] && [ "$RENDER_NODES" -eq 0 ]; then
        # No GPU device at all (e.g. QEMU without pass-through): never trust
        # stray PCI vendor IDs; select the software profile explicitly.
        DRIVER_CLASS="software"
        FALLBACK=1
        log "no GPU or DRM render node detected (cards=$CARDS render_nodes=$RENDER_NODES) — falling back to SOFTWARE rendering (llvmpipe/swrast)"
    else
        DRIVER_CLASS="$(driver_class_from_list "$VENDORS" "$KMODS" "$VIRT")"
        if [ "$DRIVER_CLASS" = "software" ]; then
            FALLBACK=1
            log "GPU present but no matching driver profile (drm_vendors=$VENDORS drm_drivers=$KMODS) — falling back to SOFTWARE rendering (llvmpipe/swrast)"
        else
            FALLBACK=0
            log "detected driver class '$DRIVER_CLASS' on $ARCH (cards=$CARDS render_nodes=$RENDER_NODES virt=$VIRT)"
        fi
    fi

    if ! probe_profile_row "$ARCH|$DRIVER_CLASS"; then
        if ! probe_profile_row "$ARCH|software"; then
            probe_profile_row "any|software" || die "no usable profile in $PROFILES_FILE"
        fi
    fi
}

find_video4linux() {
    local d n=0
    for d in /sys/class/video4linux/video*; do
        [ -d "$d" ] || continue
        n=$((n + 1))
    done
    printf '%s' "$n"
}

describe_drm() {
    local c v d drv
    for c in /sys/class/drm/card[0-9]*; do
        [ -d "$c" ] || continue
        v="$(read_sysfs "$c/device/vendor")"
        d="$(read_sysfs "$c/device/device")"
        drv="$(basename "$(readlink -f "$c/device/driver" 2>/dev/null || true)")"
        printf '      %s vendor=%s device=%s driver=%s\n' \
            "$(basename "$c")" "${v:-?}" "${d:-?}" "${drv:-?}"
    done
}

describe_v4l2() {
    local f n
    for f in /sys/class/video4linux/video*; do
        [ -d "$f" ] || continue
        n="$(read_sysfs "$f/name")"
        printf '      %s name=%s\n' "$(basename "$f")" "${n:-?}"
    done
}

print_audit() {
    printf 'GB Graphics audit (#2481) — sysfs only; no packages installed, no files written\n'
    printf '  timestamp       : %s\n' "$(date -u '+%FT%TZ')"
    printf '  architecture    : %s\n' "$ARCH"
    printf '  drm_cards       : %s\n' "$CARDS"
    describe_drm
    printf '  drm_render_nodes: %s' "$RENDER_NODES"
    if [ "$RENDER_NODES" -gt 0 ]; then
        printf ' (/dev/dri/renderD* present — VA-API kernel interface available)\n'
    else
        printf ' (none — VA-API kernel interface unavailable)\n'
    fi
    printf '  video4linux     : %s device(s)\n' "$V4L2"
    describe_v4l2
    printf '  pci_vendors     : %s (informational; includes non-GPU devices)\n' "${PCI_VENDORS:-(none)}"
    printf '  drm_vendors     : %s (classification evidence)\n' "${VENDORS:-(none)}"
    printf '  drm_drivers     : %s\n' "${KMODS:-(none)}"
    printf '  virtual_firmware: %s\n' "$([ "$VIRT" = "1" ] && printf yes || printf no)"
    printf '  driver_class    : %s\n' "$DRIVER_CLASS"
    printf '  matched_profile : %s — %s\n' "$PROFILE_KEY" "$P_DESC"
    local vadrv_hint=""
    if [ "$P_VADRV" != "-" ] && [ -n "$P_VADRV" ]; then
        vadrv_hint=" LIBVA_DRIVER_NAME=$P_VADRV"
    fi
    printf '  planned_env     : WEBKIT_DISABLE_COMPOSITING_MODE=0 GBM_BACKEND=drm LIBGL_ALWAYS_SOFTWARE=%s%s\n' \
        "$P_SW" "$vadrv_hint"
    if [ "$RENDER_NODES" -gt 0 ]; then
        printf '  va-api verdict  : hardware decode POSSIBLE on this host; run `vainfo` for the exact profile (not done in --audit)\n'
    else
        printf '  va-api verdict  : SOFTWARE decode path — no DRM render node; llvmpipe/swrast fallback\n'
    fi
    if [ "$FALLBACK" = "1" ]; then
        printf '  fallback        : YES — software rendering profile selected (no usable GPU/driver)\n'
    fi
}

pkg_installed() {
    local s
    command -v dpkg-query >/dev/null 2>&1 || return 1
    s="$(dpkg-query -W -f='${Status}' "$1" 2>/dev/null || true)"
    [ "$s" = "install ok installed" ]
}

detect_pm() {
    if command -v apt-get >/dev/null 2>&1; then printf 'apt-get'
    elif command -v apt >/dev/null 2>&1; then printf 'apt'
    else printf ''; fi
}

install_profile_packages() {
    local pm missing=() p
    pm="$(detect_pm)"
    for p in $P_MESA $P_VA $P_FW; do
        [ -n "$p" ] || continue
        if pkg_installed "$p"; then
            log "package already installed: $p"
        else
            missing+=("$p")
        fi
    done
    if [ "${#missing[@]}" -eq 0 ]; then
        log "all driver packages for profile '$PROFILE_KEY' are present — nothing to install"
        return 0
    fi
    if is_dry; then
        printf '  [dry-run] would install via %s: %s\n' \
            "${pm:-<no package manager>}" "${missing[*]}" >&2
        return 0
    fi
    if [ -z "$pm" ]; then
        warn "no supported package manager (apt-get/apt) — cannot install:${missing[*]}"
        return 0
    fi
    if [ "$(id -u)" -ne 0 ]; then
        warn "not running as root — cannot install:${missing[*]}"
        return 0
    fi
    log "installing driver packages for '$PROFILE_KEY': ${missing[*]}"
    if "$pm" install -y --no-install-recommends "${missing[@]}" >&2; then
        log "package installation finished"
    else
        warn "package installation failed for:${missing[*]} — software fallback remains available"
    fi
}

render_env() {
    printf '# Generated by gb-graphics/detect-graphics.sh (#2481) — do not edit by hand.\n'
    printf '# Detected profile: %s — %s\n' "$PROFILE_KEY" "$P_DESC"
    printf 'WEBKIT_DISABLE_COMPOSITING_MODE=0\n'
    printf 'GBM_BACKEND=drm\n'
    printf 'LIBGL_ALWAYS_SOFTWARE=%s\n' "$P_SW"
    if [ "$P_VADRV" != "-" ] && [ -n "$P_VADRV" ]; then
        printf 'LIBVA_DRIVER_NAME=%s\n' "$P_VADRV"
    fi
}

write_env() {
    local dir tmp
    dir="$(dirname "$ENV_FILE")"
    if is_dry; then
        log "dry-run: would write $ENV_FILE with:"
        render_env | sed 's/^/      /' >&2
        return 0
    fi
    mkdir -p "$dir"
    tmp="$(mktemp "$dir/.gb-graphics.XXXXXX")" || die "cannot create temp file in $dir"
    render_env > "$tmp"
    chmod 0644 "$tmp"
    if [ -f "$ENV_FILE" ] && cmp -s "$tmp" "$ENV_FILE"; then
        log "$ENV_FILE already up to date (no change)"
        rm -f "$tmp"
        return 0
    fi
    mv -f "$tmp" "$ENV_FILE" || die "cannot write $ENV_FILE"
    log "wrote $ENV_FILE"
}

print_plan() {
    log "DRY RUN — no packages installed, no files written"
    printf '  [plan] driver class : %s\n' "$DRIVER_CLASS" >&2
    printf '  [plan] profile      : %s — %s\n' "$PROFILE_KEY" "$P_DESC" >&2
    printf '  [plan] mesa packages: %s\n' "$P_MESA" >&2
    printf '  [plan] va packages  : %s\n' "$P_VA" >&2
    printf '  [plan] fw packages  : %s\n' "${P_FW:-(none)}" >&2
    if [ "$FALLBACK" = "1" ]; then
        log "no GPU/driver — software fallback profile '$PROFILE_KEY' selected (LIBGL_ALWAYS_SOFTWARE=1)"
    fi
    install_profile_packages
    write_env
}

main() {
    case "${1:-}" in
        --audit) MODE="audit" ;;
        --dry-run|-n) MODE="dry" ;;
        -h|--help) usage; return 0 ;;
        '') MODE="apply" ;;
        *) die "unknown option: $1 (try --help)" ;;
    esac
    if [ "$MODE" = "dry" ]; then DRY_RUN=1; fi
    skip_if_requested
    load_profiles
    detect
    case "$MODE" in
        audit) print_audit ;;
        dry)   print_plan ;;
        apply)
            if [ "$FALLBACK" = "1" ]; then
                log "software fallback active (llvmpipe/swrast) — hardware acceleration DISABLED for this host"
            else
                log "hardware acceleration profile '$PROFILE_KEY' active (LIBGL_ALWAYS_SOFTWARE=$P_SW)"
            fi
            install_profile_packages
            write_env
            ;;
    esac
}

main "$@"
