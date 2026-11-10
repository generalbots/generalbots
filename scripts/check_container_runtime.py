#!/usr/bin/env python3
"""Container-runtime gate (#2126): `incus` only, spawned through `SafeCommand`.

AGENTS.md, "Containers": everything runs on **Incus**; `lxc`/`lxd` are never the
container layer. This gate makes that executable instead of reviewer memory:

A. runtime-lxc      no spawn of `lxc`, `lxd` or `lxc-execute` — not even through
                    `SafeCommand` (the allowlist accepting the binary does not
                    make LXC a supported runtime).
B. direct-incus     `incus` must be spawned through
                    `crate::security::command_guard::SafeCommand`, never a raw
                    `std::process::Command::new` / `tokio::process::Command::new`
                    (which skips the binary allowlist, argv validation and the
                    scrubbed environment).

`portable_pty::CommandBuilder::new("incus")` stays allowed: it is the PTY
spawn path used by the live suite terminal (`botserver/src/api/terminal/`),
which builds an argv for the container shell rather than a host command.

Allowlist
---------
`scripts/container_runtime_allowlist.txt` records pre-existing debt as
`<rule>:<path>` (optionally `# reason`). It is *shrink-only*: an entry that no
longer matches a violation is itself reported, so the file can only get shorter.

Usage:
    python3 scripts/check_container_runtime.py --check
    python3 scripts/check_container_runtime.py --self-test

Exit codes: 0 = clean, 1 = violations (or a stale allowlist entry),
2 = usage error.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ALLOWLIST = Path("scripts/container_runtime_allowlist.txt")
SKIP_DIRS = {".git", "target", "node_modules", "dist", "botserver-installers"}
MAX_REPORTED = 60

RULES = ("runtime-lxc", "direct-incus")

# `<callee>::new("<runtime>")` — the callee path is captured so the gate can
# tell a raw `Command` from the guarded `SafeCommand`.
SPAWN_CALL = re.compile(
    r'([A-Za-z_][A-Za-z0-9_:]*)\s*::new\(\s*"(lxc|lxd|lxc-execute|incus)"\s*\)'
)
FORBIDDEN_RUNTIMES = {"lxc", "lxd", "lxc-execute"}
COMMENT_PREFIXES = ("//", "/*", "*")


def is_comment(line: str) -> bool:
    return line.lstrip().startswith(COMMENT_PREFIXES)


def callee_kind(callee: str) -> str:
    """Classifies the callee path of a `::new(...)` call."""
    if callee.endswith("SafeCommand"):
        return "safe"
    if callee.endswith("CommandBuilder"):
        return "pty-builder"
    if callee == "Command" or callee.endswith("::Command"):
        return "raw"
    return "other"


def scan_text(text: str) -> list[tuple[str, int, str]]:
    """Returns `(rule, line_number, line)` violations for one source text."""
    violations: list[tuple[str, int, str]] = []
    for number, line in enumerate(text.splitlines(), start=1):
        if is_comment(line):
            continue
        for match in SPAWN_CALL.finditer(line):
            callee, runtime = match.group(1), match.group(2)
            kind = callee_kind(callee)
            if runtime in FORBIDDEN_RUNTIMES:
                violations.append(("runtime-lxc", number, line.strip()))
            elif runtime == "incus" and kind == "raw":
                violations.append(("direct-incus", number, line.strip()))
    return violations


def rust_sources() -> list[Path]:
    sources: list[Path] = []
    for path in ROOT.rglob("*.rs"):
        if SKIP_DIRS.intersection(path.relative_to(ROOT).parts):
            continue
        sources.append(path)
    return sorted(sources)


def load_allowlist() -> tuple[set[str], list[str]]:
    entries: set[str] = set()
    problems: list[str] = []
    if not ALLOWLIST.exists():
        return entries, problems
    for number, raw in enumerate(ALLOWLIST.read_text(encoding="utf-8").splitlines(), 1):
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        rule, _, path = line.partition(":")
        if rule not in RULES or not path:
            problems.append(f"{ALLOWLIST}:{number}: expected '<rule>:<path>', got '{raw}'")
            continue
        entries.add(f"{rule}:{path}")
    return entries, problems


def check() -> int:
    allowed, problems = load_allowlist()
    found: list[str] = []
    hit_entries: set[str] = set()

    for source in rust_sources():
        relative = source.relative_to(ROOT).as_posix()
        try:
            text = source.read_text(encoding="utf-8", errors="replace")
        except OSError as exc:  # pragma: no cover - unreadable file
            problems.append(f"{relative}: cannot read ({exc})")
            continue
        for rule, number, line in scan_text(text):
            entry = f"{rule}:{relative}"
            if entry in allowed:
                hit_entries.add(entry)
                continue
            found.append(f"{relative}:{number}: [{rule}] {line}")

    for entry in sorted(allowed - hit_entries):
        problems.append(
            f"{ALLOWLIST}: stale entry '{entry}' — the violation is gone, delete the line"
        )

    for problem in problems:
        print(problem, file=sys.stderr)
    if found:
        for item in found[:MAX_REPORTED]:
            print(item, file=sys.stderr)
        if len(found) > MAX_REPORTED:
            print(f"... and {len(found) - MAX_REPORTED} more", file=sys.stderr)
        print(
            "Container runtime: use `incus` and spawn it through SafeCommand "
            "(AGENTS.md, Containers).",
            file=sys.stderr,
        )
    if found or problems:
        return 1

    print("container runtime gate: clean (incus only, via SafeCommand)")
    return 0


def self_test() -> int:
    samples = {
        "raw lxc": 'let c = tokio::process::Command::new("lxc");',
        "guarded lxc": 'let c = SafeCommand::new("lxc")?;',
        "guarded lxd": 'let c = SafeCommand::new("lxd")?;',
        "lxc-execute": 'SafeCommand::new("lxc-execute")?',
        "pty lxc": 'portable_pty::CommandBuilder::new("lxc")',
        "direct incus": 'std::process::Command::new("incus")',
        "tokio incus": 'tokio::process::Command::new("incus")',
    }
    expected = {
        "raw lxc": ["runtime-lxc"],
        "guarded lxc": ["runtime-lxc"],
        "guarded lxd": ["runtime-lxc"],
        "lxc-execute": ["runtime-lxc"],
        "pty lxc": ["runtime-lxc"],
        "direct incus": ["direct-incus"],
        "tokio incus": ["direct-incus"],
    }
    failures = 0
    for name, source in samples.items():
        rules = [rule for rule, _, _ in scan_text(source)]
        if rules != expected[name]:
            print(f"self-test: {name}: expected {expected[name]}, got {rules}", file=sys.stderr)
            failures += 1

    accepted = {
        "guarded incus": 'SafeCommand::new("incus")?',
        "pty incus": 'portable_pty::CommandBuilder::new("incus")',
        "comment": '// Command::new("lxc") bypass (#2126)',
    }
    for name, source in accepted.items():
        rules = [rule for rule, _, _ in scan_text(source)]
        if rules:
            print(f"self-test: {name}: expected no violation, got {rules}", file=sys.stderr)
            failures += 1

    if failures:
        return 1
    print("container runtime gate: self-test passed")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="scan the tree (default)")
    parser.add_argument("--self-test", action="store_true", help="run the built-in samples")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    if args.check or len(sys.argv) == 1:
        return check()
    parser.print_usage(sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
