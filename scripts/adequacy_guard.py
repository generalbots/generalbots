#!/usr/bin/env python3
"""Structural adequacy guard (issue #1735).

The adequacy sweep (#1736) was found with hand-run greps, so the same drift can
return unnoticed. This guard makes the structural rules in `AGENTS.md`
executable: it counts first-party deviations and fails when a count grows above
the checked-in ceiling in `issues/adequacy/baseline.json`.

Rules: `#[allow(..)]`; `table!` outside `botschema/src`; production `unwrap`,
`expect`, `panic!`, `todo!`/`unimplemented!`; `Command::new` outside
SafeCommand/build scripts/own-binary spawns; `.bak`/`.old`/`.orig`/`*~`;
CDN asset references; part-suffixed Rust modules (`_2`, `_a2`, `_b`, `_new`,
`partN`, `name2`) and CSS assets (`.pN.css`); files over 450 lines; and
commented-out code.

Production Rust excludes `bottest/`, test paths (`tests/`, `*_test.rs`,
`*_tests.rs`, `test(s).rs`) and anything inside a `#[cfg(test)]` item. Comments
are stripped first, so documentation is never counted. Generated `botbook`,
`target/`, `node_modules/`, `vendor/` and the vendored `last30days` skill are
not first-party and are skipped.

The baseline is a ceiling, not a snapshot: a count may stay equal or fall, never
rise. When a change removes a deviation, lower the number in the same change
(or run `--update`). `--strict DIR` ignores the baseline for part-suffix and
over-450-line violations under DIR, so a PR can be required to clean the
directory it touches before the repo-wide backlog reaches zero.

Usage:
    python3 scripts/adequacy_guard.py --baseline issues/adequacy/baseline.json
    python3 scripts/adequacy_guard.py --update
    python3 scripts/adequacy_guard.py --strict botserver/crates/botvibe

Exit codes: 0 = at/below baseline, 1 = growth or strict violation,
2 = usage/baseline error.
"""
from __future__ import annotations

import argparse
import json
import os
import re
import sys
import time

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
DEFAULT_BASELINE = os.path.join(ROOT, "issues", "adequacy", "baseline.json")

# Generated/vendored trees are not first-party.
SKIP_DIRS = {
    ".cache", ".git", ".venv", "__pycache__", "botbook", "botserver-installers",
    "botserver-stack", "build", "dist", "node_modules", "target", "vendor",
    "venv", "work",
}
SKIP_PREFIXES = ("botskills/research/last30days",)
SCAN_EXTENSIONS = {".rs", ".js", ".html", ".css"}
HARD_LIMIT = 450

ALLOW_RE = re.compile(r"#!?\[allow\s*\(")
TABLE_RE = re.compile(r"\btable!\s*\{")
COMMAND_RE = re.compile(r"\bCommand::new\s*\(")
CDN_RE = re.compile(
    r"unpkg\.com|jsdelivr\.net|cdn\.jsdelivr|cdnjs\.cloudflare|"
    r"fonts\.googleapis|fonts\.gstatic|maxcdn\.|bootstrapcdn\.",
    re.IGNORECASE,
)
PROD_PATTERNS = {
    "rust_unwrap": re.compile(r"\.unwrap\s*\("),
    "rust_expect": re.compile(r"\.expect\s*\("),
    "rust_panic": re.compile(r"\bpanic!\s*\("),
    "rust_todo": re.compile(r"\b(?:todo|unimplemented)!\s*\("),
}
COMMENT_PREFIXES = (
    "let ", "if ", "for ", "fn ", "pub fn ", "use ", "return ", "self.",
    "match ", "while ", "const ", "static ", "impl ", "struct ", "enum ",
)
COMMAND_ALLOWED_FILES = {
    "botapp/src/desktop/safe_command.rs",
    "botapp/src/desktop/sidecar.rs",
    "botapp/src/desktop/wizard/provision.rs",
    "botserver/crates/botvideo/src/safe_command.rs",
}
COMMAND_ALLOWED_PREFIXES = (
    "botlib/src/security/command_guard/",
    # The desktop wizard spawns the bundled botserver provisioner; #1730 split
    # provision.rs into this directory, so the exemption follows the code.
    "botapp/src/desktop/wizard/provision/",
)
# Directory-scoped rules: a PR must clear these in the directories it touches
# even while the repo-wide baseline is still above zero.
STRICT_RULES = {
    "part_suffixed_rust",
    "part_suffixed_css",
    "oversize_rs",
    "oversize_js",
    "oversize_html",
    "oversize_css",
}

