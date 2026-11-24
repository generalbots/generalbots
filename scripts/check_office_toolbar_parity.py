#!/usr/bin/env python3
"""CI guard for #2448 criterion 6: an office toolbar action needs a catalog verb.

Two static sources, no browser, no build:

* toolbar actions -- `data-verb="..."` / `data-action="..."` attributes in
  ``botui/ui/suite/{docs,sheet,slides}/**/*.{html,js}``;
* verb manifest -- ``cmd(app, name, ...)`` entries under
  ``botserver/src/apps/commands/**/*.rs`` (the catalog consumed by chat and by
  ``office_parity_tests.rs``).

An action is *catalog-facing* when it explicitly declares a verb: every
``data-verb`` value, and a ``data-action`` value shaped like a catalog verb
(``<app>.<name>``). The remaining short UI actions (``refresh``, ``accept``,
...) are counted and printed as a GAP, not silently waved through: the issue
asks for a toolbar-to-verb mapping that does not exist in the tree yet.

Exit 0 only when at least one catalog-facing action was found and every one of
them has a matching ``cmd()`` verb. Exit 1 on an unmatched action, an empty
scan, or a missing input. ``--self-test`` proves three outcomes on throwaway
fixtures: catalog entry present -> PASS, renamed verb -> FAIL, one toolbar
action removed -> PASS/no-crash (plus a vacuity probe).
"""

from __future__ import annotations

import argparse
import re
import sys
import tempfile
from pathlib import Path
from typing import NamedTuple

DEFAULT_ROOT = Path(__file__).resolve().parent.parent
REL_TOOLBARS = (
    "botui/ui/suite/docs",
    "botui/ui/suite/sheet",
    "botui/ui/suite/slides",
)
REL_CATALOG = "botserver/src/apps/commands"

# Matches quotes inside JS-generated markup as well as plain HTML.
ATTR_RE = re.compile(r"\bdata-(verb|action)\s*=\s*([\"'])([^\"']*)\2")
CATALOG_VERB_RE = re.compile(r"[a-z][a-z0-9_-]*\.[a-z][a-z0-9_.]*\Z")
CMD_RE = re.compile(r'\bcmd\s*\(\s*"[^"]*"\s*,\s*"([^"]+)"', re.S)


class Action(NamedTuple):
    kind: str
    value: str
    source: str
    line: int


def is_catalog_verb(value: str) -> bool:
    """A data-action only declares a catalog verb when it is `<app>.<verb>`."""
    return bool(CATALOG_VERB_RE.fullmatch(value))


def scan_toolbar(toolbar_dirs: list[Path]) -> list[Action]:
    actions: list[Action] = []
    for root in toolbar_dirs:
        if not root.is_dir():
            raise ValueError(f"toolbar directory not found: {root}")
        for path in sorted(root.rglob("*")):
            if path.suffix not in {".html", ".js"} or not path.is_file():
                continue
            text = path.read_text(encoding="utf-8", errors="replace")
            source = f"{root.name}/{path.relative_to(root).as_posix()}"
            for match in ATTR_RE.finditer(text):
                actions.append(
                    Action(
                        kind=match.group(1),
                        value=match.group(3).strip(),
                        source=source,
                        line=text.count("\n", 0, match.start()) + 1,
                    )
                )
    return actions


def parse_catalog(catalog_root: Path) -> dict[str, str]:
    """verb name -> declaring file (the curated cmd() manifest)."""
    if catalog_root.is_file():
        files = [catalog_root]
    elif catalog_root.is_dir():
        files = sorted(catalog_root.rglob("*.rs"))
    else:
        raise ValueError(f"verb manifest not found: {catalog_root}")
    verbs: dict[str, str] = {}
    for path in files:
        if "tests" in path.name or any(p in {"test", "tests"} for p in path.parts[:-1]):
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        for match in CMD_RE.finditer(text):
            verbs.setdefault(match.group(1), path.as_posix())
    return verbs


