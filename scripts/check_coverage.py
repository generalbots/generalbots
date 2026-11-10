#!/usr/bin/env python3
"""Per-crate coverage gate (#2074).

`cargo-llvm-cov` measures one crate at a time (the workspace graph exceeds the
memory budget in one run), so the numbers are compared here instead of by the
tool: this script reads the JSON export, drops the paths listed in
`coverage-exclusions.toml`, and compares the remaining totals with the per-crate
floors in the same file.

Exclusion rules (enforced, not advisory)
---------------------------------------
* every entry needs `path`, `reason` and `issue`;
* a `reason` must actually explain the exclusion (>= 20 characters);
* the `path` must point inside a crate's `src/` tree, so a whole crate cannot
  be excluded;
* a crate that carries a floor must keep at least one measured file — an
  exclusion set that erases the crate fails instead of reporting 100%.

Usage
-----
    python3 scripts/check_coverage.py --list-crates
    python3 scripts/check_coverage.py --crate botlib --json coverage/botlib.json
    python3 scripts/check_coverage.py --self-test

Exit codes: 0 = at/above every enforced floor, 1 = below a floor or an invalid
exclusion, 2 = usage/config error.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_CONFIG = "coverage-exclusions.toml"

METRICS = ("lines", "functions", "branches")
MIN_REASON_CHARS = 20


class ConfigError(Exception):
    """The exclusions/floors file cannot be honoured."""


def load_config(path: Path) -> dict:
    if not path.is_file():
        raise ConfigError(f"{path} is missing")
    try:
        import tomllib
    except ModuleNotFoundError as error:  # pragma: no cover - python < 3.11
        raise ConfigError("python 3.11+ is required (tomllib)") from error
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError as error:
        raise ConfigError(f"{path}: {error}") from error


def glob_to_regex(pattern: str) -> re.Pattern[str]:
    """`*` stays inside one path segment, `**` crosses segments.

    A trailing `/` means "this directory and everything under it".
    """
    if pattern.endswith("/"):
        pattern += "**"
    out = []
    index = 0
    while index < len(pattern):
        char = pattern[index]
        if char == "*":
            if pattern[index : index + 2] == "**":
                out.append(".*")
                index += 2
                continue
            out.append("[^/]*")
        else:
            out.append(re.escape(char))
        index += 1
    return re.compile("(?:^|/)" + "".join(out) + "$")


def validate(config: dict, floors: dict[str, dict]) -> list[tuple[str, str, int]]:
    """Returns the parsed `(path, reason, issue)` exclusions."""
    exclusions = []
    for index, entry in enumerate(config.get("exclusion", []), start=1):
        where = f"[[exclusion]] #{index}"
        missing = [key for key in ("path", "reason", "issue") if not entry.get(key)]
        if missing:
            raise ConfigError(f"{where} is missing {', '.join(missing)}")
        path = str(entry["path"]).strip()
        reason = str(entry["reason"]).strip()
        if len(reason) < MIN_REASON_CHARS:
            raise ConfigError(
                f"{where} ({path}) needs a real reason (>= {MIN_REASON_CHARS} chars)"
            )
        if "/src/" not in path:
            raise ConfigError(
                f"{where} ({path}) must point inside a crate's src/ tree; a whole "
                "crate cannot be excluded"
            )
        for crate in floors:
            if path.rstrip("/") in (f"{crate}/src", crate):
                raise ConfigError(
                    f"{where} ({path}) would exclude the whole `{crate}` crate, "
                    "which carries a coverage floor"
                )
        exclusions.append((path, reason, int(entry["issue"])))
    return exclusions


def parse_floors(config: dict) -> dict[str, dict]:
    floors = config.get("floors", {})
    if not floors:
        raise ConfigError("no [floors] table: every measured crate needs a floor")
    parsed = {}
    for crate, entry in floors.items():
        values = {}
        for metric in METRICS:
            value = entry.get(metric)
            if not isinstance(value, (int, float)) or not 0 <= value <= 100:
                raise ConfigError(
                    f"[floors.{crate}] {metric} must be a percentage in 0..=100"
                )
            values[metric] = float(value)
        values["enforce"] = bool(entry.get("enforce", False))
        parsed[crate] = values
    return parsed


def summarize(report: dict, exclusions: list[tuple[str, str, int]]) -> dict:
    data = report.get("data") or []
    if not data:
        raise ConfigError("the coverage report has no `data` section")
    files = data[0].get("files") or []
    patterns = [glob_to_regex(path) for path, _, _ in exclusions]

    totals = {metric: {"count": 0, "covered": 0} for metric in METRICS}
    measured = 0
    excluded = 0
    for entry in files:
        filename = entry.get("filename", "")
        if any(pattern.search(filename) for pattern in patterns):
            excluded += 1
            continue
        summary = entry.get("summary") or {}
        measured += 1
        for metric in METRICS:
            bucket = summary.get(metric) or {}
            totals[metric]["count"] += int(bucket.get("count", 0))
            totals[metric]["covered"] += int(bucket.get("covered", 0))

    percentages = {}
    for metric in METRICS:
        count = totals[metric]["count"]
        percentages[metric] = (
            round(100.0 * totals[metric]["covered"] / count, 2) if count else None
        )
    return {"measured_files": measured, "excluded_files": excluded, "percent": percentages}


def format_row(crate: str, result: dict, floor: dict) -> str:
    cells = []
    for metric in METRICS:
        value = result["percent"][metric]
        shown = "n/a" if value is None else f"{value:.2f}%"
        cells.append(f"{metric} {shown} (floor {floor[metric]:.1f}%)")
    mode = "gate" if floor["enforce"] else "report-only"
    return f"| `{crate}` | " + " | ".join(cells) + f" | {result['measured_files']} | {mode} |"


def check(crate: str, report_path: Path, config: dict) -> int:
    floors = parse_floors(config)
    exclusions = validate(config, floors)
    if crate not in floors:
        raise ConfigError(f"`{crate}` has no [floors.{crate}] entry")
    floor = floors[crate]

    try:
        report = json.loads(report_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ConfigError(f"{report_path}: {error}") from error

    result = summarize(report, exclusions)
    if result["measured_files"] == 0:
        raise ConfigError(
            f"every file of `{crate}` is excluded; a floored crate must keep code "
            "to measure"
        )

    line = format_row(crate, result, floor)
    print(line)
    summary_file = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary_file:
        with open(summary_file, "a", encoding="utf-8") as handle:
            if handle.tell() == 0:
                handle.write("| crate | metrics | measured files | mode |\n")
                handle.write("|---|---|---|---|\n")
            handle.write(line + "\n")

    failures = []
    for metric in METRICS:
        value = result["percent"][metric]
        if value is None:
            continue
        if value < floor[metric]:
            failures.append(
                f"{metric} {value:.2f}% is below the {floor[metric]:.1f}% floor"
            )
    if failures:
        message = f"{crate}: " + "; ".join(failures)
        if floor["enforce"]:
            print(f"error: {message}")
            return 1
        print(f"warning (report-only): {message}")
    return 0


def list_crates(config: dict) -> int:
    floors = parse_floors(config)
    validate(config, floors)
    print(f"crates={json.dumps(list(floors))}")
    return 0


def self_test() -> int:
    """Proves the gate fails below a floor and honours exclusions."""
    failures = 0

    def expect(condition: bool, label: str) -> None:
        nonlocal failures
        if not condition:
            failures += 1
            print(f"self-test FAILED: {label}")

    def report(lines_covered: int, lines_total: int, extra: str = "") -> dict:
        files = [
            {
                "filename": f"{Path.cwd()}/botlib/src/lib.rs",
                "summary": {
                    "lines": {"count": lines_total, "covered": lines_covered},
                    "functions": {"count": 10, "covered": 10},
                    "branches": {"count": 10, "covered": 10},
                },
            }
        ]
        if extra:
            files.append(
                {
                    "filename": f"{Path.cwd()}/{extra}",
                    "summary": {
                        "lines": {"count": 100, "covered": 0},
                        "functions": {"count": 10, "covered": 0},
                        "branches": {"count": 10, "covered": 0},
                    },
                }
            )
        return {"data": [{"files": files}]}

    config = {
        "exclusion": [
            {
                "path": "bottest/src/web/",
                "reason": "browser automation that needs a live Chrome",
                "issue": 2074,
            }
        ],
        "floors": {"botlib": {"lines": 50, "functions": 50, "branches": 50, "enforce": True}},
    }

    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        good = tmp_path / "good.json"
        good.write_text(json.dumps(report(80, 100)), encoding="utf-8")
        bad = tmp_path / "bad.json"
        bad.write_text(json.dumps(report(20, 100)), encoding="utf-8")
        excluded = tmp_path / "excluded.json"
        excluded.write_text(json.dumps(report(80, 100, "bottest/src/web/browser.rs")), encoding="utf-8")

        expect(check("botlib", good, config) == 0, "coverage above the floor passes")
        expect(check("botlib", bad, config) == 1, "coverage below an enforced floor fails")
        expect(check("botlib", excluded, config) == 0, "excluded paths are dropped")

        no_reason = {"exclusion": [{"path": "bottest/src/web/", "issue": 2074}], "floors": config["floors"]}
        try:
            validate(no_reason, parse_floors(no_reason))
            expect(False, "an exclusion without a reason is rejected")
        except ConfigError:
            expect(True, "an exclusion without a reason is rejected")

        whole_crate = {
            "exclusion": [
                {
                    "path": "botlib/src",
                    "reason": "not really a reason but long enough to pass",
                    "issue": 2074,
                }
            ],
            "floors": config["floors"],
        }
        try:
            validate(whole_crate, parse_floors(whole_crate))
            expect(False, "a whole-crate exclusion is rejected")
        except ConfigError:
            expect(True, "a whole-crate exclusion is rejected")

        report_only = {
            "floors": {"botlib": {"lines": 50, "functions": 50, "branches": 50}}
        }
        expect(
            check("botlib", bad, report_only) == 0,
            "report-only mode never fails the job",
        )

    if failures:
        print(f"self-test: {failures} failure(s)")
        return 1
    print("self-test: ok")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Per-crate coverage gate (#2074)")
    parser.add_argument("--config", default=DEFAULT_CONFIG, help="coverage-exclusions.toml")
    parser.add_argument("--list-crates", action="store_true", help="matrix crates, GitHub-output form")
    parser.add_argument("--crate", help="crate whose floor to enforce")
    parser.add_argument("--json", help="cargo-llvm-cov JSON export for that crate")
    parser.add_argument("--self-test", action="store_true", help="prove the gate logic")
    args = parser.parse_args()

    if args.self_test:
        return self_test()
    try:
        config = load_config(ROOT / args.config if not Path(args.config).is_absolute() else Path(args.config))
        if args.list_crates:
            return list_crates(config)
        if not args.crate or not args.json:
            parser.error("--crate and --json are required (or use --list-crates)")
        return check(args.crate, Path(args.json), config)
    except ConfigError as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
