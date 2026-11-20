#!/usr/bin/env python3
"""Type scale gate (issue #2322).

The suite used to author typography as ad-hoc pixel literals: 2 821
`font-size: <n>px` declarations in non-vendor suite CSS, 671 of them below
12 px, all of them immune to browser/OS text scaling. There is now one rem
based scale in `botui/ui/suite/css/base/01_root.css` (`--text-2xs` …
`--text-6xl`), and every suite stylesheet must name a token.

Rules enforced per `font-size` declaration in `botui/ui/suite/**/*.css`
(vendored bundles skipped):

  1. no raw px literal — `font-size: 12px` must be `var(--text-2xs)`
  2. nothing below 12px — `rem`/`em` are allowed but must resolve to at
     least 0.75rem (12px at the default root size)

Allowed without further checks: `var(...)` (a `var()` fallback is still
checked), `clamp()`/`calc()`/`min()`/`max()` (inner px literals are checked),
`inherit`, `initial`, `unset`, `revert`, `revert-layer`, and `%` (resolved
against an unknown parent, so it cannot be judged statically).

The one exception is compact density: a declaration inside a
`[data-density="compact"]` block may go below the floor, because the user
opted into dense data. It still may not be a raw px literal.

Usage
-----
    python3 scripts/check_type_scale.py --check
    python3 scripts/check_type_scale.py --list
    python3 scripts/check_type_scale.py --self-test
    python3 scripts/check_type_scale.py --root <repo>

Exit codes: 0 clean, 1 violations found.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SCAN_ROOT = Path("botui/ui/suite")
SKIP_SEGMENTS = ("vendor", "node_modules")

DECLARATION = re.compile(r"font-size\s*:\s*([^;{}]+)", re.IGNORECASE)
# Inline `style="..."` attributes in suite HTML (issue #2322). Guarded for the
# readability floor only — inline styles legitimately carry px values computed
# from JS-measured dimensions, so the strict CSS "no raw px" rule does not
# apply here; rendering text below 12px does.
STYLE_ATTR = re.compile(r"""style\s*=\s*(["'])(.*?)\1""", re.IGNORECASE | re.DOTALL)
COMMENT = re.compile(r"/\*.*?\*/", re.DOTALL)
NUMBER_UNIT = re.compile(r"([0-9]*\.?[0-9]+)\s*(px|rem|em)\b", re.IGNORECASE)
ALLOWED_KEYWORDS = {"inherit", "initial", "unset", "revert", "revert-layer"}
FUNCTIONAL = ("clamp(", "calc(", "min(", "max(", "var(")
COMPACT = re.compile(r"""data-density\s*=\s*['"]?compact""", re.IGNORECASE)
FLOOR_PX = 12.0
ROOT_PX = 16.0


def strip_comments(text: str) -> str:
    """Blank out comments but keep newlines so line numbers stay valid."""
    return COMMENT.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), text)


def line_of(text: str, index: int) -> int:
    return text.count("\n", 0, index) + 1


def block_spans(text: str) -> list[tuple[int, int, bool]]:
    """Return (start, end, compact) spans for every `{}` block."""
    spans = []
    stack: list[tuple[int, bool]] = []
    for index, char in enumerate(text):
        if char == "{":
            header_start = max(
                text.rfind("}", 0, index),
                text.rfind(";", 0, index),
                text.rfind("{", 0, index),
            ) + 1
            header = text[header_start:index]
            parent_compact = stack[-1][1] if stack else False
            stack.append((index, parent_compact or bool(COMPACT.search(header))))
        elif char == "}" and stack:
            start, compact = stack.pop()
            spans.append((start, index, compact))
    return spans


def enclosing_compact(spans: list[tuple[int, int, bool]], index: int) -> bool:
    best: tuple[int, bool] | None = None
    for start, end, compact in spans:
        if start < index < end and (best is None or start > best[0]):
            best = (start, compact)
    return bool(best and best[1])


def px_of(number: float, unit: str) -> float | None:
    unit = unit.lower()
    if unit == "px":
        return number
    if unit == "rem":
        return number * ROOT_PX
    if unit == "em":
        return number * ROOT_PX
    return None


def classify(raw_value: str, compact: bool) -> str | None:
    """Return a violation message, or None when the declaration is legal."""
    value = raw_value.strip()
    value = re.sub(r"!\s*important\s*$", "", value, flags=re.IGNORECASE).strip()
    if not value:
        return None
    lowered = value.lower()
    if lowered in ALLOWED_KEYWORDS:
        return None
    if lowered.startswith("var("):
        inner = value[value.find("(") + 1:value.rfind(")")]
        if "," in inner:
            return classify(inner.split(",", 1)[1], compact)
        return None
    if lowered.startswith(FUNCTIONAL):
        for match in NUMBER_UNIT.finditer(value):
            px = px_of(float(match.group(1)), match.group(2))
            if px is not None and px < FLOOR_PX and not compact:
                return f"{value} resolves below the {FLOOR_PX:g}px floor"
        return None
    if lowered.endswith("%"):
        return None
    single = NUMBER_UNIT.fullmatch(value)
    if not single:
        other = re.fullmatch(r"[0-9]*\.?[0-9]+\s*([a-z]+)", lowered)
        if other and other.group(1) not in ("px", "rem", "em"):
            return None  # pt/ch/vw/… — not a px literal
        return None
    number = float(single.group(1))
    unit = single.group(2).lower()
    if unit == "px":
        return "raw px font-size — use a --text-* token"
    px = px_of(number, unit)
    if px is not None and px < FLOOR_PX and not compact:
        return f"{value} is below the {FLOOR_PX:g}px readability floor"
    return None