def audit(
    toolbar_dirs: list[Path], catalog_root: Path, quiet: bool = False
) -> tuple[int, dict[str, int]]:
    violations: list[str] = []
    try:
        actions = scan_toolbar(toolbar_dirs)
        catalog = parse_catalog(catalog_root)
    except (OSError, UnicodeDecodeError, ValueError) as exc:
        print(f"FAIL input error: {exc}")
        return 1, {}

    catalog_facing = [
        a for a in actions if a.kind == "verb" or is_catalog_verb(a.value)
    ]
    unqualified = [a for a in actions if a not in catalog_facing]
    unmatched = [a for a in catalog_facing if a.value not in catalog]

    if not catalog_facing:
        violations.append(
            "FAIL no catalog-facing toolbar action found - parity check would be vacuous"
        )
    if not catalog:
        violations.append("FAIL no cmd() verbs found in the verb manifest")
    for action in unmatched:
        violations.append(
            f"FAIL toolbar {action.kind}={action.value!r} has no catalog verb "
            f"({action.source}:{action.line})"
        )

    stats = {
        "actions": len(actions),
        "verbs_attr": sum(1 for a in actions if a.kind == "verb"),
        "action_attr": sum(1 for a in actions if a.kind == "action"),
        "catalog_verbs": len(catalog),
        "catalog_facing": len(catalog_facing),
        "matched": len(catalog_facing) - len(unmatched),
        "unmatched": len(unmatched),
        "unqualified": len(unqualified),
    }
    if not quiet:
        print("=== Office toolbar verb parity guard (#2448 criterion 6) ===")
        print(f"  toolbar roots         : {', '.join(str(p) for p in toolbar_dirs)}")
        print(f"  verb manifest         : {catalog_root}")
        print(f"  data-verb attributes  : {stats['verbs_attr']}")
        print(f"  data-action attributes: {stats['action_attr']}")
        print(f"  catalog cmd() verbs   : {stats['catalog_verbs']}")
        print(f"  catalog-facing actions: {stats['catalog_facing']}")
        print(f"  matched / unmatched   : {stats['matched']} / {stats['unmatched']}")
        if unqualified:
            print(
                f"[GAP] {len(unqualified)} unqualified data-action value(s) "
                "with no catalog verb (explicit toolbar mapping absent):"
            )
            for action in unqualified:
                print(f"  {action.value!r} ({action.source}:{action.line})")
        for violation in violations:
            print(violation)
        print("RESULT: " + ("FAIL" if violations else "PASS"))
    return (1 if violations else 0), stats


def make_fixture(
    base: Path, catalog_verb: str = "docs.save", drop_action: bool = False
) -> tuple[list[Path], Path]:
    """Throwaway toolbar + manifest; no repo files are touched."""
    toolbar = base / "toolbar"
    catalog = base / "catalog"
    toolbar.mkdir(parents=True, exist_ok=True)
    catalog.mkdir(parents=True, exist_ok=True)
    markup = ""
    if not drop_action:
        markup += '<button type="button" data-verb="docs.save">Save</button>\n'
    markup += '<button type="button" data-action="docs.save">Save</button>\n'
    markup += '<button type="button" data-action="refresh">Refresh</button>\n'
    (toolbar / "docs.html").write_text(markup, encoding="utf-8")
    (catalog / "verbs.rs").write_text(
        f'cmd("docs", "{catalog_verb}", "Save", "Save a document",\n'
        "    &[], None, false),\n",
        encoding="utf-8",
    )
    return [toolbar], catalog


def self_test() -> int:
    failures = 0
    with tempfile.TemporaryDirectory(prefix="gbtoolbar_") as tmp:
        root = Path(tmp)
        cases = (
            ("catalog entry present -> PASS", "docs.save", False, 0),
            ("renamed catalog verb -> FAIL", "docs.save_v2", False, 1),
            ("one toolbar action removed -> no crash", "docs.save", True, 0),
        )
        for index, (label, verb, drop, expected) in enumerate(cases):
            toolbar_dirs, catalog = make_fixture(root / f"case{index}", verb, drop)
            rc, stats = audit(toolbar_dirs, catalog, quiet=True)
            if rc != expected:
                failures += 1
                print(f"self-test FAIL: {label} (rc={rc}, expected {expected})")
            else:
                print(f"self-test PASS: {label} (rc={rc}, matched={stats['matched']})")
        # A scan with no catalog-facing action must fail closed, not pass.
        empty = root / "empty"
        empty.mkdir()
        (empty / "docs.html").write_text(
            '<button data-action="refresh">Refresh</button>\n', encoding="utf-8"
        )
        rc, _ = audit([empty], make_fixture(root / "empty_catalog")[1], quiet=True)
        if rc != 1:
            failures += 1
            print(f"self-test FAIL: vacuous scan accepted (rc={rc})")
        else:
            print("self-test PASS: vacuous scan rejected (rc=1)")
    if failures:
        print(f"office-toolbar-parity self-test: {failures} failure(s)")
        return 1
    print("office-toolbar-parity self-test: ok (4 cases)")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=DEFAULT_ROOT, help="workspace root")
    parser.add_argument("--toolbar", type=Path, nargs="+", help="override toolbar roots")
    parser.add_argument("--catalog", type=Path, help="override verb manifest")
    parser.add_argument("--self-test", action="store_true", help="prove each outcome offline")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    toolbars = args.toolbar or [args.root / rel for rel in REL_TOOLBARS]
    return audit(toolbars, args.catalog or args.root / REL_CATALOG)[0]


if __name__ == "__main__":
    sys.exit(main())
