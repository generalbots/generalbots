#!/usr/bin/env python3
"""CI guard for the published GB7 feature comparison (#1704, #1795).

The comparison page lives on the public site, not in this repository, so
page/product drift used to be found only by a manual audit. This guard
validates the live pages (or local fixtures) against the current contract:

- exactly 33 capability rows with a GB7 position;
- the header counts match the row positions;
- voice, locked conversations, SKILL.md import and collaborative documents
  are Yes/Parity;
- no shipped row is declared Gap;
- every Follow-up and Gap row carries a tracked issue reference;
- a voice row that promises interruptions/barge-in also references the
  duplex/barge-in implementation or #1792;
- the test-scripts page is rebaselined (zero gaps after #1695 shipped).

#1695 shipped collaborative documents, so the guard no longer expects any Gap.
The fixtures under scripts/fixtures/ keep a negative case (legacy page) and
the accepted updated page for --self-test.

Usage:
    python3 scripts/gb7_features_check.py
    python3 scripts/gb7_features_check.py --index /tmp/index.html \\
        --scripts /tmp/test-scripts.html
    python3 scripts/gb7_features_check.py --self-test

Exits non-zero on any mismatch so CI fails before a stale page ships.
"""

from __future__ import annotations

import argparse
import re
import sys
import urllib.request
from pathlib import Path

INDEX_URL = "https://sites.pragmatismo.com.br/gb7features/"
SCRIPTS_URL = "https://sites.pragmatismo.com.br/gb7features/test-scripts.html"

EXPECTED_ROWS = 33
SHIPPED_ROWS = {
    "Voice / live conversation mode": "Parity",
    "Locked / hidden conversations": "Parity",
    "Agent Skills packages (SKILL.md)": "Parity",
    "Collaborative documents and presentations": "Parity",
}
VOICE_ROW = "Voice / live conversation mode"
VOICE_INTERRUPTION_RE = re.compile(r"\binterrupt|\bbarge[\s-]?in", re.I)
VOICE_CAVEAT_RE = re.compile(r"\bduplex\b|barge[\s-]?in implementation|#1792", re.I)
FIXTURES_DIR = Path(__file__).resolve().parent / "fixtures"

ROW_RE = re.compile(r"<tr>(.*?)</tr>", re.S)
GB7_RE = re.compile(
    r'<td class="gb7-col"><span class="st st-(yes|part|no)">([^<]+)</span>'
)
POS_RE = re.compile(r'<span class="pt pt-(adv|parity|follow|gap)">([^<]+)</span>')
COUNT_RE = re.compile(r"<strong>(\d+)</strong>\s+([^<]+)")


def load(source: str | None, url: str) -> str:
    if source:
        with open(source, encoding="utf-8") as handle:
            return handle.read()
    request = urllib.request.Request(url, headers={"User-Agent": "gb7-parity-check"})
    with urllib.request.urlopen(request, timeout=30) as response:
        return response.read().decode("utf-8", "replace")


def strip_tags(value: str) -> str:
    return re.sub(r"<[^>]+>", " ", value)


def parse_rows(page: str) -> list[dict[str, str]]:
    rows = []
    for match in ROW_RE.finditer(page):
        block = match.group(1)
        gb7 = GB7_RE.search(block)
        if not gb7:
            continue
        pos = POS_RE.search(block)
        if not pos:
            continue
        cells = re.split(r"</td>", block)
        # The first cell carries the title, an optional "new" badge and the
        # description span; keep only the title text.
        first_cell = cells[0].split('<span class="desc">')[0]
        first_cell = re.sub(r'<span class="badge-new">.*?</span>', "", first_cell)
        title = " ".join(strip_tags(first_cell).split())
        rows.append(
            {
                "title": title,
                "status": gb7.group(2).strip(),
                "position": pos.group(2).strip(),
                "text": block,
            }
        )
    return rows


def header_counts(page: str) -> dict[str, int]:
    counts = {}
    for number, label in COUNT_RE.findall(page):
        counts[" ".join(label.split())] = int(number)
    return counts


