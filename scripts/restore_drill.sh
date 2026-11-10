#!/usr/bin/env bash
# Restore drill for #2117 — migrations, tenant isolation and recovery on one
# production-like dataset, scripted so it can run monthly or per release.
#
# Usage:
#   DATABASE_URL=postgres://user:pass@host:5432/db scripts/restore_drill.sh
#   scripts/restore_drill.sh --database-url URL --scale 10 --report /tmp/drill.md
#
# The target MUST be a throwaway database: the drill migrates it from scratch,
# loads ~25k deterministic rows (seed 2117) and creates/drops a scratch database
# for the pg_dump/pg_restore round-trip. Never point it at production.
#
# Exit codes: 0 the drill passed, 1 a step failed, 2 a required tool is missing.
set -euo pipefail

DATABASE_URL_ARG="${DATABASE_URL:-}"
DATASET_DIR="${INTEGRITY_DATASET:-/tmp/gb-integrity}"
SCALE="1.0"
REPORT=""
SKIP_MIGRATE="0"
ALLOW_SKIPS="0"

while [ $# -gt 0 ]; do
  case "$1" in
    --database-url) DATABASE_URL_ARG="$2"; shift 2 ;;
    --dataset) DATASET_DIR="$2"; shift 2 ;;
    --scale) SCALE="$2"; shift 2 ;;
    --report) REPORT="$2"; shift 2 ;;
    --skip-migrate) SKIP_MIGRATE="1"; shift ;;
    --allow-skips) ALLOW_SKIPS="1"; shift ;;
    -h|--help) sed -n '2,16p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

if [ -z "$DATABASE_URL_ARG" ]; then
  echo "error: pass --database-url or set DATABASE_URL (throwaway instance only)" >&2
  exit 2
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

missing=()
for tool in psql pg_dump pg_restore python3 cargo; do
  command -v "$tool" >/dev/null 2>&1 || missing+=("$tool")
done
if [ "$SKIP_MIGRATE" = "0" ]; then
  command -v diesel >/dev/null 2>&1 || missing+=("diesel")
fi
if [ ${#missing[@]} -gt 0 ]; then
  echo "error: missing required tool(s): ${missing[*]}" >&2
  exit 2
fi

# Never print credentials: keep scheme/host/port/database, drop the userinfo.
masked="$(printf '%s' "$DATABASE_URL_ARG" | sed -E 's#^([a-z]+)://[^@]*@#\1://***@#')"
[ -n "$REPORT" ] || REPORT="/tmp/gb-restore-drill-$(date +%Y%m%dT%H%M%S).md"
log="$(mktemp /tmp/gb-restore-drill-log.XXXXXX)"

echo "== restore drill: $masked (scale $SCALE)"
if [ "$SKIP_MIGRATE" = "0" ]; then
  echo "-- migrations"
  ( cd botserver && DATABASE_URL="$DATABASE_URL_ARG" diesel migration run )
fi

echo "-- dataset"
python3 scripts/gen_integrity_dataset.py --out "$DATASET_DIR" --scale "$SCALE" --report

echo "-- load"
psql "$DATABASE_URL_ARG" -v ON_ERROR_STOP=1 -q -f "$DATASET_DIR/integrity_dataset.sql"

echo "-- suite (migrations, schema, tenancy, restore)"
export DATABASE_URL="$DATABASE_URL_ARG"
export INTEGRITY_DATASET="$DATASET_DIR"
if [ "$ALLOW_SKIPS" = "0" ]; then
  export INTEGRITY_STRICT=1
fi
set +e
cargo test -p bottest --features integration --test integration data_integrity \
  -- --nocapture --test-threads=1 2>&1 | tee "$log"
status=${PIPESTATUS[0]}
set -e

rows="$(python3 - "$DATASET_DIR/integrity_manifest.json" <<'PY'
import json, sys
manifest = json.load(open(sys.argv[1], encoding="utf-8"))
print(f"{manifest['total_rows']} rows, seed {manifest['seed']}, scale {manifest['scale']}")
PY
)"
rto="$(grep -o 'RTO [0-9.]*s[^·]*' "$log" | tail -1 || true)"
rpo="$(grep -o 'RPO .*' "$log" | tail -1 || true)"

{
  echo "# Restore drill — $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo
  echo "- target: \`$masked\` (throwaway)"
  echo "- dataset: $rows"
  echo "- suite: \`cargo test -p bottest --features integration --test integration data_integrity\`"
  echo "- result: $([ "$status" -eq 0 ] && echo PASS || echo FAIL)"
  echo "- ${rto:-RTO not reported}"
  echo "- ${rpo:-RPO not reported}"
  echo
  echo '```'
  grep -E '^(ok:|SKIP|test result)' "$log" | tail -20 || true
  echo '```'
} > "$REPORT"

echo "report: $REPORT"
if [ "$status" -ne 0 ]; then
  echo "drill FAILED — see $log" >&2
  exit 1
fi
echo "drill PASSED"
