#!/usr/bin/env python3
"""Static CI guard for the #2459 office telemetry wiring.

The four metric hooks in botcore's analytics module used to be dead code: they
were defined and exported but nothing ever called them, so the /metrics
dashboard silently showed no samples. This gate fails when:

  * a hook's `fn` definition disappears from botcore,
  * a hook has no call site in production Rust outside the analytics module
    (test-only callers do not count),
  * the canonical Prometheus scrape router is no longer merged in
    feature_merge_office/sub_routes.rs, or
  * either read-only GET permission (/metrics, /api/metrics) disappears from
    the authenticated RBAC catalogue.

Pure Python, no network, no cargo. Usage:

    python3 scripts/check_metrics_wiring.py

For offline probing only, METRICS_WIRING_ROOT may point the scan at a scratch
copy of the tree; every assertion still runs unchanged.
"""

from __future__ import annotations

import os
import re
import sys
from pathlib import Path

HOOKS = (
    "track_office_workbook_memory",
    "track_office_export",
    "track_office_recalc",
    "track_route_latency",
)

ROOT_ENV = "METRICS_WIRING_ROOT"
BOTSERVER_REL = "botserver"
BOTCORE_REL = "botserver/crates/botcore"
ANALYTICS_REL = "botserver/crates/botcore/src/shared/analytics"
COLLECT_REL = "botserver/crates/botcore/src/shared/analytics/collect.rs"
SUB_ROUTES_REL = "botserver/src/main_module/routes/feature_merge_office/sub_routes.rs"
RBAC_REL = "botserver/crates/botsecurity-auth/src/rbac_middleware/routes_user/app_routes.rs"


def strip_cfg_test(text: str) -> str:
    """Blank out #[cfg(test)] items so a test-only caller cannot satisfy the gate.

    Lines are masked rather than deleted so match offsets still map to the
    exact line numbers of the original source file.
    """
    lines = text.splitlines()
    masked = list(lines)
    i = 0
    while i < len(lines):
        if lines[i].strip().startswith("#[cfg(test)]"):
            masked[i] = ""
            i += 1
            while i < len(lines) and lines[i].strip().startswith("#["):
                masked[i] = ""
                i += 1
            if i >= len(lines):
                break
            item = lines[i]
            masked[i] = ""
            i += 1
            if "{" not in item:
                while i < len(lines) and ";" not in item:
                    item = lines[i]
                    masked[i] = ""
                    i += 1
                continue
            depth = item.count("{") - item.count("}")
            while i < len(lines) and depth > 0:
                depth += lines[i].count("{") - lines[i].count("}")
                masked[i] = ""
                i += 1
            continue
        i += 1
    return "\n".join(masked)


def line_of(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def main() -> int:
    root_arg = os.environ.get(ROOT_ENV)
    root = Path(root_arg).resolve() if root_arg else Path(__file__).resolve().parents[1]

    errors: list[str] = []
    evidence: list[str] = []

    def read_text(rel: str) -> str | None:
        path = root / rel
        if not path.is_file():
            errors.append(f"MISSING file: {rel}")
            return None
        try:
            return path.read_text(encoding="utf-8")
        except (OSError, UnicodeError) as exc:
            errors.append(f"UNREADABLE file: {rel} ({exc})")
            return None

    def rust_files(base_rel: str, exclude_analytics: bool) -> list[Path]:
        base = root / base_rel
        if not base.is_dir():
            errors.append(f"MISSING directory: {base_rel}")
            return []
        analytics = (root / ANALYTICS_REL).resolve()
        try:
            paths = sorted(base.rglob("*.rs"))
        except OSError as exc:
            errors.append(f"UNREADABLE directory: {base_rel} ({exc})")
            return []
        if not exclude_analytics:
            return paths
        return [p for p in paths if analytics not in p.resolve().parents]

    collect = read_text(COLLECT_REL)
    sub_routes = read_text(SUB_ROUTES_REL)
    rbac = read_text(RBAC_REL)
    botcore_files = rust_files(BOTCORE_REL, exclude_analytics=False)
    call_files = rust_files(BOTSERVER_REL, exclude_analytics=True)

    for hook in HOOKS:
        def_re = re.compile(r"\bfn\s+" + re.escape(hook) + r"\s*\(")
        call_re = re.compile(r"\b" + re.escape(hook) + r"\s*\(")

        definition = None
        for path in botcore_files:
            try:
                text = path.read_text(encoding="utf-8")
            except (OSError, UnicodeError) as exc:
                errors.append(f"UNREADABLE file: {path.relative_to(root)} ({exc})")
                continue
            clean = strip_cfg_test(text)
            match = def_re.search(clean)
            if match is not None:
                definition = f"{path.relative_to(root)}:{line_of(clean, match.start())}"
                break
        if definition is None:
            errors.append(f"MISSING fn definition of {hook}() under {BOTCORE_REL}")
        else:
            evidence.append(f"{hook}() defined at {definition}")

        call_sites: list[str] = []
        for path in call_files:
            try:
                text = path.read_text(encoding="utf-8")
            except (OSError, UnicodeError) as exc:
                errors.append(f"UNREADABLE file: {path.relative_to(root)} ({exc})")
                continue
            clean = strip_cfg_test(text)
            match = call_re.search(clean)
            if match is not None:
                call_sites.append(f"{path.relative_to(root)}:{line_of(clean, match.start())}")
        if not call_sites:
            errors.append(
                f"MISSING call site for {hook}() outside {ANALYTICS_REL} "
                "- the hook is defined but no production code invokes it"
            )
        else:
            evidence.append(f"{hook}() called at {', '.join(call_sites)}")

    if sub_routes is not None:
        canonical_re = re.compile(
            r"\.merge\([^;]{0,400}?configure_canonical\s*\(\s*\)", re.DOTALL
        )
        clean_sub_routes = strip_cfg_test(sub_routes)
        match = canonical_re.search(clean_sub_routes)
        if match is None:
            errors.append(
                f"MISSING registration: configure_canonical() is not merged in {SUB_ROUTES_REL}"
            )
        else:
            evidence.append(
                f"configure_canonical() merged at {SUB_ROUTES_REL}:{line_of(clean_sub_routes, match.start())}"
            )

    if rbac is not None:
        clean_rbac = strip_cfg_test(rbac)
        for route in ("/metrics", "/api/metrics"):
            perm_re = re.compile(
                r'RoutePermission::new\(\s*"' + re.escape(route) + r'"\s*,\s*"GET"'
            )
            match = perm_re.search(clean_rbac)
            if match is None:
                errors.append(f"MISSING RBAC GET permission for {route} in {RBAC_REL}")
            else:
                evidence.append(
                    f"RBAC GET {route} at {RBAC_REL}:{line_of(clean_rbac, match.start())}"
                )

    if errors:
        print("check_metrics_wiring: FAIL")
        for message in errors:
            print(f"  - {message}")
        return 1

    for message in evidence:
        print(f"  OK {message}")
    print(
        f"check_metrics_wiring: OK - {len(HOOKS)} hooks defined and called, "
        "canonical scrape route merged, both RBAC GET permissions present"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
