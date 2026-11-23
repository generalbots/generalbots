#!/bin/bash

echo "=== Fast Restart: botserver + botui + botmodels ==="

killall -9 botserver botui uvicorn 2>/dev/null || true
pkill -x botserver || true
pkill -x botui || true
pkill -f "target/debug/botserver" || true
pkill -f "botmodels" || true
pkill -f "uvicorn.*src.main" || true
sleep 1


# Optional extra feature set for product/dev profiles (e.g. sources).
FEATURES_ARG=""
if [ -n "${GB_BOTSERVER_FEATURES:-}" ]; then
  FEATURES_ARG="--features ${GB_BOTSERVER_FEATURES}"
fi
cargo build -p botserver ${FEATURES_ARG}
cargo build -p botui

cd botmodels
UVICORN=$(which uvicorn 2>/dev/null || echo "$HOME/.local/bin/uvicorn")
# BotModels authenticates HTTP with API_KEY and voice sockets with ?api_key=.
# #2764 — generate a random per-run key when the operator did not supply one,
# never fall back to "change-me". The same key is exported to botserver below,
# and the service binds to loopback (only botserver on this host calls it).
GB_BOTMODELS_RUNTIME_KEY="${BOTMODELS_API_KEY:-$(python3 -c 'import secrets; print(secrets.token_hex(24))' 2>/dev/null || openssl rand -hex 24)}"
export GB_BOTMODELS_RUNTIME_KEY
API_KEY="$GB_BOTMODELS_RUNTIME_KEY" nohup $UVICORN src.main:app --host 127.0.0.1 --port 8085 > ../botmodels.log 2>&1 &
echo "  botmodels PID: $!"
cd ..

for i in $(seq 1 20); do
  if curl -s http://localhost:8085/api/health > /dev/null 2>&1; then
    echo "  botmodels ready"
    break
  fi
  sleep 1
done

set -a; source .env 2>/dev/null; set +a
# Keep botserver's key identical to the one botmodels was started with, even
# when .env defines a different BOTMODELS_API_KEY.
export BOTMODELS_API_KEY="$GB_BOTMODELS_RUNTIME_KEY"
VAULT_BIN="${VAULT_BIN:-./botserver-stack/bin/vault/vault}"
# Only resolve Drive credentials when they were not provided by the caller;
# never overwrite working env with an empty Vault read.
if [ -z "${MINIO_ACCESS_KEY:-}" ]; then
  MINIO_ACCESS_KEY=$($VAULT_BIN kv get -field=accesskey -tls-skip-verify secret/gbo/drive 2>/dev/null)
  MINIO_SECRET_KEY=$($VAULT_BIN kv get -field=secret -tls-skip-verify secret/gbo/drive 2>/dev/null)
fi
MINIO_ENDPOINT="${MINIO_ENDPOINT:-http://127.0.0.1:9100}"
MINIO_BUCKET="${MINIO_BUCKET:-default.gbai}"
MINIO_SERVER="${MINIO_SERVER:-http://127.0.0.1:9100}"
BOTMODELS_HOST="http://localhost:8085" RUST_LOG=info nohup ./target/debug/botserver --noconsole > botserver.log 2>&1 &
echo "  botserver PID: $!"

sleep 2

nohup ./target/debug/botui > botui.log 2>&1 &
echo "  botui PID: $!"

echo "Done. botserver=$(pgrep -f 'botserver --noconsole') botui=$(pgrep -f botui) botmodels=$(pgrep -f botmodels)"
