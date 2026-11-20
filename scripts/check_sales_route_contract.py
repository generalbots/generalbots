#!/usr/bin/env python3
"""Offline guard for the #2540 `/api/sales/*` -> `/api/crm/*` consolidation.

The duplicate funnel/forecast/tracked-email surface is tombstoned in
``botserver/crates/botcontacts/src/routes_sales.rs``: each retired path answers
410 Gone with a canonical hint, and the canonical successor must be a real,
mounted `/api/crm/*` route. This guard proves the three criteria that need no
browser:

  1. ``botui/ui`` contains ZERO first-party references to the retired prefix
     (the string is allowed only in botbook/docs and the tombstone Rust file).
  2. Every retired path in ``routes_sales.rs`` has a canonical ``/api/crm/*``
     successor that is actually mounted by a ``.route()`` registration.
  3. The tombstone surface is shrink-only: ``routes_sales.rs`` mounts at most
     SEVEN ``/api/sales/*`` routes (baseline from the prior slice; the surface
     is retired entirely in a future release, never grown).

Style mirrors ``scripts/route_contract_status.py`` and
``scripts/check_raw_error_strings.py``. Read-only: touches no files, wires no
CI (the driver does that). Usage:

    python3 scripts/check_sales_route_contract.py            # --check (default)
    python3 scripts/check_sales_route_contract.py --check
    python3 scripts/check_sales_route_contract.py --self-test

Exit codes: 0 = all criteria pass, 1 = at least one violation, 2 = usage/IO error.
"""
from __future__ import annotations

import argparse
import os
import re
import sys
from dataclasses import dataclass
from pathlib import Path

REPO_ROOT = Path(
    os.environ.get("GB_SALES_CONTRACT_ROOT") or Path(__file__).resolve().parents[1]
).resolve()
BOTSERVER = REPO_ROOT / "botserver"
BOTUI_UI = REPO_ROOT / "botui" / "ui"
TOMBSTONE_FILE = BOTSERVER / "crates" / "botcontacts" / "src" / "routes_sales.rs"

SALES_PREFIX = "/api/sales"
CANONICAL_PREFIX = "/api/crm/"
SALES_ROUTE_BASELINE = 7
SKIP_DIRS = {".git", ".venv", "__pycache__", "node_modules", "target", "vendor", "work"}

PARAM_PATTERN = re.compile(
    r":([A-Za-z_][A-Za-z0-9_]*)|\{([A-Za-z_][A-Za-z0-9_]*)\}|\*([A-Za-z_][A-Za-z0-9_]*)"
)
GONE_HANDLER = re.compile(
    r'gone_handler!\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*,\s*"([^"]+)"\s*\)'
)
ROUTE_PATH = re.compile(r'\.route\(\s*"([^"]+)"')
SALES_ROUTE = re.compile(
    r'\.route\(\s*"([^"]+)"\s*,\s*any\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)'
)


def _canonical(path: str) -> str:
    """Normalise ``:id`` / ``{id}`` / ``*id`` params to ``{id}`` (as in check_route_contract)."""
    return PARAM_PATTERN.sub(
        lambda found: "{" + next(group for group in found.groups() if group) + "}", path
    )


def _read(path: Path) -> str | None:
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return None


def _iter_files(base: Path) -> list[Path]:
    if not base.is_dir():
        return []
    found: list[Path] = []
    for dirpath, dirnames, filenames in os.walk(base):
        dirnames[:] = sorted(name for name in dirnames if name not in SKIP_DIRS)
        found.extend(Path(dirpath) / name for name in sorted(filenames))
    return found


def _rel(path: Path) -> str:
    try:
        return path.relative_to(REPO_ROOT).as_posix()
    except ValueError:
        return str(path)


def scan_first_party_refs() -> list[tuple[str, int]]:
    """(relative path, literal occurrence count) for the retired prefix under botui/ui."""
    hits: list[tuple[str, int]] = []
    for path in _iter_files(BOTUI_UI):
        text = _read(path)
        if text is None:
            continue
        count = text.count(SALES_PREFIX)
        if count:
            hits.append((_rel(path), count))
    return sorted(hits)


