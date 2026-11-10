#!/usr/bin/env python3
"""Supported-matrix model (#2120).

`botserver/.product.matrix` is the product decision: which build tier supports
which app, on which platforms, owned by which team. This module parses it and
cross-checks the three projections that must agree with it:

1. the `.product*` profiles (`apps=`, `preview_apps=`, `hidden_apps=`),
2. the registry ownership table (`botserver/src/apps/registry/ownership.rs`),
3. the Cargo feature that each tier's build root actually compiles.

`scripts/product_profile_check.py` is the entry point. Everything here is
offline: no cargo, no services, no network.
"""

from __future__ import annotations

import os
import re
import sys
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # Python < 3.11
    try:
        import tomli as tomllib  # type: ignore
    except ModuleNotFoundError:
        print("product_matrix: Python 3.11+ (tomllib) is required", file=sys.stderr)
        raise SystemExit(2)

# `GB_PRODUCT_ROOT` lets the gate self-test (scripts/gate_selftests.py) run the
# real CLI against a fixture copy of the matrix, the profiles and the registry.
ROOT = Path(os.environ.get("GB_PRODUCT_ROOT") or Path(__file__).resolve().parents[1])
CARGO = ROOT / "botserver" / "Cargo.toml"
REGISTRY = ROOT / "botserver" / "src" / "apps" / "registry"
MATRIX = ROOT / "botserver" / ".product.matrix"
# Every shipped profile is a `.product*` file next to the matrix. Deriving the
# list instead of hardcoding it means a new profile cannot skip classification.
PRODUCT_FILES = sorted(
    path
    for path in (ROOT / "botserver").glob(".product*")
    if path.is_file() and path != MATRIX
)
OWNERSHIP_RS = REGISTRY / "ownership.rs"

CORE_APPS = {"about", "settings"}
NON_APP_SURFACES = {"sources"}
BUCKETS = ("apps", "preview_apps", "hidden_apps")
MATRIX_BUCKETS = ("apps", "preview", "hidden", "deprecated")
MATRIX_KEYS = ("product", "level", "platforms", "owner", "cargo") + MATRIX_BUCKETS
LEVELS = {"ga", "support", "preview"}
STATUS_PRECEDENCE = ("supported", "deprecated", "preview", "hidden")

# Registry id -> botserver Cargo feature. `None` means the app ships with every
# build (UI/routes have no feature gate). Keep in sync with the catalog and
# `is_app_compiled`; the guard verifies that a tier's build root turns it on.
APP_FEATURES: dict[str, str | None] = {
    "about": None,
    "settings": None,
    "analytics": "analytics",
    "attendant": "attendant",
    "automations": "automations",
    "banking": "banking",
    "billing": "billing",
    "browser": "browser",
    "calculator": None,
    "calendar": "calendar",
    "campaigns": "marketing",
    "canvas": "canvas",
    "chat": "chat",
    "clock-widget": None,
    "clock": None,
    "compliance": "compliance",
    "concierge": None,
    "crm": "people",
    "database": "database",
    "designer": "designer",
    "desktop": "desktop",
    "docs": "docs",
    "drive": "drive",
    "editor": "editor",
    "fraud": "fraud",
    "goals": "goals",
    "handoff": "handoff",
    "hr": "hr",
    "integrations": "integrations",
    "jukebox": None,
    "kyc": "kyc",
    "learn": "learn",
    "lists": "marketing",
    "mail": "mail",
    "meet": "meet",
    "memory": "memory-os",
    "minutes": "minutes",
    "monitoring": "monitoring",
    "notes": None,
    "o365": "m365",
    "paper": "paper",
    "people": "people",
    "photos": None,
    "plan": "plan",
    "player": "player",
    "plugins": None,
    "pos": "pos",
    "products": "products",
    "project": "project",
    "recycle": None,
    "research": "research",
    "retail": "retail",
    "sheet": "sheet",
    "slides": "slides",
    "social": "social",
    "store": None,
    "tasks": "tasks",
    "tax": "tax",
    "templates": "templates",
    "terminal": "terminal",
    "tickets": "tickets",
    "timeclock": "timeclock",
    "timer": None,
    "vibe": "vibe",
    "video": "video",
    "vision": "vision",
    "weather": None,
    "workspace": "workspaces",
}


