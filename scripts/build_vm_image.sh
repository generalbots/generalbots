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
#   --compression A  image compression for `incus publish` (gzip, xz, zstd, none)
#   --export DIR     after publishing, `incus image export` the unified tarball
#                    into DIR (the General Bots Lite agent runtime asset)
#   --info FILE      write KEY=value build facts (fingerprints, marker, file)
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
COMPRESSION=
EXPORT_DIR=
INFO_FILE=
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
        --compression) COMPRESSION=${2:?--compression needs a value}; shift 2 ;;
        --export) EXPORT_DIR=${2:?--export needs a value}; shift 2 ;;
        --info) INFO_FILE=${2:?--info needs a value}; shift 2 ;;
        --verify) VERIFY_ONLY=1; shift ;;
        --keep) KEEP=1; shift ;;
        --dry-run) DRY_RUN=1; shift ;;
        -h|--help) sed -n '2,37p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) die "unknown option '$1' (try --help)" ;;
    esac
done

case "$PROFILE" in
    base|office|media|data) ;;
    *) die "invalid --profile '$PROFILE' (base|office|media|data)" ;;
esac
ALIAS=${ALIAS:-gb-workspace-$PROFILE}
case "$COMPRESSION" in
    ''|gzip|xz|zstd|bzip2|none) ;;
    *) die "invalid --compression '$COMPRESSION' (gzip|xz|zstd|bzip2|none)" ;;
esac

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

# Printed only when the builder never gets a default route: the bridge, its
# DHCP leases and the container's own view of the network, so the log tells a
# host firewall or a missing dnsmasq from a guest that never asked for a
# lease. Best effort: no command here can fail the build.
network_diagnostics() {
    local net
    net=$(incus config show --expanded "$BUILDER" 2>/dev/null | awk '$1 == "network:" {print $2; exit}') || net=
    {
        echo "--- network diagnostics: builder $BUILDER, network ${net:-unknown} ---"
        incus list "$BUILDER" || true
        if [ -n "$net" ]; then
            incus network show "$net" || true
            incus network list-leases "$net" || true
        fi
        run sh -c 'ip a; ip route; cat /proc/net/route; cat /etc/resolv.conf; systemctl is-system-running; systemctl --failed --no-pager; systemctl status systemd-networkd --no-pager 2>&1 | head -n 15; journalctl -u systemd-networkd -n 40 --no-pager' || true
        echo "--- end of network diagnostics ---"
    } >&2
}

# Pin the base image by fingerprint: a remote alias such as images:debian/13
# moves daily, and the build must record exactly what it started from.
LAUNCH_IMAGE=$BASE_IMAGE
BASE_FINGERPRINT=$(incus image info "$BASE_IMAGE" 2>/dev/null | awk '/^Fingerprint:/ {print $2; exit}') || BASE_FINGERPRINT=
if [ -n "$BASE_FINGERPRINT" ]; then
    case "$BASE_IMAGE" in
        *:*) LAUNCH_IMAGE="${BASE_IMAGE%%:*}:$BASE_FINGERPRINT" ;;
        *) LAUNCH_IMAGE=$BASE_FINGERPRINT ;;
    esac
    log "base image $BASE_IMAGE is $BASE_FINGERPRINT"
else
    log "warning: could not resolve the fingerprint of $BASE_IMAGE; launching the alias"
fi

log "launching builder container"
incus launch "$LAUNCH_IMAGE" "$BUILDER" --config limits.cpu=2 --config limits.memory=2GiB

# A fresh Debian container can come up before DHCP/DNS are ready, or without
# /etc/resolv.conf at all; apt then fails. Wait for the network, and point the
# resolver at the bridge gateway when name resolution still fails.
log "waiting for the builder network"
net_ready=0
for _ in $(seq 1 60); do
    # /proc/net/route needs no iproute2: destination 00000000 is the default route.
    if run sh -c 'grep -q "^[^[:space:]]*[[:space:]]00000000[[:space:]]" /proc/net/route'; then net_ready=1; break; fi
    sleep 2
done
if [ "$net_ready" -ne 1 ]; then
    network_diagnostics
    die "the builder container has no default route after 120 s"
fi
if ! run sh -c 'getent hosts deb.debian.org >/dev/null 2>&1'; then
    run sh -c 'GW=$(ip route | awk "/^default/ {print \$3; exit}"); { [ -n "$GW" ] && echo "nameserver $GW"; echo "nameserver 1.1.1.1"; } > /etc/resolv.conf'
fi

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
    # #2845: no --no-sandbox (the Incus VM is the sandbox); blackhole DNS and
    # reject file:/data:/javascript: input URIs. The bootstrap installs the
    # full HTML-sanitizing wrapper over this one at VM boot.
    run sh -c 'printf "%s\n" "#!/bin/sh" "set -eu" "chrome=\${GB_CHROME_BIN:-}" "if [ -z \"\$chrome\" ]; then if [ -x /opt/chromium/chrome ]; then chrome=/opt/chromium/chrome; else chrome=\$(command -v chromium) || chrome=; fi; fi" "[ -n \"\$chrome\" ] || { echo \"html2pdf: no chromium in this VM\" >&2; exit 78; }" "case \"\$1\" in http://*|https://*) uri=\$1 ;; [a-zA-Z]*:*) echo \"html2pdf: unsupported input URI scheme\" >&2; exit 64 ;; *) uri=file://\$(realpath \"\$1\") ;; esac" "exec \"\$chrome\" --headless=new --disable-gpu --disable-background-networking --host-resolver-rules=\"MAP * ~NOTFOUND\" --print-to-pdf=\"\$2\" \"\$uri\"" > /usr/local/bin/html2pdf'
    run chmod +x /usr/local/bin/html2pdf
