#!/usr/bin/env bash
# One entrypoint for a reproducible BotBook build: generate every reference
# page from source, verify the guards, build MkDocs, restore old mdBook URLs.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VENV="${BOTBOOK_VENV:-/tmp/botbook-docs-venv}"
MKDOCS="$VENV/bin/mkdocs"

if [[ ! -x "$MKDOCS" ]]; then
  python3 -m venv "$VENV"
  "$VENV/bin/pip" install mkdocs-material mkdocs-literate-nav
fi

cd "$ROOT"
python3 scripts/gen_api_reference.py
python3 scripts/gen_api_ws.py
python3 scripts/gen_basic_reference.py
python3 scripts/gen_catalogs.py
python3 scripts/gen_schema_reference.py
python3 scripts/gen_source_catalogs.py

python3 scripts/botbook_guards.py --check
for gen in gen_api_reference gen_api_ws gen_basic_reference gen_catalogs \
           gen_schema_reference gen_source_catalogs; do
  python3 "scripts/$gen.py" --check
done

(cd botbook && NO_MKDOCS_2_WARNING=1 DISABLE_MKDOCS_2_WARNING=true "$MKDOCS" build)
printf 'BotBook built: %s HTML files in botbook/book\n' "$(find botbook/book -name '*.html' | wc -l)"
