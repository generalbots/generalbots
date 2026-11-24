#!/usr/bin/env python3
"""Offline static audit of the office-suite enterprise surface (#2476).

Read-only guard: it never builds botserver, starts a server, contacts the
network or drives a browser, so it cannot fabricate a UAT screenshot. It makes
the shipped Docs / Sheets / Slides capability set CI-observable by asserting,
non-vacuously:

  1. REGISTRY  - docs, sheet and slides are AppDefinitions with id, title and
                 a non-empty description in the real application catalog.
  2. I18N      - the three main app entry HTML files use data-i18n keys that
                 all resolve in the English catalogs (no hand-added literals),
                 carry no forbidden auto-* keys, and each app root ships JS
                 using gbT() / data-i18n.
  3. LOCALES   - every one of the 8 locale catalogs contains the office feature
                 namespaces (docs-*.ftl, sheet-*.ftl, slides-*.ftl); missing
                 files are reported with their exact path.
  4. ROUTES    - the office export and share REST routes are present in the
                 endpoint inventory parsed by the existing verb-binding audit,
                 and the chat verbs that expose them are bound.
  5. SHARING   - the sharing / link handlers audited by check_sharing_acl_audit
                 still exist, and the Docs share-modal + shared-link routes are
                 wired in their handler modules.
  6. ASSETS    - no CDN / unpkg / jsdelivr reference in any HTML/JS/CSS file of
                 the three app roots (local assets only).

Two documented GAPs are printed (not failed): ODT export and the Docs shared
link are registered in the crate but absent from the endpoint inventory, so
they are not yet CI-observable there. Their evidence is itself asserted.

Usage:
    python3 scripts/check_office_enterprise_audit.py [--root /path/to/copy]

Exit 0 when every assertion passes, 1 otherwise.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import NamedTuple

DEFAULT_ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(Path(__file__).resolve().parent))

# Reuse the endpoint-inventory parser and the CDN detector from the sibling
# audits so this guard cannot drift from the source of truth they enforce.
import check_office_interop_audit as interop_audit  # noqa: E402
import check_office_verb_bindings as verb_audit  # noqa: E402

REL_REGISTRY = "botserver/src/apps/registry/entries.rs"
REL_OFFICE_ENDPOINTS = "botserver/src/apps/commands/verb_endpoints/table/office_endpoints.rs"
REL_INVENTORY_DIR = "botserver/src/core/bot"
REL_DOCS_LIB = "botserver/crates/botdocs/src/lib.rs"
REL_DOCS_SHARE_MODAL = "botserver/crates/botdocs/src/ui_fragments/mod.rs"

APPS = ("docs", "sheet", "slides")
APP_TITLES = {"docs": "Docs", "sheet": "Sheets", "slides": "Slides"}
APP_ENTRIES = {app: f"botui/ui/suite/{app}/{app}.html" for app in APPS}
LOCALES = ("en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN")
LOCALE_DIR = "botlib/locales"

# Office feature namespaces that actually ship in all 8 catalogs (docs-nav.ftl
# et al.). Deliberately explicit: a missing file is reported by exact path.
OFFICE_NAMESPACES = (
    "docs-nav.ftl", "docs-structure.ftl", "docs-layout.ftl", "docs-model.ftl",
    "docs-review.ftl", "docs-forms.ftl", "docs-equations.ftl", "docs-language.ftl",
    "docs-large.ftl", "docs-tables.ftl",
    "sheet-a11y.ftl", "sheet-audit.ftl", "sheet-charting.ftl", "sheet-controls.ftl",
    "sheet-functions.ftl", "sheet-ingest.ftl", "sheet-reporting.ftl",
    "sheet-structured.ftl", "sheet-validation.ftl",
    "slides-analytics.ftl", "slides-animation.ftl", "slides-charts.ftl",
    "slides-delivery.ftl", "slides-diagrams.ftl", "slides-insert.ftl",
    "slides-masters.ftl", "slides-object-model.ftl", "slides-outline.ftl",
    "slides-table-media.ftl",
)

# (method, path) pairs normalized the same way check_office_verb_bindings.py
# normalizes them (no leading slash).
REQUIRED_ROUTES = (
    ("GET", "api/docs/export/pdf"),
    ("GET", "api/docs/export/docx"),
    ("GET", "api/docs/export/md"),
    ("GET", "api/docs/export/html"),
    ("GET", "api/docs/export/txt"),
    ("POST", "api/sheet/export"),
    ("POST", "api/slides/export"),
    ("POST", "api/sheet/share"),
)
REQUIRED_VERBS = ("sheet.export", "sheet.share", "slides.export")

SHARING_SOURCES = (
    "botserver/crates/botdocs/src/storage_drive.rs",
    "botserver/crates/botdocs/src/handlers_api/ast.rs",
    "botserver/crates/botdocs/src/access_control/model.rs",
    "botserver/crates/botdocs/src/access_control/decision.rs",
    "botserver/crates/botdocs/src/shared_link.rs",
    "botserver/src/main_module/routes/collab_routes/dto.rs",
    "botserver/src/main_module/routes/collab_routes/permissions.rs",
    "botserver/src/security/office_owner.rs",
)

I18N_ATTR = re.compile(
    r'\bdata-i18n(?:-placeholder|-title|-aria-label)?\s*=\s*["\']([^"\']+)["\']'
)
CDN_HOST = interop_audit.CDN_HOST
ASSET_SUFFIXES = {".html", ".js", ".css"}


class AuditError(Exception):
    """An input file or directory was missing, empty or unreadable."""


class Check(NamedTuple):
    name: str
    ok: bool
    detail: str


def read(root: Path, rel: str) -> str:
    try:
        text = (root / rel).read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as exc:
        raise AuditError(f"cannot read {rel}: {exc}") from exc
    if not text.strip():
        raise AuditError(f"{rel} is empty - audit would be vacuous")
    return text


def line_of(text: str, needle: str) -> int | None:
    for number, line in enumerate(text.splitlines(), 1):
        if needle in line:
            return number
    return None


def locator(rel: str, text: str, needle: str) -> str:
    number = line_of(text, needle)
    return f"{rel}:{number if number is not None else '?'}"


def app_spec(text: str, app_id: str) -> tuple[str, int] | None:
    """Return the AppSpec body for app_id and the 1-based line of its id."""
    needle = f'id: "{app_id}"'
    at = text.find(needle)
    if at < 0:
        return None
    nxt = text.find("AppSpec", at + len(needle))
    body = text[at : nxt if nxt > 0 else at + 2000]
    return body, text.count("\n", 0, at) + 1


def english_keys(root: Path) -> set[str]:
    catalog = root / LOCALE_DIR / "en"
    if not catalog.is_dir():
        raise AuditError(f"{LOCALE_DIR}/en is not a directory")
    keys: set[str] = set()
    for path in sorted(catalog.glob("*.ftl")):
        keys.update(re.findall(r"^([A-Za-z0-9_.-]+)\s*=", path.read_text(encoding="utf-8"), re.M))
    if len(keys) < 1000:
        raise AuditError(f"{LOCALE_DIR}/en has only {len(keys)} keys - audit would be vacuous")
    return keys


def office_files(root: Path, locale: str) -> set[str]:
    directory = root / LOCALE_DIR / locale
    if not directory.is_dir():
        raise AuditError(f"{LOCALE_DIR}/{locale} is not a directory")
    return {p.name for p in directory.glob("*.ftl") if re.match(r"^(docs|sheet|slides)-", p.name)}


def asset_files(root: Path, app: str) -> list[Path]:
    directory = root / "botui" / "ui" / "suite" / app
    if not directory.is_dir():
        raise AuditError(f"botui/ui/suite/{app} is not a directory")
    return sorted(p for p in directory.rglob("*")
                  if p.is_file() and p.suffix in ASSET_SUFFIXES and p.stat().st_size > 0)


def audit(root: Path) -> tuple[list[Check], list[str]]:
    checks: list[Check] = []
    gaps: list[str] = []
    add = checks.append

    registry = read(root, REL_REGISTRY)

    # 1. REGISTRY: the three office apps ship as catalog entries.
    for app in APPS:
        block = app_spec(registry, app)
        if block is None:
            add(Check(f"registry.{app}", False,
                      f"{REL_REGISTRY}: no AppSpec with id '{app}'"))
            continue
        body, line = block
        title = re.search(r'\btitle:\s*"([^"]+)"', body)
        description = re.search(r'\bdescription:\s*"([^"]+)"', body)
        ok = (title is not None and title.group(1) == APP_TITLES[app]
              and description is not None and len(description.group(1).strip()) >= 20)
        add(Check(
            f"registry.{app}", ok,
            f"{REL_REGISTRY}:{line} id='{app}' title="
            f"'{title.group(1) if title else '?'}' description="
            f"'{(description.group(1)[:60] + '...') if description else '?'}'",
        ))

    # 2. I18N: entry files use catalog-backed keys only; app roots use gbT.
    en_keys = english_keys(root)
    for app in APPS:
        rel = APP_ENTRIES[app]
        text = read(root, rel)
        keys = set(I18N_ATTR.findall(text))
        attrs = text.count("data-i18n")
        auto = sorted(k for k in keys if k.startswith("auto-"))
        unresolved = sorted(k for k in keys if k not in en_keys)
        first = line_of(text, "data-i18n")
        ok = (attrs >= 20 and len(keys) >= 15 and not auto and not unresolved)
        add(Check(
            f"i18n.entry.{app}", ok,
            f"{rel}:{first} {attrs} data-i18n attributes, {len(keys)} distinct keys, "
            f"{len(unresolved)} not in en catalogs, {len(auto)} auto-* keys"
            + (f"; unresolved={unresolved[:5]}" if unresolved else "")
            + (f"; auto={auto[:5]}" if auto else ""),
        ))
        js = asset_files(root, app)
        with_gbt = [p for p in js if p.suffix == ".js" and "gbT(" in p.read_text(encoding="utf-8", errors="replace")]
        with_attr = [p for p in js if p.suffix == ".js" and "data-i18n" in p.read_text(encoding="utf-8", errors="replace")]
        add(Check(
            f"i18n.js.{app}", len(js) >= 20 and bool(with_gbt) and bool(with_attr),
            f"botui/ui/suite/{app}: {len(js)} assets, {len(with_gbt)} JS with gbT(), "
            f"{len(with_attr)} JS with data-i18n",
        ))

    # 3. LOCALES: each of the 8 catalogs carries the office namespaces.
    all_missing: list[str] = []
    for locale in LOCALES:
        names = office_files(root, locale)
        missing = sorted(name for name in OFFICE_NAMESPACES if name not in names)
        all_missing.extend(f"{LOCALE_DIR}/{locale}/{name}" for name in missing)
        add(Check(
            f"locales.{locale}", not missing and len(names) >= len(OFFICE_NAMESPACES),
            f"{LOCALE_DIR}/{locale}: {len(names)} office namespaces, "
            f"{len(missing)} required files missing"
            + (f": {', '.join(missing[:5])}" if missing else ""),
        ))
    if all_missing:
        add(Check("locales.all-8", False, f"missing catalog files: {', '.join(all_missing[:8])}"))
    else:
        add(Check("locales.all-8", True,
                  f"all {len(OFFICE_NAMESPACES)} office namespaces in {len(LOCALES)} locales "
                  f"({len(OFFICE_NAMESPACES) * len(LOCALES)} files)"))

    # 4. ROUTES: reuse the endpoint-inventory parser from the verb-binding audit.
    routes = verb_audit.parse_inventory(root / REL_INVENTORY_DIR)
    add(Check(
        "routes.inventory-parsed", len(routes) >= 500,
        f"{REL_INVENTORY_DIR}: parsed {len(routes)} ApiEndpoint pairs",
    ))
    absent = [(method, path) for method, path in REQUIRED_ROUTES
              if (method, path) not in routes]
    add(Check(
        "routes.office-export-share", not absent,
        f"endpoint inventory has {len(REQUIRED_ROUTES) - len(absent)}/{len(REQUIRED_ROUTES)} "
        f"office export/share routes"
        + (f"; missing={absent}" if absent else ""),
    ))
    bindings = read(root, REL_OFFICE_ENDPOINTS)
    unbound = [verb for verb in REQUIRED_VERBS
               if re.search(r'bind\(\s*"' + re.escape(verb) + r'"', bindings) is None]
    add(Check(
        "routes.office-export-share-verbs", not unbound,
        f"{REL_OFFICE_ENDPOINTS}: {len(REQUIRED_VERBS) - len(unbound)}/{len(REQUIRED_VERBS)} "
        f"export/share verbs bound"
        + (f"; missing={unbound}" if unbound else ""),
    ))

    # 5. SHARING: the audited ACL handlers and the Docs share hooks still exist.
    present = 0
    for rel in SHARING_SOURCES:
        try:
            read(root, rel)
            present += 1
        except AuditError:
            add(Check("sharing.acl-sources", False, f"{rel} missing or empty"))
            break
    else:
        add(Check("sharing.acl-sources", True,
                  f"{present}/{len(SHARING_SOURCES)} sharing/ACL sources present"))
    docs_lib = read(root, REL_DOCS_LIB)
    share_modal = read(root, REL_DOCS_SHARE_MODAL)
    hooks = (
        ("docs share-modal route", REL_DOCS_SHARE_MODAL, share_modal, "/suite/docs/modals/share"),
        ("docs shared-link route", REL_DOCS_LIB, docs_lib, "/api/docs/shared/:token"),
    )
    missing_hooks = [label for label, _rel, text, needle in hooks if needle not in text]
    add(Check(
        "sharing.docs-hooks", not missing_hooks,
        "Docs share routes: " + "; ".join(
            f"{label}@{locator(rel, text, needle)}" for label, rel, text, needle in hooks
        ) + (f"; missing={missing_hooks}" if missing_hooks else ""),
    ))

    # 6. ASSETS: local assets only in the three office app roots.
    scanned: list[tuple[str, str]] = []
    for app in APPS:
        for path in asset_files(root, app):
            rel = path.relative_to(root).as_posix()
            scanned.append((rel, path.read_text(encoding="utf-8", errors="replace")))
    cdn_hits: list[str] = []
    for rel, source in scanned:
        for match in CDN_HOST.finditer(source):
            line = source.count("\n", 0, match.start()) + 1
            cdn_hits.append(f"{rel}:{line}:{match.group(0)[:60]}")
    add(Check(
        "assets.no-cdn", len(scanned) >= 200 and not cdn_hits,
        f"scanned {len(scanned)} office app assets; CDN hits: "
        + ("; ".join(cdn_hits[:5]) if cdn_hits else "none"),
    ))

    # Documented GAP evidence: real crate routes that are not CI-observable in
    # the endpoint inventory. The gap must remain factually true, else FAIL.
    gap_specs = (
        ("odt-export", "/api/docs/export/odt", "api/docs/export/odt"),
        ("docs-shared-link", "/api/docs/shared/:token", "api/docs/shared/:token"),
    )
    gap_lines: list[str] = []
    for label, source_path, normalized in gap_specs:
        line = line_of(docs_lib, f'"{source_path}"')
        in_inventory = ("GET", normalized) in routes or ("POST", normalized) in routes
        ok = line is not None and not in_inventory
        add(Check(
            f"gaps.{label}-evidence", ok,
            f"{REL_DOCS_LIB}:{line} registers {source_path}; "
            f"present in endpoint inventory={in_inventory}",
        ))
        if ok:
            gap_lines.append(
                f"{label}: {REL_DOCS_LIB}:{line} registers {source_path} but "
                f"'{normalized}' is absent from the endpoint inventory - not yet CI-observable"
            )
    gaps.extend(gap_lines)
    return checks, gaps


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=DEFAULT_ROOT,
                        help="workspace root (point at a copy for non-vacuity probes)")
    args = parser.parse_args()
    root = args.root.resolve()
    try:
        checks, gaps = audit(root)
    except AuditError as exc:
        print(f"FAIL audit input error: {exc}")
        return 1
    failed = [c for c in checks if not c.ok]
    print("=== Office suite enterprise surface audit (#2476) ===")
    print(f"  root: {root} | assertions: {len(checks)} | passed: {len(checks) - len(failed)}")
    for check in checks:
        print(f"  {'PASS' if check.ok else 'FAIL'} {check.name}: {check.detail}")
    for gap in gaps:
        print(f"  GAP  {gap}")
    print("RESULT: " + ("FAIL" if failed else "PASS")
          + f" ({len(checks) - len(failed)}/{len(checks)} assertions, "
          f"{len(gaps)} documented GAPs, no browser evidence claimed)")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
