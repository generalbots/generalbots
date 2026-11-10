#!/usr/bin/env python3
"""#1811 / #2120 product-profile guard.

The build compiles every app by default (`all-apps` is part of `default`).
Product files are the only place that classifies each registry app as always
visible, preview or hidden. This script fails CI when:

- a product file references an id that is not in the app registry (the
  documented non-app surface `sources` is the only exception),
- an app appears in more than one bucket (or twice in the same bucket),
- a registry app is not classified exactly once,
- a product file lists a core app (about/admin/settings are implicit),
- the default/desktop build does not compile the feature an app needs,
- a tier of `botserver/.product.matrix` disagrees with its `.product*` profile,
  supports a deprecated app, or names an unknown id/feature (#2120),
- a registry app has no owner/status row, or its status disagrees with the
  matrix (#2120).

Failures print a matrix diff (profile/registry vs matrix) so the fix is
mechanical. The matrix model lives in `scripts/product_matrix.py`.

Usage:
    python3 scripts/product_profile_check.py
"""

from __future__ import annotations

import argparse
import re
import sys

from product_matrix import (
    APP_FEATURES,
    BUCKETS,
    CORE_APPS,
    NON_APP_SURFACES,
    PRODUCT_FILES,
    REGISTRY,
    ROOT,
    check_matrix,
    feature_closure,
    load_features,
    parse_product,
)

# The product profile requested in #1811. Both shipped files use this list.
EXPECTED_ALWAYS = {
    "chat", "drive", "clock-widget", "jukebox", "sheet", "player",
    "calculator", "notes", "photos", "timer", "weather", "integrations",
}


def registry_ids() -> set[str]:
    ids: set[str] = set()
    for path in REGISTRY.glob("*.rs"):
        ids.update(re.findall(r'id:\s*"([^"]+)"', path.read_text(encoding="utf-8")))
    return ids


def main() -> int:
    parser = argparse.ArgumentParser(description="#1811 / #2120 product-profile guard")
    parser.add_argument("--self-test", action="store_true", help="prove the gate fails closed")
    args = parser.parse_args()
    if args.self_test:
        from gate_selftests import run_self_test

        return run_self_test("product-profile")

    errors: list[str] = []

    features = load_features()

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
    # APP_FEATURES is the full registry inventory (core ids included), so the
    # expected count is its length — no magic number to drift (#2013).
    if len(apps) != len(APP_FEATURES):
        errors.append(
            f"expected {len(APP_FEATURES)} registry apps (APP_FEATURES), "
            f"found {len(apps)}"
        )
    missing_feature_map = sorted(apps - set(APP_FEATURES))
    extra_feature_map = sorted(set(APP_FEATURES) - apps - CORE_APPS)
    if missing_feature_map:
        errors.append(f"APP_FEATURES missing registry ids: {', '.join(missing_feature_map)}")
    if extra_feature_map:
        errors.append(f"APP_FEATURES has unknown non-core ids: {', '.join(extra_feature_map)}")

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

    # #2120 — the supported matrix is the product decision; the profiles and
    # the registry ownership table must agree with it.
    matrix_errors, matrix_diff = check_matrix(features, apps)
    errors.extend(matrix_errors)

    if errors:
        print("product profile check failed:", file=sys.stderr)
        if matrix_diff:
            print("  matrix diff (profile/registry vs matrix):", file=sys.stderr)
            for line in matrix_diff:
                print(f"    - {line}", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        return 1

    print(
        f"product profile: {len(apps)} apps classified, owned and in agreement "
        "with the supported matrix; default and desktop compile all apps"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
