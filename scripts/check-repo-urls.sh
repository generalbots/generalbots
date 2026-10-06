#!/usr/bin/env bash
# CI guard (#1473): the canonical project is the unified workspace
# generalbots/generalbots. The pre-merge generalbots/botserver URL must not
# reappear in sources or documentation.
set -euo pipefail
cd "$(dirname "$0")/.."
hits=$(git grep -niE 'github\.com/generalbots?/botserver' -- \
  '*.rs' '*.md' '*.html' '*.toml' 2>/dev/null || true)
if [ -n "$hits" ]; then
  echo "FAIL: stale pre-merge repository URL found (use github.com/generalbots/generalbots):" >&2
  echo "$hits" >&2
  exit 1
fi
echo "OK: no stale pre-merge repository URLs"
