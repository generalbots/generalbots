#!/usr/bin/env python3
"""Per-app complexity budget (#2120).

68 apps and 115 Cargo features multiply every test, doc, review and support
obligation, so the surface needs a declared ceiling instead of growing by
accretion. `scripts/app_complexity_budget.toml` states, per app:

- ``routes``     REST endpoints the app owns (`ENDPOINT_PREFIX_APPS` mapped
                 over the route inventory),
- ``js_modules`` JavaScript files in the app's suite surface,
- ``loc``        lines of UI code (JS/HTML/CSS) in that surface.

The budget is a ceiling, not a snapshot (same contract as
``scripts/adequacy_guard.py``): a metric may stay equal or fall, never rise. A
change that adds a route, a module or a thousand lines must either justify a
new ceiling in the same PR or split/delete code.

Usage:
    python3 scripts/app_complexity_budget.py
    python3 scripts/app_complexity_budget.py --update   # re-baseline

Exit codes: 0 = at/below budget, 1 = over budget, 2 = usage/budget error.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # Python < 3.11
    try:
        import tomli as tomllib  # type: ignore
    except ModuleNotFoundError:
        print("app_complexity_budget: Python 3.11+ (tomllib) is required", file=sys.stderr)
        raise SystemExit(2)

ROOT = Path(__file__).resolve().parents[1]
BUDGET = ROOT / "scripts" / "app_complexity_budget.toml"
REGISTRY = ROOT / "botserver" / "src" / "apps" / "registry"
SUITE = ROOT / "botui" / "ui" / "suite"
PREFIX_MAP = ROOT / "botserver" / "src" / "core" / "bot" / "endpoint_prefix_map.rs"
INVENTORY_DIR = ROOT / "botserver" / "src" / "core" / "bot"
METRICS = ("routes", "js_modules", "loc")
UI_SUFFIXES = {".js", ".html", ".css"}
REMEDIATION = {
    "routes": "bind fewer endpoints, or raise the ceiling with an owner note",
    "js_modules": "merge or delete modules (split at 350 lines, hard limit 450)",
    "loc": "delete dead UI code or split the app into a smaller surface",
}


def registry_ids() -> list[str]:
    ids: set[str] = set()
    for path in REGISTRY.glob("*.rs"):
        ids.update(re.findall(r'id:\s*"([^"]+)"', path.read_text(encoding="utf-8")))
    return sorted(ids)


def route_owners() -> dict[str, int]:
    """Routes per app: longest matching `ENDPOINT_PREFIX_APPS` prefix wins."""
    prefixes = [
        (prefix, app)
        for prefix, app in re.findall(
            r'\("(/[^"]*)",\s*"([^"]+)"\)',
            PREFIX_MAP.read_text(encoding="utf-8"),
        )
    ]
    prefixes.sort(key=lambda entry: len(entry[0]), reverse=True)
    paths: set[str] = set()
    for path in sorted(INVENTORY_DIR.glob("endpoint_inventory*.rs")):
        paths.update(re.findall(r'path:\s*"([^"]+)"', path.read_text(encoding="utf-8")))
    counts: dict[str, int] = {}
    for route in paths:
        for prefix, app in prefixes:
            if route == prefix or route.startswith(prefix + "/"):
                counts[app] = counts.get(app, 0) + 1
                break
    return counts


def app_surfaces(ids: list[str]) -> dict[str, list[Path]]:
    """Files of each app's suite surface (directory first, then root prefix)."""
    files: dict[str, list[Path]] = {app: [] for app in ids}
    for entry in sorted(SUITE.iterdir()):
        if entry.is_dir() and entry.name in files:
            files[entry.name].extend(
                path for path in sorted(entry.rglob("*"))
                if path.is_file() and path.suffix in UI_SUFFIXES
            )
    by_length = sorted(ids, key=len, reverse=True)
    for entry in sorted(SUITE.iterdir()):
        if not entry.is_file() or entry.suffix not in UI_SUFFIXES:
            continue
        for app in by_length:
            if entry.stem == app or entry.stem.startswith(app + "-"):
                files[app].append(entry)
                break
    return files


def count_lines(path: Path) -> int:
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return 0
    return text.count("\n") + (0 if not text or text.endswith("\n") else 1)


