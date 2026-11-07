#!/usr/bin/env bash
# Rebuild the BotBook and verify it produces a usable search index.
#
# botbook/book/ is generated (and git-ignored) on purpose: the published site
# is built from botbook/src, not from a checked-in HTML tree. A source edit
# that breaks a page or a SUMMARY link is therefore invisible until a build
# runs, and the search index (searchindex-<hash>.js) is the part most likely to
# go stale or missing.
#
# Usage: scripts/botbook_reindex.sh
# Exit codes: 0 the book and search index were produced; 1 the build failed,
# the index is missing/empty, or SUMMARY.md points at a page that does not
# exist.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if ! command -v mdbook >/dev/null 2>&1; then
    echo "botbook_reindex: mdbook is not installed (need 0.5.x)" >&2
    exit 1
fi

mdbook build botbook

index="$(find botbook/book -maxdepth 1 -name 'searchindex-*.js' -print -quit)"
if [ -z "$index" ]; then
    echo "botbook_reindex: mdbook produced no search index" >&2
    exit 1
fi

size="$(wc -c < "$index" | tr -d ' ')"
if [ "$size" -lt 1000 ]; then
    echo "botbook_reindex: search index is suspiciously small ($size bytes)" >&2
    exit 1
fi

echo "botbook_reindex: $index ($size bytes)"

# SUMMARY.md is the only TOC mdBook renders; a link to a missing file drops the
# page from the sidebar and 404s for readers.
python3 - <<'PY'
import re
import sys
from pathlib import Path

src = Path("botbook/src")
summary = (src / "SUMMARY.md").read_text(encoding="utf-8")
links = re.findall(r"\]\((\./[^)#]+\.md)\)", summary)
missing = [link for link in links if not (src / link[2:]).exists()]
if missing:
    print("botbook_reindex: SUMMARY.md points at missing pages:", file=sys.stderr)
    for link in missing[:20]:
        print(f"  {link}", file=sys.stderr)
    sys.exit(1)
print(f"botbook_reindex: {len(links)} SUMMARY links resolve")
PY

echo "botbook_reindex: book and search index rebuilt"
