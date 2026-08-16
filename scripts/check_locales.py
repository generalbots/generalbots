#!/usr/bin/env python3
"""Locale parity gate.

Verifies that every FTL catalog under `botlib/locales/<locale>/` contains all
message keys present in the English (en) reference catalog. Fails (exit 1) on
any missing key so CI can block merges on translation gaps.

Usage:
    python3 scripts/check_locales.py [--root .] [--locales es,zh-CN,fr,de,ja,ko]

Exit codes:
    0  all checked locales have full parity with en
    1  at least one locale is missing keys (details printed to stderr)
"""

import argparse
import os
import re
import sys

KEY_RE = re.compile(r"^([a-zA-Z0-9_-]+)\s*=")


def load_keys(locale_dir: str) -> dict[str, str]:
    """Return {message_key: filename} for every key defined in a catalog dir."""
    keys: dict[str, str] = {}
    if not os.path.isdir(locale_dir):
        return keys
    for name in sorted(os.listdir(locale_dir)):
        if not name.endswith(".ftl"):
            continue
        path = os.path.join(locale_dir, name)
        with open(path, encoding="utf-8") as fh:
            for line in fh:
                match = KEY_RE.match(line)
                if match:
                    keys[match.group(1)] = name
    return keys


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", default=".", help="repository root (default: .)")
    parser.add_argument(
        "--locales",
        default="es,zh-CN,fr,de,ja,ko",
        help="comma-separated locales to check (default: es,zh-CN,fr,de,ja,ko)",
    )
    args = parser.parse_args()

    locales_dir = os.path.join(args.root, "botlib", "locales")
    en = load_keys(os.path.join(locales_dir, "en"))
    if not en:
        print(f"error: no English reference catalog found under {locales_dir}", file=sys.stderr)
        return 1

    failed = False
    for locale in [part.strip() for part in args.locales.split(",") if part.strip()]:
        catalog = load_keys(os.path.join(locales_dir, locale))
        missing = sorted(set(en) - set(catalog))
        if missing:
            failed = True
            print(f"locale '{locale}': {len(missing)} missing keys", file=sys.stderr)
            for key in missing:
                print(f"  {key}  (defined in {en[key]})", file=sys.stderr)
        else:
            print(f"locale '{locale}': OK ({len(catalog)} keys, full parity)")

    if failed:
        print("i18n parity check FAILED", file=sys.stderr)
        return 1
    print(f"i18n parity check passed for {len([p for p in args.locales.split(',') if p.strip()])} locales")
    return 0


if __name__ == "__main__":
    sys.exit(main())
