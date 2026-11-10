#!/usr/bin/env python3
"""i18n catalog gate (#2088).

Checks the Fluent catalogs and the keys the UI asks for, without a framework:

    python3 scripts/i18n_lint.py            # human report
    python3 scripts/i18n_lint.py --check    # exit 1 on new violations
    python3 scripts/i18n_lint.py --json     # machine output
    python3 scripts/i18n_lint.py --write-baseline .github/i18n-baseline.json

Violation kinds:

* ``DUPLICATE``            same key defined twice (same value)
* ``CONFLICT``             same key defined twice with different values
* ``BAD_KEY``              key outside ``^[a-z0-9]+(-[a-z0-9]+)*$``
* ``UNBALANCED``           a message value with unbalanced ``{ }``
* ``PLURAL_NO_OTHER``      plural block without ``*[other]``
* ``MISSING``              a key used by the UI that ``en`` does not define
* ``PLACEHOLDER_MISMATCH`` same key uses different placeholders across locales

``--check`` compares against the baseline: the baseline may only shrink. A
violation already recorded passes; a new one (or a new instance of a recorded
one) fails the gate.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LOCALES = ROOT / "botlib" / "locales"
UI_DIRS = [ROOT / "botui" / "ui"]
FTL_KEY_RE = re.compile(r"^([A-Za-z0-9_.-]+)\s*=\s*(.*)$")
KEY_RE = re.compile(r"^[a-z0-9]+(-[a-z0-9]+)*$")
PLACEHOLDER_RE = re.compile(r"\{\s*\$([A-Za-z0-9_]+)\s*\}")
DATA_I18N_RE = re.compile(r"data-i18n(?:-[a-z-]+)?=\"([^\"]+)\"")
T_CALL_RE = re.compile(r"\b(?:t|tOr|i18n\.t)\(\s*['\"]([a-z0-9][a-z0-9.-]*)['\"]")


def parse_ftl(path: Path):
    """Yield (key, value, line) for one FTL file. Continuation lines are
    folded into the previous value."""
    entries = []
    last = None
    for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        match = FTL_KEY_RE.match(line)
        if match:
            last = [match.group(1), match.group(2).strip(), lineno]
            entries.append(last)
        elif last is not None and (
            line.startswith((" ", "\t")) or not braces_balanced(last[1])
        ):
            # Continuation lines are indented, except the `}` that closes a
            # plural/multiline block, which sits at column 0.
            last[1] += "\n" + line.strip()
    return [(key, value, line) for key, value, line in entries]


def braces_balanced(value: str) -> bool:
    depth = 0
    quoted = False
    for ch in value:
        if ch == '"':
            quoted = not quoted
        elif ch == "{" and not quoted:
            depth += 1
        elif ch == "}" and not quoted:
            depth -= 1
            if depth < 0:
                return False
    return depth == 0


def plural_has_other(value: str) -> bool:
    if "->" not in value:
        return True
    return "*[other]" in value


def load_catalog():
    catalog = {}
    for locale_dir in sorted(p for p in LOCALES.iterdir() if p.is_dir()):
        locale = locale_dir.name
        entries = defaultdict(list)
        for ftl in sorted(locale_dir.glob("*.ftl")):
            for key, value, line in parse_ftl(ftl):
                entries[key].append((value, ftl.relative_to(ROOT).as_posix(), line))
        catalog[locale] = entries
    return catalog


def used_keys():
    used = defaultdict(list)
    for base in UI_DIRS:
        if not base.exists():
            continue
        files = sorted(list(base.rglob("*.html")) + list(base.rglob("*.js")))
        for path in files:
            try:
                text = path.read_text(encoding="utf-8")
            except UnicodeDecodeError:
                continue
            for match in DATA_I18N_RE.finditer(text):
                used[match.group(1)].append(path.relative_to(ROOT).as_posix())
            for match in T_CALL_RE.finditer(text):
                used[match.group(1)].append(path.relative_to(ROOT).as_posix())
    return used


def placeholders(value: str):
    return tuple(sorted(set(PLACEHOLDER_RE.findall(value))))


def lint():
    catalog = load_catalog()
    violations = []

    def add(kind, key, location, detail=""):
        violations.append({"kind": kind, "key": key, "location": location, "detail": detail})

    for locale, entries in catalog.items():
        for key, definitions in entries.items():
            if not KEY_RE.match(key):
                add("BAD_KEY", key, definitions[0][1], locale)
            if len(definitions) > 1:
                values = {value for value, _, _ in definitions}
                kind = "CONFLICT" if len(values) > 1 else "DUPLICATE"
                where = ", ".join(f"{path}:{line}" for _, path, line in definitions)
                add(kind, key, where, locale)
            value, path, line = definitions[0]
            if not braces_balanced(value):
                add("UNBALANCED", key, f"{path}:{line}", locale)
            if not plural_has_other(value):
                add("PLURAL_NO_OTHER", key, f"{path}:{line}", locale)

    english = catalog.get("en", {})
    for key, paths in used_keys().items():
        if key not in english:
            add("MISSING", key, sorted(paths)[0], f"used in {len(set(paths))} file(s)")

    for key, definitions in english.items():
        reference = placeholders(definitions[0][0])
        for locale, entries in catalog.items():
            if locale == "en" or key not in entries:
                continue
            value, path, line = entries[key][0]
            if placeholders(value) != reference:
                add(
                    "PLACEHOLDER_MISMATCH",
                    key,
                    f"{path}:{line}",
                    f"{locale}: {placeholders(value)} != en {reference}",
                )

    # Stable output: kind, key, location.
    violations.sort(key=lambda v: (v["kind"], v["key"], v["location"]))
    return violations


def fingerprint(violation):
    return f"{violation['kind']}|{violation['key']}|{violation['location']}"


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail on violations beyond the baseline")
    parser.add_argument("--json", action="store_true", help="print machine-readable JSON")
    parser.add_argument("--baseline", default=str(ROOT / ".github" / "i18n-baseline.json"))
    parser.add_argument("--write-baseline", metavar="PATH", help="record the current violations")
    args = parser.parse_args(argv)

    violations = lint()

    if args.write_baseline:
        payload = {
            "comment": "Current i18n violations. The list may only shrink; fix and regenerate.",
            "fingerprints": sorted(fingerprint(v) for v in violations),
        }
        Path(args.write_baseline).write_text(json.dumps(payload, indent=1) + "\n", encoding="utf-8")
        print(f"wrote {len(violations)} fingerprints to {args.write_baseline}")
        return 0

    baseline = set()
    baseline_path = Path(args.baseline)
    if baseline_path.exists():
        baseline = set(json.loads(baseline_path.read_text(encoding="utf-8"))["fingerprints"])

    new = [v for v in violations if fingerprint(v) not in baseline]
    fixed = baseline - {fingerprint(v) for v in violations}

    if args.json:
        print(json.dumps({"violations": violations, "new": new, "fixed": sorted(fixed)}, indent=1))
    else:
        counts = defaultdict(int)
        for violation in violations:
            counts[violation["kind"]] += 1
        for kind in sorted(counts):
            print(f"{kind}: {counts[kind]}")
        for violation in new[:200]:
            print(f"NEW {violation['kind']} {violation['key']} @ {violation['location']} {violation['detail']}")
        if len(new) > 200:
            print(f"... {len(new) - 200} more new violations")
        if fixed:
            print(f"fixed since baseline: {len(fixed)} (regenerate the baseline)")

    if args.check and new:
        print(f"i18n gate: {len(new)} new violation(s)", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
