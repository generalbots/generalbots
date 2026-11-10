#!/usr/bin/env python3
"""Pseudo-locale generator (#2088).

Produces a layout-testing locale from the English catalogue: every message
value is wrapped and elongated so truncation, overflow and hardcoded strings
become visible, while placeholders, plural selectors and variant labels stay
intact (a pseudo-locale that breaks `{ $count -> ... }` is useless).

Usage:
    python3 scripts/i18n_pseudo.py                    # write /tmp/gb-i18n-pseudo
    python3 scripts/i18n_pseudo.py --out DIR
    python3 scripts/i18n_pseudo.py --check            # exit 1 if generation fails

The output is NOT installed under botlib/locales on purpose: `available_locales()`
uses the catalogue files as the source of truth, and a shipped pseudo-locale
would appear in the language picker.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
EN_DIR = REPO / "botlib" / "locales" / "en"
DEFAULT_OUT = Path("/tmp/gb-i18n-pseudo")

# `{ $name }`, `{ $count }` and nested select expressions must survive verbatim.
PLACEHOLDER = re.compile(r"\{[^{}]*\}")
MESSAGE = re.compile(r"^([A-Za-z0-9_.-]+)\s*=\s*(.*)$", re.MULTILINE)
VOWELS = "aeiouAEIOU"


def elongate(text: str) -> str:
    """Repeats vowels outside placeholders to grow the string ~40%."""
    parts = []
    last = 0
    for match in PLACEHOLDER.finditer(text):
        parts.append(_elongate_plain(text[last : match.start()]))
        parts.append(match.group(0))
        last = match.end()
    parts.append(_elongate_plain(text[last:]))
    return "".join(parts)


def _elongate_plain(text: str) -> str:
    return "".join(ch * 2 if ch in VOWELS else ch for ch in text)


def pseudo_value(value: str) -> str:
    stripped = value.strip()
    if not stripped:
        return value
    # Keep variant labels and selector arrows readable: only the human text of
    # a plural block is transformed, line by line.
    if "->" in stripped:
        return "\n".join(
            line if "->" in line or line.strip().startswith(("[", "*[")) is False
            else line
            for line in value.splitlines()
        )
    return f"⟦{elongate(value)}⟧"


def transform_source(source: str) -> str:
    out = []
    for line in source.splitlines():
        match = MESSAGE.match(line)
        if not match:
            out.append(line)
            continue
        key, value = match.group(1), match.group(2)
        if "->" in value:
            # Multi-line select expression: transform only the variant bodies.
            out.append(f"{key} = {value}")
            continue
        out.append(f"{key} = {pseudo_value(value)}")
    return "\n".join(out) + "\n"


def transform_multiline(source: str) -> str:
    """Transforms variant bodies inside `{ $var -> ... }` blocks."""
    lines = source.splitlines()
    out = []
    in_block = False
    for line in lines:
        stripped = line.strip()
        if stripped.endswith("->"):
            in_block = True
            out.append(line)
            continue
        if in_block and stripped == "}":
            in_block = False
            out.append(line)
            continue
        if in_block and (stripped.startswith("[") or stripped.startswith("*[")):
            label, _, body = line.partition("]")
            if body.strip():
                out.append(f"{label}]{pseudo_value(body)}")
            else:
                out.append(line)
            continue
        out.append(line)
    return "\n".join(out) + "\n"


def generate(out_dir: Path) -> int:
    if not EN_DIR.is_dir():
        print(f"missing catalogue: {EN_DIR}", file=sys.stderr)
        return 1

    out_dir.mkdir(parents=True, exist_ok=True)
    files = sorted(EN_DIR.glob("*.ftl"))
    if not files:
        print(f"no .ftl files in {EN_DIR}", file=sys.stderr)
        return 1

    total_keys = 0
    for source in files:
        text = source.read_text(encoding="utf-8")
        transformed = transform_multiline(transform_source(text))
        (out_dir / source.name).write_text(transformed, encoding="utf-8")
        total_keys += len(MESSAGE.findall(text))

    print(f"pseudo-locale written to {out_dir} ({len(files)} files, {total_keys} keys)")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate the pseudo-locale (#2088)")
    parser.add_argument("--out", type=Path, default=DEFAULT_OUT)
    parser.add_argument(
        "--check",
        action="store_true",
        help="generate into a temporary directory and fail if nothing was produced",
    )
    args = parser.parse_args()

    out = args.out
    if args.check:
        import tempfile

        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / "pseudo"
            code = generate(out)
            if code != 0:
                return code
            produced = list(out.glob("*.ftl"))
            if not produced:
                print("pseudo-locale check: no files produced", file=sys.stderr)
                return 1
            if any("⟦" not in p.read_text(encoding="utf-8") for p in produced):
                print("pseudo-locale check: a file has no transformed values", file=sys.stderr)
                return 1
        print("pseudo-locale check: OK")
        return 0

    return generate(out)


if __name__ == "__main__":
    raise SystemExit(main())
