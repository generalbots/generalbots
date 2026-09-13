#!/usr/bin/env python3
"""Register unpublished botbook pages in SUMMARY.md.

mdbook renders only what the table of contents lists, so a page that exists in
the repository but is absent from SUMMARY.md is invisible on the published
site. This finds those pages and appends each one to the section of the
chapter it belongs to.

Ordering rule: a new entry goes after the last existing entry from the same
directory; if the directory has no entry yet, after the last entry from the
same top-level chapter. Entries keep the two-space indent used for chapter
sub-items, or three where the surrounding lines use three.

Run with --check to list the gap without writing (exit 1 when pages are missing).
"""
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, "botbook/src")
SUMMARY = os.path.join(SRC, "SUMMARY.md")

# Pages that intentionally have no table-of-contents entry.
#   README.md            the book cover, which mdbook does not render
#   drive-monitor-test.md  a manual test procedure kept for developers, unreferenced
SKIP = {
    "SUMMARY.md",
    "README.md",
    "drive-monitor-test.md",
}


def title_for(path):
    """Derive a menu label from the page's first H1, else from the filename."""
    try:
        text = open(os.path.join(SRC, path), encoding="utf-8").read()
    except OSError:
        return None
    m = re.search(r"^#\s+(.+)$", text, re.M)
    if m:
        label = m.group(1).strip()
        # Strip the stability markers the book uses in titles.
        label = re.sub(r"\s*[🟢🟡🔴]\s*(GA|PREVIEW|BETA)(\s*[—-]\s*IN TEST)?", "", label)
        label = re.sub(r"\s*[🟢🟡🔴]\s*$", "", label).strip()
        return label
    return os.path.splitext(os.path.basename(path))[0].replace("-", " ").title()


def main():
    text = open(SUMMARY, encoding="utf-8").read()
    lines = text.split("\n")

    listed = {os.path.normpath(m.group(1))
              for m in re.finditer(r"\]\(([^)]+\.md)\)", text)}

    on_disk = []
    for dp, dirs, fs in os.walk(SRC):
        dirs[:] = [d for d in dirs if d != "assets"]
        for f in fs:
            if f.endswith(".md"):
                rel = os.path.relpath(os.path.join(dp, f), SRC)
                on_disk.append(rel.replace(os.sep, "/"))

    missing = sorted(p for p in on_disk if p not in listed and p not in SKIP)

    if "--check" in sys.argv[1:]:
        if missing:
            print(f"{len(missing)} page(s) not in SUMMARY.md:")
            for p in missing:
                print(f"  {p}")
            return 1
        print(f"SUMMARY.md lists every page ({len(listed)} entries)")
        return 0

    if not missing:
        print("nothing to add")
        return 0

    # Index of the last line referencing each directory and each top chapter.
    def dir_of(p):
        return os.path.dirname(p)

    def chapter_of(p):
        return p.split("/")[0]

    added = 0
    for path in missing:
        label = title_for(path)
        if not label:
            print(f"SKIP  {path}: unreadable")
            continue
        d, ch = dir_of(path), chapter_of(path)

        last_dir = last_ch = -1
        for i, ln in enumerate(lines):
            m = re.search(r"\]\(([^)]+\.md)\)", ln)
            if not m:
                continue
            listed_path = os.path.normpath(m.group(1)).replace(os.sep, "/")
            if dir_of(listed_path) == d:
                last_dir = i
            if chapter_of(listed_path) == ch:
                last_ch = i

        at = last_dir if last_dir >= 0 else last_ch
        if at < 0:
            print(f"SKIP  {path}: no anchor in {ch}")
            continue

        # Match the indent of the anchor line.
        indent = re.match(r"^(\s*)", lines[at]).group(1)
        entry = f"{indent}- [{label}](./{path})"
        lines.insert(at + 1, entry)
        added += 1

    open(SUMMARY, "w", encoding="utf-8").write("\n".join(lines))
    print(f"added {added} entries to SUMMARY.md")
    return 0


if __name__ == "__main__":
    sys.exit(main())
