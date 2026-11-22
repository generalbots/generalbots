#!/usr/bin/env python3
"""Office large-file memory gate (#2530).

Declaration lives in ``ops/office-slo.yaml`` under ``office_memory``: a
deterministic synthetic corpus (5-100 MB DOCX/XLSX/PPTX-like ZIPs), per-request
RSS ceilings and a 20-cycle soak policy. This checker validates that table and,
when a measured snapshot is supplied, enforces the ceilings. It never invents
RSS numbers: without ``--snapshot`` the runtime criteria are reported as
UNVERIFIED, never as PASS.

Snapshot format (``cases`` keys are the corpus case ids from the YAML)::

    {
      "measured_at": "2026-11-22T00:00:00Z",
      "cgroup_limit_mb": 4096,
      "cases": {
        "sheets-large": {
          "peak_rss_mb": 980, "baseline_rss_mb": 640, "return_rss_mb": 660,
          "cycles": 20, "session_cache_growth_mb": 2, "events_with_content": 0
        }
      }
    }

Usage::

    python3 scripts/check_office_memory.py --self-test
    python3 scripts/check_office_memory.py --check
    python3 scripts/check_office_memory.py --check --snapshot /tmp/office-rss.json
    python3 scripts/check_office_memory.py --check --snapshot /tmp/office-rss.json --json

Exit codes: 0 declaration valid and every measured budget met, 1 a problem or
breach (fail closed), 2 the config/snapshot is unusable.
"""

from __future__ import annotations

import argparse
import copy
import json
import sys
from pathlib import Path
from typing import Any, Callable

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
DEFAULT_CONFIG = ROOT / "ops" / "office-slo.yaml"
KINDS = ("docx", "xlsx", "pptx")
MEASURED_FIELDS = (
    "peak_rss_mb",
    "baseline_rss_mb",
    "return_rss_mb",
    "cycles",
    "session_cache_growth_mb",
    "events_with_content",
)


def _extract_section(text: str, key: str) -> str:
    """Take one top-level YAML key as its own block (for the fallback parser)."""
    lines = text.splitlines()
    starts = [i for i, line in enumerate(lines) if line.strip() == f"{key}:"]
    if len(starts) != 1:
        raise ValueError(f"expected exactly one top-level {key!r} block")
    start = starts[0]
    stop = len(lines)
    for index in range(start + 1, len(lines)):
        line = lines[index]
        if line.strip() and not line[0].isspace():
            stop = index
            break
    return "\n".join(lines[start:stop])


def load_config(path: Path = DEFAULT_CONFIG) -> dict[str, Any]:
    """Load the #2530 section with PyYAML, falling back to the repo's subset parser."""
    text = path.read_text(encoding="utf-8")
    try:
        import yaml  # type: ignore[import-not-found]
    except ImportError:
        section = _extract_section(text, "office_memory")
        if str(HERE) not in sys.path:
            sys.path.insert(0, str(HERE))
        from slo_common import parse_simple_yaml

        office = parse_simple_yaml(section).get("office_memory")
    else:
        loaded = yaml.safe_load(text)
        if not isinstance(loaded, dict):
            raise ValueError(f"{path}: top level must be a mapping")
        office = loaded.get("office_memory")
    if not isinstance(office, dict):
        raise ValueError(f"{path}: missing 'office_memory' mapping")
    return office


def load_snapshot(path: Path) -> dict[str, Any]:
    try:
        snapshot = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        raise ValueError(f"{path}: invalid JSON: {error}") from error
    if not isinstance(snapshot, dict) or not isinstance(snapshot.get("cases"), dict):
        raise ValueError(f"{path}: expected an object with a 'cases' object")
    return snapshot


def _number(value: Any) -> bool:
    return isinstance(value, (int, float)) and not isinstance(value, bool)


