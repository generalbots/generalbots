#!/usr/bin/env python3
"""Wire orphaned suite SVG assets into their documentation pages.

Each orphan is mapped to the page that documents the app (or flow) it depicts.
The image is inserted after the page's intro block, matching the existing
convention: a full-width <img> on its own line.
"""
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, "botbook/src")
APPS = "07-user-interface/apps"

# asset stem -> (page path relative to SRC, alt text)
SCREENS = {
    "docs-screen": (f"{APPS}/docs.md", "Docs editor screen"),
    "slides-screen": (f"{APPS}/slides.md", "Slides editor screen"),
    "sources-screen": (f"{APPS}/sources.md", "Sources configuration screen"),
}

FLOWS = {
    "chat-flow": (f"{APPS}/chat.md", "Chat message flow"),
    "drive-flow": (f"{APPS}/drive.md", "Drive file flow"),
    "mail-flow": (f"{APPS}/mail.md", "Mail delivery flow"),
    "meet-flow": (f"{APPS}/meet.md", "Meeting lifecycle flow"),
    "paper-flow": (f"{APPS}/paper.md", "Document flow"),
    "tasks-flow": (f"{APPS}/tasks.md", "Task execution flow"),
    "analytics-flow": (f"{APPS}/analytics.md", "Analytics pipeline flow"),
    "calendar-flow": (f"{APPS}/calendar.md", "Calendar event flow"),
    "compliance-flow": (f"{APPS}/compliance.md", "Compliance request flow"),
    "designer-flow": (f"{APPS}/designer.md", "Designer pipeline flow"),
    "research-flow": (f"{APPS}/research.md", "Research pipeline flow"),
    "sources-flow": (f"{APPS}/sources.md", "Source ingestion flow"),
    "player-flow": (f"{APPS}/player.md", "Media viewer flow"),
    "suite-layout": ("07-user-interface/ui-structure.md", "Suite desktop layout"),
    "app-launcher": (f"{APPS}/README.md", "Application launcher"),
}

# Assets depicting applications that no longer exist. ERP and ITSM were unified
# into Billing and Tickets respectively, so the mockups would send a reader
# looking for apps that are not in the launcher or the catalog.
RETIRED = ["erp-screen", "itsm-screen"]


def insert(page_rel, stem, alt):
    path = os.path.join(SRC, page_rel)
    if not os.path.exists(path):
        return f"SKIP  {stem}: no page {page_rel}"
    text = open(path, encoding="utf-8").read()
    if f"assets/suite/{stem}.svg" in text:
        return f"OK    {stem}: already referenced in {page_rel}"

    img = (f'<img src="../../assets/suite/{stem}.svg" alt="{alt}" '
           f'style="max-width: 100%; height: auto;">\n')

    # Insert after the blockquote intro (the first "> ..." run), else after the H1.
    lines = text.split("\n")
    out, inserted, i = [], False, 0
    while i < len(lines):
        out.append(lines[i])
        if not inserted and lines[i].startswith("> "):
            # consume the rest of the contiguous quote block
            while i + 1 < len(lines) and lines[i + 1].startswith(">"):
                i += 1
                out.append(lines[i])
            out.append("")
            out.append(img.rstrip("\n"))
            inserted = True
        elif not inserted and lines[i].startswith("# ") and (
            i + 1 >= len(lines) or not lines[i + 1].startswith(">")
        ):
            # H1 with no following quote: insert right after it
            out.append("")
            out.append(img.rstrip("\n"))
            inserted = True
        i += 1

    if not inserted:
        return f"SKIP  {stem}: no anchor in {page_rel}"
    open(path, "w", encoding="utf-8").write("\n".join(out))
    return f"WIRED {stem} -> {page_rel}"


def main():
    log = []
    for stem, (page, alt) in {**SCREENS, **FLOWS}.items():
        log.append(insert(page, stem, alt))

    for stem in RETIRED:
        path = os.path.join(SRC, "assets/suite", f"{stem}.svg")
        if os.path.exists(path):
            os.remove(path)
            log.append(f"RM    {stem}.svg (retired application)")
        else:
            log.append(f"OK    {stem}.svg already gone")

    for line in sorted(log):
        print(line)
    return 0


if __name__ == "__main__":
    sys.exit(main())
