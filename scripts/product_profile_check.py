#!/usr/bin/env python3
"""#1811 product-profile guard.

The build compiles every app by default (`all-apps` is part of `default`).
Product files are the only place that classifies each registry app as always
visible, preview or hidden. This script fails CI when:

- a product file references an id that is not in the app registry (the
  documented non-app surface `sources` is the only exception),
- an app appears in more than one bucket (or twice in the same bucket),
- a registry app is not classified exactly once,
- a product file lists a core app (about/admin/settings are implicit),
- the default/desktop build does not compile the feature an app needs.

Usage:
    python3 scripts/product_profile_check.py
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
        print("product_profile_check: Python 3.11+ (tomllib) is required", file=sys.stderr)
        raise SystemExit(2)

ROOT = Path(__file__).resolve().parents[1]
CARGO = ROOT / "botserver" / "Cargo.toml"
REGISTRY = ROOT / "botserver" / "src" / "apps" / "registry"
PRODUCT_FILES = [ROOT / "botserver" / ".product", ROOT / "botserver" / ".product.desktop-local"]

CORE_APPS = {"about", "admin", "settings"}
NON_APP_SURFACES = {"sources"}
BUCKETS = ("apps", "preview_apps", "hidden_apps")

# The product profile requested in #1811. Both shipped files use this list.
EXPECTED_ALWAYS = {
    "chat", "drive", "clock-widget", "jukebox", "sheet", "player",
    "calculator", "notes", "photos", "timer", "weather", "integrations",
}

# Registry id -> botserver Cargo feature. `None` means the app ships with every
# build (UI/routes have no feature gate). Keep in sync with the catalog and
# `is_app_compiled`; the guard below then verifies `all-apps` really turns it on.
APP_FEATURES: dict[str, str | None] = {
    "about": None,
    "admin": None,
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


def registry_ids() -> set[str]:
    ids: set[str] = set()
    for path in REGISTRY.glob("*.rs"):
        ids.update(re.findall(r'id:\s*"([^"]+)"', path.read_text(encoding="utf-8")))
    return ids


def parse_product(path: Path) -> dict[str, list[str]]:
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


def main() -> int:
    errors: list[str] = []

    with CARGO.open("rb") as handle:
        cargo = tomllib.load(handle)
    features = cargo.get("features", {})

    if "all-apps" not in features:
        errors.append("botserver/Cargo.toml has no `all-apps` feature")
    default = features.get("default", [])
    if "all-apps" not in default:
        errors.append("botserver/Cargo.toml `default` does not include `all-apps`")
    default_closure = feature_closure(features, default)
    desktop_closure = feature_closure(features, features.get("desktop-local", []))
    if "all-apps" not in desktop_closure:
        errors.append("`desktop-local` does not include `all-apps`")
    # #1820 — the desktop edition serves the Mail app and the Data Sources
    # IMAP account routes. Keep the feature explicit so removing it from
    # `all-apps` cannot leave those UI surfaces posting to absent routes.
    if "mail" not in desktop_closure:
        errors.append("`desktop-local` does not compile `mail` (IMAP account routes)")

    apps = registry_ids()
    if len(apps) != 68:
        errors.append(f"expected 68 registry apps, found {len(apps)}")

    known = apps | NON_APP_SURFACES
    for path in PRODUCT_FILES:
        rel = path.relative_to(ROOT)
        profile = parse_product(path)

        for bucket in BUCKETS:
            values = profile[bucket]
            if len(values) != len(set(values)):
                duplicates = sorted({value for value in values if values.count(value) > 1})
                errors.append(f"{rel}: duplicate ids in {bucket}: {', '.join(duplicates)}")

        classified: dict[str, str] = {}
        for bucket in BUCKETS:
            for app in profile[bucket]:
                if app not in known:
                    errors.append(f"{rel}: unknown app id `{app}` in {bucket}")
                    continue
                if app in CORE_APPS:
                    errors.append(f"{rel}: core app `{app}` must not be listed in {bucket}")
                if app in classified:
                    errors.append(
                        f"{rel}: `{app}` is classified twice "
                        f"({classified[app]} and {bucket})"
                    )
                else:
                    classified[app] = bucket
                if app in apps and APP_FEATURES.get(app):
                    required = APP_FEATURES[app]
                    if required not in default_closure:
                        errors.append(
                            f"{rel}: `{app}` needs feature `{required}`, "
                            "which the default build does not compile"
                        )
                    if required not in desktop_closure:
                        errors.append(
                            f"{rel}: `{app}` needs feature `{required}`, "
                            "which the desktop-local build does not compile"
                        )

        for app in sorted(apps - CORE_APPS):
            if app not in classified:
                errors.append(f"{rel}: registry app `{app}` is not classified")
        missing_always = sorted(EXPECTED_ALWAYS - set(profile["apps"]))
        extra_always = sorted(set(profile["apps"]) - EXPECTED_ALWAYS)
        if missing_always:
            errors.append(f"{rel}: always list is missing {', '.join(missing_always)}")
        if extra_always:
            errors.append(f"{rel}: always list has unexpected {', '.join(extra_always)}")

    if any(APP_FEATURES.get(app) and APP_FEATURES[app] not in default_closure for app in apps):
        missing = sorted(
            app for app in apps
            if APP_FEATURES.get(app) and APP_FEATURES[app] not in default_closure
        )
        errors.append(f"`all-apps` does not compile features for: {', '.join(missing)}")

    if errors:
        print("product profile check failed:", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        return 1

    print("product profile: 68 apps fully classified; default and desktop compile all apps")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
