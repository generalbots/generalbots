#!/usr/bin/env python3
"""Static audit: map the 100 UAT.md scenarios to automated test coverage.

Offline only. This script reads UAT.md plus the automated suite sources under
``botui/ui/suite/tests/`` and ``bottest/tests/`` and reports, per scenario,
whether a test carries its stable ID (for example ``DOC-05``) as a label.
It does NOT run a test, a browser, cargo or a model call. A scenario whose ID
is absent from every automated suite source is a GAP (manual / LLM / hardware
only), even if a module harness happens to exercise the same app code.

Exit code 0 requires BOTH:
  * >= 60% of the parsed scenarios mapped to a suite test, and
  * the deterministic, previously verified scenarios mapped as well.

Otherwise exit 1. Usage::

    python3 scripts/check_uat_scenario_coverage.py [--root /path/to/repo]
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

MIN_SCENARIOS = 90
MIN_COVERAGE = 0.60

# Only scenarios with a deterministic (LLM-independent), previously verified
# re-run on record. DOC-05 was re-verified through the seeded-doc deep-link
# flow in issue #2031; it must be backed by an automated test, not a manual run.
DETERMINISTIC_IDS = frozenset({"DOC-05"})

# "### DOC-05 · \"Open my last document about the Q3 report\" · en"
SCENARIO_RE = re.compile(r"^###\s+([A-Z]+-\d{2})(?:\s|$)", re.MULTILINE)
SUITE_DIRS = ("botui/ui/suite/tests", "bottest/tests")
SUITE_SUFFIXES = frozenset({".js", ".rs"})

# Heuristics only: a hint why a GAP is not automatable offline. They never
# change the PASS/GAP verdict, which is decided solely by the ID search.
HARDWARE_MARKERS = (
    "attach", "anex", "camera", "microphone", "device", "folder",
    "ffprobe", "reload the page", "private window", "second browser",
)
LLM_MARKERS = (
    "generate", "generated", "gere ", "gerar", "crie", "create", "write",
    "escreva", "traduz", "translate", "resum", "summar", "pesquise",
    "research", "find ", "notícias", "news", "mp3", "músic", "music",
    "vídeo", "video", "cotações", "explique", "explain", "compare",
    "compare", "compare", "analis", "score de risco", "faça um resumo",
)


class AuditError(Exception):
    """Fatal audit input problem (missing or renamed UAT file, unreadable tree)."""


def parse_scenarios(uat_path: Path) -> list[tuple[str, str]]:
    """Return (scenario_id, scenario_block) pairs from UAT.md.

    A block runs from the ID heading to the next scenario heading (or the end
    of the file). Fails loudly when fewer than MIN_SCENARIOS are found so an
    emptied, renamed or restructured file cannot silently pass.
    """
    try:
        text = uat_path.read_text(encoding="utf-8", errors="replace")
    except OSError as exc:
        raise AuditError(f"cannot read {uat_path}: {exc}") from exc

    matches = list(SCENARIO_RE.finditer(text))
    scenarios: list[tuple[str, str]] = []
    for index, match in enumerate(matches):
        end = matches[index + 1].start() if index + 1 < len(matches) else len(text)
        scenarios.append((match.group(1).upper(), text[match.end():end]))

    if len(scenarios) < MIN_SCENARIOS:
        raise AuditError(
            f"parsed only {len(scenarios)} scenario IDs from {uat_path}; "
            f"expected >= {MIN_SCENARIOS} - UAT.md is empty, renamed or "
            "its '### ID' headings changed"
        )
    return scenarios


def inventory_suite_sources(root: Path) -> dict[str, str]:
    """Read every automated suite source once, keyed by repo-relative path."""
    sources: dict[str, str] = {}
    for relative_dir in SUITE_DIRS:
        directory = root / relative_dir
        if not directory.is_dir():
            raise AuditError(f"missing automated suite directory: {directory}")
        try:
            files = [p for p in directory.rglob("*")
                     if p.is_file() and p.suffix.lower() in SUITE_SUFFIXES]
        except OSError as exc:
            raise AuditError(f"cannot list {directory}: {exc}") from exc
        for path in sorted(files):
            try:
                text = path.read_text(encoding="utf-8", errors="replace")
            except OSError as exc:
                raise AuditError(f"cannot read {path}: {exc}") from exc
            sources[path.relative_to(root).as_posix()] = text.upper()
    return sources


def id_pattern(scenario_id: str) -> re.Pattern[str]:
    """Match the stable ID as a standalone label, case-insensitively."""
    return re.compile(r"(?<![A-Z0-9])" + re.escape(scenario_id) + r"(?![A-Z0-9])")


def gap_reason(block: str) -> str:
    """Best-effort hint for why a GAP has no automated coverage."""
    lowered = block.lower()
    if any(marker in lowered for marker in HARDWARE_MARKERS):
        return "hardware/manual"
    if any(marker in lowered for marker in LLM_MARKERS):
        return "LLM"
    return "manual"


def map_scenarios(
    scenarios: list[tuple[str, str]], sources: dict[str, str]
) -> list[tuple[str, str, str, str]]:
    """Return (id, status, evidence, reason) for every scenario."""
    results: list[tuple[str, str, str, str]] = []
    for scenario_id, block in scenarios:
        pattern = id_pattern(scenario_id)
        evidence = next(
            (path for path, text in sources.items() if pattern.search(text)), ""
        )
        if evidence:
            results.append((scenario_id, "PASS", evidence, ""))
        else:
            results.append((scenario_id, "GAP", "", gap_reason(block)))
    return results


def report(results: list[tuple[str, str, str, str]], root: Path) -> int:
    """Print per-scenario lines plus a compact summary. Return the exit code."""
    mapped = [r for r in results if r[1] == "PASS"]
    gaps = [r for r in results if r[1] == "GAP"]
    total = len(results)
    coverage = len(mapped) / total if total else 0.0
    mapped_ids = {r[0] for r in mapped}

    print(f"UAT scenario coverage audit (root: {root})")
    for scenario_id, status, evidence, reason in results:
        if status == "PASS":
            print(f"PASS {scenario_id:<11} automated: {evidence}")
        else:
            print(f"GAP  {scenario_id:<11} reason: {reason or 'unknown'}")

    reasons: dict[str, int] = {}
    for _, _, _, reason in gaps:
        reasons[reason or "unknown"] = reasons.get(reason or "unknown", 0) + 1

    print("-" * 68)
    print(
        f"total={total} mapped={len(mapped)} gaps={len(gaps)} "
        f"coverage={coverage:.1%} (threshold {MIN_COVERAGE:.0%})"
    )
    print("gap reasons: " + ", ".join(
        f"{name}={count}" for name, count in sorted(reasons.items())
    ) or "gap reasons: none")
    print("first 10 GAP ids: " + ", ".join(r[0] for r in gaps[:10]) or "-")

    failed = False
    if coverage < MIN_COVERAGE:
        print(
            f"FAIL coverage {coverage:.1%} is below {MIN_COVERAGE:.0%}; "
            f"{len(gaps)} of {total} scenarios have no automated test carrying "
            "their UAT ID"
        )
        failed = True
    missing_deterministic = sorted(DETERMINISTIC_IDS - mapped_ids)
    if missing_deterministic:
        print(
            "FAIL deterministic scenario(s) not mapped to an automated test: "
            + ", ".join(missing_deterministic)
            + " (previously verified only by a manual browser re-run)"
        )
        failed = True
    if failed:
        print("RESULT: FAIL (exit 1)")
        return 1
    print("RESULT: PASS (exit 0)")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root", type=Path, default=Path(__file__).resolve().parent.parent,
        help="workspace root containing UAT.md and the suite directories",
    )
    args = parser.parse_args(argv)
    root = args.root.resolve()

    try:
        scenarios = parse_scenarios(root / "UAT.md")
        sources = inventory_suite_sources(root)
    except AuditError as exc:
        print(f"FAIL {exc}", file=sys.stderr)
        return 1

    return report(map_scenarios(scenarios, sources), root)


if __name__ == "__main__":
    sys.exit(main())
