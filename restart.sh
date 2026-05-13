#!/bin/bash

echo "=== Fast Restart: botserver + botui + botmodels ==="

killall -9 botserver botui uvicorn 2>/dev/null || true
pkill -f "botserver" || true
pkill -f "botui" || true
pkill -f "botmodels" || true
pkill -f "uvicorn.*src.main" || true
sleep 1

rm -f botserver.log botmodels.log botui.log

cargo build -p botserver
cargo build -p botui

cd botmodels
UVICORN=$(which uvicorn 2>/dev/null || echo "$HOME/.local/bin/uvicorn")
nohup $UVICORN src.main:app --host 0.0.0.0 --port 8085 > ../botmodels.log 2>&1 &
echo "  botmodels PID: $!"
cd ..

for i in $(seq 1 20); do
  if curl -s http://localhost:8085/api/health > /dev/null 2>&1; then
    echo "  botmodels ready"
    break
  fi
  sleep 1
done

BOTMODELS_HOST="http://localhost:8085" BOTMODELS_API_KEY="starter" RUST_LOG=info nohup ./target/debug/botserver --noconsole > botserver.log 2>&1 &
echo "  botserver PID: $!"

sleep 2

nohup ./target/debug/botui > botui.log 2>&1 &
echo "  botui PID: $!"

sleep 3
curl -s http://localhost:8080/health > /dev/null 2>&1 && echo "✅ botserver ready" || echo "❌ botserver failed"

echo "Done. botserver=$(pgrep -f 'botserver --noconsole') botui=$(pgrep -f botui) botmodels=$(pgrep -f botmodels)"
