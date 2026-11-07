#!/usr/bin/env bash
# Rebuild the BotBook and verify the committed HTML/search index is current.
#
# mdBook writes the search index (searchindex-<hash>.js) and every rendered
# page under botbook/book/. The directory is tracked so the published site can
# be served without a build step; that also means a source edit that was not
# followed by `mdbook build` silently leaves readers on an old index.
#
# Usage: scripts/botbook_reindex.sh
# Exit codes: 0 the committed book matches the sources; 1 a rebuild is needed
# or the build failed.
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

if ! git diff --quiet -- botbook/book; then
    echo "botbook_reindex: botbook/book is stale; commit the regenerated book" >&2
    git status --porcelain -- botbook/book | head -20 >&2
    exit 1
fi

echo "botbook_reindex: committed book is in sync with botbook/src"
