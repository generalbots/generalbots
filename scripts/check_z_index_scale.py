#!/usr/bin/env python3
"""Z-index scale gate (issue #2321).

The suite used to spread 120 distinct z-index values from -10 to 2147483647:
each overlay that needed to win an arms race picked a bigger integer than the
last. There is now one scale in `botui/ui/suite/css/base/01_root.css`
(`--z-base` … `--z-top`), and every suite-wide layer must name a token instead
of a literal.

Allowed without a token:
  * `var(--z-*)`, `inherit`, `auto`, `initial`, `unset`, `revert`
  * integers 0-99 — local stacking inside one component's own stacking context
  * the token definitions themselves (`--z-modal: 1300`), which are not
    `z-index` declarations

Anything at 100 or above, or negative, is a suite-wide layer and must be a
token. Vendored bundles and the browser test harness are skipped: they are not
ours to edit. The cloud and login surfaces are separate documents with their
own stylesheets and do not load the suite token layer, so they are out of
scope here.

Usage
-----
    python3 scripts/check_z_index_scale.py --check
    python3 scripts/check_z_index_scale.py --list
    python3 scripts/check_z_index_scale.py --self-test
    python3 scripts/check_z_index_scale.py --root <repo>

Exit codes: 0 clean, 1 violations found.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCAN_ROOT = Path("botui/ui/suite")
SUFFIXES = (".css", ".js", ".html")
SKIP_SEGMENTS = ("vendor", "node_modules")

# `z-index: 1400`, `z-index:1400 !important`, `zIndex = "1400"`
DECLARATION = re.compile(
    r"(?:z-index\s*:\s*|zIndex\s*=\s*['\"]?)(-?\d+)\b",
    re.IGNORECASE,
)
LOCAL_ONLY_MAX = 99


def line_of(text: str, index: int) -> int:
    return text.count("\n", 0, index) + 1


def scan_text(text: str) -> list[tuple[int, str]]:
    found = []
    for m in DECLARATION.finditer(text):
        value = int(m.group(1))
        if 0 <= value <= LOCAL_ONLY_MAX:
            continue
        snippet = " ".join(text[max(0, m.start() - 40):m.end() + 20].split())
        found.append((line_of(text, m.start()), snippet[:160]))
    return found


def scan_file(path: Path) -> list[tuple[int, str]]:
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError as exc:
        print(f"warning: cannot read {path}: {exc}", file=sys.stderr)
        return []
    return scan_text(text)


def candidates(root: Path) -> list[Path]:
    base = root / SCAN_ROOT
    if not base.is_dir():
        return []
    files = []
    for path in base.rglob("*"):
        if not path.is_file() or path.suffix.lower() not in SUFFIXES:
            continue
        if any(part in SKIP_SEGMENTS for part in path.parts):
            continue
        if "tests" in path.parts:
            continue
        if path.name.endswith((".orig", ".min.js", ".min.css")):
            continue
        files.append(path)
    return sorted(files)


def check(root: Path) -> int:
    violations = 0
    for path in candidates(root):
        for line, snippet in scan_file(path):
            print(f"{path.relative_to(root)}:{line}: literal z-index outside the scale")
            print(f"    {snippet}")
            violations += 1
    if violations:
        print(f"\n{violations} literal z-index value(s) — use a --z-* token")
        return 1
    print("no literal z-index values outside the scale")
    return 0


def list_sites(root: Path) -> int:
    total = 0
    for path in candidates(root):
        for line, snippet in scan_file(path):
            print(f"{path.relative_to(root)}:{line}: {snippet}")
            total += 1
    print(f"\n{total} site(s)")
    return 0


def self_test() -> int:
    good = [
        ".modal { z-index: var(--z-modal); }",
        ".card { z-index: 2; }",
        ".label { z-index: 1 !important; }",
        ".below { z-index: 0; }",
        ".auto { z-index: auto; }",
        "el.style.zIndex = '5';",
        "    --z-modal: 1300;",
    ]
    bad = [
        ".modal { z-index: 1300; }",
        ".toast { z-index: 9999 !important; }",
        ".splash { z-index: 2147483000; }",
        ".under { z-index: -10; }",
        "el.style.zIndex = '100000';",
        "'position:fixed;z-index:10050;'",
    ]
    failures = 0
    for sample in good:
        if scan_text(sample):
            print(f"self-test: false positive on {sample}")
            failures += 1
    for sample in bad:
        if not scan_text(sample):
            print(f"self-test: missed violation in {sample}")
            failures += 1
    if failures:
        return 1
    print("self-test ok")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--list", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--root", type=Path, default=ROOT)
    args = parser.parse_args()
    if not (args.check or args.list or args.self_test):
        parser.print_help()
        return 2
    status = 0
    if args.self_test:
        status |= self_test()
    if args.list:
        status |= list_sites(args.root)
    if args.check:
        status |= check(args.root)
    return status


if __name__ == "__main__":
    raise SystemExit(main())