def measure() -> dict[str, dict[str, int]]:
    ids = registry_ids()
    routes = route_owners()
    surfaces = app_surfaces(ids)
    measured: dict[str, dict[str, int]] = {}
    for app in ids:
        files = surfaces[app]
        measured[app] = {
            "routes": routes.get(app, 0),
            "js_modules": sum(1 for path in files if path.suffix == ".js"),
            "loc": sum(count_lines(path) for path in files),
        }
    return measured


def load_budget(path: Path) -> tuple[dict[str, int], dict[str, dict[str, int]]]:
    try:
        with path.open("rb") as handle:
            data = tomllib.load(handle)
    except OSError as error:
        raise SystemExit(f"app_complexity_budget: cannot read {path}: {error}")
    defaults = data.get("defaults")
    if not isinstance(defaults, dict) or any(key not in defaults for key in METRICS):
        raise SystemExit(f"app_complexity_budget: {path} needs [defaults] with {', '.join(METRICS)}")
    apps = data.get("apps", {})
    if not isinstance(apps, dict):
        raise SystemExit(f"app_complexity_budget: {path} has no [apps] table")
    ceilings: dict[str, dict[str, int]] = {}
    for app, entry in apps.items():
        if not isinstance(entry, dict):
            raise SystemExit(f"app_complexity_budget: [apps.\"{app}\"] must be a table")
        ceilings[app] = {
            metric: entry.get(metric, defaults[metric]) for metric in METRICS
        }
    return defaults, ceilings


def write_budget(path: Path, measured: dict[str, dict[str, int]], defaults: dict[str, int]) -> None:
    lines = [
        "# Per-app complexity budget (#2120).",
        "#",
        "# Ceilings, not snapshots: a metric may stay equal or fall, never rise.",
        "# A PR that grows one of them must justify the new ceiling here (owner",
        "# note) or split/delete code. Re-baseline with --update.",
        "#",
        "# routes     REST endpoints owned through ENDPOINT_PREFIX_APPS",
        "# js_modules JavaScript files under botui/ui/suite/<app>/",
        "# loc        lines of UI code (JS/HTML/CSS) of the app surface",
        "",
        "[defaults]",
        *(f"{metric} = {defaults[metric]}" for metric in METRICS),
        "",
    ]
    for app in sorted(measured):
        lines.append(f'[apps."{app}"]')
        lines.extend(f"{metric} = {measured[app][metric]}" for metric in METRICS)
        lines.append("")
    path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")
    print(f"app_complexity_budget: budget written to {path.relative_to(ROOT)}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Per-app complexity budget (#2120)")
    parser.add_argument("--budget", default=str(BUDGET), help="budget TOML path")
    parser.add_argument("--update", action="store_true", help="re-baseline the budget from the tree")
    parser.add_argument("--self-test", action="store_true", help="prove the gate fails closed")
    args = parser.parse_args()
    if args.self_test:
        from gate_selftests import run_self_test

        return run_self_test("app-budget")
    path = Path(args.budget)

    measured = measure()
    defaults = {metric: 0 for metric in METRICS}
    if path.exists():
        defaults, ceilings = load_budget(path)
    elif not args.update:
        print(f"app_complexity_budget: missing {path.relative_to(ROOT)} (run --update)", file=sys.stderr)
        return 2
    else:
        ceilings = {}

    if args.update:
        for app in measured:
            for metric in METRICS:
                defaults[metric] = max(defaults[metric], measured[app][metric])
        write_budget(path, measured, defaults)
        return 0

    over: list[str] = []
    missing = sorted(set(measured) - set(ceilings))
    stale = sorted(set(ceilings) - set(measured))
    for app in missing:
        over.append(f'`{app}` has no budget row (add [apps."{app}"])')
    for app in stale:
        over.append(f'budget row `{app}` names no registry app (delete it)')
    for app in sorted(measured):
        ceiling = ceilings.get(app)
        if ceiling is None:
            continue
        for metric in METRICS:
            current = measured[app][metric]
            if current > ceiling[metric]:
                over.append(
                    f"{app}: {metric} {current} > budget {ceiling[metric]} "
                    f"(+{current - ceiling[metric]})"
                )
                print(f"  remediation: {REMEDIATION[metric]}")

    if over:
        print("app complexity budget exceeded:", file=sys.stderr)
        for line in over:
            print(f"  - {line}", file=sys.stderr)
        return 1

    totals = {metric: sum(app[metric] for app in measured.values()) for metric in METRICS}
    print(
        f"app complexity budget: OK — {len(measured)} apps within budget "
        f"({totals['routes']} routes, {totals['js_modules']} JS modules, "
        f"{totals['loc']} UI lines)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
