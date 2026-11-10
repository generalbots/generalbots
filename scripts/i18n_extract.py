#!/usr/bin/env python3
"""Translator handoff for the Fluent catalogs (#2088).

    python3 scripts/i18n_extract.py --locale pt-BR
    python3 scripts/i18n_extract.py --locale de --format csv --output /tmp/de.csv

Emits the keys that locale is missing (or whose placeholders disagree with
``en``), with the English source, the placeholder names and whether the message
is a plural block. Existing translations are never rewritten: the file is a
handoff, and merging stays with the translator/reviewer.
"""

from __future__ import annotations

import argparse
import csv
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from i18n_lint import load_catalog, placeholders  # noqa: E402


def rows_for(locale: str):
    catalog = load_catalog()
    english = catalog.get("en", {})
    target = catalog.get(locale, {})
    rows = []
    for key in sorted(english):
        source = english[key][0][0]
        current = target.get(key)
        if current is None:
            status = "missing"
        elif placeholders(current[0][0]) != placeholders(source):
            status = "placeholder-mismatch"
        else:
            continue
        rows.append(
            {
                "key": key,
                "status": status,
                "source": source,
                "placeholders": ", ".join(placeholders(source)),
                "plural": "yes" if "->" in source else "no",
                "current": current[0][0] if current else "",
            }
        )
    return rows


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--locale", required=True, help="target locale, for example pt-BR")
    parser.add_argument("--format", choices=["json", "csv"], default="json")
    parser.add_argument("--output", help="write to this file instead of stdout")
    args = parser.parse_args(argv)

    rows = rows_for(args.locale)
    if args.format == "csv":
        import io

        buffer = io.StringIO()
        writer = csv.DictWriter(
            buffer,
            fieldnames=["key", "status", "source", "placeholders", "plural", "current"],
        )
        writer.writeheader()
        writer.writerows(rows)
        payload = buffer.getvalue()
    else:
        payload = json.dumps({"locale": args.locale, "rows": rows}, indent=1, ensure_ascii=False) + "\n"

    if args.output:
        Path(args.output).write_text(payload, encoding="utf-8")
        print(f"wrote {len(rows)} row(s) for {args.locale} to {args.output}")
    else:
        sys.stdout.write(payload)
    return 0


if __name__ == "__main__":
    sys.exit(main())
