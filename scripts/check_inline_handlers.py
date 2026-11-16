#!/usr/bin/env python3
"""Guard (#2376): ratchet inline on* handlers in suite HTML until strict CSP is enforceable.

Counts `on...="` attributes in botui/ui/suite/**/*.html (excluding vendor). The count
must not increase above the recorded budget; reduce it over time by moving handlers to
external JS so script-src 'self' can be enforced without 'unsafe-inline'.
"""
from __future__ import annotations

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1] / "botui" / "ui" / "suite"
# Budget as of 2026-11-16 (#2376 evidence). Must only go down.
BUDGET = 1196
PATTERN = re.compile(r'\bon[a-z]+\s*=\s*"', re.IGNORECASE)

count = 0
by_file = []
for p in ROOT.rglob("*.html"):
    if "vendor" in p.parts:
        continue
    try:
        text = p.read_text(encoding="utf-8", errors="ignore")
    except Exception:
        continue
    n = len(PATTERN.findall(text))
    if n:
        count += n
        by_file.append((str(p.relative_to(ROOT)), n))

by_file.sort(key=lambda x: -x[1])
print(f"Inline on* handlers in suite HTML (non-vendor): {count} (budget {BUDGET})")
for path, n in by_file[:10]:
    print(f"  {n:4d}  {path}")

if count > BUDGET:
    print(f"FAIL: inline handler count increased above budget ({count} > {BUDGET})")
    sys.exit(1)
print("OK: inline handler count within budget")
