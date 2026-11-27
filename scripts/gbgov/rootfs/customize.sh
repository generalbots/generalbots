#!/bin/sh
# mmdebstrap customize hook for the General Bots Lite WSL root filesystem.
# Called by scripts/gbgov/build_wsl_rootfs.sh as `customize.sh <rootdir>`;
# GB_AGENT_RUNTIME_VERSION, GB_DEBIAN_SNAPSHOT, GB_ROOTFS_OVERLAY and
# GB_ROOTFS_FACTS come from that script.
set -eu

ROOT=${1:?usage: customize.sh <rootdir>}
OVERLAY=${GB_ROOTFS_OVERLAY:?}
FACTS=${GB_ROOTFS_FACTS:?}
VERSION=${GB_AGENT_RUNTIME_VERSION:?}
SNAPSHOT=${GB_DEBIAN_SNAPSHOT:?}

# Overlay files with explicit owners and modes: the checkout's modes depend
# on the machine that cloned the repository.
install -D -o root -g root -m 0644 "$OVERLAY/etc/wsl.conf" "$ROOT/etc/wsl.conf"
install -D -o root -g root -m 0644 "$OVERLAY/etc/systemd/system/gb-incus-firstboot.service" \
    "$ROOT/etc/systemd/system/gb-incus-firstboot.service"
install -D -o root -g root -m 0755 "$OVERLAY/usr/local/libexec/gb-incus-firstboot" \
    "$ROOT/usr/local/libexec/gb-incus-firstboot"
install -D -o root -g root -m 0644 "$OVERLAY/usr/local/share/generalbots/incus-preseed.yaml" \
    "$ROOT/usr/local/share/generalbots/incus-preseed.yaml"

chroot "$ROOT" systemctl enable gb-incus-firstboot.service
# systemd-firstboot prompts on the console for locale/keymap/root password
# when the machine id is new; a WSL distro has no console to answer it.
ln -sf /dev/null "$ROOT/etc/systemd/system/systemd-firstboot.service"

# Root's subordinate id range for unprivileged containers (the range Incus
# documents; it is not added by the Debian package).
for f in subuid subgid; do
    touch "$ROOT/etc/$f"
    grep -q '^root:' "$ROOT/etc/$f" || echo 'root:1000000:1000000000' >> "$ROOT/etc/$f"
done

# After import the distro updates from the live mirrors, not the snapshot.
rm -f "$ROOT/etc/apt/sources.list"
mkdir -p "$ROOT/etc/apt/sources.list.d"
cat > "$ROOT/etc/apt/sources.list.d/debian.sources" <<'EOF'
Types: deb
URIs: https://deb.debian.org/debian
Suites: trixie trixie-updates
Components: main
Signed-By: /usr/share/keyrings/debian-archive-keyring.gpg

Types: deb
URIs: https://deb.debian.org/debian-security
Suites: trixie-security
Components: main
Signed-By: /usr/share/keyrings/debian-archive-keyring.gpg
EOF

INCUS_VERSION=$(chroot "$ROOT" dpkg-query -W -f='${Version}' incus-base)
SYSTEMD_VERSION=$(chroot "$ROOT" dpkg-query -W -f='${Version}' systemd)

# What the desktop app reads to tell which runtime a distro carries.
mkdir -p "$ROOT/usr/lib/generalbots"
cat > "$ROOT/usr/lib/generalbots/agent-runtime-release" <<EOF
GB_AGENT_RUNTIME_VERSION=$VERSION
GB_DEBIAN_SUITE=trixie
GB_DEBIAN_SNAPSHOT=$SNAPSHOT
GB_INCUS_VERSION=$INCUS_VERSION
EOF
chmod 0644 "$ROOT/usr/lib/generalbots/agent-runtime-release"

# Every imported distro must generate its own machine id on first boot.
: > "$ROOT/etc/machine-id"
rm -f "$ROOT/var/lib/dbus/machine-id"
find "$ROOT/var/log" -type f -delete
rm -f "$ROOT"/var/cache/debconf/*-old "$ROOT"/var/lib/dpkg/*-old

{
    printf 'GB_INCUS_VERSION=%s\n' "$INCUS_VERSION"
    printf 'GB_SYSTEMD_VERSION=%s\n' "$SYSTEMD_VERSION"
    printf 'GB_INSTALLED_BYTES=%s\n' "$(du -sbx "$ROOT" | cut -f1)"
} > "$FACTS"
