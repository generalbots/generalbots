#!/bin/bash

echo "=== Fast Restart: botserver + botmodels only ==="

# Kill only the app services, keep infra running
pkill -f "botserver --noconsole" || true
pkill -f "botmodels" || true

# Clean logs
rm -f botserver.log botmodels.log

# Build only botserver (botui likely already built)
cargo build -p botserver

# Start botmodels
cd botmodels
source venv/bin/activate
uvicorn src.main:app --host 0.0.0.0 --port 8085 > ../botmodels.log 2>&1 &
echo "  botmodels PID: $!"
cd ..

# Wait for botmodels
for i in $(seq 1 20); do
  if curl -s http://localhost:8085/api/health > /dev/null 2>&1; then
    echo "  botmodels ready"
    break
  fi
  sleep 1
done

# Start botserver (keep botui running if already up)
if ! pgrep -f "botui" > /dev/null; then
  echo "Starting botui..."
  cargo build -p botui
  cd botui
  BOTSERVER_URL="http://localhost:8080" ./target/debug/botui > ../botui.log 2>&1 &
  echo "  botui PID: $!"
  cd ..
fi

# Start botserver
BOTMODELS_HOST="http://localhost:8085" BOTMODELS_API_KEY="starter" RUST_LOG=info ./target/debug/botserver --noconsole > botserver.log 2>&1 &
echo "  botserver PID: $!"

# Quick health check
sleep 2
curl -s http://localhost:8080/health > /dev/null 2>&1 && echo "✅ botserver ready" || echo "❌ botserver failed"

echo "Done. botserver $(pgrep -f 'botserver --noconsole') botui $(pgrep -f botui) botmodels $(pgrep -f botmodels)"
