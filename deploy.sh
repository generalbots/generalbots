#!/bin/bash
# Manual deploy script - run this on alm-ci container
echo "1. Stop..."
sudo incus exec system -- systemctl stop botserver || true
sudo incus exec system -- pkill -x botserver || true
sleep 1

echo "2. Copy..."
sudo incus file push /opt/gbo/work/botserver/target/debug/botserver system:/opt/gbo/bin/botserver --mode=0755

echo "3. Start..."
sudo incus exec system -- systemctl start botserver
sleep 2

echo "4. Verify..."
sudo incus exec system -- pgrep -x botserver && echo "✅ SUCCESS" || echo "❌ FAILED"
