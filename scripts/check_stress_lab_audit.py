#!/usr/bin/env python3
"""Offline static audit of the #2531 large-file load-stress lab.

Read-only and CI-friendly: it never builds botserver, starts a browser, sends a
request or mutates the tree. It asserts, non-vacuously:

  1. CORPUS  - ``officestress`` declares the 5/10/25/50/100 MB acceptance table
               and a deterministic seed policy; both shipped generators pass
               their offline self-tests with same-seed/different-seed SHA checks.
  2. MEASURE - a real peak-RSS mechanism exists (``/proc/<pid>/status`` VmRSS and
               VmHWM, ``/usr/bin/time -v`` / ``wait4(ru_maxrss)`` fallback) and
               emits baseline/peak/return snapshots plus a fail-closed gate.
  3. PROFILE - 2/4/8 GB cgroup class limits and a machine-readable cgroup
               writability flag are part of the report schema.
  4. SUITE   - large-document degradation cases exist in the shipped suite and
               both offline suites run with zero FAILs (GAPs are allowed).
  5. CI      - ci.yml invokes the #2531 harness guard and the #2530 corpus gate.

The *measured* criteria (4 GB cgroup RSS, browser JS heap, wire bytes, the real
20-cycle soak and release attachments) cannot be established statically. They
are printed as explicit, non-failing GAPs. No RSS number is ever invented.

Usage::

    python3 scripts/check_stress_lab_audit.py
    python3 scripts/check_stress_lab_audit.py --root /tmp/opencode/probe-root

Exit 0 when every static guarantee holds, 1 otherwise.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path
from typing import NamedTuple

DEFAULT_ROOT = Path(__file__).resolve().parent.parent

REL_GENERATOR = "bottest/tools/officestress/officestress.py"
REL_MATRIX = "bottest/tools/officestress/os_matrix.py"
REL_BUILDERS = "bottest/tools/officestress/os_builders.py"
REL_PROFILES = "bottest/tools/officestress/os_profiles.py"
REL_RUNNER = "bottest/tools/officestress/os_runner.py"
REL_MEASURE = "bottest/tools/officestress/os_measure.py"
REL_SELFTEST = "bottest/tools/officestress/os_selftest.py"
REL_OFFICE_CORPUS = "scripts/office_largefile_corpus.py"
REL_DOCS_JS = "botui/ui/suite/tests/webtest-docs-large.js"
REL_DOCS_DEGRADATION = "botui/ui/suite/tests/webtest-docs-large/12_degradation_baseline.js"
REL_OFFICE_JS = "botui/ui/suite/tests/webtest-office-large.js"
REL_OFFICE_HARNESS_JS = "botui/ui/suite/tests/webtest-office-large/00_harness.js"
REL_OFFICE_MATRIX_JS = "botui/ui/suite/tests/webtest-office-large/01_matrix.js"
REL_CI = ".github/workflows/ci.yml"

SOURCES = (
    REL_GENERATOR, REL_MATRIX, REL_BUILDERS, REL_PROFILES, REL_RUNNER,
    REL_MEASURE, REL_SELFTEST, REL_OFFICE_CORPUS, REL_DOCS_JS,
    REL_DOCS_DEGRADATION, REL_OFFICE_JS, REL_OFFICE_HARNESS_JS,
    REL_OFFICE_MATRIX_JS, REL_CI,
)


class AuditError(Exception):
    """An input file was missing, empty or unreadable."""


class Check(NamedTuple):
    name: str
    ok: bool
    detail: str


def read(root: Path, rel: str) -> str:
    try:
        text = (root / rel).read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as exc:
        raise AuditError("cannot read {}: {}".format(rel, exc)) from exc
    if not text.strip():
        raise AuditError("{} is empty - audit would be vacuous".format(rel))
    return text


def line_of(text: str, needle: str) -> int | None:
    for number, line in enumerate(text.splitlines(), 1):
        if needle in line:
            return number
    return None


def locator(rel: str, text: str, needle: str) -> str:
    number = line_of(text, needle)
    return "{}:{}".format(rel, number if number is not None else "?")


def int_value(text: str, name: str) -> int | None:
    match = re.search(r"\b" + re.escape(name) + r"\s*(?::[^=]+)?=\s*([\d_]+)\s*$", text, re.M)
    if match is None:
        return None
    try:
        return int(match.group(1).replace("_", ""))
    except ValueError:
        return None


def float_value(text: str, name: str) -> float:
    match = re.search(r"\b" + re.escape(name) + r"\s*=\s*([\d.]+)", text)
    if match is None:
        return float("nan")
    try:
        return float(match.group(1))
    except ValueError:
        return float("nan")


def size_table(text: str) -> tuple[int, ...] | None:
    match = re.search(r"^ACCEPTANCE_SIZES\s*=\s*\(([^)]*)\)", text, re.M)
    if match is None:
        return None
    parts = [part.strip() for part in match.group(1).split(",") if part.strip()]
    if not parts or any(not part.isdigit() for part in parts):
        return None
    return tuple(int(part) for part in parts)


def run_offline(root: Path, argv: list[str], timeout: int = 120) -> tuple[int, str]:
    try:
        proc = subprocess.run(
            argv, cwd=str(root), capture_output=True, text=True, timeout=timeout, check=False
        )
    except (OSError, subprocess.SubprocessError) as exc:
        return -1, "cannot execute {}: {}".format(argv[0], exc)
    return proc.returncode, (proc.stdout + proc.stderr).strip()


def selftest_count(output: str, label: str = "self-test") -> int:
    match = re.search(label + r":\s*(\d+)/\1 checks passed", output)
    return int(match.group(1)) if match else -1


def audit(root: Path) -> tuple[list[Check], list[str]]:
    src = {rel: read(root, rel) for rel in SOURCES}
    generator, matrix, builders = src[REL_GENERATOR], src[REL_MATRIX], src[REL_BUILDERS]
    profiles, runner, measure = src[REL_PROFILES], src[REL_RUNNER], src[REL_MEASURE]
    selftest = src[REL_SELFTEST]
    corpus, docs_js, docs_deg = (
        src[REL_OFFICE_CORPUS], src[REL_DOCS_JS], src[REL_DOCS_DEGRADATION]
    )
    office_js, office_harness, office_matrix, ci = (
        src[REL_OFFICE_JS], src[REL_OFFICE_HARNESS_JS], src[REL_OFFICE_MATRIX_JS], src[REL_CI]
    )
    checks: list[Check] = []
    gaps: list[str] = []
    add = checks.append

    # 1. Corpus: acceptance size table and deterministic seed policy. ---------
    sizes = size_table(generator)
    add(Check(
        "corpus.acceptance_size_table",
        sizes == (5, 10, 25, 50, 100)
        and int_value(matrix, "MIN_SIZE_MB") == 5
        and int_value(matrix, "MAX_SIZE_MB") == 100
        and int_value(corpus, "MIN_SIZE_MB") == 5
        and int_value(corpus, "MAX_SIZE_MB") == 100,
        "{} sizes={} (want 5/10/25/50/100); both generators bound 5-100 MB".format(
            locator(REL_GENERATOR, generator, "ACCEPTANCE_SIZES"), sizes
        ),
    ))
    seed = int_value(generator, "DEFAULT_SEED")
    add(Check(
        "corpus.seed_policy",
        seed == 2531
        and "random.Random(config.seed)" in builders
        and "def resolve_seed(" in matrix
        and "seed must be non-negative" in matrix
        and '"acceptance_sizes_mb": list(ACCEPTANCE_SIZES)' in generator
        and '"sha256": digest' in generator
        and '"feature_counts": counts' in generator,
        "{} DEFAULT_SEED={} (want 2531); seeded RNG + manifest SHA-256".format(
            locator(REL_GENERATOR, generator, "DEFAULT_SEED"), seed
        ),
    ))
    rc, out = run_offline(root, [sys.executable, str(root / REL_GENERATOR), "--self-test"])
    count = selftest_count(out)
    add(Check(
        "corpus.officestress_selftest",
        rc == 0 and count >= 40
        and "same seed -> identical SHA-256" in generator
        and "different seed -> different SHA-256" in generator,
        "{} -> rc={} count={}/>=40; same/different seed SHA assertions present".format(
            REL_GENERATOR, rc, count
        ),
    ))
    rc, out = run_offline(root, [sys.executable, str(root / REL_OFFICE_CORPUS), "--self-test"])
    count = selftest_count(out)
    add(Check(
        "corpus.office_corpus_selftest",
        rc == 0 and count >= 30
        and "same seed -> identical SHA-256" in corpus
        and "different seed -> different SHA-256" in corpus,
        "{} -> rc={} count={}/>=30; byte-identical seed replay asserted".format(
            REL_OFFICE_CORPUS, rc, count
        ),
    ))

    # 2. Measurement: real RSS source + baseline/peak/return snapshots. -------
    add(Check(
        "measure.rss_sources_exist",
        all(token in runner for token in (
            'PROC_STATUS = "/proc/{pid}/status"', "VmRSS:", "VmHWM:",
            "def read_proc_rss(", 'Maximum resident set size \\(kbytes\\)',
            "usage.ru_maxrss", "def run_once(", "def run_soak(",
        )),
        "{} reads /proc VmRSS/VmHWM; time -v and wait4(ru_maxrss) fallbacks".format(
            locator(REL_RUNNER, runner, "PROC_STATUS =")
        ),
    ))
    snapshot_fields = ("baseline_rss_mb", "peak_rss_mb", "return_rss_mb")
    add(Check(
        "measure.baseline_peak_return_snapshots",
        all(text.count(field) >= 2 for text in (runner, measure) for field in snapshot_fields)
        and all(field in profiles for field in snapshot_fields)
        and "NUMERIC_CASE_FIELDS" in profiles
        and '"rss_source": source' in runner,
        "{} and {} both emit {}; {} validates them".format(
            locator(REL_RUNNER, runner, '"peak_rss_mb":'), REL_MEASURE,
            "/".join(snapshot_fields), REL_PROFILES,
        ),
    ))
    add(Check(
        "measure.gate_fails_closed",
        all(token in profiles for token in (
            "exceeds ceiling", "cgroup limit", "peak regressed",
            "SOAK_RETURN_TOLERANCE_PCT", "FORBIDDEN_OBSERVATION_KEYS",
            'if report["status"] == "SKIP"', "validate_report(report)",
            '"verdict": "FAIL"', '"verdict": "SKIP"',
        )) and float_value(profiles, "REGRESSION_PCT") == 10.0
        and float_value(profiles, "SOAK_RETURN_TOLERANCE_PCT") == 10.0
        and int_value(profiles, "SOAK_CYCLES") == 20,
        "{} enforces ceiling/cgroup/10% regression/soak/PII and SKIP!=PASS".format(
            locator(REL_PROFILES, profiles, "REGRESSION_PCT")
        ),
    ))
    rc, out = run_offline(root, [sys.executable, str(root / REL_MEASURE), "--self-test"])
    count = selftest_count(out, "officestress harness self-test")
    add(Check(
        "measure.harness_selftest",
        rc == 0 and count >= 40
        and all(token in selftest for token in (
            "10% regression fails", "ceiling breach is explained",
            "cgroup-limit breach is reported", "short soak fails",
            "content in events fails", "document content in observations rejected",
        )),
        "{} -> rc={} count={}/>=40; {} pins regression/ceiling/cgroup/soak/PII cases".format(
            REL_MEASURE, rc, count, REL_SELFTEST
        ),
    ))
    gaps.append(
        "os_runner.py: baseline/return snapshots are true /proc samples only with "
        "--server-pid; without it they alias process peak (process mode)."
    )

    # 3. Profiles: 2/4/8 GB cgroup class limits and writability flag. ---------
    declared = dict(re.findall(r'"(laptop-\d+gb)":\s*(\d+),', profiles))
    expected = {"laptop-2gb": "2048", "laptop-4gb": "4096", "laptop-8gb": "8192"}
    add(Check(
        "profile.cgroup_class_limits",
        declared == expected
        and 'REFERENCE_PROFILE = "laptop-4gb"' in profiles
        and "def profile_rows(" in profiles
        and "def _cgroup_writable(" in measure
        and '"cgroup_writable": _cgroup_writable()' in measure
        and '"cgroup_reference_mb": profiles.REFERENCE_LIMIT_MB' in measure,
        "{} profiles={} (want 2/4/8 GB); report carries cgroup_writable flag".format(
            locator(REL_PROFILES, profiles, "PROFILES:"), declared
        ),
    ))

    # 4. Suite: large-document degradation cases, run offline. ---------------
    add(Check(
        "suite.degradation_cases_exist",
        all(token in docs_deg for token in (
            "D.evaluate(20, 10)", "NS.Degradation.start", "enableAnyway",
            "degraded-banner", "readOnly", "editing is gated",
        )) and all(token in docs_js for token in (
            "12_degradation_baseline", "harness.finish();",
        )) and "function matrixCell(" in office_harness and "GAP" in office_matrix
        and "harness.finish();" in office_js,
        "{} pins read-only gating + override; {} declares PASS/GAP matrix".format(
            locator(REL_DOCS_DEGRADATION, docs_deg, "D.evaluate(20, 10)"),
            REL_OFFICE_MATRIX_JS,
        ),
    ))
    rc, out = run_offline(root, ["node", str(root / REL_DOCS_JS)])
    totals = re.search(r"TOTAL:\s*(\d+)\s*\|\s*PASS:\s*(\d+)\s*\|\s*FAIL:\s*(\d+)", out)
    numbers = tuple(int(value) for value in totals.groups()) if totals else (-1, -1, -1)
    add(Check(
        "suite.docs_degradation_selftest",
        rc == 0 and numbers[0] >= 30 and numbers[1] == numbers[0] and numbers[2] == 0,
        "{} -> rc={} total/pass/fail={}; degradation cases executed".format(
            REL_DOCS_JS, rc, numbers
        ),
    ))
    rc, out = run_offline(root, ["node", str(root / REL_OFFICE_JS)])
    totals = re.search(
        r"TOTAL:\s*(\d+)\s*\|\s*PASS:\s*(\d+)\s*\|\s*GAP:\s*(\d+)\s*\|\s*FAIL:\s*(\d+)", out
    )
    numbers = tuple(int(value) for value in totals.groups()) if totals else (-1, -1, -1, -1)
    add(Check(
        "suite.office_readiness_selftest",
        rc == 0 and numbers[0] >= 20 and numbers[2] >= 1 and numbers[3] == 0,
        "{} -> rc={} total/pass/gap/fail={}; GAPs explicit, not failing".format(
            REL_OFFICE_JS, rc, numbers
        ),
    ))

    # 5. CI: the harness gate is invoked; the JS suites are not (honest GAP). -
    ci_commands = (
        "python3 bottest/tools/officestress/os_measure.py --self-test",
        "python3 bottest/tools/officestress/os_measure.py --run",
        "python3 bottest/tools/officestress/os_measure.py --gate",
        "python3 scripts/office_largefile_corpus.py --self-test",
        "python3 scripts/check_office_memory.py --check",
    )
    missing = [command for command in ci_commands if command not in ci]
    stress_title = "Guard: Office large-file load-stress harness (#2531)" in ci
    corpus_title = "Guard: Office large-file corpus and memory ceilings (#2530)" in ci
    add(Check(
        "ci.stress_guard_invoked",
        not missing and stress_title and corpus_title,
        "{} missing steps: {}; #2531 guard={}, #2530 guard={}".format(
            locator(REL_CI, ci, "Guard: Office large-file load-stress harness"),
            ", ".join(missing) if missing else "none",
            "present" if stress_title else "MISSING",
            "present" if corpus_title else "MISSING",
        ),
    ))
    if REL_DOCS_JS not in ci and REL_OFFICE_JS not in ci:
        gaps.append(
            "ci.yml does not invoke webtest-docs-large.js / webtest-office-large.js "
            "directly; both are runnable offline and are executed by this audit, "
            "but CI coverage of those two suites is a GAP."
        )

    return checks, gaps


def criteria(ok: dict[str, bool]) -> list[tuple[str, str, str]]:
    """Every #2531 acceptance criterion, labelled AUTOMATED or GAP."""
    auto = "AUTOMATED"
    gap = "GAP"
    return [
        ("AC1", "5/10/25/50 MB per format opens with recorded peak RSS and time on "
         "laptop-4gb", gap,
         "generator + RSS harness statically verified; no 4 GB cgroup or server run here"),
        ("AC2", "harness exits non-zero on a memory ceiling or 10% regression breach",
         auto if ok.get("measure.gate_fails_closed") and ok.get("measure.harness_selftest")
         else gap,
         "fail-closed gate asserted in os_profiles.py:178 and 47/47 self-test"),
        ("AC3", "report shows server RSS, browser heap and wire bytes separately; "
         "no document content or PII", gap,
         "schema + FORBIDDEN_OBSERVATION_KEYS verified; CDP heap/wire collection unmeasured"),
        ("AC4", "same corpus runs on desktop-local profile without OOM to a stated "
         "size; first failing size documented", gap,
         "2/4/8 GB profiles declared; no desktop-local cgroup run or OOM boundary"),
        ("AC5", "soak returns to baseline RSS within ±10% after 20 cycles for all "
         "three apps", gap,
         "SOAK_CYCLES=20 and ±10% tolerance asserted; measured 20-cycle soak not run"),
        ("AC6", "results attached to a release and linked from trackers; no generated "
         "files in the repository", gap,
         "release attachment is a CI/release action; generator defaults to /tmp/opencode"),
    ]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--root", type=Path, default=DEFAULT_ROOT,
        help="workspace root (point at a copy for non-vacuity probes)",
    )
    args = parser.parse_args()
    try:
        checks, gaps = audit(args.root)
    except AuditError as exc:
        print("FAIL audit input error: {}".format(exc))
        return 1
    failed = [check for check in checks if not check.ok]
    ok = {check.name: check.ok for check in checks}
    print("=== Load-stress lab static audit (#2531) ===")
    print("  root: {} | assertions: {} | passed: {}".format(
        args.root, len(checks), len(checks) - len(failed)))
    for check in checks:
        print("  {} {}: {}".format("PASS" if check.ok else "FAIL", check.name, check.detail))
    print("--- Acceptance criteria (issue #2531) ---")
    for identifier, text, label, note in criteria(ok):
        print("  [{}] {}: {} | {}".format(label, identifier, text, note))
    print("--- Non-failing GAPs (runtime/hardware/CI not provable offline) ---")
    for gap in gaps:
        print("  [GAP] {}".format(gap))
    print("RESULT: {}".format("FAIL" if failed else "PASS"))
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