def parse_product(path: Path) -> dict[str, list[str]]:
    """`apps=`, `preview_apps=` and `hidden_apps=` of one `.product*` file."""
    result = {key: [] for key in BUCKETS}
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        key = key.strip().lower()
        if key not in result:
            continue
        result[key] = [part.strip().lower() for part in value.split(",") if part.strip()]
    return result


def feature_closure(features: dict, roots: list[str]) -> set[str]:
    """Every feature reachable from `roots` (transitive, ignoring dep:/crate)."""
    seen: set[str] = set()
    stack = list(roots)
    while stack:
        name = stack.pop()
        if name in seen:
            continue
        seen.add(name)
        entries = features.get(name)
        if entries is None:
            continue
        for entry in entries:
            if entry.startswith("dep:") or "/" in entry or entry.startswith("-"):
                continue
            stack.append(entry)
    return seen


def parse_matrix(path: Path = MATRIX) -> dict[str, dict]:
    """`[tier name]` sections of `botserver/.product.matrix`."""
    tiers: dict[str, dict] = {}
    current: dict | None = None
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("[tier ") and line.endswith("]"):
            name = line[len("[tier ") : -1].strip()
            current = tiers.setdefault(name, {})
            continue
        if current is None or "=" not in line:
            continue
        key, value = line.split("=", 1)
        key = key.strip().lower()
        value = value.strip()
        if key in MATRIX_BUCKETS:
            current[key] = [part.strip().lower() for part in value.split(",") if part.strip()]
        else:
            current[key] = value
    return tiers


def matrix_status(tiers: dict[str, dict], app: str) -> str | None:
    """Support status the matrix implies for `app`, or `None` if unclassified."""
    seen: set[str] = set()
    for tier in tiers.values():
        if app in tier.get("apps", []):
            seen.add("supported")
        elif app in tier.get("deprecated", []):
            seen.add("deprecated")
        elif app in tier.get("preview", []):
            seen.add("preview")
        elif app in tier.get("hidden", []):
            seen.add("hidden")
    for candidate in STATUS_PRECEDENCE:
        if candidate in seen:
            return candidate
    return None


def load_features() -> dict:
    """Cargo `[features]` table of botserver."""
    with CARGO.open("rb") as handle:
        return tomllib.load(handle).get("features", {})


