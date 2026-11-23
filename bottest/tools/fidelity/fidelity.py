#!/usr/bin/env python3
"""DOCX/PPTX/XLSX round-trip fidelity comparison tool (#2457).

Compares two corpus directories (original vs round-tripped), extracts
structural features from each OOXML package and emits a deterministic JSON or
Markdown diff report with a pass/fail threshold. Test-only and stdlib-only: no
server, no browser, no CI job, no network.

Usage::

    python3 bottest/tools/fidelity/fidelity.py --self-test
    python3 bottest/tools/fidelity/fidelity.py \\
        --original /tmp/original --roundtrip /tmp/roundtripped \\
        --threshold 0.95 --json /tmp/fidelity.json --md /tmp/fidelity.md

Exit codes: 0 PASS (or self-test passed), 1 FAIL (below threshold or self-test
failure), 2 bad arguments / missing input directory / I/O error.
"""

from __future__ import annotations

import argparse
import sys
import tempfile
from pathlib import Path

import fx_compare as compare
import fx_extract as fx
import fx_fixtures as fixtures
import fx_report as report

SELF_TEST_DIR = "/tmp/opencode"
SELF_TEST_THRESHOLD = 0.95
SELF_TEST_TEXT_WEIGHT = 3.0


def _self_test() -> bool:
    """Run deterministic checks on synthetic fixtures; print each verdict."""
    Path(SELF_TEST_DIR).mkdir(parents=True, exist_ok=True)
    root = Path(tempfile.mkdtemp(prefix="fidelity-", dir=SELF_TEST_DIR))
    original = fixtures.write_corpus(root / "original", fixtures.ORIGINAL_SPECS)
    clean = fixtures.write_corpus(root / "clean", fixtures.ORIGINAL_SPECS)
    degraded = fixtures.write_corpus(root / "degraded", fixtures.DEGRADED_SPECS)
    fixtures.write_corpus(degraded, fixtures.EXTRA_SPECS)
    missing = fixtures.write_corpus(root / "missing", fixtures.ORIGINAL_SPECS)
    (missing / "slides_b.pptx").unlink()

    results: list[bool] = []

    def check(name: str, condition: bool, detail: str = "") -> None:
        verdict = "PASS" if condition else "FAIL"
        suffix = f" - {detail}" if detail else ""
        print(f"[{verdict}] {name}{suffix}")
        results.append(condition)

    doc_spec = fixtures.ORIGINAL_SPECS["doc_a.docx"][1]
    extracted = fx.extract(original / "doc_a.docx", "docx")
    check("extract docx counts", extracted["paragraphs"] == doc_spec["paragraphs"]
          and extracted["charts"] == doc_spec["charts"],
          f"paragraphs={extracted['paragraphs']} charts={extracted['charts']}")

    clean_report = compare.build_report(original, clean, SELF_TEST_THRESHOLD,
                                        SELF_TEST_TEXT_WEIGHT)
    clean_json = report.render_json(clean_report)
    check("identical corpus passes",
          bool(clean_report["summary"]["passed"])
          and clean_report["summary"]["overall_score"] == 1.0,
          f"score={clean_report['summary']['overall_score']}")

    again = compare.build_report(original, clean, SELF_TEST_THRESHOLD,
                                 SELF_TEST_TEXT_WEIGHT)
    check("deterministic json", clean_json == report.render_json(again))
    check("deterministic markdown",
          report.render_markdown(clean_report) == report.render_markdown(again))

    degraded_report = compare.build_report(original, degraded,
                                           SELF_TEST_THRESHOLD,
                                           SELF_TEST_TEXT_WEIGHT)
    summary = degraded_report["summary"]
    check("degraded corpus fails",
          not summary["passed"] and summary["overall_score"] < 0.95,
          f"score={summary['overall_score']}")
    statuses = {feature["status"]
                for entry in degraded_report["files"]
                for feature in entry.get("features", [])}
    check("dropped chart detected", "dropped" in statuses,
          f"statuses={sorted(statuses)}")
    check("extra file detected", summary["files_extra"] == 1)

    missing_report = compare.build_report(original, missing,
                                          SELF_TEST_THRESHOLD,
                                          SELF_TEST_TEXT_WEIGHT)
    check("missing file scored zero",
          missing_report["summary"]["files_missing"] == 1
          and any(entry["status"] == "missing"
                  for entry in missing_report["files"]))

    check("markdown verdict", "PASS" in report.render_markdown(clean_report)
          and "FAIL" in report.render_markdown(degraded_report))
    print(f"fixtures: {root}")
    return all(results)


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Compare original vs round-tripped Office corpora (#2457).")
    parser.add_argument("--original", type=Path,
                        help="directory of original DOCX/PPTX/XLSX files")
    parser.add_argument("--roundtrip", type=Path,
                        help="directory of round-tripped DOCX/PPTX/XLSX files")
    parser.add_argument("--threshold", type=float,
                        default=compare.DEFAULT_THRESHOLD,
                        help="pass threshold in [0,1] (default: 0.95)")
    parser.add_argument("--text-weight", type=float,
                        default=compare.DEFAULT_TEXT_WEIGHT,
                        help="weight of the text feature (default: 3.0)")
    parser.add_argument("--json", dest="json_path", type=Path,
                        help="write the JSON report to this path")
    parser.add_argument("--md", dest="md_path", type=Path,
                        help="write the Markdown report to this path")
    parser.add_argument("--self-test", action="store_true",
                        help="run deterministic checks on synthetic fixtures")
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = _parser()
    args = parser.parse_args(argv)
    if args.self_test:
        return 0 if _self_test() else 1

    if args.original is None or args.roundtrip is None:
        parser.error("--original and --roundtrip are required (or --self-test)")
    if not 0.0 <= args.threshold <= 1.0:
        parser.error("--threshold must be in [0,1]")
    if args.text_weight <= 0:
        parser.error("--text-weight must be positive")
    if not args.original.is_dir():
        print(f"error: original is not a directory: {args.original}",
              file=sys.stderr)
        return 2
    if not args.roundtrip.is_dir():
        print(f"error: roundtrip is not a directory: {args.roundtrip}",
              file=sys.stderr)
        return 2

    result = compare.build_report(args.original, args.roundtrip,
                                  args.threshold, args.text_weight)
    json_text = report.render_json(result)
    markdown_text = report.render_markdown(result)
    try:
        if args.json_path is not None:
            args.json_path.parent.mkdir(parents=True, exist_ok=True)
            args.json_path.write_text(json_text, encoding="utf-8")
        if args.md_path is not None:
            args.md_path.parent.mkdir(parents=True, exist_ok=True)
            args.md_path.write_text(markdown_text, encoding="utf-8")
    except OSError as exc:
        print(f"error: cannot write report: {exc}", file=sys.stderr)
        return 2
    if args.json_path is None and args.md_path is None:
        print(markdown_text, end="")
    return 0 if result["summary"]["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
