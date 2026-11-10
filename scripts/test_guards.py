#!/usr/bin/env python3
"""Offline unit-suite policy guards (issue #2074).

`cargo test -p bottest --test unit` is the cross-crate offline suite. It must
stay hermetic and reviewable, so four structural rules are executable here
instead of living in reviewer memory:

A. module-registration  every `*.rs` under `bottest/tests/unit/` is declared as
                        `mod <name>;` in its parent `mod.rs` (an orphan file is
                        compiled by nobody and silently rots).
B. ignore-reason        no bare `#[ignore]`: a skipped test must carry
                        `#[ignore = "reason"]` so the skip is documented.
C. file-size            no file over 450 lines (AGENTS.md split requirement).
D. offline-only         no network/service calls: `reqwest::blocking`,
                        `postgres://`, `redis://` or `http://localhost` in the
                        unit tree means the "offline" suite needs a service.

Allowlist
---------
`bottest/tests/unit/guards-allowlist.txt` records pre-existing debt as
`<rule>:<path>` (optionally `# reason`). It is *shrink-only*: an entry that no
longer matches a violation is itself reported, so the file can only get shorter.
Rule names are the four above.

Usage:
    python3 scripts/test_guards.py --check
    python3 scripts/test_guards.py --self-test

Exit codes: 0 = clean, 1 = violations (or a stale allowlist entry),
2 = usage error.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
UNIT_DIR = Path("bottest/tests/unit")
ALLOWLIST = Path("bottest/tests/unit/guards-allowlist.txt")
MAX_LINES = 450

RULES = ("module-registration", "ignore-reason", "file-size", "offline-only")

# A unit test that opens a socket or a database is not a unit test.
SERVICE_PATTERNS = (
    "reqwest::blocking",
    "postgres://",
    "redis://",
    "http://localhost",
)

BARE_IGNORE = re.compile(r"^\s*#\[ignore\]\s*$")
IGNORE_EMPTY_REASON = re.compile(r'^\s*#\[ignore\s*=\s*""\]\s*$')


class Violation:
    """One rule hit. `path` is always repo-relative so keys are stable."""

    def __init__(self, rule: str, path: Path, line: int, message: str) -> None:
        self.rule = rule
        self.path = path
        self.line = line
        self.message = message

    def key(self) -> str:
        return f"{self.rule}:{self.path.as_posix()}"

    def render(self) -> str:
        return f"{self.path}:{self.line}: {self.rule}: {self.message}"


def rust_files(root: Path) -> list[Path]:
    """Every Rust file under the unit tree, relative to `root`, sorted."""
    unit = root / UNIT_DIR
    if not unit.is_dir():
        return []
    return sorted(p.relative_to(root) for p in unit.rglob("*.rs") if p.is_file())


def read(root: Path, relative: Path) -> str:
    return (root / relative).read_text(encoding="utf-8")


def check_registration(root: Path, files: list[Path]) -> list[Violation]:
    violations = []
    for path in files:
        if path.name == "mod.rs":
            continue
        parent_mod = path.parent / "mod.rs"
        if not (root / parent_mod).is_file():
            violations.append(
                Violation(
                    "module-registration",
                    path,
                    1,
                    f"parent {parent_mod.as_posix()} is missing; every directory needs a mod.rs",
                )
            )
            continue
        declared = re.compile(rf"(?:^|\s)mod\s+{re.escape(path.stem)}\s*;")
        if not declared.search(read(root, parent_mod)):
            violations.append(
                Violation(
                    "module-registration",
                    path,
                    1,
                    f"not declared in {parent_mod.as_posix()}: add `mod {path.stem};` "
                    "or delete the file",
                )
            )
    return violations


def check_ignores(root: Path, files: list[Path]) -> list[Violation]:
    violations = []
    for path in files:
        for lineno, raw in enumerate(read(root, path).splitlines(), start=1):
            if BARE_IGNORE.match(raw):
                violations.append(
                    Violation(
                        "ignore-reason",
                        path,
                        lineno,
                        'bare `#[ignore]`; use `#[ignore = "why it is skipped"]`',
                    )
                )
            elif IGNORE_EMPTY_REASON.match(raw):
                violations.append(
                    Violation(
                        "ignore-reason",
                        path,
                        lineno,
                        "empty ignore reason; document why the test is skipped",
                    )
                )
    return violations


def check_sizes(root: Path, files: list[Path]) -> list[Violation]:
    violations = []
    for path in files:
        lines = len(read(root, path).splitlines())
        if lines > MAX_LINES:
            violations.append(
                Violation(
                    "file-size",
                    path,
                    1,
                    f"{lines} lines (limit {MAX_LINES}); split it by responsibility",
                )
            )
    return violations


def check_offline(root: Path, files: list[Path]) -> list[Violation]:
    violations = []
    for path in files:
        for lineno, raw in enumerate(read(root, path).splitlines(), start=1):
            for needle in SERVICE_PATTERNS:
                if needle in raw:
                    violations.append(
                        Violation(
                            "offline-only",
                            path,
                            lineno,
                            f"`{needle}` needs a service or the network; the unit "
                            "suite must run offline",
                        )
                    )
    return violations


def read_allowlist(root: Path) -> list[tuple[str, str, int]]:
    """Returns (rule, repo-relative path, line number) triples."""
    path = root / ALLOWLIST
    if not path.is_file():
        return []
    entries = []
    for lineno, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        rule, _, target = line.partition(":")
        entries.append((rule.strip(), target.strip(), lineno))
    return entries


def run_check(root: Path = ROOT, quiet: bool = False) -> int:
    files = rust_files(root)
    if not files:
        print(f"error: no Rust files under {UNIT_DIR.as_posix()}", file=sys.stderr)
        return 2

    violations = [
        *check_registration(root, files),
        *check_ignores(root, files),
        *check_sizes(root, files),
        *check_offline(root, files),
    ]

    allowlist = read_allowlist(root)
    allowed = {f"{rule}:{target}" for rule, target, _ in allowlist}
    unknown = [entry for entry in allowlist if entry[0] not in RULES]

    reported = [v for v in violations if v.key() not in allowed]
    stale = sorted(allowed - {v.key() for v in violations})

    if not quiet:
        for violation in reported:
            print(violation.render())
        for rule, target, lineno in unknown:
            print(
                f"{ALLOWLIST.as_posix()}:{lineno}: error: unknown rule `{rule}` "
                f"(expected one of {', '.join(RULES)})"
            )
        for entry in stale:
            print(
                f"{ALLOWLIST.as_posix()}: error: stale allowlist entry `{entry}` no "
                "longer matches a violation; the allowlist may only shrink"
            )

    failures = len(reported) + len(stale) + len(unknown)
    if failures:
        if not quiet:
            print(f"\n{failures} unit-suite policy violation(s) in {len(files)} file(s)")
        return 1

    if not quiet:
        print(f"ok: {len(files)} unit file(s) satisfy the {len(RULES)} policy rules")
    return 0


def self_test() -> int:
    """Proves each rule fires and that the allowlist is shrink-only."""
    import tempfile

    failures = 0

    def expect(condition: bool, label: str) -> None:
        nonlocal failures
        if not condition:
            failures += 1
            print(f"self-test FAILED: {label}")

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        unit = root / UNIT_DIR
        area = unit / "area"
        area.mkdir(parents=True)
        allowlist = unit / "guards-allowlist.txt"
        (unit / "mod.rs").write_text("mod area;\n", encoding="utf-8")
        (area / "mod.rs").write_text("mod good;\n", encoding="utf-8")
        (area / "good.rs").write_text("#[test]\nfn ok() {}\n", encoding="utf-8")

        expect(run_check(root, quiet=True) == 0, "clean tree passes")

        (area / "orphan.rs").write_text("#[test]\nfn orphan() {}\n", encoding="utf-8")
        expect(run_check(root, quiet=True) == 1, "unregistered module fails")

        (area / "mod.rs").write_text("mod good;\nmod orphan;\n", encoding="utf-8")
        expect(run_check(root, quiet=True) == 0, "registration fixes the failure")

        (area / "orphan.rs").write_text(
            '#[ignore]\n#[test]\nfn orphan() { let _ = "postgres://x"; }\n', encoding="utf-8"
        )
        expect(run_check(root, quiet=True) == 1, "bare ignore and service call fail")

        (area / "orphan.rs").write_text(
            '#[ignore = "needs a browser"]\n#[test]\nfn orphan() {}\n', encoding="utf-8"
        )
        expect(run_check(root, quiet=True) == 0, "documented ignore passes")

        allowlist.write_text("file-size:bottest/tests/unit/area/good.rs\n", encoding="utf-8")
        expect(run_check(root, quiet=True) == 1, "stale allowlist entry fails")

        allowlist.write_text("", encoding="utf-8")
        (area / "big.rs").write_text("// x\n" * (MAX_LINES + 1), encoding="utf-8")
        (area / "mod.rs").write_text("mod good;\nmod orphan;\nmod big;\n", encoding="utf-8")
        expect(run_check(root, quiet=True) == 1, "oversized file fails")

        allowlist.write_text(
            "file-size:bottest/tests/unit/area/big.rs  # baseline\n", encoding="utf-8"
        )
        expect(run_check(root, quiet=True) == 0, "allowlisted debt passes")

        allowlist.write_text("mystery-rule:bottest/tests/unit/area/good.rs\n", encoding="utf-8")
        expect(run_check(root, quiet=True) == 1, "unknown allowlist rule fails")

    if failures:
        print(f"self-test: {failures} failure(s)")
        return 1
    print("self-test: ok")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Offline unit-suite policy guards (#2074)",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("--check", action="store_true", help="run the guards (default)")
    parser.add_argument("--self-test", action="store_true", help="prove each rule fires")
    args = parser.parse_args()

    if args.self_test:
        return self_test()
    return run_check()


if __name__ == "__main__":
    sys.exit(main())
