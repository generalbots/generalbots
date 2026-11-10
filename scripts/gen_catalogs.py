#!/usr/bin/env python3
"""Generate the BotBook connector and skills catalogs (#2080).

One entry point, two catalogs, everything derived from source — no
timestamps, no network, sorted output, so two runs are byte-identical.

* `scripts/gen_connector_catalog.py` — providers, auth profiles, actions and
  the derived `built`/`partial`/`planned`/`unsupported` status from the
  integration catalog seeds plus the live adapter registry.
* `scripts/gen_skills_catalog.py` — the curated skills in `botskills/**` and
  the linked-only entries from `botskills/README.md`.

Both generated blocks are registered in `botbook/src/SUMMARY.md` between
markers, so the table of contents stays in sync with the pages.

Usage
-----
    python3 scripts/gen_catalogs.py            # write the pages
    python3 scripts/gen_catalogs.py --report   # inventory only
    python3 scripts/gen_catalogs.py --check    # exit 1 when pages are stale
"""

from __future__ import annotations

import argparse
import sys

from catalog_common import (
    CONN_BEGIN, CONN_DIR, CONN_END, SKILL_BEGIN, SKILL_END, SKILLS_PAGE, SUMMARY,
    read, update_summary,
)
from gen_connector_catalog import build_connectors, report as report_connectors
from gen_connector_catalog import summary_block as connector_block
from gen_skills_catalog import build_skills, report as report_skills
from gen_skills_catalog import summary_block as skills_block

CATALOG_SERVICE_ANCHOR = "   - [Service Catalog](./06-channels/catalog.md)"
MARKETPLACE_ANCHOR = "   - [Skills Marketplace](./13-ai-os/marketplace.md)"


def build() -> dict:
    """Every generated file, keyed by path."""
    _, connector_pages = build_connectors()
    skills_body, _, _ = build_skills()
    files = {CONN_DIR / name: body for name, body in connector_pages.items()}
    files[SKILLS_PAGE] = skills_body
    summary = read(SUMMARY)
    summary = update_summary(summary, connector_block(connector_pages),
                             CONN_BEGIN, CONN_END, CATALOG_SERVICE_ANCHOR)
    summary = update_summary(summary, skills_block(),
                             SKILL_BEGIN, SKILL_END, MARKETPLACE_ANCHOR)
    files[SUMMARY] = summary
    return files


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--report", action="store_true", help="print the inventory")
    parser.add_argument("--check", action="store_true", help="fail when pages are stale")
    args = parser.parse_args()
    if args.report:
        report_connectors()
        report_skills()
        return 0

    files = build()
    stale = [path for path, body in files.items() if not path.is_file() or read(path) != body]
    if args.check:
        for path in stale:
            print(f"stale: {path}")
        if stale:
            print("gen_catalogs: run python3 scripts/gen_catalogs.py to regenerate")
            return 1
        print(f"gen_catalogs: {len(files)} pages up to date")
        return 0

    for path in stale:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(files[path], encoding="utf-8")
    for orphan in sorted(CONN_DIR.glob("*.md")):
        if orphan not in files:
            orphan.unlink()
            print(f"removed stale page: {orphan}")
    over = [p for p in files if p != SUMMARY and files[p].count("\n") > 450]
    print(f"wrote {len(stale)} of {len(files)} files")
    for path in over:
        print(f"WARNING: {path} exceeds 450 lines")
    return 0


if __name__ == "__main__":
    sys.exit(main())