fi

if [ "$PROFILE" = media ]; then
    run sh -c 'export DEBIAN_FRONTEND=noninteractive; apt-get -o DPkg::Lock::Timeout=300 install -y -q --no-install-recommends ffmpeg'
fi
if [ "$PROFILE" = data ]; then
    run python3 -m pip install --break-system-packages --no-cache-dir --quiet numpy pandas matplotlib scipy
fi

MARKER="v7-$PROFILE"
# The two identities of #1658 (`claude` owns subscription credentials,
# `runner` executes generated code). The marker below makes the server skip
# its workspace preparation, so the image must already carry both users.
run sh -c "id -u runner >/dev/null 2>&1 || useradd -m -s /bin/bash runner; id -u claude >/dev/null 2>&1 || useradd -m -s /bin/bash claude; mkdir -p /home/claude/.claude; chown -R claude:claude /home/claude; chmod 700 /home/claude /home/claude/.claude; chown -R runner:runner /opt/vibe; echo $MARKER > /opt/vibe/.provisioned"

# Leave no caches, logs or host identity in the image: every workspace
# launched from it must get its own machine id, and the download stays small.
log "cleaning the builder before publishing"
run sh -c 'apt-get clean; rm -rf /var/lib/apt/lists/* /root/.cache /root/.npm /tmp/* /var/tmp/*; find /var/log -type f -delete; : > /etc/machine-id; rm -f /var/lib/dbus/machine-id'

log "publishing $ALIAS"
incus stop "$BUILDER" >/dev/null
publish_args=(publish "$BUILDER" --alias "$ALIAS" --reuse)
[ -n "$COMPRESSION" ] && publish_args+=(--compression "$COMPRESSION")
publish_args+=("description=General Bots workspace ($PROFILE profile, $MARKER)")
incus "${publish_args[@]}" >/dev/null
FINGERPRINT=$(incus image info "$ALIAS" | awk '/^Fingerprint:/ {print $2; exit}')
[ -n "$FINGERPRINT" ] || die "cannot read the fingerprint of $ALIAS"
log "image ready: $ALIAS $FINGERPRINT (launch it with AGENT_VM_IMAGE=$ALIAS)"

EXPORT_FILE=
if [ -n "$EXPORT_DIR" ]; then
    mkdir -p "$EXPORT_DIR"
    log "exporting $ALIAS to $EXPORT_DIR"
    # A published image is unified: the export is one tarball named after
    # its fingerprint, and that fingerprint is the tarball's SHA-256.
    incus image export "$ALIAS" "$EXPORT_DIR/" >/dev/null
    for candidate in "$EXPORT_DIR/$FINGERPRINT".tar*; do
        [ -f "$candidate" ] && EXPORT_FILE=$candidate
    done
    [ -n "$EXPORT_FILE" ] || die "incus image export left no $FINGERPRINT.tar* in $EXPORT_DIR (split image?)"
    actual=$(sha256sum "$EXPORT_FILE" | awk '{print $1}')
    [ "$actual" = "$FINGERPRINT" ] || die "exported $EXPORT_FILE hashes to $actual, not the fingerprint $FINGERPRINT"
    log "exported $EXPORT_FILE ($(du -h "$EXPORT_FILE" | cut -f1))"
fi

if [ -n "$INFO_FILE" ]; then
    {
        printf 'ALIAS=%s\n' "$ALIAS"
        printf 'PROFILE=%s\n' "$PROFILE"
        printf 'MARKER=%s\n' "$MARKER"
        printf 'FINGERPRINT=%s\n' "$FINGERPRINT"
        printf 'BASE_IMAGE=%s\n' "$BASE_IMAGE"
        printf 'BASE_FINGERPRINT=%s\n' "$BASE_FINGERPRINT"
        printf 'COMPRESSION=%s\n' "${COMPRESSION:-default}"
        if [ "$PROFILE" = base ]; then chromium=none; elif [ -f "$CHROMIUM_TARBALL" ]; then chromium=static; else chromium=distro; fi
        printf 'CHROMIUM=%s\n' "$chromium"
        printf 'EXPORT_FILE=%s\n' "$EXPORT_FILE"
    } > "$INFO_FILE"
fi

if [ "$CHECK_SIZE" -eq 1 ]; then
    log "checking the +${SIZE_LIMIT_MB} MB budget"
    GB_BASE_IMAGE="$BASE_IMAGE" GB_OFFICE_IMAGE="$ALIAS" GB_VM_SIZE_LIMIT_MB="$SIZE_LIMIT_MB" \
        "$REPO_ROOT/botserver/images/office/check-size.sh"
fi
