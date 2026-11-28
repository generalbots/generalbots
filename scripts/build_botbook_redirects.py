#!/usr/bin/env python3
"""Emit mdBook-compatible redirects into the built BotBook site.

mdBook wrote a chapter's README.md as ``<chapter>/index.html``; MkDocs writes
``<chapter>/README.html``. Any old link to the former must still land on the
same content. Run after ``mkdocs build``.

Usage:
    python3 scripts/build_botbook_redirects.py            # botbook/book
    python3 scripts/build_botbook_redirects.py <site_dir>
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SUMMARY = ROOT / "botbook" / "src" / "SUMMARY.md"
DEFAULT_SITE = ROOT / "botbook" / "book"
LINK_RE = re.compile(r"\]\(([^)\s]+\.md)\)")

PAGE = """<!doctype html>
<html lang="en"><head><meta charset="utf-8">
<meta http-equiv="refresh" content="0;url={target}">
<link rel="canonical" href="{target}">
<title>Redirect</title></head>
<body><a href="{target}">Documentation moved to {target}</a></body></html>
"""


def main() -> int:
    site = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_SITE
    targets: dict[str, str] = {}
    for link in LINK_RE.findall(SUMMARY.read_text(encoding="utf-8")):
        if not link.endswith("/README.md"):
            continue
        old = link[2:-len("README.md")] + "index.html"
        new = "/" + link[2:-len(".md")] + ".html"
        targets[old] = new
    for old, new in sorted(targets.items()):
        path = site / old
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(PAGE.format(target=new), encoding="utf-8")
    print(f"build_botbook_redirects: wrote {len(targets)} mdBook-compatible redirect(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
