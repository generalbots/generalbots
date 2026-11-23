#!/usr/bin/env python3
"""Answer "is my installation affected?" from the API or an offline bundle."""
import argparse
import json
import re
import sys
from pathlib import Path


def parse_version(value):
    return tuple(int(part) if part.isdigit() else 0 for part in re.findall(r"[0-9]+", value))


def affected(ranges, version):
    for item in ranges:
        match = re.match(r"^<\s*([0-9.]+)$", item.strip())
        if match and parse_version(version) < parse_version(match.group(1)):
            return True
    return False


def load(args):
    if args.bundle:
        path = Path(args.bundle)
        return json.loads(path.read_text(encoding="utf-8"))["advisories"]
    import urllib.request
    with urllib.request.urlopen(args.api, timeout=15) as response:
        payload = json.loads(response.read().decode("utf-8"))
    return payload["advisories"] if isinstance(payload, dict) else payload


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("product")
    parser.add_argument("version")
    parser.add_argument("--bundle")
    parser.add_argument("--api", default="https://generalbots.org/api/advisories")
    args = parser.parse_args()
    try:
        advisories = load(args)
    except (OSError, ValueError, KeyError) as error:
        print("audit: ERROR — " + str(error))
        return 2
    hits = [item for item in advisories
            if item.get("product") == args.product and affected(item.get("affected", []), args.version)]
    if hits:
        print("AFETADO / AFFECTED — " + args.product + " " + args.version)
        for item in hits:
            print(f"  {item['id']} — fixed in {item.get('fixed', 'unknown')}")
        return 1
    print("NAO AFETADO / NOT AFFECTED — " + args.product + " " + args.version)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
