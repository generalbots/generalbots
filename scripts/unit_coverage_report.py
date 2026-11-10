#!/usr/bin/env python3
"""Generate botbook/src/12-ecosystem-reference/unit-coverage.md from the tree.

The report is deterministic: crates are discovered by walking
``botserver/crates/*`` plus the root crates, tests are counted by parsing
``#[test]`` / ``#[tokio::test]`` attributes, and every list is sorted. Running
the script twice on the same tree produces byte-identical output, so a review
diff only shows real coverage changes.

Usage:
    python3 scripts/unit_coverage_report.py            # write the page
    python3 scripts/unit_coverage_report.py --stdout   # print, do not write
"""

from __future__ import annotations

import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUTPUT = os.path.join(
    ROOT, "botbook", "src", "12-ecosystem-reference", "unit-coverage.md"
)

# Directories that never contain hand-written Rust.
SKIP_DIRS = {"target", "node_modules", ".git", "work", "botserver-stack"}

TEST_ATTR = re.compile(r"^\s*#\[(?:tokio::)?test(?:\([^\]]*\))?\]", re.MULTILINE)

ROOT_CRATES = ("botapp", "botlib", "botschema", "bottest", "botui")

# One-line status for every crate that still has no test, with the reason it
# has none. Crates with pure helpers but no tests say so explicitly instead of
# being labelled "thin" — the difference is the remaining work.
NO_TEST_REASONS = {
    "botchannelbindings": "Diesel models plus three routes; the only pure function is the `ChannelBinding::empty()` constructor.",
    "botchannels-core": "Provider metadata and token-expiry helpers that read the system clock; no injected clock, so the expiry branches need a small refactor first.",
    "botcoresession": "Session store over Diesel and Valkey; the one pure helper (`is_placeholder_title`) is a four-line predicate.",
    "boteditor": "File CRUD plus a local `html_escape` that duplicates `botuifragments::htmx_escape` (covered there).",
    "bothandoff": "Thin CRUD: handlers/db/routes/storage over the `handoff_*` tables.",
    "bothr": "Thin CRUD: handlers/db/routes/storage over the `hr_*` tables.",
    "botitsm": "Thin CRUD: handlers/db/routes/storage over the `itsm_*` tables.",
    "botlegal": "Contract DTOs and expiry helpers that read the clock; no pure parser.",
    "botmultimodal": "Configuration readers over the `ConfigProvider` trait; every path is I/O.",
    "botnvidia": "Shells out to `nvidia-smi`; no pure logic.",
    "botpublic": "Twelve lines of route registration.",
    "botsales": "Thin CRUD: leads and quotes over the `sales_*` tables.",
    "bottasks": "**Gap:** the scheduler queries `system_automations`; `should_run_now` reads the clock directly, so it needs an injected `now` before it can be unit-tested.",
}


def crate_dirs() -> list[str]:
    """Every workspace crate that holds hand-written Rust, sorted by name."""
    found: list[str] = []
    crates_root = os.path.join(ROOT, "botserver", "crates")
    if os.path.isdir(crates_root):
        for name in sorted(os.listdir(crates_root)):
            path = os.path.join(crates_root, name)
            if os.path.isdir(path) and os.path.isfile(os.path.join(path, "Cargo.toml")):
                found.append(name)
    for name in ROOT_CRATES:
        path = os.path.join(ROOT, name)
        if os.path.isdir(path) and os.path.isfile(os.path.join(path, "Cargo.toml")):
            found.append(name)
    return sorted(set(found))


def rust_files(crate_path: str) -> list[str]:
    paths: list[str] = []
    for dirpath, dirnames, filenames in os.walk(crate_path):
        dirnames[:] = sorted(d for d in dirnames if d not in SKIP_DIRS)
        for filename in sorted(filenames):
            if filename.endswith(".rs"):
                paths.append(os.path.join(dirpath, filename))
    return paths


def analyse(crate: str) -> tuple[int, list[str], int]:
    """Return ``(test_count, test_topics, file_count)`` for one crate."""
    base = os.path.join(ROOT, "botserver", "crates", crate)
    if not os.path.isdir(base):
        base = os.path.join(ROOT, crate)

    count = 0
    topics: list[str] = []
    files = rust_files(base)
    for path in files:
        try:
            with open(path, encoding="utf-8", errors="ignore") as handle:
                text = handle.read()
        except OSError:
            continue
        count += len(TEST_ATTR.findall(text))

        name = os.path.basename(path)
        if name.endswith("_tests.rs"):
            topics.append(name[: -len("_tests.rs")])
        elif os.sep + "tests" + os.sep in path:
            topics.append(os.path.splitext(name)[0] + " (integration)")

    return count, sorted(set(topics)), len(files)


