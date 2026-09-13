#!/usr/bin/env python3
"""Verify every internal botbook link against what the site actually publishes.

mdbook renders only the pages listed in SUMMARY.md, so a link to a page that
exists on disk but is absent from the table of contents works in the editor and
404s for every reader. Checking against the filesystem alone therefore misses
the most common breakage in this book.

Each relative link is resolved against its own file and classified:

  MISSING     no such file on disk
  UNPUBLISHED file exists but SUMMARY.md never lists it

Absolute URLs, anchors and non-markdown targets are ignored. Exits non-zero
when any link fails, so it can gate a change.
"""
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, "botbook/src")
SUMMARY = os.path.join(SRC, "SUMMARY.md")


def main():
    summary_text = open(SUMMARY, encoding="utf-8").read()
    listed = {os.path.normpath(m.group(1)).replace(os.sep, "/")
              for m in re.finditer(r"\]\(([^)]+\.md)\)", summary_text)}

    missing = {}
    unpublished = {}
    checked = 0

    for dp, dirs, fs in os.walk(SRC):
        dirs[:] = [d for d in dirs if d not in {"assets", "book"}]
        for f in fs:
            if not f.endswith(".md"):
                continue
            full = os.path.join(dp, f)
            rel = os.path.relpath(full, SRC).replace(os.sep, "/")
            try:
                text = open(full, encoding="utf-8").read()
            except OSError as e:
                print(f"unreadable {rel}: {e}")
                continue
            for m in re.finditer(r"\]\(([^)\s]+)\)", text):
                target = m.group(1)
                if target.startswith(("http://", "https://", "#", "mailto:", "/")):
                    continue
                path = target.partition("#")[0]
                if not path.endswith(".md"):
                    continue
                checked += 1
                resolved = os.path.normpath(
                    os.path.join(os.path.dirname(rel), path)
                ).replace(os.sep, "/")
                line = text[:m.start()].count("\n") + 1
                if not os.path.exists(os.path.join(SRC, resolved)):
                    missing.setdefault(rel, []).append((line, target))
                elif resolved not in listed:
                    unpublished.setdefault(rel, []).append((line, target))

    print(f"{checked} internal link(s) checked against {len(listed)} published pages")
    for rel, hits in sorted(missing.items()):
        for line, target in hits:
            print(f"  MISSING     {rel}:{line} -> {target}")
    for rel, hits in sorted(unpublished.items()):
        for line, target in hits:
            print(f"  UNPUBLISHED {rel}:{line} -> {target}")

    broken = sum(len(v) for v in missing.values()) + sum(len(v) for v in unpublished.values())
    if broken:
        print(f"{broken} broken link(s)")
        return 1
    print("no broken links")
    return 0


if __name__ == "__main__":
    sys.exit(main())
