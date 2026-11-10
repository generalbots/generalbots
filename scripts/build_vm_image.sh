#!/bin/bash
# #2062 — build the pre-fabricated agent workspace image.
#
# The image is Debian 13 plus one profile from
# `botserver/crates/botagent/src/vm/profile/mod.rs`:
#
#   base    today's runtime (python3, pip, node, git) — no document engine
#   office  base + static headless Chromium + fonts + document/PDF libraries
#           (the default; +<= 200 MB, which is why documents are generated with
#           python-docx/openpyxl/python-pptx/reportlab and HTML becomes PDF
#           through `chromium --headless=new --print-to-pdf`)
#   media   office + ffmpeg, for offline hosts only (media jobs run in BotModels)
#   data    office + numpy/pandas/matplotlib/scipy
#
# No secrets and no credentials are involved: everything comes from the local
# artifact cache or from the distribution mirror.
#
# Usage:
#   scripts/build_vm_image.sh --profile office
#   scripts/build_vm_image.sh --profile office --offline --check-size
#
# Options:
#   --profile NAME   base|office|media|data          (default: office)
#   --cache DIR      static artifacts to bake in      (default: $GB_VM_ARTIFACT_CACHE
#                                                      or /opt/vibe/artifacts)
#   --base IMAGE     base Incus image alias           (default: images:debian/13)
#   --alias NAME     published image alias            (default: gb-workspace-<profile>)
#   --offline        never touch the network; fail when an artifact is missing
#   --check-size     enforce base + 200 MB after publishing
#   --verify         only check this script against the profile table in Rust
#   --keep           keep the builder container for inspection
#   --dry-run        print the plan and exit
#   -h, --help       this help
set -euo pipefail

REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
PROFILE_TABLE="$REPO_ROOT/botserver/crates/botagent/src/vm/profile/mod.rs"
PROFILE=office
CACHE=${GB_VM_ARTIFACT_CACHE:-/opt/vibe/artifacts}
BASE_IMAGE=images:debian/13
ALIAS=
OFFLINE=0
CHECK_SIZE=0
VERIFY_ONLY=0
KEEP=0
DRY_RUN=0
SIZE_LIMIT_MB=200

log() { printf '[build_vm_image] %s\n' "$*"; }
die() { printf '[build_vm_image] error: %s\n' "$*" >&2; exit 1; }

while [ $# -gt 0 ]; do
    case "$1" in
        --profile) PROFILE=${2:?--profile needs a value}; shift 2 ;;
        --cache) CACHE=${2:?--cache needs a value}; shift 2 ;;
        --base) BASE_IMAGE=${2:?--base needs a value}; shift 2 ;;
        --alias) ALIAS=${2:?--alias needs a value}; shift 2 ;;
        --offline) OFFLINE=1; shift ;;
        --check-size) CHECK_SIZE=1; shift ;;
        --verify) VERIFY_ONLY=1; shift ;;
        --keep) KEEP=1; shift ;;
        --dry-run) DRY_RUN=1; shift ;;
        -h|--help) sed -n '2,40p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) die "unknown option '$1' (try --help)" ;;
    esac
done

case "$PROFILE" in
    base|office|media|data) ;;
    *) die "invalid --profile '$PROFILE' (base|office|media|data)" ;;
esac
ALIAS=${ALIAS:-gb-workspace-$PROFILE}

# Artifact ids this script installs. `--verify` keeps them in sync with the
# Rust table, which is the source of truth for sizes and probes.
OFFICE_IDS="chromium-headless fonts doc-libs pdf-libs html-libs"
MEDIA_IDS="ffmpeg"
DATA_IDS="scientific-python"

verify_table() {
    [ -f "$PROFILE_TABLE" ] || die "profile table not found: $PROFILE_TABLE"
    local ids="$OFFICE_IDS" missing=0
    case "$PROFILE" in
        base) ids="" ;;
        media) ids="$OFFICE_IDS $MEDIA_IDS" ;;
        data) ids="$OFFICE_IDS $DATA_IDS" ;;
    esac
    for id in $ids; do
        grep -q "id: \"$id\"" "$PROFILE_TABLE" || { log "artifact '$id' is not in $PROFILE_TABLE"; missing=1; }
    done
    [ "$missing" -eq 0 ] || die "this script drifted from the profile table"
    log "profile '$PROFILE' matches the Rust artifact table"
}

verify_table
[ "$VERIFY_ONLY" -eq 1 ] && exit 0

command -v incus >/dev/null 2>&1 || die "incus is required (https://linuxcontainers.org/incus/)"

CHROMIUM_TARBALL="$CACHE/chromium-headless.tar.zst"
if [ "$PROFILE" != base ] && [ ! -f "$CHROMIUM_TARBALL" ]; then
    if [ "$OFFLINE" -eq 1 ]; then
        die "offline build needs $CHROMIUM_TARBALL (the static headless Chromium build). Place it in --cache or build online once."
    fi
    log "warning: $CHROMIUM_TARBALL missing; the build falls back to the distro chromium package (+230 MB, outside the budget)"
