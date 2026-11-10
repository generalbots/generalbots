#!/usr/bin/env python3
"""Shared paths and helpers for the BotBook catalog generators (#2080).

`gen_catalogs.py` is the entry point; `gen_connector_catalog.py` and
`gen_skills_catalog.py` own one catalog each. Everything here is pure: no
timestamps, no network, sorted output, so a re-run is byte-identical.
"""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CATALOG = ROOT / "botserver" / "src" / "apps" / "integration_catalog"
ADAPTERS = ROOT / "botserver" / "crates" / "botintegrations" / "src" / "providers"
SKILLS = ROOT / "botskills"
BOOK = ROOT / "botbook" / "src"
CONN_DIR = BOOK / "06-channels" / "connectors"
SKILLS_PAGE = BOOK / "13-ai-os" / "skills-catalog.md"
SUMMARY = BOOK / "SUMMARY.md"

CONN_BEGIN = "<!-- BEGIN GENERATED CONNECTOR CATALOG -->"
CONN_END = "<!-- END GENERATED CONNECTOR CATALOG -->"
SKILL_BEGIN = "<!-- BEGIN GENERATED SKILLS CATALOG -->"
SKILL_END = "<!-- END GENERATED SKILLS CATALOG -->"

MAX_PAGE_LINES = 400


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def cell(text: str) -> str:
    """Escape a value for a Markdown table cell."""
    return text.replace("|", "\\|").replace("\n", " ").strip()


def anchor(text: str) -> str:
    """mdBook heading slug for `text`."""
    slug = re.sub(r"[^\w\s-]", "", text.lower()).strip()
    return re.sub(r"\s+", "-", slug)


def update_summary(current: str, block: str, begin: str, end: str, anchor_line: str) -> str:
    """Replace the generated block, or insert it after `anchor_line` once."""
    if begin in current and end in current:
        start = current.index(begin)
        stop = current.index(end) + len(end) + 1
        return current[:start] + block + current[stop:]
    marker = anchor_line + "\n"
    if marker not in current:
        return current + "\n" + block
    return current.replace(marker, marker + block, 1)
