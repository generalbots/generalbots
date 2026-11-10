#!/usr/bin/env python3
"""Per-locale and per-app translation coverage report (#2088).

    python3 scripts/i18n_report.py
    python3 scripts/i18n_report.py --json
    python3 scripts/i18n_report.py --app chat

Reads the same catalog and UI scan as ``i18n_lint.py``; it reports, it does not
gate (use ``i18n_lint.py --check`` for that).
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from i18n_lint import ROOT, load_catalog, used_keys  # noqa: E402

APP_ROOT = ROOT / "botui" / "ui" / "suite"


def locale_report(catalog):
    english = catalog.get("en", {})
    rows = []
    for locale, entries in sorted(catalog.items()):
        total = len(entries)
        missing = sum(1 for key in english if key not in entries)
        duplicates = sum(1 for defs in entries.values() if len(defs) > 1)
        plurals = sum(1 for defs in entries.values() if "->" in defs[0][0])
        rows.append(
            {
                "locale": locale,
                "keys": total,
                "missing_vs_en": missing,
                "duplicates": duplicates,
                "plural_blocks": plurals,
            }
        )
    return rows


def app_of(path: str) -> str:
    marker = "botui/ui/suite/"
    if marker in path:
        rest = path.split(marker, 1)[1]
        return rest.split("/", 1)[0]
    if "botui/ui/login" in path:
        return "login"
    if "botui/ui/cloud" in path:
        return "cloud"
    if "botui/ui/wizard" in path:
        return "wizard"
    return "shell"


def app_report(catalog):
    english = catalog.get("en", {})
    used = used_keys()
    per_app = defaultdict(lambda: {"used": 0, "missing": 0})
    for key, paths in used.items():
        app = app_of(paths[0])
        per_app[app]["used"] += 1
        if key not in english:
            per_app[app]["missing"] += 1
    return [dict(app=app, **values) for app, values in sorted(per_app.items())]


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--json", action="store_true", help="machine-readable output")
    parser.add_argument("--app", help="only this app directory")
    args = parser.parse_args(argv)

    catalog = load_catalog()
    locales = locale_report(catalog)
    apps = app_report(catalog)
    if args.app:
        apps = [row for row in apps if row["app"] == args.app]

    if args.json:
        print(json.dumps({"locales": locales, "apps": apps}, indent=1))
        return 0

    print("Locale coverage")
    print(f"{'locale':8} {'keys':>6} {'missing':>8} {'dupes':>6} {'plurals':>8}")
    for row in locales:
        print(
            f"{row['locale']:8} {row['keys']:6} {row['missing_vs_en']:8} "
            f"{row['duplicates']:6} {row['plural_blocks']:8}"
        )

    print("\nApp coverage (keys requested by the UI)")
    print(f"{'app':20} {'used':>6} {'missing':>8}")
    for row in apps:
        print(f"{row['app']:20} {row['used']:6} {row['missing']:8}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