def parse_tombstones(text: str) -> tuple[dict[str, str], list[str], int]:
    """Return (retired -> canonical, orphan handler names, retired route count)."""
    canonical_by_handler = {name: canon for name, canon in GONE_HANDLER.findall(text)}
    pairs = [
        (path, handler)
        for path, handler in SALES_ROUTE.findall(text)
        if path.startswith(SALES_PREFIX + "/")
    ]
    mapping: dict[str, str] = {}
    used_handlers: set[str] = set()
    for retired, handler in pairs:
        canon = canonical_by_handler.get(handler)
        if canon:
            mapping[_canonical(retired)] = _canonical(canon)
            used_handlers.add(handler)
    orphans = sorted(set(canonical_by_handler) - used_handlers)
    return mapping, orphans, len(pairs)


def parse_mounted_paths() -> set[str]:
    """Canonicalised literal ``.route("...")`` paths across the whole botserver tree."""
    mounted: set[str] = set()
    for path in _iter_files(BOTSERVER):
        if path.suffix != ".rs":
            continue
        text = _read(path)
        if text is None:
            continue
        mounted.update(_canonical(route) for route in ROUTE_PATH.findall(text))
    return mounted


@dataclass(frozen=True)
class Criterion:
    label: str
    observed: str
    problems: tuple[str, ...]


def evaluate(
    ui_hits: list[tuple[str, int]],
    mapping: dict[str, str],
    orphans: list[str],
    tombstone_count: int,
    mounted: set[str],
    tombstone_file_present: bool,
) -> list[Criterion]:
    """Pure evaluator shared by the CLI and the self-test."""
    selected = sorted(canon for canon in mapping.values() if canon.startswith(CANONICAL_PREFIX))
    missing = sorted(canon for canon in selected if canon not in mounted)
    non_crm = sorted(set(mapping.values()) - set(selected))

    ui_problems = [f"{path} references {SALES_PREFIX} ({count}x)" for path, count in ui_hits]
    map_problems = [
        f"{canon} has no mounted /api/crm/* route" for canon in missing
    ] + [
        f"{next((r for r, c in mapping.items() if c == canon), '?')} maps outside "
        f"{CANONICAL_PREFIX}: {canon}"
        for canon in non_crm
    ] + [
        f"gone_handler! {handler} has no retired /api/sales/* route" for handler in orphans
    ]
    if not tombstone_file_present:
        map_problems = [
            problem for problem in map_problems if "gone_handler!" not in problem
        ]

    shrink_problems: list[str] = []
    if tombstone_count > SALES_ROUTE_BASELINE:
        shrink_problems.append(
            f"{tombstone_count} retired routes exceed baseline {SALES_ROUTE_BASELINE}"
        )

    if not tombstone_file_present:
        observed_map = "tombstone file absent (surface fully retired)"
        observed_shrink = "0 routes (fully retired)"
    else:
        observed_map = (
            f"{len(mapping)} retired paths, {len(selected)} canonical /api/crm/*, "
            f"{len(missing)} unmounted"
        )
        observed_shrink = (
            f"{tombstone_count} routes (baseline {SALES_ROUTE_BASELINE}, max "
            f"{SALES_ROUTE_BASELINE})"
        )
    total_refs = sum(count for _path, count in ui_hits)
    return [
        Criterion(
            "no first-party /api/sales refs in botui/ui",
            f"{total_refs} literal occurrences in {len(ui_hits)} files",
            tuple(ui_problems),
        ),
        Criterion(
            "retired paths -> mounted /api/crm/* successors",
            observed_map,
            tuple(map_problems),
        ),
        Criterion(
            "tombstone surface is shrink-only",
            observed_shrink,
            tuple(shrink_problems),
        ),
    ]


def _print_table(criteria: list[Criterion]) -> None:
    rows = [
        (criterion.label, criterion.observed, "PASS" if not criterion.problems else "FAIL")
        for criterion in criteria
    ]
    label_width = max(len("criterion"), *(len(row[0]) for row in rows))
    observed_width = max(len("observed"), *(len(row[1]) for row in rows))
    print("check_sales_route_contract (#2540) — offline guard")
    print(f"{'#':<2} | {'criterion':<{label_width}} | {'observed':<{observed_width}} | result")
    print("-+-".join(["-" * 2, "-" * label_width, "-" * observed_width, "-" * 6]))
    for index, (label, observed, result) in enumerate(rows, 1):
        print(f"{index:<2} | {label:<{label_width}} | {observed:<{observed_width}} | {result}")


