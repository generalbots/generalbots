#!/bin/bash
# Build the WSL2 root filesystem of the General Bots Lite agent runtime.
#
# The result is the tarball the desktop app imports as the WSL distro
# `GeneralBotsLite` (`wsl --import GeneralBotsLite <dir> <file> --version 2`):
# Debian 13 (trixie) from a pinned snapshot.debian.org timestamp, systemd as
# init, Incus from Debian main, and a one-time unit that initialises Incus
# (dir storage pool, incusbr0 NAT bridge, no network listener) on first boot.
#
# Reproducible: the package set comes from one snapshot timestamp,
# SOURCE_DATE_EPOCH is that timestamp, mmdebstrap sorts and clamps the tar
# members, and gzip runs with -n. The same inputs give the same bytes.
#
# Usage (as root, on a Debian/Ubuntu host with mmdebstrap and
# debian-archive-keyring installed):
#   scripts/gbgov/build_wsl_rootfs.sh --version 1.0.0 --out dist/rootfs.tar.gz
#
# Options:
#   --snapshot TS    snapshot.debian.org timestamp, YYYYMMDDTHHMMSSZ
#                    (default: $GB_DEBIAN_SNAPSHOT or the pin below)
#   --version V      agent runtime version written into the image (default: dev)
#   --out FILE       output tarball, must end in .tar.gz (required)
#   --info FILE      write KEY=value build facts (incus version, snapshot)
#   --keyring FILE   keyring that verifies the snapshot's Release files
#                    (default: /usr/share/keyrings/debian-archive-keyring.gpg)
#   --dry-run        print the plan and exit
#   -h, --help       this help
set -euo pipefail

HERE=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
# Bump deliberately: a new pin is a new runtime release (security updates).
DEFAULT_SNAPSHOT=20261120T000000Z
SNAPSHOT=${GB_DEBIAN_SNAPSHOT:-$DEFAULT_SNAPSHOT}
SUITE=trixie
VERSION=dev
OUT=
INFO=
KEYRING=/usr/share/keyrings/debian-archive-keyring.gpg
DRY_RUN=0

# Everything beyond minbase. incus-base runs containers only (the `incus`
# package adds QEMU for virtual machines, which WSL does not need);
# dnsmasq-base is only a Recommends of incus-base but incusbr0 needs it for
# DHCP and DNS.
PACKAGES=(
    systemd systemd-sysv dbus
    ca-certificates debian-archive-keyring
    incus-base incus-client dnsmasq-base lxcfs uidmap
    nftables iproute2 squashfs-tools xz-utils zstd
    procps
)

log() { printf '[build_wsl_rootfs] %s\n' "$*"; }
die() { printf '[build_wsl_rootfs] error: %s\n' "$*" >&2; exit 1; }

while [ $# -gt 0 ]; do
    case "$1" in
        --snapshot) SNAPSHOT=${2:?--snapshot needs a value}; shift 2 ;;
        --version) VERSION=${2:?--version needs a value}; shift 2 ;;
        --out) OUT=${2:?--out needs a value}; shift 2 ;;
        --info) INFO=${2:?--info needs a value}; shift 2 ;;
        --keyring) KEYRING=${2:?--keyring needs a value}; shift 2 ;;
        --dry-run) DRY_RUN=1; shift ;;
        -h|--help) sed -n '2,27p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) die "unknown option '$1' (try --help)" ;;
    esac
done

[[ "$SNAPSHOT" =~ ^[0-9]{8}T[0-9]{6}Z$ ]] || die "--snapshot '$SNAPSHOT' is not YYYYMMDDTHHMMSSZ"
[[ "$VERSION" =~ ^[0-9A-Za-z][0-9A-Za-z.+-]*$ ]] || die "--version '$VERSION' has characters outside [0-9A-Za-z.+-]"
[ -n "$OUT" ] || die "--out is required"
case "$OUT" in
    *.tar.gz) ;;
    *) die "--out must end in .tar.gz (the compression every wsl --import accepts)" ;;
esac

SOURCE_DATE_EPOCH=$(date -u -d "${SNAPSHOT:0:4}-${SNAPSHOT:4:2}-${SNAPSHOT:6:2}T${SNAPSHOT:9:2}:${SNAPSHOT:11:2}:${SNAPSHOT:13:2}Z" +%s) \
    || die "cannot convert $SNAPSHOT to an epoch"
export SOURCE_DATE_EPOCH

