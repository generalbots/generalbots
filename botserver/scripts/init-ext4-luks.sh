#!/bin/bash
# Initialize ext4 with LUKS encryption (Issue #529)
# Replaces BTRFS which got corrupted with no free space.

set -e

DISK="${1:-/dev/sdb}"
MOUNT="${2:-/opt/gbo/data}"
LABEL="${3:-gbo-data}"

echo "=== Initializing ext4 with LUKS on $DISK ==="

# Check if already initialized
if cryptsetup isLuks "$DISK" 2>/dev/null; then
    echo "LUKS already configured on $DISK"
    cryptsetup luksOpen "$DISK" "$LABEL" 2>/dev/null || true
else
    echo "Formatting $DISK with LUKS..."
    cryptsetup luksFormat "$DISK"
    echo "Opening LUKS container..."
    cryptsetup luksOpen "$DISK" "$LABEL"
fi

echo "Creating ext4 filesystem..."
mkfs.ext4 -L "$LABEL" "/dev/mapper/$LABEL"

echo "Mounting to $MOUNT..."
mkdir -p "$MOUNT"
mount "/dev/mapper/$LABEL" "$MOUNT"

echo "Adding to fstab..."
if ! grep -q "$LABEL" /etc/fstab; then
    echo "/dev/mapper/$LABEL $MOUNT ext4 defaults,noatime,nodiratime 0 2" >> /etc/fstab
fi

echo "Done. ext4 with LUKS ready at $MOUNT"
