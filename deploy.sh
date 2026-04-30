#!/bin/bash
# Manual deploy script - run this on alm-ci container
echo "0. Backup..."
sudo incus snapshot create system backup-before-update-$(date +%Y%m%d-%H%M)

echo "1. Stop..."
sudo incus exec system -- systemctl stop botserver || true
sudo incus exec system -- systemctl stop ui || true
sudo incus exec system -- pkill -x botserver || true
sudo incus exec system -- pkill -x botui || true
sleep 1

echo "2. Copy..."
sudo incus file push /opt/gbo/work/botserver/target/debug/botserver system:/opt/gbo/bin/botserver --mode=0755
sudo incus file push /opt/gbo/work/botui/target/debug/botui system:/opt/gbo/bin/botui --mode=0755

echo "3. Start..."
sudo incus exec system -- systemctl start botserver
sudo incus exec system -- systemctl start ui
sleep 2

echo "4. Verify..."
sudo incus exec system -- pgrep -x botserver && echo "✅ botserver: SUCCESS" || echo "❌ botserver: FAILED"
sudo incus exec system -- pgrep -x botui && echo "✅ botui: SUCCESS" || echo "❌ botui: FAILED"
