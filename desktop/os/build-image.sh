#!/usr/bin/env bash
# Build the General Bots OS appliance with an EXPLICIT Secure Boot policy
# (#2478). Two modes:
#
#   dev      unsigned UKI (mkosi.conf default), Secure Boot off. This is the
#            locally bootable image — `mkosi vm` boots it without any keys.
#   release  signed UKI + signed bootloader. Signing keys are MANDATORY: the
#            build aborts loudly if they are missing, unreadable or malformed.
#            Pass them with --key/--cert or GB_SB_KEY/GB_SB_CERT.
#
# No private key is ever stored in this repository. Generate a throwaway pair
# for development/CI with desktop/os/generate-secure-boot-keys.sh; production
# must inject an offline-held key pair via the environment/flags.
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

die() { printf 'build-image: %s\n' "$*" >&2; exit 1; }

usage() {
    cat <<'USAGE'
Usage: desktop/os/build-image.sh [options] [-- <extra mkosi args>]

  --mode dev|release   build policy (default: ${GB_OS_BUILD_MODE:-dev})
  --key PATH           PEM signing key (release; or GB_SB_KEY)
  --cert PATH          PEM signing certificate (release; or GB_SB_CERT)
  --output-dir DIR     mkosi output directory (or GB_OS_OUTPUT_DIR)
  --mkosi PATH         mkosi executable (default: mkosi)
  --smoke              after a release build, boot the image in OVMF with
                       Secure Boot enabled and Microsoft keys + our MOK enrolled
  -h, --help           show this help

Anything after `--` is appended to the mkosi invocation (e.g. -- -f).
USAGE
}

mode="${GB_OS_BUILD_MODE:-dev}"
key="${GB_SB_KEY:-}"
cert="${GB_SB_CERT:-}"
out_dir="${GB_OS_OUTPUT_DIR:-}"
mkosi_bin="mkosi"
smoke=0
extra=()

while [ "$#" -gt 0 ]; do
    case "$1" in
        --mode)        mode="${2:-}"; [ "$#" -ge 2 ] || die "--mode needs a value"; shift ;;
        --mode=*)      mode="${1#*=}" ;;
        --key)         key="${2:-}"; [ "$#" -ge 2 ] || die "--key needs a value"; shift ;;
        --key=*)       key="${1#*=}" ;;
        --cert)        cert="${2:-}"; [ "$#" -ge 2 ] || die "--cert needs a value"; shift ;;
        --cert=*)      cert="${1#*=}" ;;
        --output-dir)  out_dir="${2:-}"; [ "$#" -ge 2 ] || die "--output-dir needs a value"; shift ;;
        --output-dir=*) out_dir="${1#*=}" ;;
        --mkosi)       mkosi_bin="${2:-}"; [ "$#" -ge 2 ] || die "--mkosi needs a value"; shift ;;
        --mkosi=*)     mkosi_bin="${1#*=}" ;;
        --smoke)       smoke=1 ;;
        -h|--help)     usage; exit 0 ;;
        --)            shift; extra=("$@"); break ;;
        *)             die "unknown option: $1" ;;
    esac
    shift
done

case "$mode" in
    dev|release) : ;;
    *) die "--mode must be 'dev' or 'release' (got '$mode')" ;;
esac

args=()
if [ -n "$out_dir" ]; then
    mkdir -p -- "$out_dir"
    args+=("--output-dir=$out_dir")
fi

if [ "$mode" = "release" ]; then
    if [ -z "$key" ] || [ -z "$cert" ]; then
        die "RELEASE BUILD REFUSED: signing keys are missing (#2478).
Set GB_SB_KEY / GB_SB_CERT (or pass --key / --cert) to a PEM private key and
certificate. For a throwaway local pair run:
  desktop/os/generate-secure-boot-keys.sh --output-dir /tmp/gb-sb-keys
Development builds without keys: desktop/os/build-image.sh --mode dev"
    fi
    for f in "$key" "$cert"; do
        [ -f "$f" ] || die "signing key file not found: $f"
        [ -s "$f" ] || die "signing key file is empty: $f"
    done
    grep -q -- 'PRIVATE KEY' "$key" \
        || die "signing key is not a PEM private key: $key"
    grep -q -- 'BEGIN CERTIFICATE' "$cert" \
        || die "signing certificate is not PEM: $cert"

    # A key/cert pair that does not match yields an image signed by a key the
    # shipped certificate cannot verify — a silent boot failure. Check it when
    # openssl is available (keygen/CI have it); mkosi still performs the signing.
    if command -v openssl >/dev/null 2>&1; then
        if openssl pkey -in "$key" -noout >/dev/null 2>&1 \
                && openssl x509 -in "$cert" -noout >/dev/null 2>&1; then
            key_fp="$(openssl pkey -in "$key" -pubout -outform DER 2>/dev/null | sha256sum | cut -d' ' -f1 || true)"
            cert_fp="$(openssl x509 -in "$cert" -pubkey -noout 2>/dev/null | openssl pkey -pubin -outform DER 2>/dev/null | sha256sum | cut -d' ' -f1 || true)"
            if [ -n "$key_fp" ] && [ -n "$cert_fp" ] && [ "$key_fp" != "$cert_fp" ]; then
                die "signing key and certificate do not match: the signed image would not boot"
            fi
        fi
    fi

    profile="$SCRIPT_DIR/mkosi.profiles/secure-boot-release.conf"
    [ -f "$profile" ] || die "missing release signing policy: $profile"
    args+=(
        "--profile=secure-boot-release"
        "--secure-boot-key=$key"
        "--secure-boot-certificate=$cert"
    )
    printf 'build-image: RELEASE mode — signed UKI/bootloader with %s\n' "$cert"
else
    printf 'build-image: DEV mode — unsigned UKI, Secure Boot off (bootable without keys)\n'
fi

args+=("${extra[@]}")
log_prefix="${out_dir:-/tmp}"

command -v "$mkosi_bin" >/dev/null 2>&1 \
    || die "mkosi executable not found: $mkosi_bin"

"$mkosi_bin" "${args[@]}" build

if [ "$smoke" -eq 1 ]; then
    [ "$mode" = "release" ] \
        || die "--smoke requires --mode release (an unsigned image proves nothing with Secure Boot on)"
    smoke_log="${log_prefix}/secure-boot-smoke.log"
    printf 'build-image: OVMF Secure Boot smoke (Microsoft keys + MOK), log: %s\n' "$smoke_log"
    set +e
    timeout 480 "$mkosi_bin" "${args[@]}" \
        --firmware=uefi-secure-boot \
        --firmware-variables=microsoft-mok \
        vm >"$smoke_log" 2>&1
    smoke_rc=$?
    set -e
    printf 'mkosi vm (Secure Boot) exit code: %s\n' "$smoke_rc"
    if [ "$smoke_rc" -ne 0 ] && [ "$smoke_rc" -ne 124 ]; then
        tail -80 "$smoke_log" >&2 || true
        die "OVMF Secure Boot smoke failed with exit $smoke_rc"
    fi
    if grep -aq 'Started General Bots kiosk shell' "$smoke_log"; then
        printf 'Secure Boot smoke: green (kiosk shell started with Secure Boot enabled)\n'
    else
        tail -80 "$smoke_log" >&2 || true
        die 'OVMF Secure Boot smoke did not confirm generalbots-shell.service'
    fi
fi