def validate_config(config: dict[str, Any]) -> list[str]:
    """Return every schema/policy problem; an empty list means the table is sound."""
    problems: list[str] = []
    if config.get("issue") != 2530:
        problems.append("office_memory.issue must be 2530")
    corpus = config.get("corpus")
    admission = config.get("admission")
    ceilings = config.get("rss_ceilings_mb")
    soak = config.get("soak")
    events = config.get("events")
    if not isinstance(corpus, dict):
        problems.append("office_memory.corpus is missing")
    if not isinstance(admission, dict):
        problems.append("office_memory.admission is missing")
    if not isinstance(ceilings, dict):
        problems.append("office_memory.rss_ceilings_mb is missing")
    if not isinstance(soak, dict):
        problems.append("office_memory.soak is missing")
    if not isinstance(events, dict):
        problems.append("office_memory.events is missing")
    if any(not isinstance(value, dict) for value in (corpus, admission, ceilings, soak, events)):
        return problems

    if corpus.get("synthetic_only") is not True:
        problems.append("corpus.synthetic_only must be true (no real client data)")
    if not isinstance(corpus.get("seed"), int) or corpus["seed"] < 0:
        problems.append("corpus.seed must be a non-negative integer")
    low, high = corpus.get("min_size_mb"), corpus.get("max_size_mb")
    bounds_ok = bool(_number(low) and _number(high) and 0 < low < high)
    if not bounds_ok:
        problems.append("corpus.min_size_mb/max_size_mb must be positive and ordered")
    elif low != 5 or high != 100:
        problems.append("corpus size policy must be the declared 5-100 MB range")

    cases = corpus.get("cases")
    if not isinstance(cases, list) or not cases:
        problems.append("corpus.cases must be a non-empty list")
        return problems
    ids: set[str] = set()
    for case in cases:
        if not isinstance(case, dict):
            problems.append("every corpus case must be a mapping")
            continue
        identifier = case.get("id")
        kind = case.get("kind")
        size = case.get("size_mb")
        if not isinstance(identifier, str) or not identifier:
            problems.append("every corpus case needs a non-empty id")
        elif identifier in ids:
            problems.append(f"duplicate corpus case id: {identifier}")
        else:
            ids.add(identifier)
        if kind not in KINDS:
            problems.append(f"case {identifier}: kind must be one of {KINDS}")
        if not _number(size) or (bounds_ok and not (low <= size <= high)):
            problems.append(f"case {identifier}: size_mb outside the corpus policy")

    if set(ceilings) != ids:
        problems.append(
            f"rss_ceilings_mb keys {sorted(ceilings)} must match case ids {sorted(ids)}"
        )
    for identifier, ceiling in ceilings.items():
        if not _number(ceiling) or ceiling <= 0:
            problems.append(f"ceiling for {identifier} must be a positive number")

    multipliers = admission.get("peak_multiplier")
    if not isinstance(multipliers, dict) or set(multipliers) != set(KINDS):
        problems.append("admission.peak_multiplier must declare every Office kind")
    else:
        for kind, multiplier in multipliers.items():
            if not _number(multiplier) or multiplier <= 0:
                problems.append(f"admission.peak_multiplier.{kind} must be positive")
    if admission.get("max_concurrent_per_class") != 1:
        problems.append("admission.max_concurrent_per_class must be 1 (one heavy job per class)")
    if admission.get("queue_observable") is not True:
        problems.append("admission.queue_observable must be true")

    if soak.get("cycles") != 20:
        problems.append("soak.cycles must be 20")
    if not _number(soak.get("rss_return_tolerance_pct")) or soak["rss_return_tolerance_pct"] <= 0:
        problems.append("soak.rss_return_tolerance_pct must be positive")
    if not _number(soak.get("max_session_cache_growth_mb")) or soak["max_session_cache_growth_mb"] < 0:
        problems.append("soak.max_session_cache_growth_mb must be non-negative")
    if soak.get("evidence") != "runtime-uat-required":
        problems.append("soak.evidence must be 'runtime-uat-required' (no fabricated PASS)")
    if events.get("structured") is not True:
        problems.append("events.structured must be true")
    if events.get("include_document_content") is not False:
        problems.append("events.include_document_content must be false")
    return problems


