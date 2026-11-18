#!/usr/bin/env bash
# Single entry point for the locale-catalogue gate (#2088, #2542).
#
# Runs both checks that protect the UI from rendering a raw key:
#   1. scripts/i18n_lint.py --check   catalog hygiene + keys the UI asks for
#   2. scripts/i18n_pseudo.py --check the pseudo-locale generator stays valid
#
# Usage:
#   scripts/check-i18n.sh            # gate (exit 1 on a new violation)
#   scripts/check-i18n.sh --report   # human report only, never fails
#
# CI runs the plain form (.github/workflows/ci.yml, job "i18n catalog gate").
set -euo pipefail

cd "$(dirname "$0")/.."

mode="${1:-}"
if [[ "$mode" == "--report" ]]; then
  python3 scripts/i18n_lint.py
  python3 scripts/i18n_pseudo.py --check
  exit 0
fi

python3 scripts/i18n_lint.py --check
python3 scripts/i18n_pseudo.py --check
echo "i18n gate: OK"