def self_test() -> int:
    """Prove each criterion fires and the clean case passes, using synthetic strings."""
    failures: list[str] = []
    checks: list[str] = []

    def verify(condition: bool, label: str) -> None:
        checks.append(label)
        if not condition:
            failures.append(label)

    clean = """
        gone_handler!(funnel, "/api/crm/pipeline");
        gone_handler!(forecast_move, "/api/crm/deals/:id");
        .route("/api/sales/funnel", any(funnel))
        .route("/api/sales/funnel/move", any(forecast_move))
    """
    mapping, orphans, count = parse_tombstones(clean)
    verify(mapping == {
        "/api/sales/funnel": "/api/crm/pipeline",
        "/api/sales/funnel/move": "/api/crm/deals/{id}",
    }, "tombstone mapping parse")
    verify(not orphans and count == 2, "clean tombstone parse")

    mounted = {"/api/crm/pipeline", "/api/crm/deals/{id}"}
    good = evaluate([], mapping, orphans, count, mounted, True)
    verify(all(not criterion.problems for criterion in good), "clean contract rejected")

    ui_bad = evaluate([("botui/ui/suite/x.js", 1)], mapping, orphans, count, mounted, True)
    verify(any("references /api/sales" in p for p in ui_bad[0].problems), "UI reference not detected")

    unmounted = evaluate([], mapping, orphans, count, set(), True)
    verify(any("no mounted /api/crm/*" in p for p in unmounted[1].problems), "unmounted canonical not detected")

    outside = evaluate([], {"/api/sales/x": "/api/nowhere"}, [], 1, mounted, True)
    verify(any("maps outside" in p for p in outside[1].problems), "non-CRM canonical not detected")

    orphan = evaluate([], mapping, ["stale_handler"], count, mounted, True)
    verify(any("gone_handler! stale_handler" in p for p in orphan[1].problems), "orphan not detected")

    grown = evaluate([], mapping, orphans, SALES_ROUTE_BASELINE + 1, mounted, True)
    verify(any("exceed baseline" in p for p in grown[2].problems), "tombstone growth not detected")

    verify(_canonical("/api/crm/deals/:id") == "/api/crm/deals/{id}", "param canonicalisation")

    retired = evaluate([], {}, [], 0, mounted, False)
    verify(all(not criterion.problems for criterion in retired), "fully retired surface rejected")

    if failures:
        for failure in failures:
            print(f"FAIL: {failure}", file=sys.stderr)
        print(f"self-test FAILED ({len(failures)} checks)", file=sys.stderr)
        return 1
    print(f"self-test: {len(checks)} checks passed")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Offline /api/sales/* -> /api/crm/* guard (#2540)")
    parser.add_argument("--check", action="store_true", help="run the guard (default)")
    parser.add_argument("--self-test", action="store_true", help="test rules with synthetic strings")
    args = parser.parse_args(argv)
    if args.self_test:
        return self_test()

    try:
        ui_hits = scan_first_party_refs()
        mounted = parse_mounted_paths()
        present = TOMBSTONE_FILE.is_file()
        text = _read(TOMBSTONE_FILE) if present else ""
        if present and text is None:
            print(f"check_sales_route_contract: cannot read {_rel(TOMBSTONE_FILE)}", file=sys.stderr)
            return 2
        mapping, orphans, count = parse_tombstones(text or "")
    except OSError as error:
        print(f"check_sales_route_contract: scan failed: {error}", file=sys.stderr)
        return 2

    criteria = evaluate(ui_hits, mapping, orphans, count, mounted, present)
    _print_table(criteria)
    violations = [problem for criterion in criteria for problem in criterion.problems]
    for problem in violations:
        print(f"FAIL: {problem}", file=sys.stderr)
    if violations:
        print(f"sales route contract FAILED ({len(violations)} violations)", file=sys.stderr)
        return 1
    print("sales route contract OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
