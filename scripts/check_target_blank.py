#!/usr/bin/env python3
"""Guard (#2327): no target=_blank without noopener/noreferrer in suite (non-vendor)."""
from __future__ import annotations

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1] / "botui" / "ui" / "suite"
EXCLUDED_PARTS = {"vendor", "vs"}
SUFFIXES = {".html", ".js"}

issues = []
for p in ROOT.rglob("*"):
    if not p.is_file() or p.suffix not in SUFFIXES:
        continue
    if any(part in EXCLUDED_PARTS for part in p.parts) or p.name.endswith(".orig"):
        continue
    s = p.read_text(encoding="utf-8", errors="ignore")
    rel = p.relative_to(ROOT)

    # Static/templated anchors with target="_blank" but no noopener in tag.
    for m in re.finditer(r"<a\b[^>]*target\s*=\s*[\"']?_blank[\"']?[^>]*>", s, re.I):
        tag = m.group(0)
        if "noopener" not in tag.lower():
            issues.append((str(rel), s.count("\n", 0, m.start()) + 1, "anchor-missing-noopener", tag[:160]))

    # JS-generated anchors: element.target="_blank" must be paired with rel/noopener nearby.
    for m in re.finditer(r"\.target\s*=\s*[\"']_blank[\"']", s):
        ahead = s[m.start() : m.start() + 300]
        if ".rel" not in ahead and "noopener" not in ahead:
            issues.append((str(rel), s.count("\n", 0, m.start()) + 1, "js-target-no-rel", s[max(0, m.start() - 20) : m.end() + 60].replace("\n", " ")))

    # window.open(..., "_blank", ...) must include noopener/noreferrer or nullify opener.
    for m in re.finditer(r"window\.open\s*\([^)]*[\"']_blank[\"']?[^)]*\)", s):
        call = m.group(0)
        ctx = s[max(0, m.start() - 40) : m.end() + 140]
        if "noopener" not in call and "noreferrer" not in call and "opener = null" not in ctx and "opener=null" not in ctx:
            issues.append((str(rel), s.count("\n", 0, m.start()) + 1, "window-open-noopener", call[:180]))

if issues:
    print(f"FAIL: {len(issues)} unsafe target=_blank usage(s)")
    for path, line, kind, detail in issues:
        print(f"{path}:{line}: {kind}: {detail}")
    sys.exit(1)
print("OK: no unsafe target=_blank usages in suite (non-vendor)")
