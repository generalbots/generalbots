#!/usr/bin/env bash
# CI guard (#1139): server-rendered fragments must never emit hx-get
# against POST-only /suite fragment/modal/form routes.
set -euo pipefail
cd "$(dirname "$0")/.."
if grep -rEn 'hx-get="/suite/(docs|slides|sheet)/(fragments|modals|forms)' \
     botserver/crates/botdocs/src botserver/crates/botslides/src botserver/crates/botsheet/src 2>/dev/null; then
  echo "FAIL: hx-get found on POST-only suite fragment routes (use hx-post + json-enc)." >&2
  exit 1
fi
echo "OK: no hx-get on POST-only fragment routes"
