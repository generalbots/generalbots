#!/usr/bin/env python3
"""Regenerate Atom/RSS feeds from published advisory front matter (#2719)."""
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "security" / "advisories"
OUT = ROOT / "security" / "feeds"


def entries():
    result = []
    for path in sorted(SOURCE.glob("*.md")):
        text = path.read_text(encoding="utf-8")
        if not text.startswith("---"):
            continue
        values = dict(re.findall(r"^([a-z_]+):\s*(.+)$", text.split("---", 2)[1], re.M))
        result.append(values)
    return result


def main():
    rows = entries()
    atom = ['<?xml version="1.0" encoding="UTF-8"?>', '<feed xmlns="http://www.w3.org/2005/Atom">',
            '<title>General Bots security advisories</title>', '<id>https://generalbots.org/security/</id>',
            '<updated>2026-11-23T00:00:00Z</updated>']
    for row in rows:
        ident = row.get("id", "GB-SA-0000-0000")
        atom.append('<entry><title>' + row.get("title", "") + '</title>')
        atom.append('<id>tag:generalbots.org,2026:' + ident + '</id>')
        atom.append('<updated>2026-11-23T00:00:00Z</updated>')
        atom.append('<link href="https://generalbots.org/security/advisories/' + ident + '.html"/>')
        atom.append('<summary>' + row.get("summary", "") + '</summary></entry>')
    atom.append('</feed>')
    (OUT / "feed.atom").write_text("\n".join(atom) + "\n", encoding="utf-8")
    print(f"gen_feeds: wrote {len(rows)} Atom entry(ies); rss.xml kept as the checked-in example")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
