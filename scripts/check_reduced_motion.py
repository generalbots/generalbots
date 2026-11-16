#!/usr/bin/env python3
"""Guard (#2326): animated suite CSS files must honor prefers-reduced-motion."""
from __future__ import annotations
import pathlib, re, sys

root = pathlib.Path(__file__).resolve().parents[1] / "botui" / "ui" / "suite"
bad = []
animated = 0
for p in root.rglob("*.css"):
    if "vendor" in p.parts:
        continue
    try:
        s = p.read_text(encoding="utf-8", errors="ignore")
    except Exception:
        continue
    if re.search(r"@keyframes|animation\s*:|transition\s*:", s):
        animated += 1
        if "prefers-reduced-motion" not in s:
            bad.append(str(p.relative_to(root)))
if bad:
    print(f"FAIL: {len(bad)} animated CSS file(s) missing prefers-reduced-motion")
    for b in sorted(bad):
        print(b)
    sys.exit(1)
print(f"OK: {animated} animated suite CSS files honor prefers-reduced-motion")
