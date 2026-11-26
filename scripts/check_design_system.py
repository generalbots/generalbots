#!/usr/bin/env python3
"""Static guard for DESIGN-SYSTEM.md compliance.

Checks (all fail closed, no runtime/browser needed):
  D1 every --id-* token documented in DESIGN-SYSTEM.md is defined in
     botui/ui/suite/css/identity-tokens.css
  D2 every non-default theme overrides ONLY the accent --id-* tokens
     (the default theme equals the design system; other themes vary colour)
  D3 no `outline: ... var(--id-focus)` (--id-focus is a box-shadow token)
  D4 no non-accent --id-* override keyed by a theme id in non-default themes

Usage:
  python3 scripts/check_design_system.py            # scan the repo
  python3 scripts/check_design_system.py --self-test # non-vacuity probes
  python3 scripts/check_design_system.py --strict   # fail on any GAP

Exit 0 = all checks green, 1 = at least one failure.
"""

from __future__ import annotations

import argparse
import re
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
UI = ROOT / "botui" / "ui"
DOC = ROOT / "DESIGN-SYSTEM.md"
TOKENS = UI / "suite" / "css" / "identity-tokens.css"
THEMES = UI / "suite" / "public" / "themes"

# The seven tokens a non-default theme is allowed to change.
ACCENT_TOKENS = frozenset(
    {
        "--id-accent",
        "--id-accent-hover",
        "--id-accent-light",
        "--id-accent-text",
        "--id-accent-soft",
        "--id-accent-glow",
        "--id-on-accent",
    }
)

TOKEN_DECL = re.compile(r"(?m)^\s*(--id-[a-z0-9-]+)\s*:")
DOC_TOKEN = re.compile(r"`(--id-[a-z0-9-]+)`")
FOCUS_OUTLINE = re.compile(
    r"(?is)\boutline\s*:[^;{}]*var\(\s*--id-focus\s*\)"
)
CSS_COMMENT = re.compile(r"/\*.*?\*/", re.S)
THEME_ID_SELECTOR = re.compile(r":root\s*\[\s*data-theme\s*=")
# Tokens written at runtime by identity/accent.js, not declared in the CSS file.
RUNTIME_TOKENS = frozenset({"--id-paper-band"})


def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as exc:
        raise SystemExit(f"[design-system] cannot read {path}: {exc}") from exc


def check_doc_token_parity(doc_text: str, token_text: str) -> list[str]:
    """D1: tokens cited in the design doc must exist in the token file."""
    documented = {
        t for t in DOC_TOKEN.findall(doc_text)
        if t != "--id-*" and t not in RUNTIME_TOKENS
    }
    declared = set(TOKEN_DECL.findall(token_text))
    missing = sorted(documented - declared)
    return [f"D1 documented token not defined in identity-tokens.css: {t}" for t in missing]


def check_theme_accent_only(theme_files: list[Path]) -> list[str]:
    """D2/D4: non-default themes may override only accent tokens."""
    failures: list[str] = []
    for path in theme_files:
        text = read(path)
        overrides = set(TOKEN_DECL.findall(text))
        forbidden = sorted(overrides - ACCENT_TOKENS)
        for token in forbidden:
            failures.append(f"D2 {path.name} overrides non-accent token {token}")
        if THEME_ID_SELECTOR.search(text) and forbidden:
            failures.append(f"D4 {path.name} keys a non-accent identity override by theme id")
    return failures


def check_focus_usage(ui_root: Path) -> list[str]:
    """D3: --id-focus is a box-shadow, never an outline colour."""
    failures: list[str] = []
    for path in ui_root.rglob("*"):
        if not path.is_file() or path.suffix not in {".css", ".html", ".js"}:
            continue
        if "vendor" in path.parts or "tests" in path.parts:
            continue
        text = CSS_COMMENT.sub("", read(path))
        for match in FOCUS_OUTLINE.finditer(text):
            line = text.count("\n", 0, match.start()) + 1
            failures.append(f"D3 {path.relative_to(ui_root)}:{line} outline uses --id-focus")
    return failures


def scan() -> list[str]:
    failures: list[str] = []
    if not DOC.is_file() or not TOKENS.is_file() or not THEMES.is_dir():
        return ["D0 required design-system files are missing"]
    failures += check_doc_token_parity(read(DOC), read(TOKENS))
    theme_files = sorted(
        p for p in THEMES.glob("*.css") if p.name != "default.css"
    )
    if not theme_files:
        failures.append("D0 no non-default theme files found (vacuous accent check)")
    failures += check_theme_accent_only(theme_files)
    failures += check_focus_usage(UI)
    return failures


def self_test() -> list[str]:
    """Non-vacuity: prove each check rejects a deliberately broken fixture."""
    failures: list[str] = []
    with tempfile.TemporaryDirectory(prefix="gb-design-system-") as tmp:
        base = Path(tmp)
        doc = base / "DESIGN-SYSTEM.md"
        tokens = base / "identity-tokens.css"
        theme = base / "evil.css"
        doc.write_text("Use `--id-paper` and `--id-accent`.\n", encoding="utf-8")
        tokens.write_text(":root {\n  --id-accent: #F28C28;\n}\n", encoding="utf-8")
        if not check_doc_token_parity(read(doc), read(tokens)):
            failures.append("SELFTEST D1 did not detect a documented-but-undefined token")

        theme.write_text(":root {\n  --id-paper: #fff;\n}\n", encoding="utf-8")
        if not check_theme_accent_only([theme]):
            failures.append("SELFTEST D2 did not detect a non-accent theme override")

        theme.write_text(
            ':root[data-theme="evil"] {\n  --id-ink: #000;\n}\n', encoding="utf-8"
        )
        if not check_theme_accent_only([theme]):
            failures.append("SELFTEST D4 did not detect a theme-id-scoped override")

        bad = base / "bad.css"
        bad.write_text(".x { outline: 2px solid var(--id-focus); }\n", encoding="utf-8")
        ui = base / "ui"
        ui.mkdir()
        (ui / "bad.css").write_text(read(bad), encoding="utf-8")
        if not check_focus_usage(ui):
            failures.append("SELFTEST D3 did not detect --id-focus used as outline")

        (ui / "bad.css").unlink()
        comment = ui / "comment.css"
        comment.write_text(
            "/* Use box-shadow: var(--id-focus), NEVER outline: var(--id-focus). */\n",
            encoding="utf-8",
        )
        if check_focus_usage(ui):
            failures.append("SELFTEST D3 flagged a documentation comment")
    return failures


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true", help="run non-vacuity probes")
    parser.add_argument("--strict", action="store_true", help="fail on any GAP line")
    args = parser.parse_args()

    if args.self_test:
        problems = self_test()
        label = "SELF-TEST"
    else:
        problems = scan()
        label = "SCAN"

    if problems:
        print(f"[design-system] {label}: FAIL ({len(problems)} problem(s))")
        for problem in problems[:200]:
            print(f"  - {problem}")
        return 1
    print(f"[design-system] {label}: PASS (no problems)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
