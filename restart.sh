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

# Load operator-provided secrets before choosing the BotModels key, so a value
# from .env/Vault-injected env is honoured. Never fall back to a shipped literal.
set -a; source .env 2>/dev/null; set +a
if [ -z "${BOTMODELS_API_KEY:-}" ] || [ ${#BOTMODELS_API_KEY} -lt 16 ]; then
  if [ -n "${BOTMODELS_API_KEY:-}" ]; then
    echo "WARNING: BOTMODELS_API_KEY is too short; generating an ephemeral key instead" >&2
  fi
  if command -v python3 >/dev/null 2>&1; then
    BOTMODELS_API_KEY=$(python3 -c 'import secrets; print(secrets.token_hex(24))')
  elif command -v openssl >/dev/null 2>&1; then
    BOTMODELS_API_KEY=$(openssl rand -hex 24)
  else
    echo "ERROR: no CSPRNG available (python3 or openssl) to generate BOTMODELS_API_KEY" >&2
    exit 1
  fi
  if [ -z "${BOTMODELS_API_KEY:-}" ]; then
    echo "ERROR: failed to generate a BotModels API key" >&2
    exit 1
  fi
fi
export BOTMODELS_API_KEY
echo "  botmodels API key ready (length ${#BOTMODELS_API_KEY})"

cd botmodels
UVICORN=$(which uvicorn 2>/dev/null || echo "$HOME/.local/bin/uvicorn")
# BotModels authenticates HTTP with API_KEY and voice sockets with ?api_key=.
# One shared key goes to both BotModels and botserver; bind to loopback only so
# there is no guessable default and no external exposure (#2764).
API_KEY="$BOTMODELS_API_KEY" nohup $UVICORN src.main:app --host 127.0.0.1 --port 8085 > ../botmodels.log 2>&1 &
echo "  botmodels PID: $!"
cd ..

for i in $(seq 1 20); do
  if curl -s http://localhost:8085/api/health > /dev/null 2>&1; then
    echo "  botmodels ready"
    break
  fi
  sleep 1
done

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
