#!/usr/bin/env python3
"""Office large-file load-stress measurement harness (#2531).

Builds on the deterministic corpus generator in this directory (commit
2df2ca9fc) and adds the measurement half of the lab:

* ``--run`` generates the bounded 25/25/50 MB DOCX/XLSX/PPTX corpus, drives a
  representative import/export workload and records peak RSS, then emits a JSON
  report with the 2/4/8 GB profile comparison and a PASS/FAIL gate.
* ``--run --soak --cycles N`` runs a bounded open/edit/save/close leak soak.
* ``--gate`` re-evaluates a stored report against ceilings, a 10% regression
  baseline and the soak policy. It fails closed on a breach.
* ``--self-test`` validates all of the above logic offline, with no binary.

The harness is honest about its environment: with no ``--workload-cmd`` it
emits an explicit SKIP report (exit 0, never PASS). Use ``--require-runtime``
to turn a SKIP into exit 2 on a job that genuinely needs hardware. Server RSS,
browser heap and wire bytes are separate report fields; browser figures are
only merged from a validated ``--observations`` payload (never guessed), and no
document content, tenant name or path is ever written to a report.

All outputs default to ``/tmp/opencode/officestress-measure`` — never the repo.

Exit codes: 0 PASS or SKIP, 1 FAIL (gate breach), 2 bad args / I/O / no runtime
when ``--require-runtime`` is set.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parent))

import os_profiles as profiles  # noqa: E402
import os_runner as runner  # noqa: E402

DEFAULT_OUT = Path("/tmp/opencode/officestress-measure")
DEFAULT_REPORT = DEFAULT_OUT / "report.json"


def _load_json(path: Path) -> dict[str, Any]:
    try:
        document = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError) as error:
        raise ValueError(f"cannot read {path}: {error}") from error
    except json.JSONDecodeError as error:
        raise ValueError(f"{path}: invalid JSON: {error}") from error
    if not isinstance(document, dict):
        raise ValueError(f"{path}: expected a JSON object")
    return document


def _import_generator() -> Any:
    try:
        import officestress
    except ImportError as error:  # pragma: no cover - generator ships with this file
        raise ValueError(f"cannot import the corpus generator: {error}") from error
    return officestress


def _generate_corpus(case_ids: list[str], out_dir: Path) -> dict[str, dict[str, Any]]:
    """Generate exactly the selected acceptance cases with the shipped generator."""
    generator = _import_generator()
    out_dir.mkdir(parents=True, exist_ok=True)
    corpus: dict[str, dict[str, Any]] = {}
    for case_id in case_ids:
        kind, size_mb, _ = profiles.CASE_SPECS[case_id]
        config = generator.resolve_config(kind, size_mb, generator.DEFAULT_SEED)
        entry = generator.generate(config, out_dir)
        corpus[case_id] = {"entry": entry, "path": str(out_dir / entry["file"])}
    return corpus


def _environment_dict(env: runner.Environment, args: argparse.Namespace) -> dict[str, Any]:
    return {
        "python": env.python,
        "time_binary": env.time_binary,
        "server_pid": env.server_pid,
        "proc_readable": env.proc_readable,
        "workload_configured": bool(env.workload_template),
        "cgroup_reference_mb": profiles.REFERENCE_LIMIT_MB,
        "cgroup_writable": _cgroup_writable(),
        "profile": args.profile,
        "mode": args.mode,
    }


def _cgroup_writable() -> bool:
    try:
        mount = Path("/sys/fs/cgroup")
        return mount.is_dir() and bool(os_access_writable(mount))
    except OSError:
        return False


def os_access_writable(path: Path) -> bool:
    """Best-effort writability probe that never creates a file."""
    import os

    return bool(os.access(str(path), os.W_OK))


def _skip_report(env: runner.Environment, args: argparse.Namespace) -> dict[str, Any]:
    report: dict[str, Any] = {
        "schema": profiles.SCHEMA,
        "issue": profiles.ISSUE,
        "status": "SKIP",
        "mode": args.mode,
        "profile": args.profile,
        "skip_reason": env.reason(),
        "environment": _environment_dict(env, args),
        "corpus": {},
        "profiles": [],
        "cases": [],
    }
    try:
        baseline = _load_optional(args.baseline)
    except ValueError:
        baseline = None
    report["gate"] = profiles.evaluate(report, baseline)
    return report


def _load_optional(path: str | None) -> dict[str, Any] | None:
    return _load_json(Path(path)) if path else None


def _load_observations(path: str | None) -> dict[str, dict[str, Any]]:
    if not path:
        return {}
    document = _load_json(Path(path))
    problems = profiles.validate_observations(document)
    if problems:
        raise ValueError("; ".join(problems))
    return document["cases"]


def _build_case(
    case_id: str,
    corpus: dict[str, dict[str, Any]],
    observed: dict[str, Any],
    observations: dict[str, dict[str, Any]],
    mode: str,
) -> dict[str, Any]:
    kind, size_mb, _ = profiles.CASE_SPECS[case_id]
    entry = corpus[case_id]["entry"]
    external = observations.get(case_id, {})
    case: dict[str, Any] = {
        "id": case_id,
        "kind": kind,
        "size_mb": size_mb,
        "sha256": entry["sha256"],
        "exit_code": int(observed["exit_code"]),
        "baseline_rss_mb": observed["baseline_rss_mb"],
        "peak_rss_mb": observed["peak_rss_mb"],
        "return_rss_mb": observed["return_rss_mb"],
        "cycles": int(observed.get("cycles", 1 if mode == "measure" else profiles.SOAK_CYCLES)),
        "session_cache_growth_mb": observed.get("session_cache_growth_mb", 0.0),
        "rss_source": observed.get("rss_source", "unknown"),
        "events_with_content": 0,
        "browser_heap_mb": external.get("browser_heap_mb"),
        "wire_bytes": external.get("wire_bytes"),
        "browser_heap_source": "observations" if "browser_heap_mb" in external else None,
        "wire_bytes_source": "observations" if "wire_bytes" in external else None,
    }
    if observed.get("error"):
        case["error"] = str(observed["error"])
    return case


def _write_report(report: dict[str, Any], path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def _print_result(report: dict[str, Any], as_json: bool) -> None:
    if as_json:
        print(json.dumps(report, indent=2, sort_keys=True))
        return
    gate = report["gate"]
    print(profiles.render(gate))
    if report["status"] == "SKIP":
        print(f"SKIP: {report['skip_reason']}")
    print(f"report: {report.get('report_path', '(not written)')}")


def run_mode(args: argparse.Namespace) -> int:
    """Generate the corpus, measure each case and write a gated JSON report."""
    if args.mode == "soak" and not 1 <= args.cycles <= profiles.MAX_SOAK_CYCLES:
        print(
            f"error: --cycles must be between 1 and {profiles.MAX_SOAK_CYCLES}",
            file=sys.stderr,
        )
        return 2
    env = runner.detect_environment(args.workload_cmd, args.server_pid)
    if not env.available():
        report = _skip_report(env, args)
        report["report_path"] = str(args.report)
        _write_report({k: v for k, v in report.items() if k != "report_path"}, args.report)
        _print_result(report, args.json)
        return 2 if args.require_runtime else 0

    case_ids = args.case or list(profiles.CASE_SPECS)
    try:
        observations = _load_observations(args.observations)
        baseline = _load_optional(args.baseline)
        corpus = _generate_corpus(case_ids, args.out)
    except (ValueError, OSError, KeyError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2

    cases: list[dict[str, Any]] = []
    for case_id in case_ids:
        entry = corpus[case_id]["entry"]
        fields = {
            "file": corpus[case_id]["path"],
            "id": case_id,
            "kind": profiles.CASE_SPECS[case_id][0],
            "size_mb": profiles.CASE_SPECS[case_id][1],
            "sha256": entry["sha256"],
            "mode": args.mode,
        }
        try:
            if args.mode == "soak":
                observed = runner.run_soak(
                    env, fields, args.cycles, timeout=args.timeout, prefer_time=not args.no_time
                )
            else:
                observed = runner.run_once(
                    env, fields, timeout=args.timeout, prefer_time=not args.no_time
                )
        except runner.WorkloadError as error:
            observed = {
                "exit_code": -1, "peak_rss_mb": 0.0, "baseline_rss_mb": 0.0,
                "return_rss_mb": 0.0, "cycles": 0, "error": str(error),
                "rss_source": "unavailable",
            }
        cases.append(_build_case(case_id, corpus, observed, observations, args.mode))

    peak = max((case["peak_rss_mb"] for case in cases), default=0.0)
    report: dict[str, Any] = {
        "schema": profiles.SCHEMA,
        "issue": profiles.ISSUE,
        "status": "MEASURED",
        "mode": args.mode,
        "profile": args.profile,
        "environment": _environment_dict(env, args),
        "corpus": {
            case_id: {"file": corpus[case_id]["entry"]["file"],
                      "sha256": corpus[case_id]["entry"]["sha256"]}
            for case_id in case_ids
        },
        "profiles": profiles.profile_rows(peak),
        "cases": cases,
    }
    report["gate"] = profiles.evaluate(report, baseline)
    report["report_path"] = str(args.report)
    _write_report({k: v for k, v in report.items() if k != "report_path"}, args.report)
    _print_result(report, args.json)
    return 0 if report["gate"]["verdict"] in ("PASS", "SKIP") else 1


def gate_mode(args: argparse.Namespace) -> int:
    """Re-evaluate a stored report; never modify it."""
    try:
        report = _load_json(args.report)
        baseline = _load_optional(args.baseline)
    except ValueError as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
    result = profiles.evaluate(report, baseline)
    if args.json:
        print(json.dumps(result, indent=2, sort_keys=True))
    else:
        print(profiles.render(result))
    return 0 if result["verdict"] in ("PASS", "SKIP") else 1


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument("--run", action="store_true", help="generate corpus and measure")
    action.add_argument("--gate", action="store_true", help="evaluate a stored report")
    action.add_argument("--self-test", action="store_true", help="offline logic checks")
    parser.add_argument("--report", type=Path, default=DEFAULT_REPORT)
    parser.add_argument("--out", type=Path, default=DEFAULT_OUT, help="corpus directory")
    parser.add_argument("--case", action="append", choices=list(profiles.CASE_SPECS),
                        help="acceptance case id (repeatable; default all)")
    parser.add_argument("--profile", choices=list(profiles.PROFILES), default=profiles.REFERENCE_PROFILE)
    parser.add_argument("--mode", choices=("measure", "soak"), default="measure")
    parser.add_argument("--soak", action="store_true", help="shorthand for --mode soak")
    parser.add_argument("--cycles", type=int, default=profiles.SOAK_CYCLES)
    parser.add_argument("--timeout", type=int, default=runner.DEFAULT_TIMEOUT_S)
    parser.add_argument("--workload-cmd", default=None,
                        help="shell-free command template with {file} {id} {kind} {mode} fields")
    parser.add_argument("--server-pid", type=int, default=None,
                        help="measure this server's /proc VmRSS/VmHWM instead of the workload process")
    parser.add_argument("--no-time", action="store_true", help="use wait4 even when /usr/bin/time exists")
    parser.add_argument("--observations", default=None, help="JSON with browser_heap_mb/wire_bytes per case")
    parser.add_argument("--baseline", default=None, help="prior report for the 10%% regression gate")
    parser.add_argument("--require-runtime", action="store_true", help="exit 2 on SKIP")
    parser.add_argument("--json", action="store_true", help="machine-readable output")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    if args.soak:
        args.mode = "soak"
    if args.self_test:
        import os_selftest

        return os_selftest.run()
    if args.gate:
        return gate_mode(args)
    return run_mode(args)


if __name__ == "__main__":
    sys.exit(main())
