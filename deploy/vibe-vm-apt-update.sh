#!/bin/bash
# #1260 — Daily maintenance for vibe project VMs: apt-upgrade every vibe VM,
# taking a snapshot BEFORE the change and clearing the previous snapshot so
# only one rollback point is ever kept (rotation).
#
# Targets ONLY vibe project VMs — containers named `{project}-development` or
# `{project}-prod` (created by botvibe's vm_lifecycle). Core infra containers
# (bot, proxy, tables, vault, drive, ...) are never touched.
#
# Install as a systemd timer on the HOST (incus lives on the host):
#   /opt/gbo/bin/vibe-vm-apt-update.sh   (this script)
#   /etc/systemd/system/vibe-vm-apt-update.service
#   /etc/systemd/system/vibe-vm-apt-update.timer
set -u

SNAPSHOT_TAG="daily-apt"
LOG_FILE="${VIBE_VM_UPDATE_LOG:-/opt/gbo/logs/vibe-vm-apt-update.log}"
INCUS="${INCUS_BIN:-incus}"

mkdir -p "$(dirname "$LOG_FILE")"
log() { echo "[$(date -Is)] $*" >> "$LOG_FILE"; }

log "=== vibe VM apt maintenance starting ==="

# List containers whose name ends in -development or -prod (vibe project VMs).
mapfile -t VMS < <("$INCUS" list --format csv 2>/dev/null | awk -F, '{
  name=$1
  if (name ~ /-(development|prod)$/) print name
}')

if [ "${#VMS[@]}" -eq 0 ]; then
  log "no vibe VMs found — nothing to do"
  exit 0
fi

for vm in "${VMS[@]}"; do
  log "--- processing $vm ---"

  # Snapshot rotation: remove the previous daily snapshot (if any), then
  # snapshot the CURRENT state so a broken upgrade can be rolled back.
  "$INCUS" snapshot delete "$vm/$SNAPSHOT_TAG" >/dev/null 2>&1
  if ! "$INCUS" snapshot create "$vm/$SNAPSHOT_TAG" >/dev/null 2>&1; then
    log "WARN: could not snapshot $vm before upgrade — skipping"
    continue
  fi
  log "snapshot $SNAPSHOT_TAG created for $vm"

  # apt update + upgrade inside the VM. Accept both apt-get and apt.
  if "$INCUS" exec "$vm" -- bash -c 'command -v apt-get >/dev/null 2>&1' >/dev/null 2>&1; then
    if "$INCUS" exec "$vm" -- bash -c 'DEBIAN_FRONTEND=noninteractive apt-get update -qq && DEBIAN_FRONTEND=noninteractive apt-get upgrade -y -qq' >> "$LOG_FILE" 2>&1; then
      log "apt upgrade OK for $vm"
    else
      log "ERROR: apt upgrade failed for $vm — snapshot kept for rollback"
    fi
  else
    log "WARN: no apt-get in $vm — skipped"
  fi
done

log "=== vibe VM apt maintenance complete ==="