def scan_text(text: str) -> list[tuple[int, str, str]]:
    cleaned = strip_comments(text)
    spans = block_spans(cleaned)
    found = []
    for match in DECLARATION.finditer(cleaned):
        value = match.group(1)
        compact = enclosing_compact(spans, match.start())
        problem = classify(value, compact)
        if problem:
            snippet = " ".join(f"font-size: {value}".split())
            found.append((line_of(cleaned, match.start()), snippet[:160], problem))
    return found


def scan_inline_html(text: str) -> list[tuple[int, str, str]]:
    """Check inline `style=` font-size declarations for the readability floor.

    Unlike stylesheet rules, inline px values are allowed (they are often
    computed from JS-measured dimensions); only a value that resolves below
    FLOOR_PX is a violation. `#2322` found a `.65rem` label that the CSS-only
    guard could not see.
    """
    found = []
    for attr in STYLE_ATTR.finditer(text):
        value = attr.group(2)
        for match in DECLARATION.finditer(value):
            raw = match.group(1).strip()
            problem = floor_problem(raw)
            if problem:
                snippet = " ".join(f"font-size: {raw}".split())
                found.append((line_of(text, attr.start()), snippet[:160], problem))
    return found


def floor_problem(raw_value: str) -> str | None:
    """Return a violation only when a value resolves below the floor.

    `var(--text-*)` is trusted (the token itself is guarded in CSS); functional
    values are checked by numeric literals, as in `classify`.
    """
    value = re.sub(r"!\s*important\s*$", "", raw_value.strip(), flags=re.IGNORECASE).strip()
    lowered = value.lower()
    if not value or lowered in ALLOWED_KEYWORDS or lowered.endswith("%"):
        return None
    if lowered.startswith("var("):
        inner = value[value.find("(") + 1:value.rfind(")")]
        if "," in inner:
            return floor_problem(inner.split(",", 1)[1])
        return None
    for number, unit in NUMBER_UNIT.findall(value):
        px = px_of(float(number), unit)
        if px is not None and px < FLOOR_PX:
            return f"{value} resolves below the {FLOOR_PX:g}px readability floor"
    return None


def scan_file(path: Path) -> list[tuple[int, str, str]]:
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError as exc:
        print(f"warning: cannot read {path}: {exc}", file=sys.stderr)
        return []
    if path.suffix == ".html":
        return scan_inline_html(text)
    return scan_text(text)


def candidates(root: Path) -> list[Path]:
    base = root / SCAN_ROOT
    if not base.is_dir():
        return []
    files = []
    for path in base.rglob("*"):
        if not path.is_file():
            continue
        if path.suffix not in (".css", ".html"):
            continue
        if any(part in SKIP_SEGMENTS for part in path.parts):
            continue
        if path.name.endswith((".orig", ".min.css", ".min.html")):
            continue
        files.append(path)
    return sorted(files)


def check(root: Path) -> int:
    violations = 0
    for path in candidates(root):
        for line, snippet, problem in scan_file(path):
            print(f"{path.relative_to(root)}:{line}: {problem}")
            print(f"    {snippet}")
            violations += 1
    if violations:
        print(f"\n{violations} font-size declaration(s) outside the type scale")
        return 1
    print("no font-size declarations outside the type scale (CSS + inline HTML)")
    return 0


def list_sites(root: Path) -> int:
    total = 0
    tokens = 0
    for path in candidates(root):
        for line, snippet, problem in scan_file(path):
            print(f"{path.relative_to(root)}:{line}: {snippet}")
            total += 1
        text = strip_comments(path.read_text(encoding="utf-8", errors="replace"))
        tokens += len(re.findall(r"font-size\s*:\s*var\(--text-", text))
    print(f"\n{total} violation(s), {tokens} token-based declaration(s)")
    return 0


def self_test() -> int:
    good = [
        ".a { font-size: var(--text-sm); }",
        ".a { font-size: var(--text-2xs) !important; }",
        ".a { font-size: 0.75rem; }",
        ".a { font-size: 1.125rem; }",
        ".a { font-size: inherit; }",
        ".a { font-size: revert-layer; }",
        ".a { font-size: 90%; }",
        ".a { font-size: clamp(0.875rem, 2vw, 1.5rem); }",
        ".a { font-size: var(--x, 1rem); }",
        '[data-density="compact"] .grid { font-size: 0.6875rem; }',
        ".a { font-family: 12px; }",
        "@media (max-width: 600px) { .a { font-size: var(--text-xs); } }",
    ]
    bad = [
        ".a { font-size: 12px; }",
        ".a { font-size: 11px !important; }",
        ".a { font-size:11px; }",
        ".a { font-size: 0.6875rem; }",
        ".a { font-size: 0.7em; }",
        ".a { font-size: var(--x, 10px); }",
        ".a { font-size: clamp(10px, 2vw, 1rem); }",
        ".dark .a { font-size: 13.5px; }",
    ]
    inline_good = [
        '<span style="font-size: var(--text-2xs)" data-i18n="x">(optional)</span>',
        '<div style="width: 290px; font-size: 14px; color: red">ok</div>',
        '<b style="font-size: 0.75rem">ok</b>',
        '<i style="font-size: inherit">ok</i>',
    ]
    inline_bad = [
        '<span style="font-size:.65rem" data-i18n="setup-optional">(optional)</span>',
        '<div style="font-size: 11px">small</div>',
        '<b style="font-size: 0.6875rem; color: red">small</b>',
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
    for sample in inline_good:
        if scan_inline_html(sample):
            print(f"self-test: inline false positive on {sample}")
            failures += 1
    for sample in inline_bad:
        if not scan_inline_html(sample):
            print(f"self-test: inline missed violation in {sample}")
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
