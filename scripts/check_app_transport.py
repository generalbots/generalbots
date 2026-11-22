#!/usr/bin/env python3
"""#2337 — every active .product app has a documented transport classification.

The registry is a machine-readable JS object in the suite; the active app list
comes from botserver/.product. A new app without a classification fails the gate.
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PRODUCT = ROOT / "botserver/.product"
REGISTRY = ROOT / "botui/ui/suite/js/app-transport.js"
ALLOWED = {"htmx-first", "hybrid", "stateful-exception"}
ENTRY = re.compile(r'"([a-z0-9-]+)"\s*:\s*"(htmx-first|hybrid|stateful-exception)"')


def load() -> tuple[set[str], dict[str, str]]:
    apps: set[str] = set()
    for line in PRODUCT.read_text().splitlines():
        line = line.strip()
        if line.startswith("apps="):
            apps = {a.strip() for a in line.split("=", 1)[1].split(",") if a.strip()}
    registry = dict(ENTRY.findall(REGISTRY.read_text()))
    return apps, registry


def check() -> int:
    apps, registry = load()
    missing = sorted(apps - registry.keys())
    stale = sorted(registry.keys() - apps)
    bad = {k: v for k, v in registry.items() if v not in ALLOWED}
    if missing or stale or bad:
        print(f"app transport: FAIL (missing={missing}, stale={stale}, invalid={bad})")
        return 1
    counts = {k: sum(v == k for v in registry.values()) for k in sorted(ALLOWED)}
    print(f"app transport: OK ({len(registry)} apps, {counts})")
    return 0


def self_test() -> int:
    apps, registry = load()
    assert apps and registry, "empty product or registry"
    assert all(v in ALLOWED for v in registry.values())
    assert apps <= registry.keys(), f"unclassified apps: {sorted(apps - registry.keys())}"
    print(f"app transport self-test: ok ({len(apps)} active apps)")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    return self_test() if args.self_test else check()


if __name__ == "__main__":
    sys.exit(main())