# rule -> (owner that lowers the count, remediation printed in CI)
RULE_META = {
    "allow_attributes": (None, "remove the attribute and fix the lint (AGENTS.md)"),
    "table_outside_botschema": ("#1520", "move the table! into botschema (AGENTS.md)"),
    "rust_unwrap": (None, "propagate with ? or log the error (AGENTS.md)"),
    "rust_expect": (None, "propagate with ? or log the error (AGENTS.md)"),
    "rust_panic": (None, "return Err(...) instead (AGENTS.md)"),
    "rust_todo": (None, "implement it or return Err(NotImplemented) (AGENTS.md)"),
    "raw_command_new": ("unassigned", "use SafeCommand (AGENTS.md: Command Execution)"),
    "backup_files": ("#1734", "delete it; backups live in /tmp (AGENTS.md)"),
    "cdn_refs": ("#1733", "vendor the asset locally (AGENTS.md: Local Assets Only)"),
    "part_suffixed_rust": ("#1728", "use a self-describing name (AGENTS.md: splits)"),
    "part_suffixed_css": ("#1729", "rename the part after what it styles (AGENTS.md)"),
    "oversize_rs": ("#1730", "split at 350 lines, hard limit 450 (AGENTS.md)"),
    "oversize_js": ("#1731", "split at 350 lines, hard limit 450 (AGENTS.md)"),
    "oversize_html": ("#1731", "split at 350 lines, hard limit 450 (AGENTS.md)"),
    "oversize_css": ("#1731", "split at 350 lines, hard limit 450 (AGENTS.md)"),
    "commented_code": ("#1734", "delete or restore it (AGENTS.md)"),
}

BLOCK_COMMENT_RE = re.compile(r"/\*.*?\*/", re.DOTALL)
CFG_TEST_RE = re.compile(r"#\[cfg\(\s*test\s*\)\]")
Violation = tuple[str, int, str]