def check_matrix(features: dict, apps: set[str]) -> tuple[list[str], list[str]]:
    """Cross-check the matrix, the profiles, the owners and the tier builds.

    Returns `(errors, diff)`; `diff` is the human-readable matrix diff printed
    by the entry point when a check fails.
    """
    errors: list[str] = []
    diff: list[str] = []
    if not MATRIX.exists():
        return [f"missing {MATRIX.relative_to(ROOT)} (#2120)"], diff
    tiers = parse_matrix()
    rel_matrix = MATRIX.relative_to(ROOT)
    if not tiers:
        errors.append(f"{rel_matrix}: no [tier ...] sections")
    known = apps | NON_APP_SURFACES
    claimed: dict[str, str] = {}

    for name, tier in tiers.items():
        missing_keys = [key for key in MATRIX_KEYS if key not in tier]
        if missing_keys:
            errors.append(f"{rel_matrix}: tier `{name}` misses {', '.join(missing_keys)}")
        if tier.get("level") not in LEVELS:
            errors.append(f"{rel_matrix}: tier `{name}` has invalid level `{tier.get('level')}`")
        for key in ("platforms", "owner"):
            if not str(tier.get(key, "")).strip():
                errors.append(f"{rel_matrix}: tier `{name}` has an empty `{key}`")
        buckets = {key: tier.get(key, []) for key in MATRIX_BUCKETS}
        for key, values in buckets.items():
            if len(values) != len(set(values)):
                errors.append(f"{rel_matrix}: tier `{name}` repeats ids in `{key}`")
            unknown = sorted(set(values) - known)
            if unknown:
                errors.append(
                    f"{rel_matrix}: tier `{name}` names unknown ids in `{key}`: "
                    f"{', '.join(unknown)}"
                )
        # `apps=` is the supported surface: it can never hold a preview, hidden
        # or deprecated app. Deprecated apps may stay in `preview=`/`hidden=`
        # during the announce window (#2120 deprecation policy).
        for bucket in ("preview", "hidden", "deprecated"):
            both = sorted(set(buckets["apps"]) & set(buckets[bucket]))
            if both:
                errors.append(
                    f"{rel_matrix}: tier `{name}` supports {', '.join(both)} "
                    f"while also listing them in `{bucket}`"
                )

        cargo_root = str(tier.get("cargo", "-")).strip()
        if cargo_root not in ("-", ""):
            if cargo_root not in features:
                errors.append(
                    f"{rel_matrix}: tier `{name}` names unknown Cargo feature `{cargo_root}`"
                )
            else:
                closure = feature_closure(features, [cargo_root])
                for app in buckets["apps"]:
                    required = APP_FEATURES.get(app)
                    if required and required not in closure:
                        errors.append(
                            f"{rel_matrix}: tier `{name}` supports `{app}` but "
                            f"`{cargo_root}` does not compile `{required}`"
                        )

        product = str(tier.get("product", "-")).strip()
        if product in ("-", ""):
            continue
        path = ROOT / "botserver" / product
        if path not in PRODUCT_FILES:
            errors.append(f"{rel_matrix}: tier `{name}` names unknown profile `{product}`")
            continue
        if product in claimed:
            errors.append(
                f"{rel_matrix}: `{product}` is claimed by `{claimed[product]}` and `{name}`"
            )
        claimed[product] = name
        profile = parse_product(path)
        rel_profile = path.relative_to(ROOT)
        for app in profile["apps"]:
            if app in buckets["apps"]:
                continue
            where = next(
                (bucket for bucket in ("deprecated", "preview", "hidden") if app in buckets[bucket]),
                "unsupported",
            )
            errors.append(
                f"{rel_profile}: `{app}` is in apps= but tier `{name}` says {where} (#2120)"
            )
            diff.append(f"tier {name}: apps= has `{app}`, matrix says {where}")
        for app in profile["preview_apps"]:
            if app in buckets["preview"] or app in buckets["apps"] or app in buckets["deprecated"]:
                continue
            errors.append(
                f"{rel_profile}: `{app}` is in preview_apps= but tier `{name}` does not ship it"
            )
            diff.append(f"tier {name}: preview_apps= has `{app}`, matrix does not")
        for app in profile["hidden_apps"]:
            if app in buckets["hidden"] or app in buckets["deprecated"]:
                continue
            errors.append(
                f"{rel_profile}: `{app}` is in hidden_apps= but tier `{name}` does not hide it"
            )
            diff.append(f"tier {name}: hidden_apps= has `{app}`, matrix does not")
        unadvertised = sorted(set(buckets["apps"]) - set(profile["apps"]) - CORE_APPS)
        for app in unadvertised:
            errors.append(
                f"{rel_profile}: tier `{name}` supports `{app}` but the profile does not "
                "advertise it (#2120)"
            )
            diff.append(f"tier {name}: matrix supports `{app}`, profile does not")

    for path in PRODUCT_FILES:
        if path.name not in claimed:
            errors.append(f"{path.relative_to(ROOT)} is not claimed by any matrix tier (#2120)")

    ownership = read_ownership(errors)
    for app in sorted(apps):
        entry = ownership.get(app)
        if entry is None:
            errors.append(f"registry app `{app}` has no owner/status row (#2120)")
            continue
        owner, status = entry
        if not owner.strip():
            errors.append(f"registry app `{app}` has an empty owner (#2120)")
        expected = matrix_status(tiers, app)
        if expected is None:
            errors.append(f"registry app `{app}` is classified in no matrix tier (#2120)")
        elif status != expected:
            errors.append(
                f"registry app `{app}` declares status `{status}` but the matrix says `{expected}`"
            )
            diff.append(f"status `{app}`: registry {status}, matrix {expected}")
    for app in sorted(set(ownership) - apps):
        errors.append(f"ownership table names `{app}`, which is not a registry app (#2120)")
    return errors, diff


def read_ownership(errors: list[str]) -> dict[str, tuple[str, str]]:
    """`id -> (owner, status)` from `botserver/src/apps/registry/ownership.rs`."""
    ownership: dict[str, tuple[str, str]] = {}
    if not OWNERSHIP_RS.exists():
        errors.append(f"{OWNERSHIP_RS.relative_to(ROOT)} is missing (#2120)")
        return ownership
    text = OWNERSHIP_RS.read_text(encoding="utf-8")
    pattern = r'\("([^"]+)",\s*"([^"]*)",\s*AppStatus::(\w+)\)'
    for app, owner, status in re.findall(pattern, text):
        if app in ownership:
            errors.append(f"ownership table lists `{app}` twice (#2120)")
        ownership[app] = (owner, status.lower())
    return ownership
