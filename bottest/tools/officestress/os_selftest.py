#!/usr/bin/env python3
"""Offline self-tests for the #2531 load-stress harness.

No server, no binary, no cargo, no network. Everything validated here is pure
logic, report schema parsing, the SKIP path and a small deterministic corpus
smoke test. Run via ``python3 os_measure.py --self-test``.
"""

from __future__ import annotations

import contextlib
import copy
import io
import json
import tempfile
from pathlib import Path
from typing import Any, Callable

import os_measure
import os_profiles as profiles
import os_runner as runner

FAILURES: list[str] = []
CHECKS = 0


def check(condition: bool, label: str) -> None:
    global CHECKS
    CHECKS += 1
    if not condition:
        FAILURES.append(label)


def rejects(label: str, action: Callable[[], Any], errors: tuple[type[BaseException], ...] = (ValueError,)) -> None:
    try:
        action()
    except errors:
        check(True, label)
    else:
        check(False, label)


def good_report(mode: str = "measure", profile: str = "laptop-4gb") -> dict[str, Any]:
    """Build a synthetic in-budget report (numbers here are test fixtures only)."""
    cases: list[dict[str, Any]] = []
    for case_id in profiles.CASE_SPECS:
        cases.append({
            "id": case_id,
            "baseline_rss_mb": 300.0,
            "peak_rss_mb": 320.0,
            "return_rss_mb": 310.0,
            "exit_code": 0,
            "cycles": profiles.SOAK_CYCLES if mode == "soak" else 1,
            "session_cache_growth_mb": 1.0,
            "events_with_content": 0,
            "browser_heap_mb": 120.0,
            "wire_bytes": 1024,
        })
    return {
        "schema": profiles.SCHEMA,
        "issue": profiles.ISSUE,
        "status": "MEASURED",
        "mode": mode,
        "profile": profile,
        "cases": cases,
    }


def test_profiles() -> None:
    check(len(profiles.PROFILES) == 3, "three memory profiles declared")
    check(set(profiles.PROFILES.values()) == {2048, 4096, 8192}, "profile limits are 2/4/8 GB")
    check(profiles.REFERENCE_PROFILE in profiles.PROFILES, "reference profile exists")
    check(profiles.ceiling_for("docs-large", "laptop-4gb") == 900.0, "4 GB docs ceiling")
    check(profiles.ceiling_for("docs-large", "laptop-2gb") == 450.0, "2 GB ceiling scales down")
    check(profiles.ceiling_for("docs-large", "laptop-8gb") == 1800.0, "8 GB ceiling scales up")
    rows = profiles.profile_rows(3000.0)
    check([row["name"] for row in rows] == list(profiles.profile_names()), "profile rows ordered")
    check(not rows[0]["fits_limit"] and rows[1]["fits_limit"] and rows[2]["fits_limit"],
          "peak evaluated against every profile")
    rejects("unknown case rejected", lambda: profiles.ceiling_for("nope", "laptop-4gb"))
    rejects("unknown profile rejected", lambda: profiles.ceiling_for("docs-large", "nope"))


def test_schema_and_gate() -> None:
    report = good_report()
    check(not profiles.validate_report(report), "synthetic report is valid")
    check(profiles.evaluate(report)["verdict"] == "PASS", "in-budget report passes")
    check(profiles.evaluate(report)["regression"] == "UNVERIFIED", "no baseline leaves regression UNVERIFIED")

    skip = {
        "schema": profiles.SCHEMA, "issue": profiles.ISSUE, "status": "SKIP",
        "mode": "measure", "profile": "laptop-4gb", "cases": [],
        "skip_reason": "no workload command",
    }
    check(not profiles.validate_report(skip), "SKIP report is valid")
    result = profiles.evaluate(skip)
    check(result["verdict"] == "SKIP" and not result["breaches"], "SKIP never becomes PASS")

    over = copy.deepcopy(report)
    over["cases"][0]["peak_rss_mb"] = 901.0
    check(profiles.evaluate(over)["verdict"] == "FAIL", "over-ceiling fails")
    breach = profiles.evaluate(over)["breaches"][0]["detail"]
    check("exceeds ceiling" in breach, "ceiling breach is explained")

    cgroup = copy.deepcopy(report)
    cgroup["profile"] = "laptop-2gb"
    cgroup["cases"][0]["peak_rss_mb"] = 2100.0
    check(any("cgroup limit" in item["detail"] for item in profiles.evaluate(cgroup)["breaches"]),
          "cgroup-limit breach is reported")

    regressed = copy.deepcopy(report)
    regressed["cases"][0]["peak_rss_mb"] = 400.0
    baseline = {"cases": {"docs-large": {"peak_rss_mb": 300.0}}}
    reg = profiles.evaluate(regressed, baseline)
    check(reg["verdict"] == "FAIL" and any("regressed" in item["detail"] for item in reg["breaches"]),
          "10% regression fails")

    short = good_report(mode="soak")
    short["cases"][0]["cycles"] = 3
    check(profiles.evaluate(short)["verdict"] == "FAIL", "short soak fails")
    leaky = good_report(mode="soak")
    leaky["cases"][0]["session_cache_growth_mb"] = 99.0
    check(profiles.evaluate(leaky)["verdict"] == "FAIL", "session cache growth fails")
    drifted = good_report(mode="soak")
    drifted["cases"][0]["return_rss_mb"] = 1000.0
    check(profiles.evaluate(drifted)["verdict"] == "FAIL", "RSS not returning to baseline fails")
    content = copy.deepcopy(report)
    content["cases"][0]["events_with_content"] = 1
    check(profiles.evaluate(content)["verdict"] == "FAIL", "content in events fails")
    bad_exit = copy.deepcopy(report)
    bad_exit["cases"][0]["exit_code"] = 500
    check(profiles.evaluate(bad_exit)["verdict"] == "FAIL", "nonzero workload exit fails")
    errored = copy.deepcopy(report)
    errored["cases"][0]["error"] = "binary missing"
    check(profiles.evaluate(errored)["verdict"] == "FAIL", "workload error fails")

    broken = copy.deepcopy(report)
    del broken["cases"][1]["cycles"]
    check(bool(profiles.validate_report(broken)), "missing numeric field invalidates report")
    skip["cases"] = report["cases"]
    check(bool(profiles.validate_report(skip)), "SKIP report with cases is invalid")


