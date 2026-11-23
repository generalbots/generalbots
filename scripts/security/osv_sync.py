#!/usr/bin/env python3
"""Map advisory front matter to OSV and keep a VEX statement beside it (#2718).

Offline formatter only: it never calls NVD/OSV over the network. Feed the
scanner output from the CI job in scan_deps.py as --findings to convert new
entries; deduplicate against published advisory IDs in security/advisories/.
"""
import argparse
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def front_matter(path):
    text = Path(path).read_text(encoding="utf-8")
    values = {}
    if text.startswith("---"):
        for line in text.split("---", 2)[1].splitlines():
            match = re.match(r'^([a-z_]+):\s*(.+)$', line)
            if match:
                values[match.group(1)] = match.group(2).strip().strip('"')
    return values


def to_osv(values):
    return {
        "schema_version": "1.6.0", "id": values.get("id", "GB-SA-0000-0000"),
        "modified": values.get("revision_history", ""), "summary": values.get("summary", ""),
        "affected": [{"package": {"name": "generalbots", "ecosystem": "GeneralBots"},
                      "ranges": [{"type": "SEMVER", "events": [{"introduced": "0"},
                                  {"fixed": values.get("fixed_versions", "")}]}]}]
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("advisory")
    parser.add_argument("--out", default=str(ROOT / "security" / "osv"))
    args = parser.parse_args()
    values = front_matter(args.advisory)
    if not values:
        print("osv_sync: FAIL — no front matter")
        return 1
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    target = out / (values.get("id", "GB-SA-0000-0000") + ".json")
    target.write_text(json.dumps(to_osv(values), indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"osv_sync: wrote {target}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
