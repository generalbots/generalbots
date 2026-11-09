#!/usr/bin/env bash
# Install the bundled skill pack into a bot (#2063).
#
# Every `botskills/<category>/<slug>/SKILL.md` is posted to
# `POST /api/sources/skills`, which parses the front-matter, hashes the document
# and stores it with `enabled = TRUE` — imported skills are on by default, so
# the prompt advertises them as `name: description` immediately and the model
# pulls a body with `skills.find {query}`.
#
# Usage:
#   scripts/install-skills.sh --token "$TOKEN" [--bot default] [--url http://localhost:8080]
#   scripts/install-skills.sh --token "$TOKEN" --dry-run
#
# Env fallbacks: GB_TOKEN, GB_BOT_ID, GB_URL, GB_SKILLS_DIR.
# The token is a normal session bearer token; nothing is stored on disk.
set -euo pipefail

url="${GB_URL:-http://localhost:8080}"
bot="${GB_BOT_ID:-default}"
token="${GB_TOKEN:-}"
dry_run=0
root="${GB_SKILLS_DIR:-$(cd "$(dirname "$0")/.." && pwd)/botskills}"

while [ $# -gt 0 ]; do
    case "$1" in
        --token) token="${2:-}"; shift 2 ;;
        --bot) bot="${2:-}"; shift 2 ;;
        --url) url="${2:-}"; shift 2 ;;
        --dir) root="${2:-}"; shift 2 ;;
        --dry-run) dry_run=1; shift ;;
        -h|--help) sed -n '2,18p' "$0"; exit 0 ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
done

if [ ! -d "$root" ]; then
    echo "skills directory not found: $root" >&2
    exit 1
fi
if [ "$dry_run" -eq 0 ] && [ -z "$token" ]; then
    echo "a bearer token is required (--token or GB_TOKEN)" >&2
    exit 1
fi

installed=0
failed=0
while IFS= read -r file; do
    name=$(awk '/^name:/{sub(/^name:[ ]*/, ""); print; exit}' "$file")
    if [ -z "$name" ]; then
        echo "SKIP (no name in front-matter): $file" >&2
        failed=$((failed + 1))
        continue
    fi
    if [ "$dry_run" -eq 1 ]; then
        echo "WOULD INSTALL $name ($file)"
        installed=$((installed + 1))
        continue
    fi
    payload=$(python3 - "$file" "$bot" <<'PY'
import json, sys
path, bot = sys.argv[1], sys.argv[2]
with open(path, "r", encoding="utf-8") as handle:
    content = handle.read()
print(json.dumps({"bot_id": bot, "content": content}))
PY
)
    status=$(curl -sS -o /tmp/gb_skill_install.json -w '%{http_code}' \
        -X POST "$url/api/sources/skills" \
        -H "Authorization: Bearer $token" \
        -H "Content-Type: application/json" \
        -d "$payload")
    if [ "$status" -ge 200 ] && [ "$status" -lt 300 ]; then
        echo "installed $name"
        installed=$((installed + 1))
    else
        echo "FAILED $name (HTTP $status): $(cat /tmp/gb_skill_install.json)" >&2
        failed=$((failed + 1))
    fi
done < <(find "$root" -name SKILL.md | sort)

echo "----"
echo "installed=$installed failed=$failed bot=$bot url=$url"
[ "$failed" -eq 0 ]