def test_observations() -> None:
    check(not profiles.validate_observations({"cases": {"docs-large": {"browser_heap_mb": 1.5, "wire_bytes": 2}}}),
          "valid observations accepted")
    check(bool(profiles.validate_observations({"cases": {"docs-large": {"content": "secret"}}})),
          "document content in observations rejected")
    check(bool(profiles.validate_observations({"cases": {"unknown": {"wire_bytes": 2}}})),
          "unknown case in observations rejected")
    check(bool(profiles.validate_observations({"cases": {"docs-large": {"wire_bytes": -1}}})),
          "negative wire bytes rejected")


def test_runner() -> None:
    text = "Command being timed: x\n\tMaximum resident set size (kbytes): 204800\n"
    check(runner.parse_time_stderr(text) == 200.0, "time -v peak parsed")
    check(runner.parse_time_stderr("no peak here") is None, "missing time peak is None")
    env = runner.detect_environment(None, None)
    check(not env.available() and "--workload-cmd" in env.reason(), "absent runtime yields SKIP reason")
    check(runner.detect_environment("true {file}", None).available(), "workload command enables runtime")
    check(not runner.detect_environment("true", 999999).available(), "unreadable server PID disables runtime")
    fields = runner._format_command("tool --in {file} --case {id} --mode {mode}",
                                    {"file": "/tmp/a.xlsx", "id": "sheets-large", "mode": "measure"})
    check(fields == ["tool", "--in", "/tmp/a.xlsx", "--case", "sheets-large", "--mode", "measure"],
          "workload template formats with fields")
    rejects("empty workload template rejected", lambda: runner._format_command("", {}),
            (runner.WorkloadError,))
    proc = runner.read_proc_rss(__import__("os").getpid())
    check(not proc or (proc["rss_mb"] > 0 and proc["peak_mb"] > 0), "proc RSS reader is sane")


def test_cli_skip_and_generator(tmp: Path) -> None:
    quiet = io.StringIO()
    report_path = tmp / "skip-report.json"
    args = os_measure.build_parser().parse_args(["--run", "--report", str(report_path)])
    with contextlib.redirect_stdout(quiet):
        skip_code = os_measure.run_mode(args)
    check(skip_code == 0, "SKIP run exits 0 without --require-runtime")
    stored = json.loads(report_path.read_text(encoding="utf-8"))
    check(stored["status"] == "SKIP" and stored["gate"]["verdict"] == "SKIP",
          "stored SKIP report is explicit")
    check(stored["cases"] == [] and "skip_reason" in stored, "SKIP carries no fabricated cases")
    strict = os_measure.build_parser().parse_args(
        ["--run", "--report", str(report_path), "--require-runtime"]
    )
    with contextlib.redirect_stdout(quiet):
        strict_code = os_measure.run_mode(strict)
    check(strict_code == 2, "--require-runtime turns SKIP into exit 2")

    generator = os_measure._import_generator()
    config = generator.resolve_config("xlsx", 5, generator.DEFAULT_SEED, units=64)
    entry = generator.generate(config, tmp / "corpus")
    check(entry["size_bytes"] == config.target_bytes, "generator honours target size")
    check(len(entry["sha256"]) == 64, "generator emits a SHA-256")

    measured = tmp / "measured.json"
    measured.write_text(json.dumps(good_report()), encoding="utf-8")
    gate_args = os_measure.build_parser().parse_args(["--gate", "--report", str(measured)])
    with contextlib.redirect_stdout(quiet):
        pass_code = os_measure.gate_mode(gate_args)
    check(pass_code == 0, "gate CLI passes an in-budget report")
    breach = tmp / "breach.json"
    over = good_report()
    over["cases"][0]["peak_rss_mb"] = 2000.0
    breach.write_text(json.dumps(over), encoding="utf-8")
    with contextlib.redirect_stdout(quiet):
        fail_code = os_measure.gate_mode(os_measure.build_parser().parse_args(
            ["--gate", "--report", str(breach)]))
    check(fail_code == 1, "gate CLI fails closed on a breach")


def run() -> int:
    try:
        with tempfile.TemporaryDirectory(prefix="officestress-selftest-") as folder:
            tmp = Path(folder)
            test_profiles()
            test_schema_and_gate()
            test_observations()
            test_runner()
            test_cli_skip_and_generator(tmp)
    except (OSError, ValueError, KeyError, TypeError, runner.WorkloadError) as error:
        print(f"self-test: unexpected error: {error}")
        return 1
    print(f"officestress harness self-test: {CHECKS}/{CHECKS} checks passed")
    if FAILURES:
        for label in FAILURES:
            print(f"  FAIL {label}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(run())