def check_index(page: str) -> list[str]:
    errors = []
    rows = parse_rows(page)
    if len(rows) != EXPECTED_ROWS:
        errors.append(f"expected {EXPECTED_ROWS} capability rows, found {len(rows)}")

    counts = header_counts(page)
    positions = [row["position"] for row in rows]
    expected = {
        "GB7 advantages": positions.count("Advantage"),
        "at parity": positions.count("Parity"),
        "follow-ups tracked": positions.count("Follow-up"),
        "gap not yet available": positions.count("Gap"),
    }
    for key, value in expected.items():
        found = counts.get(key)
        if found != value:
            errors.append(f"header says {found} '{key}', rows say {value}")

    by_title = {row["title"]: row for row in rows}
    for title, position in SHIPPED_ROWS.items():
        row = by_title.get(title)
        if not row:
            errors.append(f"missing row: {title}")
        elif row["position"] == "Gap":
            errors.append(f"{title}: shipped row is still declared Gap")
        elif row["position"] != position:
            errors.append(f"{title}: position {row['position']} != {position}")

    for row in rows:
        if row["position"] in ("Follow-up", "Gap") and not re.search(
            r"#\d{3,}", row["text"]
        ):
            errors.append(f"{row['title']}: {row['position']} row has no tracked issue")

    # #1792: the voice row may promise interruptions/barge-in only when it
    # also names the duplex/barge-in implementation or its tracking issue.
    voice = by_title.get(VOICE_ROW)
    if voice:
        promises_interruptions = bool(VOICE_INTERRUPTION_RE.search(voice["text"]))
        has_caveat = bool(VOICE_CAVEAT_RE.search(voice["text"]))
        if promises_interruptions and not has_caveat:
            errors.append(
                f"{VOICE_ROW}: interruption/barge-in promise needs a "
                "duplex/barge-in implementation reference or #1792"
            )
    return errors


def check_scripts(page: str) -> list[str]:
    errors = []
    if "four declared gaps" in page or "four\n      declared gaps" in page:
        errors.append("test scripts still describe four declared gaps")
    # #1695 shipped collaborative documents; no gap should remain.
    if re.search(r"one honest gap|one declared gap", page, re.I):
        errors.append(
            "test scripts still reference a gap for collaborative documents "
            "(shipped in #1695)"
        )
    if re.search(r"<strong>[1-9]\d*</strong>\s+gap", page):
        errors.append("script header still reports gaps (should be 0 after #1695)")
    return errors


def self_test() -> int:
    """Run check_index against the legacy and updated fixtures.

    The legacy page must be rejected (collaborative documents still Gap and
    no voice caveat); the updated page must be accepted.
    """
    cases = [
        ("legacy", FIXTURES_DIR / "gb7-index-legacy.html", False),
        ("updated", FIXTURES_DIR / "gb7-index-updated.html", True),
    ]
    failed = False
    for name, path, should_accept in cases:
        try:
            page = path.read_text(encoding="utf-8")
        except OSError as error:
            print(f"gb7 self-test: cannot read {path}: {error}")
            failed = True
            continue
        errors = check_index(page)
        accepted = not errors
        if accepted == should_accept:
            expected = "accepted" if should_accept else "rejected"
            print(f"gb7 self-test: {name} fixture {expected} as expected")
        else:
            wanted = "acceptance" if should_accept else "rejection"
            print(f"gb7 self-test: {name} fixture FAILED (expected {wanted})")
            failed = True
        if not accepted:
            for error in errors:
                print(f"  - {error}")
    if failed:
        print("gb7 self-test: FAIL")
        return 1
    print("gb7 self-test: PASS (legacy rejected, updated accepted)")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--index", help="local index.html copy")
    parser.add_argument("--scripts", help="local test-scripts.html copy")
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="validate the bundled legacy/updated index fixtures",
    )
    args = parser.parse_args()

    if args.self_test:
        return self_test()

    try:
        index = load(args.index, INDEX_URL)
        scripts = load(args.scripts, SCRIPTS_URL)
    except Exception as error:  # network or filesystem failure
        print(f"gb7 features check: cannot load pages: {error}")
        return 2

    errors = check_index(index) + check_scripts(scripts)
    if errors:
        print("gb7 features check: FAIL")
        for error in errors:
            print(f"  - {error}")
        return 1
    print("gb7 features check: PASS (33 rows, counts match, no untracked gaps)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