def describe(topics: list[str], tests: int) -> str:
    if tests == 0:
        return "—"
    if not topics:
        return "inline `#[cfg(test)]` modules"
    shown = topics[:4]
    text = ", ".join(f"`{topic}`" for topic in shown)
    if len(topics) > len(shown):
        text += f", +{len(topics) - len(shown)} more"
    return text


def build() -> str:
    rows = []
    for crate in crate_dirs():
        tests, topics, files = analyse(crate)
        rows.append((crate, tests, topics, files))

    rows.sort(key=lambda row: (-row[1], row[0]))

    total_tests = sum(row[1] for row in rows)
    total_files = sum(row[3] for row in rows)
    zero = [row for row in rows if row[1] == 0]
    under_five = [row for row in rows if 0 < row[1] < 5]

    lines: list[str] = []
    lines.append("# Unit test coverage by crate")
    lines.append("")
    lines.append(
        "Deterministic inventory of the offline unit tests in the Rust workspace:"
        " one row per crate, the number of `#[test]` / `#[tokio::test]` functions"
        " it owns, and the modules those tests live in. It is generated from the"
        " tree — never edited by hand:"
    )
    lines.append("")
    lines.append("```bash")
    lines.append("python3 scripts/unit_coverage_report.py")
    lines.append("```")
    lines.append("")
    lines.append(
        "The count is a *function* count, not a line-coverage percentage. A crate"
        " with many tests can still miss a branch; the per-module line and branch"
        " numbers come from the `cargo-llvm-cov` job (#2074). What this page"
        " guarantees is that no crate is silently absent from the unit gate."
    )
    lines.append("")
    lines.append("## Summary")
    lines.append("")
    lines.append(f"- Crates in the workspace: **{len(rows)}**")
    lines.append(f"- `.rs` files scanned: **{total_files}**")
    lines.append(f"- Test functions: **{total_tests}**")
    lines.append(f"- Crates with no tests: **{len(zero)}**")
    lines.append(f"- Crates with 1–4 tests: **{len(under_five)}**")
    lines.append("")
    lines.append(
        "Run one crate at a time (`cargo test -p <crate> --lib`); the workspace"
        " graph does not fit the 4.5 GiB dev container in one go (AGENTS.md)."
    )
    lines.append("")
    lines.append("## Coverage by crate")
    lines.append("")
    lines.append("| Crate | Tests | What the tests cover |")
    lines.append("|-------|------:|----------------------|")
    for crate, tests, topics, _files in rows:
        lines.append(f"| `{crate}` | {tests} | {describe(topics, tests)} |")
    lines.append("")

    lines.append("## Crates without unit tests")
    lines.append("")
    lines.append(
        "Every crate below has zero test functions. The reason states whether it"
        " is thin glue (Tier C: covered by the integration suite and listed in"
        " `coverage-exclusions.toml`) or a **gap** — a crate that does hold pure"
        " logic and still needs a test."
    )
    lines.append("")
    lines.append("| Crate | Reason |")
    lines.append("|-------|--------|")
    for crate, _tests, _topics, _files in zero:
        reason = NO_TEST_REASONS.get(crate, "**Unclassified** — needs a tier decision.")
        lines.append(f"| `{crate}` | {reason} |")
    lines.append("")

    lines.append("## Crates with fewer than five tests")
    lines.append("")
    lines.append("| Crate | Tests | What the tests cover |")
    lines.append("|-------|------:|----------------------|")
    for crate, tests, topics, _files in under_five:
        lines.append(f"| `{crate}` | {tests} | {describe(topics, tests)} |")
    lines.append("")

    return "\n".join(lines)


def main() -> int:
    page = build()
    if "--stdout" in sys.argv:
        sys.stdout.write(page)
        return 0
    os.makedirs(os.path.dirname(OUTPUT), exist_ok=True)
    with open(OUTPUT, "w", encoding="utf-8") as handle:
        handle.write(page)
    print(f"wrote {os.path.relpath(OUTPUT, ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
