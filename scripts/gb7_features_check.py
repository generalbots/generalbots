#!/usr/bin/env python3
"""CI guard for the published GB7 feature comparison (#1704).

The comparison page lives on the public site, not in this repository, so
page/product drift used to be found only by a manual audit. This guard
validates the live pages (or local copies) against the contract reached on
2026-11-05:

- exactly 33 capability rows with a GB7 position;
- the header counts match the row positions;
- voice, locked conversations and SKILL.md import are Yes/Parity;
- collaborative documents is the only remaining Gap, linked to its issue;
- every Follow-up and Gap row carries a tracked issue reference;
- the test-scripts page is rebaselined (one gap negative-tested).

Usage:
    python3 scripts/gb7_features_check.py
    python3 scripts/gb7_features_check.py --index /tmp/index.html \\
        --scripts /tmp/test-scripts.html

Exits non-zero on any mismatch so CI fails before a stale page ships.
"""

from __future__ import annotations

import argparse
import re
import sys
import urllib.request

INDEX_URL = "https://sites.pragmatismo.com.br/gb7features/"
SCRIPTS_URL = "https://sites.pragmatismo.com.br/gb7features/test-scripts.html"

EXPECTED_ROWS = 33
SHIPPED_ROWS = {
    "Voice / live conversation mode": "Parity",
    "Locked / hidden conversations": "Parity",
    "Agent Skills packages (SKILL.md)": "Parity",
}
LAST_GAP = "Collaborative documents and presentations"
GAP_ISSUE = "#1695"

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
        elif row["position"] != position:
            errors.append(f"{title}: position {row['position']} != {position}")

    gaps = [row for row in rows if row["position"] == "Gap"]
    if [row["title"] for row in gaps] != [LAST_GAP]:
        errors.append(
            "declared gaps are "
            + (", ".join(row["title"] for row in gaps) or "none")
            + f" (expected only {LAST_GAP})"
        )
    for row in rows:
        if row["position"] in ("Follow-up", "Gap") and not re.search(
            r"#\d{3,}", row["text"]
        ):
            errors.append(f"{row['title']}: {row['position']} row has no tracked issue")
    gap_row = by_title.get(LAST_GAP)
    if gap_row and GAP_ISSUE not in gap_row["text"]:
        errors.append(f"{LAST_GAP}: missing {GAP_ISSUE} reference")
    return errors


def check_scripts(page: str) -> list[str]:
    errors = []
    if "four declared gaps" in page or "four\n      declared gaps" in page:
        errors.append("test scripts still describe four declared gaps")
    if "Shipped checks and one honest gap" not in page:
        errors.append("TS-05 is not rebaselined (missing new heading)")
    if not re.search(r"<strong>1</strong>\s+gap negative-tested", page):
        errors.append("script header does not say 1 gap negative-tested")
    if "tracked in #1695" not in page and "#1695" not in page:
        errors.append("TS-05 does not reference the tracked gap issue")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--index", help="local index.html copy")
    parser.add_argument("--scripts", help="local test-scripts.html copy")
    args = parser.parse_args()

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
    print("gb7 features check: PASS (33 rows, counts match, one tracked gap)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
