#!/usr/bin/env bash
# Rebuild the BotBook with MkDocs and verify the published artifacts.
#
# botbook/book/ is generated (and git-ignored) on purpose: the published site is
# built from botbook/src, not from a checked-in HTML tree. This script proves
# that the build works, the search index exists, the generated nav is current
# and every SUMMARY.md link resolves to a real page.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if ! command -v mkdocs >/dev/null 2>&1; then
    echo "botbook_reindex: mkdocs is not installed (need MkDocs 1.6 + Material)" >&2
    exit 1
fi

python3 scripts/gen_source_catalogs.py --check
(cd botbook && NO_MKDOCS_2_WARNING=1 DISABLE_MKDOCS_2_WARNING=true mkdocs build)

index="botbook/book/search/search_index.json"
if [ ! -s "$index" ]; then
    echo "botbook_reindex: MkDocs produced no search index" >&2
    exit 1
fi
size="$(wc -c < "$index" | tr -d ' ')"
if [ "$size" -lt 1000 ]; then
    echo "botbook_reindex: search index is suspiciously small ($size bytes)" >&2
    exit 1
fi
echo "botbook_reindex: $index ($size bytes)"

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

echo "botbook_reindex: $(find botbook/book -name '*.html' | wc -l) HTML pages built"