fi

BUILDER="gb-vm-build-$$"
log "profile=$PROFILE base=$BASE_IMAGE alias=$ALIAS cache=$CACHE offline=$OFFLINE"
if [ "$DRY_RUN" -eq 1 ]; then
    log "dry run: would launch $BUILDER, install the '$PROFILE' artifacts and publish $ALIAS"
    exit 0
fi

cleanup() {
    if [ "$KEEP" -eq 0 ]; then
        incus delete --force "$BUILDER" >/dev/null 2>&1 || true
    else
        log "kept builder container: $BUILDER"
    fi
}
trap cleanup EXIT

run() { incus exec "$BUILDER" -- "$@"; }

log "launching builder container"
incus launch "$BASE_IMAGE" "$BUILDER" --config limits.cpu=2 --config limits.memory=2GiB

run mkdir -p /opt/vibe/artifacts /opt/vibe/conversations
if [ -d "$CACHE" ] && [ -n "$(ls -A "$CACHE" 2>/dev/null)" ]; then
    log "pushing artifact cache from $CACHE"
    incus file push -r "$CACHE/." "$BUILDER/opt/vibe/artifacts/"
fi

export DEBIAN_FRONTEND=noninteractive
run sh -c 'export DEBIAN_FRONTEND=noninteractive; apt-get -o DPkg::Lock::Timeout=300 update -q'
run sh -c 'export DEBIAN_FRONTEND=noninteractive; apt-get -o DPkg::Lock::Timeout=300 install -y -q --no-install-recommends python3 python3-pip nodejs npm git ca-certificates iptables iproute2 cron curl'

if [ "$PROFILE" != base ]; then
    # chromium-headless: static build from the cache, distro package only as the
    # online fallback. Both provide --headless=new --print-to-pdf.
    if [ -f "$CHROMIUM_TARBALL" ]; then
        run sh -c 'mkdir -p /opt/chromium && tar -xf /opt/vibe/artifacts/chromium-headless.tar.zst -C /opt/chromium'
    else
        run sh -c 'export DEBIAN_FRONTEND=noninteractive; apt-get -o DPkg::Lock::Timeout=300 install -y -q --no-install-recommends chromium'
    fi
    run sh -c 'export DEBIAN_FRONTEND=noninteractive; apt-get -o DPkg::Lock::Timeout=300 install -y -q --no-install-recommends fonts-noto-core fonts-dejavu-core'
    run python3 -m pip install --break-system-packages --no-cache-dir --quiet \
        python-docx openpyxl python-pptx lxml reportlab pikepdf pypdf markdown jinja2
    # WeasyPrint is deliberately absent: the embedded Chromium prints HTML to
    # PDF, which is faster and 55 MB smaller.
    run sh -c 'printf "%s\n" "#!/bin/sh" "set -eu" "chrome=\${GB_CHROME_BIN:-}" "if [ -z \"\$chrome\" ]; then if [ -x /opt/chromium/chrome ]; then chrome=/opt/chromium/chrome; else chrome=\$(command -v chromium) || chrome=; fi; fi" "[ -n \"\$chrome\" ] || { echo \"html2pdf: no chromium in this VM\" >&2; exit 78; }" "exec \"\$chrome\" --headless=new --no-sandbox --disable-gpu --print-to-pdf=\"\$2\" \"file://\$(realpath \"\$1\")\"" > /usr/local/bin/html2pdf'
    run chmod +x /usr/local/bin/html2pdf
fi

if [ "$PROFILE" = media ]; then
    run sh -c 'export DEBIAN_FRONTEND=noninteractive; apt-get -o DPkg::Lock::Timeout=300 install -y -q --no-install-recommends ffmpeg'
fi
if [ "$PROFILE" = data ]; then
    run python3 -m pip install --break-system-packages --no-cache-dir --quiet numpy pandas matplotlib scipy
fi

run sh -c "id -u runner >/dev/null 2>&1 || useradd -m -s /bin/bash runner; chown -R runner:runner /opt/vibe; echo v7-$PROFILE > /opt/vibe/.provisioned"

log "publishing $ALIAS"
incus stop "$BUILDER" >/dev/null
incus publish "$BUILDER" --alias "$ALIAS" --reuse >/dev/null
log "image ready: $ALIAS (launch it with AGENT_VM_IMAGE=$ALIAS)"

if [ "$CHECK_SIZE" -eq 1 ]; then
    log "checking the +${SIZE_LIMIT_MB} MB budget"
    GB_BASE_IMAGE="$BASE_IMAGE" GB_OFFICE_IMAGE="$ALIAS" GB_VM_SIZE_LIMIT_MB="$SIZE_LIMIT_MB" \
        "$REPO_ROOT/botserver/images/office/check-size.sh"
fi
