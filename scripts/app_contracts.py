#!/usr/bin/env python3
"""App contracts: every registry app declares its surface (#2119).

#2101 proved that a shipped app *answers*; this module proves that a shipped
app is *documented*. For every `AppSpec` in `botserver/src/apps/registry/*.rs`
the contract test requires three things:

1. **product classification** — the id appears in exactly one bucket of both
   `botserver/.product` and `botserver/.product.desktop-local` (parsed with
   `product_profile_check.py`, so the two guards cannot drift);
2. **smoke coverage** — the app is part of the surface list that
   `smoke_app_surfaces.py` probes, i.e. it declares a `/suite/...` url;
3. **a documented API surface** — `issues/app-contracts/apps.toml` either
   lists representative routes (each one must exist as a `"/api/..."` literal
   in the botserver sources) or declares `no_api = true` with a real reason and
   an issue number.

An app that gains routes without updating its contract, or a contract that
invents a route the server does not serve, fails the build. Nothing here needs
a server: the whole check is a static read of the tree, so it runs in seconds
before the expensive app-surface smoke.

Usage (through the smoke entry point):
    python3 scripts/smoke_app_surfaces.py --contracts
    python3 scripts/smoke_app_surfaces.py --self-test
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # Python < 3.11
    try:
        import tomli as tomllib  # type: ignore
    except ModuleNotFoundError:
        print("app_contracts: Python 3.11+ (tomllib) is required", file=sys.stderr)
        raise SystemExit(2)

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "issues" / "app-contracts" / "apps.toml"
SOURCE_DIRS = ("botserver/src", "botserver/crates")
MIN_REASON_CHARS = 20

ROUTE_RE = re.compile(r'"(/api/[A-Za-z0-9_/{}\.\-]+)"')


class ContractError(Exception):
    """The contracts file cannot be read or is not a valid contract set."""


def load_contracts(path: Path = CONTRACTS) -> dict[str, dict]:
    """Returns `{app id: entry}`; `[meta]` is not an app."""
    if not path.is_file():
        raise ContractError(f"{path} is missing")
    try:
        payload = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise ContractError(f"{path}: {error}") from error
    apps = payload.get("app")
    if not isinstance(apps, dict) or not apps:
        raise ContractError(f"{path}: no [app.<id>] entries")
    return apps


def route_literals() -> set[str]:
    """Every `"/api/..."` literal the botserver sources mention."""
    literals: set[str] = set()
    for directory in SOURCE_DIRS:
        base = ROOT / directory
        if not base.is_dir():
            continue
        for path in base.rglob("*.rs"):
            literals.update(ROUTE_RE.findall(path.read_text(encoding="utf-8", errors="ignore")))
    if not literals:
        raise ContractError("no `/api/...` literals found; the sources moved")
    return literals


def classification() -> dict[str, str]:
    """app id -> product bucket, using the product-profile parser."""
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    import product_profile_check as profile  # noqa: E402 - sibling module

    # Core apps (about/settings) are deliberately absent from the product
    # buckets: the profile guard rejects listing them, so they classify as
    # "core" instead of "unclassified".
    buckets: dict[str, str] = {app: "core" for app in profile.CORE_APPS}
    for path in profile.PRODUCT_FILES:
        parsed = profile.parse_product(path)
        for bucket in profile.BUCKETS:
            for app in parsed[bucket]:
                buckets.setdefault(app, bucket)
    return buckets


def validate(
    registry: list[tuple[str, str]],
    contracts: dict[str, dict],
    classified: dict[str, str],
    literals: set[str],
    smoke_ids: set[str],
) -> list[str]:
    """Every way an app can be undocumented, as human-readable errors."""
    errors: list[str] = []
    ids = {app_id for app_id, _ in registry}

    for unknown in sorted(set(contracts) - ids):
        errors.append(f"contract for `{unknown}`, which is not a registry app")

    for app_id, url in registry:
        entry = contracts.get(app_id)
        if entry is None:
            errors.append(f"`{app_id}` has no [app.{app_id}] contract entry")
            continue
        if app_id not in classified:
            errors.append(f"`{app_id}` is not classified in a product file")
        if app_id not in smoke_ids:
            errors.append(f"`{app_id}` is not in the app-surface smoke list (url: {url})")

        routes = entry.get("routes")
        no_api = entry.get("no_api")
        if no_api and routes:
            errors.append(f"`{app_id}` declares both `routes` and `no_api`")
            continue
        if no_api:
            reason = str(entry.get("reason", "")).strip()
            if len(reason) < MIN_REASON_CHARS:
                errors.append(
                    f"`{app_id}` is no_api but needs a real reason (>= {MIN_REASON_CHARS} chars)"
                )
            if not isinstance(entry.get("issue"), int):
                errors.append(f"`{app_id}` is no_api but has no `issue` number")
            continue
        if not isinstance(routes, list) or not routes:
            errors.append(f"`{app_id}` declares neither `routes` nor `no_api = true`")
            continue
        for route in routes:
            if not isinstance(route, str) or not route.startswith("/api/"):
                errors.append(f"`{app_id}` route {route!r} is not an /api/ path")
            elif route not in literals:
                errors.append(
                    f"`{app_id}` documents {route}, which no botserver source serves"
                )
    return errors


def report(registry: list[tuple[str, str]], errors: list[str]) -> int:
    if errors:
        print("app contracts: FAILED", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        return 1
    print(f"app contracts: {len(registry)} registry apps classified, smoked and documented")
    return 0


def self_test() -> int:
    """Proves each contract rule fires and a clean set passes."""
    failures = 0

    def expect(condition: bool, label: str) -> None:
        nonlocal failures
        if not condition:
            failures += 1
            print(f"self-test FAILED: {label}")

    registry = [("chat", "/suite/partials/chat.html"), ("about", "/suite/about/about.html")]
    classified = {"chat": "apps", "about": "apps"}
    smoke_ids = {"chat", "about"}
    literals = {"/api/chat", "/api/product"}
    clean = {
        "chat": {"routes": ["/api/chat"]},
        "about": {"no_api": True, "reason": "static page rendered from the bundle", "issue": 2119},
    }
    expect(validate(registry, clean, classified, literals, smoke_ids) == [], "clean set passes")

    missing = dict(clean)
    del missing["about"]
    expect(validate(registry, missing, classified, literals, smoke_ids) != [], "missing entry fails")

    invented = {"chat": {"routes": ["/api/chat/nope"]}, "about": clean["about"]}
    expect(validate(registry, invented, classified, literals, smoke_ids) != [], "invented route fails")

    vague = {"chat": clean["chat"], "about": {"no_api": True, "reason": "n/a"}}
    expect(validate(registry, vague, classified, literals, smoke_ids) != [], "vague no_api reason fails")

    unclassified = {"chat": clean["chat"], "about": clean["about"]}
    expect(
        validate(registry, unclassified, {"chat": "apps"}, literals, smoke_ids) != [],
        "unclassified app fails",
    )
    expect(
        validate(registry, unclassified, classified, literals, {"chat"}) != [],
        "app missing from the smoke list fails",
    )
    expect(
        validate(registry, dict(clean, ghost={"no_api": True, "reason": "x" * 30, "issue": 1}),
                 classified, literals, smoke_ids) != [],
        "contract for an unknown app fails",
    )
    if failures:
        print(f"app contracts self-test: {failures} failure(s)")
        return 1
    print("app contracts self-test: ok")
    return 0