MIRROR="https://snapshot.debian.org/archive/debian/$SNAPSHOT"
SECURITY="https://snapshot.debian.org/archive/debian-security/$SNAPSHOT"
# Snapshot Release files are past their Valid-Until by design.
SOURCES=(
    "deb [check-valid-until=no] $MIRROR $SUITE main"
    "deb [check-valid-until=no] $MIRROR $SUITE-updates main"
    "deb [check-valid-until=no] $SECURITY $SUITE-security main"
)
INCLUDE=$(IFS=,; printf '%s' "${PACKAGES[*]}")

log "suite=$SUITE snapshot=$SNAPSHOT (epoch $SOURCE_DATE_EPOCH) version=$VERSION out=$OUT"
log "packages: $INCLUDE"
if [ "$DRY_RUN" -eq 1 ]; then
    printf '  source: %s\n' "${SOURCES[@]}"
    log "dry run: would run mmdebstrap, customize with $HERE/rootfs and write $OUT"
    exit 0
fi

[ "$(id -u)" -eq 0 ] || die "run as root (mmdebstrap --mode=root)"
command -v mmdebstrap >/dev/null 2>&1 || die "mmdebstrap is required (apt-get install mmdebstrap)"
[ -f "$KEYRING" ] || die "keyring not found: $KEYRING (apt-get install debian-archive-keyring)"
[ -f "$HERE/rootfs/customize.sh" ] || die "missing $HERE/rootfs/customize.sh"

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
TAR="$WORK/rootfs.tar"

# Read by rootfs/customize.sh (mmdebstrap hooks inherit the environment).
export GB_AGENT_RUNTIME_VERSION="$VERSION"
export GB_DEBIAN_SNAPSHOT="$SNAPSHOT"
export GB_ROOTFS_OVERLAY="$HERE/rootfs/overlay"
export GB_ROOTFS_FACTS="$WORK/facts"

# Documentation and translations are dropped to keep the download small;
# every package's copyright file stays (licence compliance).
mmdebstrap \
    --mode=root \
    --variant=minbase \
    --format=tar \
    --architectures=amd64 \
    --keyring="$KEYRING" \
    --include="$INCLUDE" \
    --aptopt='Acquire::Check-Valid-Until "false"' \
    --aptopt='Acquire::Retries "5"' \
    --aptopt='APT::Install-Recommends "false"' \
    --dpkgopt='path-exclude=/usr/share/man/*' \
    --dpkgopt='path-exclude=/usr/share/info/*' \
    --dpkgopt='path-exclude=/usr/share/lintian/*' \
    --dpkgopt='path-exclude=/usr/share/locale/*' \
    --dpkgopt='path-include=/usr/share/locale/locale.alias' \
    --dpkgopt='path-exclude=/usr/share/doc/*' \
    --dpkgopt='path-include=/usr/share/doc/*/copyright' \
    --customize-hook="sh \"$HERE/rootfs/customize.sh\" \"\$1\"" \
    "$SUITE" "$TAR" "${SOURCES[@]}"

# WSL reads plain and gzip tarballs on every supported Windows build
# (10 2004/19041+ and 11); xz and zstd are not accepted by older WSL releases.
mkdir -p "$(dirname "$OUT")"
log "compressing"
gzip -n -9 -c "$TAR" > "$OUT"

tar -tzf "$OUT" | sed 's#^\./##' > "$WORK/members"
for member in etc/wsl.conf etc/systemd/system/gb-incus-firstboot.service \
    etc/systemd/system/multi-user.target.wants/gb-incus-firstboot.service \
    usr/local/libexec/gb-incus-firstboot usr/local/share/generalbots/incus-preseed.yaml \
    usr/lib/generalbots/agent-runtime-release usr/bin/incus; do
    grep -qxF "$member" "$WORK/members" || die "the tarball has no $member"
done

INCUS_VERSION=$(sed -n 's/^GB_INCUS_VERSION=//p' "$GB_ROOTFS_FACTS")
[ -n "$INCUS_VERSION" ] || die "customize.sh did not record the incus version"
if [ -n "$INFO" ]; then
    {
        printf 'SUITE=%s\n' "$SUITE"
        printf 'SNAPSHOT=%s\n' "$SNAPSHOT"
        printf 'SOURCE_DATE_EPOCH=%s\n' "$SOURCE_DATE_EPOCH"
        printf 'INCUS_VERSION=%s\n' "$INCUS_VERSION"
        printf 'SYSTEMD_VERSION=%s\n' "$(sed -n 's/^GB_SYSTEMD_VERSION=//p' "$GB_ROOTFS_FACTS")"
        printf 'INSTALLED_BYTES=%s\n' "$(sed -n 's/^GB_INSTALLED_BYTES=//p' "$GB_ROOTFS_FACTS")"
    } > "$INFO"
fi
log "rootfs ready: $OUT ($(du -h "$OUT" | cut -f1), incus $INCUS_VERSION)"
