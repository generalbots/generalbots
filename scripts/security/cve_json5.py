#!/usr/bin/env python3
"""Generate a CVE JSON 5.0 record from advisory front matter (#2715).

Reads the YAML-like front matter used by security/ADVISORY-TEMPLATE.md without
third-party packages and writes the canonical 5.0 record to stdout.
"""
import json
import re
import sys

REQUIRED = ["id", "title", "summary", "affected_versions", "fixed_versions", "cwe", "revision_history", "author"]


def front_matter(path):
    text = open(path, encoding="utf-8").read()
    if not text.startswith("---"):
        raise ValueError("advisory has no front matter")
    body = text.split("---", 2)[1]
    values = {}
    for line in body.splitlines():
        match = re.match(r'^([a-z_]+):\s*(.+)$', line)
        if match:
            value = match.group(2).strip().strip('"')
            values[match.group(1)] = value
    return values


def record(values):
    missing = [key for key in REQUIRED if key not in values]
    if missing:
        raise ValueError("missing required front matter: " + ", ".join(missing))
    return {
        "format": "MITRE CVE 5.0",
        "dataVersion": "5.0",
        "cveMetadata": {"cveId": values.get("cve", "CVE-2026-0000"), "state": "PUBLIC"},
        "containers": {"cna": {
            "title": values["title"],
            "descriptions": [{"lang": "en", "value": values["summary"]}],
            "affected": [{"vendor": "General Bots", "product": "General Bots",
                          "versions": [{"version": values["affected_versions"], "status": "affected"}]}],
            "problemTypes": [{"descriptions": [{"cweId": values["cwe"], "lang": "en", "description": values["cwe"]}]}],
            "references": [],
            "providerMetadata": {"orgId": "generalbots", "shortName": "General Bots"}
        }}
    }


def main(argv):
    if len(argv) != 1:
        print("usage: cve_json5.py <advisory.md>")
        return 2
    try:
        data = record(front_matter(argv[0]))
    except (ValueError, OSError) as error:
        print("cve_json5: FAIL — " + str(error))
        return 1
    print(json.dumps(data, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
