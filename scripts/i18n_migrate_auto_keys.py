#!/usr/bin/env python3
"""One-shot migration that retires the machine-generated ``auto-`` key class (#2542).

The ``auto-*`` keys were produced by extracting English literals
(``ui-auto-01.ftl`` … ``ui-auto-04.ftl``). They carry no semantics, nobody
references them, and AGENTS.md forbids inventing them. This script:

* renames the handful of ``auto-`` keys the UI still references through an
  explicit ``RENAMES`` mapping (semantic ``<app>-<element>`` key, English value
  kept, every locale that already had a translation keeps it), and
* deletes every unreferenced ``auto-`` key from every locale catalog.

Run once from the repository root, then commit the script output:

    python3 scripts/i18n_migrate_auto_keys.py --dry-run
    python3 scripts/i18n_migrate_auto_keys.py

After the migration ``grep -rn '^auto-' botlib/locales`` must be empty; the
linter enforces that with its ``AUTO_KEY`` violation.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LOCALES = ROOT / "botlib" / "locales"
UI = ROOT / "botui" / "ui"

KEY_RE = re.compile(r"^([A-Za-z0-9][^=]*?) = (.*)$")
UI_ATTR_RE = re.compile(
    r"data-i18n(?:-placeholder|-title|-aria-label|-document-title)?=\"([^\"]+)\""
)
UI_CALL_RE = re.compile(r"""(?:\bgbT|\btOr|\bt)\(\s*['"]([^'"]+)['"]""")

# ``auto-`` keys the UI still uses → semantic replacement. Keep this list
# explicit: a rename must be a deliberate human decision, not a slug move.
RENAMES = {
    "auto-rotate-2": "chat-projector-rotate",
}


def locale_files() -> list[Path]:
    return sorted(LOCALES.glob("*/*.ftl"))


def ui_files() -> list[Path]:
    return [
        path
        for path in UI.rglob("*")
        if path.suffix in {".html", ".js"} and "vendor" not in path.parts
    ]


def ui_references() -> dict[str, set[str]]:
    refs: dict[str, set[str]] = {}
    for path in ui_files():
        text = path.read_text(encoding="utf-8", errors="ignore")
        for match in UI_ATTR_RE.finditer(text):
            refs.setdefault(match.group(1), set()).add(str(path.relative_to(ROOT)))
        for match in UI_CALL_RE.finditer(text):
            refs.setdefault(match.group(1), set()).add(str(path.relative_to(ROOT)))
    return refs


def catalog_keys(locale: str) -> set[str]:
    keys: set[str] = set()
    for path in sorted((LOCALES / locale).glob("*.ftl")):
        for line in path.read_text(encoding="utf-8").splitlines():
            match = KEY_RE.match(line)
            if match:
                keys.add(match.group(1).strip())
    return keys


def rewrite_key(path: Path, old: str, new: str) -> bool:
    """Rename one definition in place, keeping the value and continuation lines."""
    lines = path.read_text(encoding="utf-8").splitlines(keepends=True)
    changed = False
    for index, line in enumerate(lines):
        match = KEY_RE.match(line.rstrip("\n"))
        if match and match.group(1).strip() == old:
            lines[index] = line.replace(f"{old} =", f"{new} =", 1)
            changed = True
    if changed:
        path.write_text("".join(lines), encoding="utf-8")
    return changed


def drop_key(path: Path, key: str) -> bool:
    """Remove one definition and its indented continuation lines."""
    lines = path.read_text(encoding="utf-8").splitlines(keepends=True)
    out: list[str] = []
    dropped = False
    skip = False
    for line in lines:
        if skip:
            if line.startswith((" ", "\t")):
                continue
            skip = False
        match = KEY_RE.match(line.rstrip("\n"))
        if match and match.group(1).strip() == key:
            dropped = True
            skip = True
            continue
        out.append(line)
    if dropped:
        path.write_text("".join(out), encoding="utf-8")
    return dropped


def rename_references(files: list[Path], old: str, new: str) -> list[str]:
    touched: list[str] = []
    for path in files:
        text = path.read_text(encoding="utf-8", errors="ignore")
        if f'"{old}"' not in text and f"'{old}'" not in text:
            continue
        updated = text.replace(f'"{old}"', f'"{new}"').replace(f"'{old}'", f"'{new}'")
        path.write_text(updated, encoding="utf-8")
        touched.append(str(path.relative_to(ROOT)))
    return touched


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dry-run", action="store_true", help="report, change nothing")
    args = parser.parse_args()

    keys: set[str] = set()
    for locale_dir in sorted(LOCALES.iterdir()):
        if locale_dir.is_dir():
            keys |= catalog_keys(locale_dir.name)
    auto_keys = {key for key in keys if key.startswith("auto-")}
    if not auto_keys:
        print("nothing to migrate: no auto- keys left")
        return 0

    refs = ui_references()
    referenced = {key for key in auto_keys if key in refs}
    orphaned = auto_keys - referenced
    unmapped = referenced - set(RENAMES)

    print(f"auto- keys across all locales: {len(auto_keys)}")
    print(f"  referenced by the UI: {len(referenced)}")
    print(f"  orphaned (delete):    {len(orphaned)}")
    if unmapped:
        for key in sorted(unmapped):
            print(f"  UNMAPPED, add it to RENAMES: {key} -> {sorted(refs[key])}")
        return 1

    files = [path for path in locale_files() if path.suffix == ".ftl"]
    if args.dry_run:
        for old, new in RENAMES.items():
            if old in auto_keys:
                print(f"  rename {old} -> {new} in {len(files)} catalogs")
        print(f"  delete {len(orphaned)} orphaned keys from {len(files)} catalogs")
        return 0

    sources = ui_files()
    for old, new in sorted(RENAMES.items()):
        if old not in auto_keys:
            continue
        touched = rename_references(sources, old, new)
        renamed = [path for path in files if rewrite_key(path, old, new)]
        print(f"  {old} -> {new}: {len(touched)} UI files, {len(renamed)} catalogs")

    removed = 0
    for path in files:
        for key in sorted(orphaned):
            if drop_key(path, key):
                removed += 1
    print(f"  deleted {removed} definitions")

    for path in files:
        if path.name.startswith("ui-auto-") and not path.read_text(encoding="utf-8").strip():
            path.unlink()
            print(f"  removed empty {path.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