def evaluate_snapshot(config: dict[str, Any], snapshot: dict[str, Any]) -> dict[str, Any]:
    """Check one measured RSS snapshot against the declared ceilings and soak policy."""
    ceilings = config["rss_ceilings_mb"]
    tolerance = float(config["soak"]["rss_return_tolerance_pct"])
    max_growth = float(config["soak"]["max_session_cache_growth_mb"])
    required_cycles = int(config["soak"]["cycles"])
    measured = snapshot["cases"]
    results: list[dict[str, Any]] = []
    breaches: list[dict[str, Any]] = []

    for identifier, ceiling in sorted(ceilings.items()):
        entry = measured.get(identifier)
        if not isinstance(entry, dict):
            breaches.append({"case": identifier, "detail": "no measured case data"})
            continue
        problems: list[str] = []
        values: dict[str, float] = {}
        for field in MEASURED_FIELDS:
            value = entry.get(field)
            if not _number(value):
                problems.append(f"missing numeric {field}")
            else:
                values[field] = float(value)
        if problems:
            breaches.append({"case": identifier, "detail": "; ".join(problems)})
            continue
        if values["peak_rss_mb"] > float(ceiling):
            problems.append(
                f"peak RSS {values['peak_rss_mb']} MB exceeds ceiling {ceiling} MB"
            )
        if values["cycles"] < required_cycles:
            problems.append(f"only {int(values['cycles'])} soak cycles, need {required_cycles}")
        if values["baseline_rss_mb"] > 0:
            allowed = values["baseline_rss_mb"] * (1 + tolerance / 100.0)
            if values["return_rss_mb"] > allowed:
                problems.append(
                    f"RSS returned to {values['return_rss_mb']} MB, over {allowed:.1f} MB "
                    f"({tolerance:.0f}% of {values['baseline_rss_mb']} MB)"
                )
        else:
            problems.append("baseline_rss_mb must be positive")
        if values["session_cache_growth_mb"] > max_growth:
            problems.append(
                f"session cache grew {values['session_cache_growth_mb']} MB, over {max_growth} MB"
            )
        if values["events_with_content"] != 0:
            problems.append("a structured event included document content")
        results.append({
            "case": identifier,
            "peak_rss_mb": values["peak_rss_mb"],
            "ceiling_mb": float(ceiling),
            "cycles": int(values["cycles"]),
            "met": not problems,
        })
        if problems:
            breaches.append({"case": identifier, "detail": "; ".join(problems)})

    return {
        "ok": not breaches,
        "checked": len(results),
        "cases": results,
        "breaches": breaches,
        "verdict": "PASS" if not breaches else "FAIL",
    }


def render(report: dict[str, Any]) -> str:
    lines = [f"Office RSS gate: checked={report['checked']} breaches={len(report['breaches'])}"]
    for case in report["cases"]:
        status = "PASS" if case["met"] else "FAIL"
        lines.append(
            f"  {status} {case['case']}: peak {case['peak_rss_mb']} MB "
            f"(ceiling {case['ceiling_mb']} MB), {case['cycles']} soak cycles"
        )
    for breach in report["breaches"]:
        lines.append(f"  BREACH {breach['case']}: {breach['detail']}")
    lines.append("RESULT: " + report["verdict"])
    return "\n".join(lines)


