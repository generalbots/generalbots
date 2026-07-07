#!/bin/bash
set -e

echo "╔══════════════════════════════════════════════════╗"
echo "║           General Bots Stack Reset              ║"
echo "╚══════════════════════════════════════════════════╝"
echo ""
echo "This will:"
echo "  • Clean botserver-stack/ and work/ directories"
echo "  • Kill all botserver/botui/botmodels processes"
echo "  • Rebuild botserver and botui from source"
echo "  • Restart the full stack"
echo ""

# Pre-flight check: warn if services are running
BOTSRV=$(pgrep -f 'botserver' 2>/dev/null | head -3 | tr '\n' ' ')
BOTUI=$(pgrep -f 'botui' 2>/dev/null | head -3 | tr '\n' ' ')
if [ -n "$BOTSRV" ] || [ -n "$BOTUI" ]; then
  echo "⚠ WARNING: Services currently running:"
  [ -n "$BOTSRV" ] && echo "  botserver PIDs: $BOTSRV"
  [ -n "$BOTUI" ] && echo "  botui PIDs: $BOTUI"
  echo ""
fi

# Require 'PRUNE' confirmation (safety: must type explicitly to avoid accidental reset)
read -r -p $'Type \e[1;31mPRUNE\e[0m to proceed with reset: ' CONFIRM
if [ "$CONFIRM" != "PRUNE" ]; then
  echo "Reset cancelled."
  exit 1
fi
echo ""

echo "Cleaning up..."
rm -rf botserver-stack/ botserver/botserver-stack/ ./work/ botserver/work/ .env botserver/.env \
  botserver.log botserver/botserver.log botui.log botserver/botui.log botmodels.log botserver/botmodels.log

echo "Killing any remaining botserver/botui/botmodels processes..."

# Save PIDs to files for precise tracking
pgrep -f 'target/debug/botserver' > /tmp/botserver.pid 2>/dev/null || true
pgrep -f 'target/debug/botui' > /tmp/botui.pid 2>/dev/null || true
pgrep -f 'uvicorn.*src.main' > /tmp/botmodels.pid 2>/dev/null || true

pkill -f 'target/debug/botserver' 2>/dev/null || true
pkill -f 'target/debug/botui' 2>/dev/null || true
pkill -f 'uvicorn.*src.main' 2>/dev/null || true
fuser -k 8080/tcp 2>/dev/null || true
fuser -k 3000/tcp 2>/dev/null || true
sleep 3

# Verify processes are gone using PID files
for pidfile in /tmp/botserver.pid /tmp/botui.pid /tmp/botmodels.pid; do
  if [ -f "$pidfile" ]; then
    for pid in $(cat "$pidfile"); do
      if kill -0 "$pid" 2>/dev/null; then
        echo "WARNING: Process $pid still running, sending SIGKILL"
        kill -9 "$pid" 2>/dev/null || true
      fi
    done
    rm -f "$pidfile"
  fi
done

echo "Starting services..."
./restart.sh

echo ""
echo "=== Post-Reset Verification ==="

# Wait for botserver health
echo -n "Waiting for botserver health (up to 180s) ."
for i in $(seq 1 36); do
  STATUS=$(curl -s -o /dev/null -w '%{http_code}' http://localhost:8080/health 2>/dev/null)
  if [ "$STATUS" = "200" ]; then
    echo ""
    echo "✅ botserver health: 200 OK"
    break
  fi
  echo -n "."
  sleep 5
done

# Check botui
BOTUI_PID=$(pgrep -f 'target/debug/botui' 2>/dev/null || echo "not running")
echo "  botui PID: $BOTUI_PID"

# Check botmodels
BOTMODELS_PID=$(pgrep -f 'uvicorn.*src.main' 2>/dev/null || echo "not running")
echo "  botmodels PID: $BOTMODELS_PID"

# Quick log scan for errors (skip MinIO 403 noise)
ERR_COUNT=$(grep -c -E " ERROR | panic" botserver.log 2>/dev/null || echo 0)
echo "  Errors in botserver.log: $ERR_COUNT"
if [ "$ERR_COUNT" -gt 0 ]; then
  echo "  (non-MinIO errors shown below)"
  grep -E " ERROR " botserver.log 2>/dev/null | grep -v "InvalidAccessKeyId\|tokio_backend\|Got HTTP 403\|Retrying" | head -5
fi

echo ""
echo "=== Reset complete! ==="