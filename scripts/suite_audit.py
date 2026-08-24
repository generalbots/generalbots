#!/usr/bin/env python3
"""Suite mindfulness audit (#936 wave): flags suite app files exceeding the
450-line budget and counts inline style attributes (design-token debt).

Usage: python3 scripts/suite_audit.py [suite_dir]   # default botui/ui/suite
Exit code 1 when any hard violation (>450 lines) exists.
"""
import sys, os, re

LIMIT = 450
root = sys.argv[1] if len(sys.argv) > 1 else os.path.join("botui", "ui", "suite")
violations = []
inline_total = 0

for dirpath, _, files in os.walk(root):
    if any(seg in dirpath for seg in ("node_modules", "vendor", "webfonts", "default.gbui", os.path.join(root, "public"), os.path.join(root, "webfonts"))):
        continue
    for name in files:
        if not name.endswith((".js", ".html", ".css")):
            continue
        if name.endswith(".min.js") or name.endswith(".orig"):
            continue
        # Generated/design-token stylesheets are exempt from the line budget.
        path = os.path.join(dirpath, name)
        try:
            with open(path, encoding="utf-8", errors="ignore") as fh:
                lines = sum(1 for _ in fh)
        except OSError:
            continue
        rel = os.path.relpath(path, root)
        if name == "theme-sentient.css":
            continue
        if lines > LIMIT:
            violations.append((rel, lines))
        if name.endswith(".html"):
            with open(path, encoding="utf-8", errors="ignore") as fh:
                inline_total += len(re.findall(r"\sstyle=\"", fh.read()))

print(f"Suite audit — limit {LIMIT} lines | inline style attrs: {inline_total}")
if violations:
    print("\nOver-budget files:")
    for rel, n in sorted(violations, key=lambda v: -v[1]):
        print(f"  {n:5d}  {rel}")
    sys.exit(1)
print("OK: no over-budget files")