def _line_of(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def strip_comments(text: str) -> str:
    """Remove // and /* */ comments, preserving line numbers.

    Lexical approximation: it cuts at `//` even inside a string, which can hide
    code after a URL but never invents a violation.
    """
    text = BLOCK_COMMENT_RE.sub(lambda m: "\n" * m.group(0).count("\n"), text)
    return "\n".join(line.split("//", 1)[0] for line in text.split("\n"))


def strip_cfg_test(text: str) -> str:
    """Remove `#[cfg(test)]` items, preserving line numbers."""
    pieces, pos = [], 0
    for match in CFG_TEST_RE.finditer(text):
        if match.start() < pos:
            continue
        brace = text.find("{", match.end())
        semi = text.find(";", match.end())
        if brace == -1 or (semi != -1 and semi < brace):
            end = semi + 1 if semi != -1 else match.end()
        else:
            depth, end = 0, len(text)
            for index in range(brace, len(text)):
                if text[index] == "{":
                    depth += 1
                elif text[index] == "}":
                    depth -= 1
                    if depth == 0:
                        end = index + 1
                        break
        pieces.append(text[pos:match.start()])
        pieces.append("\n" * text[match.start():end].count("\n"))
        pos = end
    pieces.append(text[pos:])
    return "".join(pieces)


def is_production_path(rel: str) -> bool:
    parts = rel.split("/")
    if parts[0] == "bottest":
        return False
    if any(part in {"tests", "test", "testdata", "benches"} for part in parts[:-1]):
        return False
    stem = os.path.splitext(parts[-1])[0]
    return stem not in {"test", "tests"} and not stem.endswith(("_test", "_tests"))


def is_part_suffixed_rust(path: str) -> bool:
    stem = os.path.splitext(os.path.basename(path))[0]
    directory = os.path.dirname(path)
    if re.search(r"(?:^|_)part\d+$", stem) or re.search(r"_\d+$", stem):
        return True
    if re.search(r"_[a-z]$", stem) or re.search(r"_[a-d]2$", stem):
        return True
    if stem.endswith("_new"):
        return True
    # `installer_vault2.rs` / `installer_regs2.rs`: numeric twin of a real base
    # file or directory. `soc2.rs` (SOC 2) has no base and is not a split.
    if stem.endswith("2") and len(stem) > 1:
        base = stem[:-1]
        if os.path.exists(os.path.join(directory, base + ".rs")):
            return True
        if os.path.isdir(os.path.join(directory, base)):
            return True
        try:
            siblings = os.listdir(directory)
        except OSError:
            siblings = []
        if any(name.startswith(base + ".") and name != os.path.basename(path) for name in siblings):
            return True
    return False


def is_part_suffixed_css(path: str) -> bool:
    stem = os.path.splitext(os.path.basename(path))[0]
    return bool(re.search(r"(?:^|[._-])p\d+$", stem)) or bool(
        re.search(r"(?:^|_)part\d+$", stem)
    )


def count_lines(text: str) -> int:
    if not text:
        return 0
    return text.count("\n") + (0 if text.endswith("\n") else 1)


def scan_tree() -> tuple[dict[str, list[Violation]], int]:
    found: dict[str, list[Violation]] = {rule: [] for rule in RULE_META}
    scanned = 0
    for dirpath, dirnames, filenames in os.walk(ROOT):
        dirnames[:] = sorted(
            name for name in dirnames if name not in SKIP_DIRS and not name.startswith(".")
        )
        for name in sorted(filenames):
            path = os.path.join(dirpath, name)
            rel = os.path.relpath(path, ROOT).replace(os.sep, "/")
            if any(rel == p or rel.startswith(p + "/") for p in SKIP_PREFIXES):
                continue
            if name.endswith(("~", ".bak", ".old", ".orig")):
                found["backup_files"].append((rel, 0, ""))
                continue
            ext = os.path.splitext(name)[1]
            if ext not in SCAN_EXTENSIONS:
                continue
            scanned += 1
            try:
                with open(path, "r", encoding="utf-8", errors="replace") as handle:
                    text = handle.read()
            except OSError as error:
                print(f"adequacy_guard: cannot read {rel}: {error}", file=sys.stderr)
                continue

            lines = count_lines(text)
            # Generated bundles are not first-party sources: the Tailwind CLI
            # writes output.css (header identifies the generator), and splitting
            # it by hand would be overwritten on the next build. Vendored trees
            # are already skipped by SKIP_DIRS.
            if ext == ".css" and "tailwindcss v" in text[:200].lower():
                continue
            if lines > HARD_LIMIT:
                found["oversize_" + ext[1:]].append((rel, 0, f"{lines} lines"))
            if ext == ".css" and is_part_suffixed_css(rel):
                found["part_suffixed_css"].append((rel, 0, ""))
            if ext != ".rs":
                for number, line in enumerate(text.split("\n"), 1):
                    if CDN_RE.search(line):
                        found["cdn_refs"].append((rel, number, line.strip()[:120]))
                continue

            if is_part_suffixed_rust(path):
                found["part_suffixed_rust"].append((rel, 0, ""))
            cleaned = strip_comments(text)
            for match in ALLOW_RE.finditer(cleaned):
                found["allow_attributes"].append((rel, _line_of(cleaned, match.start()), ""))
            if not rel.startswith("botschema/"):
                for match in TABLE_RE.finditer(cleaned):
                    found["table_outside_botschema"].append(
                        (rel, _line_of(cleaned, match.start()), "")
                    )

            production = strip_cfg_test(cleaned) if is_production_path(rel) else None
            if production is not None:
                command_allowed = (
                    rel in COMMAND_ALLOWED_FILES
                    or name == "build.rs"
                    or any(rel.startswith(p) for p in COMMAND_ALLOWED_PREFIXES)
                )
                if not command_allowed:
                    for match in COMMAND_RE.finditer(production):
                        found["raw_command_new"].append(
                            (rel, _line_of(production, match.start()), "")
                        )
                for rule, pattern in PROD_PATTERNS.items():
                    for match in pattern.finditer(production):
                        found[rule].append((rel, _line_of(production, match.start()), ""))

            for number, line in enumerate(text.split("\n"), 1):
                stripped = line.strip()
                if not stripped.startswith("//") or stripped.startswith(("///", "//!")):
                    continue
                content = stripped[2:].strip()
                if not content or not content.startswith(COMMENT_PREFIXES):
                    continue
                statement = content.split(" //", 1)[0].rstrip()
                if statement.endswith((";", "{", "}")):
                    found["commented_code"].append((rel, number, content[:100]))
    return found, scanned


def load_baseline(path: str) -> dict:
    try:
        with open(path, "r", encoding="utf-8") as handle:
            data = json.load(handle)
    except (OSError, json.JSONDecodeError) as error:
        raise SystemExit(f"adequacy_guard: cannot read baseline {path}: {error}")
    rules = data.get("rules")
    if not isinstance(rules, dict):
        raise SystemExit(f"adequacy_guard: baseline {path} has no `rules` object")
    missing = [rule for rule in RULE_META if rule not in rules]
    if missing:
        raise SystemExit(f"adequacy_guard: baseline {path} is missing: {', '.join(missing)}")
    return data


def baseline_count(entry) -> int:
    if isinstance(entry, int):
        return entry
    if isinstance(entry, dict) and isinstance(entry.get("count"), int):
        return entry["count"]
    raise SystemExit(f"adequacy_guard: malformed baseline entry: {entry!r}")


def write_baseline(path: str, counts: dict[str, int], previous: dict | None) -> None:
    old_rules = (previous or {}).get("rules", {})
    rules = {
        rule: {
            "count": counts[rule],
            "owner": (old_rules.get(rule) or {}).get("owner", RULE_META[rule][0]),
        }
        for rule in RULE_META
    }
    data = {
        "issue": "#1735",
        "generated": time.strftime("%Y-%m-%d"),
        "contract": (
            "Ceilings, not snapshots: a count may stay equal or fall, never rise. "
            "Lower a number in the change that removes the deviation."
        ),
        "rules": rules,
    }
    directory = os.path.dirname(path)
    if directory:
        os.makedirs(directory, exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(data, handle, indent=2)
        handle.write("\n")
    print(f"adequacy_guard: baseline written to {os.path.relpath(path, ROOT)}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Structural adequacy guard (#1735)")
    parser.add_argument("--baseline", default=DEFAULT_BASELINE, help="baseline JSON path")
    parser.add_argument("--update", action="store_true", help="recompute the baseline from the tree")
    parser.add_argument(
        "--strict", action="append", default=[], metavar="DIR",
        help="fail on part-suffix/over-limit violations under DIR (repeatable)",
    )
    args = parser.parse_args()

    started = time.monotonic()
    try:
        found, scanned = scan_tree()
    except OSError as error:
        print(f"adequacy_guard: scan failed: {error}", file=sys.stderr)
        return 2
    counts = {rule: len(items) for rule, items in found.items()}

    if args.update:
        previous = None
        if os.path.exists(args.baseline):
            try:
                with open(args.baseline, "r", encoding="utf-8") as handle:
                    previous = json.load(handle)
            except (OSError, json.JSONDecodeError):
                previous = None
        write_baseline(args.baseline, counts, previous)
        return 0

    baseline = load_baseline(args.baseline)
    ceilings = {rule: baseline_count(baseline["rules"][rule]) for rule in RULE_META}

    strict = []
    for item in args.strict:
        normalized = item.replace("\\", "/").strip("/")
        if normalized.startswith("./"):
            normalized = normalized[2:]
        if normalized and normalized != ".":
            strict.append(normalized)

    failures: list[str] = []
    drops: list[str] = []
    for rule in RULE_META:
        hint = RULE_META[rule][1]
        current, ceiling = counts[rule], ceilings[rule]
        if current > ceiling:
            failures.append(f"{rule}: {current} > baseline {ceiling} (+{current - ceiling})")
            for path, line, detail in found[rule]:
                where = f"{path}:{line}" if line else path
                print(f"  {where}{f' ({detail})' if detail else ''}")
            print(f"  remediation: {hint}")
        elif current < ceiling:
            drops.append(f"{rule}: {current} < baseline {ceiling}")
        for path, line, detail in found[rule]:
            if rule not in STRICT_RULES:
                continue
            if any(path == root or path.startswith(root + "/") for root in strict):
                where = f"{path}:{line}" if line else f"{path}{f' ({detail})' if detail else ''}"
                failures.append(f"{rule}: {where} (strict: {hint})")

    if failures:
        print(f"adequacy_guard: FAIL — {len(failures)} issue(s) against {os.path.relpath(args.baseline, ROOT)}")
        for line in failures:
            print(f"- {line}")
        return 1

    elapsed = time.monotonic() - started
    print(
        f"adequacy_guard: OK — {len(RULE_META)} rules at or below baseline "
        f"({scanned} files scanned in {elapsed:.2f}s)"
    )
    for line in drops:
        print(f"  note: {line} — lower it with `--update` (counts only move down)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
