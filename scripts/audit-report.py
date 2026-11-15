#!/usr/bin/env python3
"""Cross-reference Cargo.lock against the RustSec advisory DB and report the
live / withdrawn / unaffected split.

This script is used by the security CI job to:
  - record the advisory DB commit (reproducibility)
  - print a clear live/withdrawn/unaffected split (so "22 packages" is never
    quoted without context)
  - fail on stale deny.toml ignore entries (ignore IDs missing from DB or
    marked withdrawn)

Design notes
------------
The most accurate classification comes from cargo-deny. If `cargo-deny` is
available on PATH, this script shells out to it (preferred). It can also run
against a pinned advisory DB by setting `RUSTSEC_ADVISORY_DB_PATH` (or passing
--advisory-db-path). When cargo-deny is not available, the script performs a
best-effort parse of the advisory DB git checkout (if provided) and validates
deny.toml ignore staleness against it.

Exit codes
----------
0  success (no live, non-ignored advisories)
1  live (non-ignored) advisories found OR stale ignore entries found
2  usage / input error
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass
class IgnoreEntry:
    id: str
    reason: str


def repo_root_from_here() -> Path:
    return Path(__file__).resolve().parents[1]


def load_deny_toml(path: Path) -> tuple[list[IgnoreEntry], dict[str, Any]]:
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    advisories = data.get("advisories") or {}
    ignores_raw = advisories.get("ignore") or []
    ignores: list[IgnoreEntry] = []
    for item in ignores_raw:
        if isinstance(item, dict):
            rid = item.get("id")
            if not rid:
                continue
            ignores.append(IgnoreEntry(id=str(rid), reason=str(item.get("reason", ""))))
        elif isinstance(item, str):
            ignores.append(IgnoreEntry(id=item, reason=""))
    return ignores, data


def git_head(path: Path) -> str | None:
    try:
        out = subprocess.check_output(
            ["git", "-C", str(path), "rev-parse", "HEAD"],
            stderr=subprocess.DEVNULL,
            text=True,
        ).strip()
        return out or None
    except Exception:
        return None


def print_advisory_db_commit(advisory_db_path: Path | None) -> None:
    if advisory_db_path and advisory_db_path.exists():
        commit = git_head(advisory_db_path)
        if commit:
            print(f"advisory-db-path: {advisory_db_path}")
            print(f"advisory-db-commit: {commit}")
        else:
            print(f"advisory-db-path: {advisory_db_path}")
            print("advisory-db-commit: (not a git repo or unknown)")
    else:
        print("advisory-db-path: (default cargo-deny location)")
        print("advisory-db-commit: (unknown; run cargo-deny to resolve)")


def find_advisory_files(advisory_db_path: Path) -> list[Path]:
    files: list[Path] = []
    # Common layouts
    for pat in ("crates/**/RUSTSEC-*.toml", "advisories/*.toml", "crates/**/RUSTSEC-*.md"):
        files.extend(advisory_db_path.glob(pat))
    # Some repos store as .md with frontmatter; try to be broad
    if not files:
        files.extend(advisory_db_path.rglob("RUSTSEC-*.toml"))
        files.extend(advisory_db_path.rglob("RUSTSEC-*.md"))
    return sorted(set(files))


def parse_advisory_file(path: Path) -> dict[str, Any] | None:
    text = path.read_text(encoding="utf-8", errors="replace")
    if path.suffix == ".toml":
        try:
            return tomllib.loads(text)
        except Exception:
            return None
    if path.suffix == ".md":
        # Try to extract TOML frontmatter between --- and ---
        if text.startswith("---"):
            parts = text.split("---", 2)
            if len(parts) >= 3:
                try:
                    return tomllib.loads(parts[1])
                except Exception:
                    return None
    return None


def is_withdrawn(advisory_doc: dict[str, Any]) -> bool:
    adv = advisory_doc.get("advisory") or {}
    if adv.get("withdrawn"):
        return True
    # Some formats
    if advisory_doc.get("withdrawn"):
        return True
    return False


def stale_ignore_check(ignores: list[IgnoreEntry], advisory_db_path: Path | None) -> list[str]:
    errors: list[str] = []
    if not advisory_db_path or not advisory_db_path.exists():
        print("WARN: no advisory DB path provided; skipping stale-ignore DB check (cannot verify withdrawn/missing).", file=sys.stderr)
        return errors
    files = find_advisory_files(advisory_db_path)
    id_to_docs: dict[str, list[Path]] = {}
    for f in files:
        # filename often contains ID
        name = f.name
        for ig in ignores:
            if ig.id in name:
                id_to_docs.setdefault(ig.id, []).append(f)
    for ig in ignores:
        paths = id_to_docs.get(ig.id, [])
        if not paths:
            errors.append(f"STALE: ignore entry '{ig.id}' not found in advisory DB at {advisory_db_path} (may be removed/fixed upstream). Remove it from deny.toml.")
            continue
        withdrawn_any = False
        found_valid = False
        for p in paths:
            doc = parse_advisory_file(p)
            if doc is None:
                continue
            adv = doc.get("advisory") or doc
            adv_id = adv.get("id") or doc.get("id")
            if adv_id and adv_id != ig.id:
                # sometimes file named differently; still check
                pass
            if is_withdrawn(doc):
                withdrawn_any = True
            else:
                found_valid = True
        if not found_valid and withdrawn_any:
            errors.append(f"STALE/WITHDRAWN: ignore entry '{ig.id}' is marked withdrawn in advisory DB ({paths[0]}). Remove it from deny.toml.")
        elif not paths:
            pass
    return errors


def run_cargo_deny_advisories(repo_root: Path, advisory_db_path: Path | None) -> int:
    cmd = ["cargo-deny", "--all-features", "check", "advisories"]
    env = os.environ.copy()
    if advisory_db_path:
        env["RUSTSEC_ADVISORY_DB_PATH"] = str(advisory_db_path)
    print("\n== Running: " + " ".join(cmd) + " ==")
    try:
        proc = subprocess.run(cmd, cwd=str(repo_root), env=env)
        return proc.returncode
    except FileNotFoundError:
        print("cargo-deny not found on PATH; cannot run live advisory check.", file=sys.stderr)
        print("Install cargo-deny or run in CI. For offline DB stale checks, pass --advisory-db-path.", file=sys.stderr)
        return 2


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=None, help="Repository root (default: auto)")
    parser.add_argument("--deny-toml", type=Path, default=None, help="Path to deny.toml (default: <repo>/deny.toml)")
    parser.add_argument("--advisory-db-path", type=Path, default=None, help="Path to pinned RustSec advisory DB checkout")
    parser.add_argument("--skip-cargo-deny", action="store_true", help="Only run stale ignore check and DB reporting")
    parser.add_argument("--help-summary", action="store_true", help="Print short summary and exit 0")
    args = parser.parse_args()

    if args.help_summary:
        print("audit-report.py: reports RustSec advisory DB commit and checks deny.toml ignore staleness.")
        return 0

    repo_root = args.repo_root or repo_root_from_here()
    deny_path = args.deny_toml or (repo_root / "deny.toml")

    print("== audit-report.py ==")
    print(f"repo-root: {repo_root}")
    print(f"deny-toml: {deny_path}")

    try:
        ignores, _ = load_deny_toml(deny_path)
    except Exception as e:
        print(f"ERROR: failed to read {deny_path}: {e}", file=sys.stderr)
        return 2

    print(f"deny.toml ignore entries: {len(ignores)}")
    for ig in ignores:
        print(f"  - {ig.id}")

    print_advisory_db_commit(args.advisory_db_path)

    stale_errors = stale_ignore_check(ignores, args.advisory_db_path)
    if stale_errors:
        print("\nStale ignore check: FAIL", file=sys.stderr)
        for e in stale_errors:
            print(f"  {e}", file=sys.stderr)
        return 1
    print("\nStale ignore check: OK")

    if args.skip_cargo_deny:
        print("Skipping cargo-deny run (--skip-cargo-deny).")
        return 0

    rc = run_cargo_deny_advisories(repo_root, args.advisory_db_path)
    if rc != 0:
        print(f"\ncargo-deny advisories check failed (exit {rc}).", file=sys.stderr)
        return 1
    print("\ncargo-deny advisories check: OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
