#!/usr/bin/env python3
"""Blocking clippy baseline gate for Rust error-handling debt (issue #2543).

AGENTS.md forbids ``unwrap()``, ``expect()`` and ``panic!()`` in production
Rust. Issue #2543 asks for ``clippy::unwrap_used``, ``expect_used`` and
``panic`` as ``deny`` for non-test code, with a shrink-only baseline allowlist
and a CI check that the count never grows.

Why the lint config stays advisory (``warn`` in ``[workspace.lints.clippy]``):
Cargo's workspace lint table has no per-file / per-crate exclusion mechanism,
and a globally denied lint would stop the whole workspace from compiling on
the remaining debt and on test-module unwraps. So the deny behaviour is
implemented as a blocking CI check instead: this script parses the JSON
diagnostics of ``cargo clippy --message-format=json``, keeps only the three
tracked lints in non-test production files under ``botserver/src`` and
``botserver/crates``, and fails when a count grows against the ``[clippy]``
table in ``botserver/clippy-baseline.toml``.

Scope and heuristics (deliberately documented, not hidden):
  * Only ``reason == "compiler-message"`` records with a tracked lint code are
    counted. Other cargo JSON records (artifacts, build scripts) are ignored.
  * A diagnostic is counted only when its primary span's file is under
    ``botserver/src/`` or ``botserver/crates/``; paths in ``tests/`` /
    ``test`` / ``testdata`` / ``benches`` directories and ``*_test.rs`` /
    ``*_tests.rs`` stems are excluded. These are the same directories the
    textual ratchet (``check_rust_error_ratchet.py``) scans.
  * ``#[cfg(test)]`` modules are not compiled by the default clippy targets
    (lib + bin), so their unwraps never reach this gate. Running clippy with
    ``--all-targets`` would add them; the CI step deliberately does not.
  * A source file compiled into multiple targets can be reported more than
    once; with a zero ceiling that only makes a regression fail sooner.
  * No cargo invocation happens here. The caller supplies the JSON on stdin
    or as a path. No unwrap / expect / panic call is made by this script.

Usage:
    cargo clippy --workspace --message-format=json | python3 scripts/check_clippy_baseline.py
    python3 scripts/check_clippy_baseline.py /tmp/clippy.json
    python3 scripts/check_clippy_baseline.py --update /tmp/clippy.json
    python3 scripts/check_clippy_baseline.py --self-test

Exit codes: 0 = at/below baseline, 1 = growth, 2 = usage/baseline/IO error.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import sys
import tempfile
import tomllib

ROOT = os.path.abspath(
    os.environ.get("GB_RATCHET_ROOT") or os.path.join(os.path.dirname(__file__), "..")
)
DEFAULT_BASELINE = os.path.join(ROOT, "botserver", "clippy-baseline.toml")
SCOPE_PREFIXES = ("botserver/src/", "botserver/crates/")
SKIP_PARTS = {"test", "tests", "testdata", "benches"}
SKIP_STEMS = {"test", "tests"}
# Clippy diagnostic code -> key in the baseline's [clippy] table.
TRACKED = {
    "clippy::unwrap_used": "unwrap_used",
    "clippy::expect_used": "expect_used",
    "clippy::panic": "panic",
}
LINT_KEYS = tuple(TRACKED.values())
BASELINE_VALUE_RE = re.compile(r"^(\s*(unwrap_used|expect_used|panic)\s*=\s*)\d+(\s*(?:#.*)?)$")

# Tables the textual ratchet regenerates from source. Everything else in the
# shared baseline file — notably this script's [clippy] table — must survive
# regeneration verbatim (issue #2543 regression).
GENERATED_TABLES = {"rules", "rule_notes"}
TABLE_HEADER_RE = re.compile(r"^\s*\[([^\]]+)\]\s*$")


def extract_unknown_sections(raw: str) -> list[str]:
    """Verbatim lines of table blocks the ratchet generator does not own.

    Blank/comment lines trailing a previous table are carried into the next
    block, so [clippy] keeps its documentation header. Generated tables
    ([rules], [rule_notes], [files.*]) and the root preamble are dropped.
    """
    kept: list[str] = []
    pending: list[str] = []
    block: list[str] = []
    name = ""
    for line in raw.split("\n"):
        header = TABLE_HEADER_RE.match(line)
        if header:
            if name and name not in GENERATED_TABLES and not name.startswith("files."):
                kept.extend(pending + block)
            index = len(block)
            while index and (not block[index - 1].strip()
                             or block[index - 1].lstrip().startswith("#")):
                index -= 1
            pending = block[index:] if name else []
            name, block = header.group(1).strip(), [line]
        elif name:
            block.append(line)
    if name and name not in GENERATED_TABLES and not name.startswith("files."):
        kept.extend(pending + block)
    return kept


def self_test_preservation(check, write_baseline) -> None:
    """Double regeneration keeps [clippy] and its comment (#2543)."""
    with tempfile.TemporaryDirectory() as tmp_dir:
        path = os.path.join(tmp_dir, "clippy-baseline.toml")
        totals = {rule: 0 for rule in ("unwrap", "expect", "panic", "unwrap_or_default")}
        totals["discarded_result"] = 7
        write_baseline(path, totals, {})
        with open(path, "a", encoding="utf-8") as handle:
            handle.write("\n# doc for clippy\n[clippy]\nunwrap_used = 0\n"
                         "expect_used = 0\npanic = 0\n")
        for generation in (1, 2):
            write_baseline(path, totals, {})
            with open(path, "rb") as handle:
                data = tomllib.load(handle)
            with open(path, "r", encoding="utf-8") as handle:
                text = handle.read()
            check(
                f"regeneration #{generation} preserves [clippy] and its comment",
                data.get("clippy") == {
                    "unwrap_used": 0, "expect_used": 0, "panic": 0,
                } and "# doc for clippy" in text,
            )
            check(f"regeneration #{generation} keeps [rules]", data.get("rules") == totals)


def normalize_rel(file_name: str, root: str = ROOT) -> str:
    """Return a repo-relative, forward-slash path for a diagnostic file name.

    Normalize './' and '..' segments before the scope check so a diagnostic
    can never be silently dropped (fail-open) because of a path-format quirk
    in the JSON emitted by cargo/clippy.
    """
    rel = os.path.relpath(file_name, root) if os.path.isabs(file_name) else file_name
    return os.path.normpath(rel.replace("\\", "/")).replace(os.sep, "/")


def is_scoped_production(rel: str) -> bool:
    """True for non-test .rs paths under the two ratchet scope directories."""
    if not any(rel.startswith(prefix) for prefix in SCOPE_PREFIXES):
        return False
    if not rel.endswith(".rs"):
        return False
    parts = rel.split("/")
    if any(part in SKIP_PARTS for part in parts[:-1]):
        return False
    stem = os.path.splitext(parts[-1])[0]
    if stem in SKIP_STEMS or stem.endswith(("_test", "_tests")):
        return False
    return True


def count_diagnostics(lines: list[str], root: str = ROOT) -> dict[str, int]:
    """Count tracked clippy diagnostics in scoped, non-test production files."""
    counts = {key: 0 for key in LINT_KEYS}
    for raw in lines:
        line = raw.strip()
        if not line:
            continue
        try:
            record = json.loads(line)
        except (ValueError, TypeError):
            continue
        if not isinstance(record, dict) or record.get("reason") != "compiler-message":
            continue
        message = record.get("message")
        code = message.get("code") if isinstance(message, dict) else None
        key = TRACKED.get(code.get("code")) if isinstance(code, dict) else None
        if key is None:
            continue
        spans = message.get("spans") if isinstance(message, dict) else None
        if not isinstance(spans, list) or not spans or not isinstance(spans[0], dict):
            continue
        file_name = spans[0].get("file_name")
        if not isinstance(file_name, str):
            continue
        if is_scoped_production(normalize_rel(file_name, root)):
            counts[key] += 1
    return counts


def growth(current: dict[str, int], ceilings: dict[str, int]) -> list[str]:
    """Lint keys whose count exceeds its ceiling (pure, shrink-only)."""
    return [key for key in LINT_KEYS if current[key] > ceilings[key]]


def load_baseline(path: str) -> dict[str, int] | None:
    """Parse the [clippy] table, or return None after printing a reason."""
    try:
        with open(path, "rb") as handle:
            data = tomllib.load(handle)
    except (OSError, tomllib.TOMLDecodeError) as error:
        print(f"check_clippy_baseline: cannot load baseline {path}: {error}", file=sys.stderr)
        return None
    section = data.get("clippy")
    if not isinstance(section, dict):
        print(f"check_clippy_baseline: baseline {path} has no [clippy] table", file=sys.stderr)
        return None
    ceilings: dict[str, int] = {}
    for key in LINT_KEYS:
        value = section.get(key)
        if not isinstance(value, int) or isinstance(value, bool) or value < 0:
            print(
                f"check_clippy_baseline: baseline {path} needs a non-negative "
                f"integer clippy.{key}",
                file=sys.stderr,
            )
            return None
        ceilings[key] = value
    return ceilings


def update_baseline(path: str, counts: dict[str, int]) -> bool:
    """Lower the [clippy] ceilings in place; leave every other table intact."""
    try:
        with open(path, "r", encoding="utf-8") as handle:
            raw = handle.read()
    except OSError as error:
        print(f"check_clippy_baseline: cannot read baseline {path}: {error}", file=sys.stderr)
        return False
    in_section = False
    seen: set[str] = set()
    out: list[str] = []
    for line in raw.split("\n"):
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            in_section = stripped == "[clippy]"
            out.append(line)
            continue
        if in_section:
            match = BASELINE_VALUE_RE.match(line)
            if match and match.group(2) in counts:
                out.append(f"{match.group(1)}{counts[match.group(2)]}{match.group(3)}")
                seen.add(match.group(2))
                continue
        out.append(line)
    missing = [key for key in LINT_KEYS if key not in seen]
    if missing:
        print(
            f"check_clippy_baseline: [clippy] in {path} is missing: {', '.join(missing)}",
            file=sys.stderr,
        )
        return False
    try:
        with open(path, "w", encoding="utf-8") as handle:
            handle.write("\n".join(out))
    except OSError as error:
        print(f"check_clippy_baseline: cannot write baseline {path}: {error}", file=sys.stderr)
        return False
    return True


def synthetic(code: str, file_name: str) -> str:
    """Build one synthetic cargo JSON compiler-message record (self-test only)."""
    return json.dumps({
        "reason": "compiler-message",
        "message": {
            "message": "synthetic diagnostic",
            "code": {"code": code},
            "level": "warning",
            "spans": [{"file_name": file_name, "line": 1, "column": 1}],
        },
    })


def self_test() -> int:
    """Prove parsing, scope filtering and shrink-only logic on synthetic input."""
    failures: list[str] = []
    checks: list[str] = []

    def check(name: str, condition: bool) -> None:
        checks.append(name)
        if condition:
            print(f"  ok: {name}")
        else:
            print(f"  FAIL: {name}")
            failures.append(name)

    lines = [
        synthetic("clippy::unwrap_used", "botserver/src/a.rs"),
        synthetic("clippy::unwrap_used", "botserver/crates/b/src/c.rs"),
        synthetic("clippy::unwrap_used", os.path.join(ROOT, "botserver/src/d.rs")),
        synthetic("clippy::expect_used", "botserver/src/a.rs"),
        synthetic("clippy::panic", "botserver/crates/b/src/c.rs"),
        synthetic("clippy::unwrap_used", "botserver/crates/b/tests/it.rs"),
        synthetic("clippy::unwrap_used", "botserver/src/a_test.rs"),
        synthetic("clippy::unwrap_used", "botui/src/main.rs"),
        json.dumps({"reason": "compiler-artifact", "message": {}}),
        "not json at all",
    ]
    expected = {"unwrap_used": 3, "expect_used": 1, "panic": 1}
    counts = count_diagnostics(lines)
    check(f"synthetic diagnostic counts {counts}", counts == expected)
    check(
        "test paths and out-of-scope files are excluded",
        not is_scoped_production("botserver/crates/b/tests/it.rs")
        and not is_scoped_production("botserver/src/a_test.rs")
        and is_scoped_production("botserver/src/a.rs"),
    )
    check(
        "./ and ..-prefixed scoped paths are normalized into scope (no fail-open)",
        is_scoped_production(normalize_rel("./botserver/src/a.rs"))
        and is_scoped_production(normalize_rel("botserver/src/../src/a.rs"))
        and not is_scoped_production(normalize_rel("../outside/elsewhere.rs")),
    )
    check("equal counts are not growth", growth(expected.copy(), expected) == [])
    lowered = expected.copy()
    lowered["panic"] = 0
    check("lower count is not growth", growth(lowered, expected) == [])
    raised = expected.copy()
    raised["expect_used"] = 2
    check("raised count is growth", growth(raised, expected) == ["expect_used"])
    if failures:
        print(f"check_clippy_baseline: self-test FAIL — {len(failures)} check(s)")
        return 1
    print(f"check_clippy_baseline: self-test OK — {len(checks)} checks passed")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Clippy error-handling baseline gate (#2543)")
    parser.add_argument(
        "json_path", nargs="?", default="-",
        help="file of cargo clippy --message-format=json lines, or - for stdin",
    )
    parser.add_argument("--baseline", default=DEFAULT_BASELINE, help="baseline TOML path")
    parser.add_argument(
        "--update", action="store_true",
        help="lower the [clippy] ceilings to the current counts (refuses raises)",
    )
    parser.add_argument(
        "--self-test", action="store_true",
        help="check parsing, scope filtering and shrink-only logic on synthetic JSON",
    )
    args = parser.parse_args()

    if args.self_test:
        return self_test()

    if args.json_path == "-":
        lines = sys.stdin.readlines()
    else:
        try:
            with open(args.json_path, "r", encoding="utf-8", errors="replace") as handle:
                lines = handle.readlines()
        except OSError as error:
            print(f"check_clippy_baseline: cannot read {args.json_path}: {error}", file=sys.stderr)
            return 2

    counts = count_diagnostics(lines)
    ceilings = load_baseline(args.baseline)
    if ceilings is None:
        return 2
    rel = os.path.relpath(args.baseline, ROOT)

    if args.update:
        if growth(counts, ceilings):
            for key in growth(counts, ceilings):
                print(
                    f"check_clippy_baseline: refusing to raise {key} "
                    f"({counts[key]} > baseline {ceilings[key]}); fix the growth first"
                )
            return 1
        if not update_baseline(args.baseline, counts):
            return 2
        summary = ", ".join(f"{key}: {ceilings[key]}->{counts[key]}" for key in LINT_KEYS)
        print(f"check_clippy_baseline: baseline written to {rel} ({summary})")
        return 0

    grown = growth(counts, ceilings)
    if grown:
        print(f"check_clippy_baseline: FAIL — {len(grown)} lint(s) grew against {rel}:")
        for key in grown:
            print(
                f"- {key}: {counts[key]} > baseline {ceilings[key]} "
                f"(+{counts[key] - ceilings[key]})"
            )
        print("  remediation: propagate with ? / typed errors; never add #[allow]")
        return 1

    drops = [f"{key}: {counts[key]} < {ceilings[key]}" for key in LINT_KEYS
             if counts[key] < ceilings[key]]
    summary = ", ".join(f"{key}={counts[key]}" for key in LINT_KEYS)
    print(f"check_clippy_baseline: OK — {summary} at/below {rel}")
    for line in drops:
        print(f"  note: {line} — lower it with --update (counts only move down)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
