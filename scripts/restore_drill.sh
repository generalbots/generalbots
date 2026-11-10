#!/usr/bin/env bash
# Restore drill (#2117) — take a real dump, restore it into a throwaway
# database, and prove the copy is complete and still tenant-isolated.
#
# Usage:
#   scripts/restore_drill.sh --dsn postgres://user:pass@host:5432/botserver
#   scripts/restore_drill.sh --dump /opt/gbo/data/backup/botserver.dump --dsn "$URL"
#   scripts/restore_drill.sh --dsn "$URL" --verify-only      # assert, no restore
#   scripts/restore_drill.sh --self-test                     # no server needed
#
# What it does, in order:
#   1. takes `pg_dump -Fc` of the source (unless --dump names an existing dump)
#      and records the source row counts as a manifest;
#   2. creates the throwaway database `gb_restore_drill_<pid>` on the same
#      server, restores the dump into it with `pg_restore --exit-on-error`;
#   3. asserts, on the restored copy, that the core tables exist, that the
#      migration history survived, that every table has exactly the source row
#      count, and that every cross-tenant probe returns zero rows;
#   4. drops the throwaway database (unless --keep).
#
# NEVER point --dsn at production: the drill creates and drops databases and
# needs a role allowed to do so. Restore into a copy, never into the live DB.
#
# Exit codes: 0 the drill passed, 1 a mismatch, 2 a missing tool or bad input.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
checks="$here/restore_drill_checks.py"

dsn="${DATABASE_URL:-}"
dump=""
expected=""
report=""
min_migrations="1"
keep="0"
verify_only="0"
self_test="0"

usage() {
  sed -n '2,24p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
}

while [ $# -gt 0 ]; do
  case "$1" in
    --dump) dump="${2:?--dump needs a file}"; shift 2 ;;
    --dsn|--database-url) dsn="${2:?--dsn needs a connection URL}"; shift 2 ;;
    --expected) expected="${2:?--expected needs a file}"; shift 2 ;;
    --report) report="${2:?--report needs a file}"; shift 2 ;;
    --min-migrations) min_migrations="${2:?--min-migrations needs a number}"; shift 2 ;;
    --keep) keep="1"; shift ;;
    --verify-only) verify_only="1"; shift ;;
    --self-test) self_test="1"; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "error: unknown argument: $1 (try --help)" >&2; exit 2 ;;
  esac
done

if [ "$self_test" = "1" ]; then
  exec python3 "$checks" --self-test
fi

if [ -z "$dsn" ]; then
  echo "error: pass --dsn (or set DATABASE_URL) — a throwaway database only" >&2
  exit 2
fi

# Never print credentials: keep scheme/host/port/database, drop the userinfo.
masked="$(printf '%s' "$dsn" | sed -E 's#^([a-zA-Z][a-zA-Z0-9+.-]*)://[^@/]*@#\1://***@#')"
scratch="gb_restore_drill_$$"
workdir="$(mktemp -d "${TMPDIR:-/tmp}/gb-restore-drill.XXXXXX")"
created_dump=""
scratch_created="0"
report="${report:-/tmp/gb-restore-drill-$(date -u +%Y%m%dT%H%M%SZ).md}"

needed=(psql python3)
if [ "$verify_only" = "0" ]; then
  needed+=(pg_dump pg_restore)
fi
missing=()
for tool in "${needed[@]}"; do
  command -v "$tool" >/dev/null 2>&1 || missing+=("$tool")
done
if [ ${#missing[@]} -gt 0 ]; then
  echo "error: missing required tool(s): ${missing[*]} — install the PostgreSQL client" >&2
  exit 2
fi

drop_scratch() {
  if [ "$scratch_created" = "1" ]; then
    if psql "$dsn" -v ON_ERROR_STOP=1 -q -c "drop database if exists \"$scratch\" with (force)" >/dev/null 2>&1 ||
       psql "$dsn" -v ON_ERROR_STOP=1 -q -c "drop database if exists \"$scratch\"" >/dev/null 2>&1; then
      echo "-- dropped throwaway database: $scratch"
    else
      echo "warning: could not drop $scratch — drop it by hand: dropdb $scratch" >&2
    fi
    scratch_created="0"
  fi
}

cleanup() {
  status=$?
  if [ "$keep" = "0" ]; then
    drop_scratch
    if [ -n "$created_dump" ]; then
      rm -f "$created_dump" "$created_dump.manifest.json"
    fi
  fi
  rm -rf "$workdir"
  exit "$status"
}
trap cleanup EXIT

echo "== restore drill: $masked"

if [ "$verify_only" = "1" ]; then
  echo "-- mode: verify-only (no dump, no restore)"
  args=(--dsn "$dsn" --label "verify-only (no restore)" --report "$report" --min-migrations "$min_migrations")
  if [ -n "$expected" ]; then
    args+=(--expected "$expected")
  fi
  set +e
  python3 "$checks" "${args[@]}"
  status=$?
  set -e
  echo "report: $report"
  exit "$status"
fi

if [ -n "$dump" ] && [ ! -f "$dump" ]; then
  echo "error: dump not found: $dump" >&2
  exit 2
fi

if [ -z "$dump" ]; then
  dump="${TMPDIR:-/tmp}/gb-restore-drill-$$.dump"
  created_dump="$dump"
  echo "-- dump: $masked -> $dump"
  pg_dump -Fc -d "$dsn" -f "$dump"
  expected="${expected:-$dump.manifest.json}"
  python3 "$checks" --dsn "$dsn" --manifest "$expected"
elif [ -z "$expected" ] && [ -f "${dump}.manifest.json" ]; then
  expected="${dump}.manifest.json"
fi
if [ -z "$expected" ]; then
  echo "note: no manifest for $(basename "$dump") — asserting schema and tenancy only;" >&2
  echo "      pass --expected <file> (or keep <dump>.manifest.json next to it) for row counts" >&2
fi

note="\`$(basename "$dump")\` restored with pg_restore --exit-on-error"

echo "-- throwaway database: $scratch"
psql "$dsn" -v ON_ERROR_STOP=1 -q -c "create database \"$scratch\""
scratch_created="1"
scratch_url="$(python3 "$checks" --scratch-url "$dsn" --name "$scratch")"

echo "-- restore: $(basename "$dump") -> $scratch"
if ! pg_restore --no-owner --no-privileges --exit-on-error -d "$scratch_url" "$dump"; then
  echo "error: pg_restore failed — the dump is not restorable into an empty database" >&2
  exit 1
fi

args=(--dsn "$scratch_url" --label "pg_dump -Fc → pg_restore (throwaway \`$scratch\`)"
      --report "$report" --note "$note" --min-migrations "$min_migrations"
      --out "$workdir/restored.json")
if [ -n "$expected" ]; then
  args+=(--expected "$expected")
fi

set +e
python3 "$checks" "${args[@]}"
status=$?
set -e

if [ "$keep" = "1" ]; then
  echo "-- kept: database \"$scratch\" and dump $dump"
else
  drop_scratch
fi
echo "report: $report"

if [ "$status" -ne 0 ]; then
  echo "error: restore drill FAILED — the restored copy does not match the source" >&2
  exit 1
fi
echo "restore drill PASSED"