def self_test(config_path: Path = DEFAULT_CONFIG) -> int:
    checks = 0
    failures: list[str] = []

    def check(condition: bool, label: str) -> None:
        nonlocal checks
        checks += 1
        if not condition:
            failures.append(label)

    def rejects(label: str, action: Callable[[], Any], errors: tuple[type[BaseException], ...] = (ValueError,)) -> None:
        try:
            action()
        except errors:
            check(True, label)
        else:
            check(False, label)

    try:
        config = load_config(config_path)
        check(not validate_config(config), "real ops/office-slo.yaml is valid")

        good: dict[str, Any] = {"cases": {}}
        for identifier, ceiling in config["rss_ceilings_mb"].items():
            good["cases"][identifier] = {
                "peak_rss_mb": float(ceiling) * 0.9,
                "baseline_rss_mb": 600.0,
                "return_rss_mb": 620.0,
                "cycles": 20,
                "session_cache_growth_mb": 1.0,
                "events_with_content": 0,
            }
        check(evaluate_snapshot(config, good)["ok"], "measured snapshot inside ceilings passes")

        cases = sorted(config["rss_ceilings_mb"])
        first = cases[0]
        over = copy.deepcopy(good)
        over["cases"][first]["peak_rss_mb"] = float(config["rss_ceilings_mb"][first]) + 1
        over_report = evaluate_snapshot(config, over)
        check(not over_report["ok"] and any("exceeds ceiling" in b["detail"] for b in over_report["breaches"]),
              "over-ceiling RSS fails")

        missing = copy.deepcopy(good)
        del missing["cases"][first]
        missing_report = evaluate_snapshot(config, missing)
        check(not missing_report["ok"] and any("no measured case data" in b["detail"] for b in missing_report["breaches"]),
              "missing case evidence fails")

        short = copy.deepcopy(good)
        short["cases"][first]["cycles"] = 19
        check(not evaluate_snapshot(config, short)["ok"], "short soak fails")

        leaky = copy.deepcopy(good)
        leaky["cases"][first]["session_cache_growth_mb"] = 999
        check(not evaluate_snapshot(config, leaky)["ok"], "session cache growth fails")

        drifted = copy.deepcopy(good)
        drifted["cases"][first]["return_rss_mb"] = 2000
        check(not evaluate_snapshot(config, drifted)["ok"], "RSS not returning to baseline fails")

        leaky_events = copy.deepcopy(good)
        leaky_events["cases"][first]["events_with_content"] = 1
        check(not evaluate_snapshot(config, leaky_events)["ok"], "content in events fails")

        bad_size = copy.deepcopy(config)
        bad_size["corpus"]["cases"][0]["size_mb"] = 101
        check(bool(validate_config(bad_size)), "out-of-policy corpus size is rejected")

        bad_ceiling = copy.deepcopy(config)
        del bad_ceiling["rss_ceilings_mb"][first]
        check(bool(validate_config(bad_ceiling)), "missing ceiling is rejected")

        from office_largefile_corpus import ACCEPTANCE_CASES, build_payload, resolve_size_mb

        check(resolve_size_mb(25) == 25.0, "generator accepts a 25 MB case")
        for size in (0, 101, float("nan")):
            rejects(f"generator rejects {size} MB", lambda size=size: resolve_size_mb(size))
        check(
            all(resolve_size_mb(size) == size for _, size, _ in ACCEPTANCE_CASES),
            "generator acceptance table matches the corpus policy",
        )

        import tempfile

        with tempfile.TemporaryDirectory(prefix="office-memory-selftest-") as folder:
            missing_path = Path(folder) / "absent.yaml"
            rejects("missing config fails closed", lambda: load_config(missing_path), (OSError, ValueError))

            seed = config["corpus"]["seed"]
            gen_a = build_payload("xlsx", 65536, seed, Path(folder) / "a.xlsx")
            gen_b = build_payload("xlsx", 65536, seed, Path(folder) / "b.xlsx")
            check(gen_a["sha256"] == gen_b["sha256"], "same seed -> identical SHA-256")

            bad_json = Path(folder) / "bad.json"
            bad_json.write_text("{not json", encoding="utf-8")
            rejects("malformed snapshot JSON rejected", lambda: load_snapshot(bad_json))
            no_cases = Path(folder) / "no-cases.json"
            no_cases.write_text('{"cases": []}', encoding="utf-8")
            rejects("snapshot without a cases mapping rejected", lambda: load_snapshot(no_cases))
            bad_case = evaluate_snapshot(config, {"cases": {first: "not-an-object"}})
            check(not bad_case["ok"] and any("no measured case data" in b["detail"] for b in bad_case["breaches"]),
                  "malformed snapshot case fails closed")
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"self-test: unexpected error: {error}", file=sys.stderr)
        return 1

    print(f"self-test: {checks}/{checks} checks passed")
    if failures:
        for label in failures:
            print(f"  FAIL {label}", file=sys.stderr)
        return 1
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--config", type=Path, default=DEFAULT_CONFIG)
    parser.add_argument("--snapshot", type=Path)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args(argv)

    if args.self_test:
        return self_test(args.config)
    try:
        config = load_config(args.config)
        problems = validate_config(config)
        if problems:
            for problem in problems:
                print(f"  PROBLEM {problem}", file=sys.stderr)
            print("RESULT: FAIL (invalid declaration)", file=sys.stderr)
            return 1
        if args.snapshot is None:
            message = (
                "Office memory gate: corpus declaration and RSS ceilings valid; "
                "runtime soak UNVERIFIED (supply --snapshot with measured RSS)"
            )
            if args.json:
                print(json.dumps({
                    "declaration": "PASS", "runtime": "UNVERIFIED",
                    "cases": sorted(config["rss_ceilings_mb"]),
                }, sort_keys=True))
            else:
                print(message)
            return 0
        report = evaluate_snapshot(config, load_snapshot(args.snapshot))
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
    if args.json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print(render(report))
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
