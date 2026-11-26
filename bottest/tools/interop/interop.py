#!/usr/bin/env python3
"""``interop`` — deterministic Office conversion-report generator + gate (#2463).

Offline, stdlib-only companion to the fidelity/officestress tools. Given a
source Office file (DOCX/PPTX/XLSX) and its GB-converted output, it emits a
deterministic JSON report of structural losses (headings, tables, images,
formulas, fonts) plus a machine-readable metric-compatible font-fallback
decision for every source font. The gate FAILS when a required feature is
silently dropped; a documented font substitution with its fallback present in
the converted output is a visible WARN, not a silent drop.

This slice deliberately does NOT implement WOPI/Office Online, Google import,
embedded-object round-trip or any live-server connector -- those require a
running server and are out of scope here (see #2463).

Usage::

    python3 bottest/tools/interop/interop.py --self-test
    python3 bottest/tools/interop/interop.py \\
        --source /tmp/source.docx --converted /tmp/converted.docx \\
        --json /tmp/opencode/conversion-report.json
    python3 bottest/tools/interop/interop.py \\
        --validate /tmp/opencode/conversion-report.json

Exit codes: 0 PASS / self-test ok, 1 gate FAIL, 2 bad args, bad input or I/O.
"""

from __future__ import annotations

import argparse
import ast
import json
import sys
import tempfile
from pathlib import Path
from typing import Any

import ix_extract
import ix_fixtures
import ix_report

SELF_TEST_DIR = "/tmp/opencode"
MODULES = ("interop.py", "ix_extract.py", "ix_fonts.py",
           "ix_fixtures.py", "ix_report.py")
STDLIB_ROOTS = {
    "__future__", "argparse", "ast", "fnmatch", "io", "json", "pathlib",
    "sys", "tempfile", "typing", "xml", "zipfile",
}


def _print_summary(report: dict[str, Any]) -> None:
    """Print a compact, deterministic verdict summary."""
    gate = report["gate"]
    losses = report["losses"]
    dropped = sum(1 for item in losses
                  if item.get("type") == "count" and item.get("severity") == "dropped")
    partial = sum(1 for item in losses
                  if item.get("type") == "count" and item.get("severity") == "partial")
    fonts = sum(1 for item in losses if item.get("type") == "font")
    verdict = "PASS" if gate["passed"] else "FAIL"
    print(f"source: {report['source']['name']} ({report['source']['kind']})")
    print(f"converted: {report['converted']['name']} ({report['converted']['kind']})")
    print(f"losses: {len(losses)} (dropped={dropped}, partial={partial}, "
          f"missing_fonts={fonts})")
    print(f"font_decisions: {len(report['font_decisions'])}")
    print(f"gate: {verdict} (failures={len(gate['failures'])}, "
          f"warnings={len(gate['warnings'])})")
    for failure in gate["failures"]:
        print(f"  FAIL {failure['feature']}: {failure['reason']}")
    for warning in gate["warnings"]:
        print(f"  WARN {warning['feature']}: {warning['reason']}")


def _self_test() -> bool:
    """Run deterministic checks on synthetic OOXML fixtures; print verdicts."""
    Path(SELF_TEST_DIR).mkdir(parents=True, exist_ok=True)
    root = Path(tempfile.mkdtemp(prefix="interop-", dir=SELF_TEST_DIR))
    results: list[bool] = []

    def check(name: str, condition: bool, detail: str = "") -> None:
        verdict = "PASS" if condition else "FAIL"
        suffix = f" - {detail}" if detail else ""
        print(f"[{verdict}] {name}{suffix}")
        results.append(condition)

    docx = root / "source.docx"
    docx.write_bytes(ix_fixtures.build_docx(
        3, 2, 1, 4, ["Calibri", "Arial"]))
    pptx = root / "source.pptx"
    pptx.write_bytes(ix_fixtures.build_pptx(
        1, 2, 3, 1, ["Calibri"]))
    xlsx = root / "source.xlsx"
    xlsx.write_bytes(ix_fixtures.build_xlsx(5, 2, 1, ["Calibri", "Arial"]))

    doc_expected = ix_fixtures.expected("docx", 3, 2, 1, 4,
                                        ["Calibri", "Arial"])
    check("extract docx feature vector",
          ix_extract.extract(docx) == doc_expected,
          str(ix_extract.extract(docx)["features"]))
    check("extract pptx feature vector",
          ix_extract.extract(pptx) == ix_fixtures.expected(
              "pptx", 1, 2, 3, 1, ["Calibri"]))
    check("extract xlsx feature vector",
          ix_extract.extract(xlsx) == ix_fixtures.expected(
              "xlsx", 0, 2, 1, 5, ["Calibri", "Arial"]))
    check("fixtures are deterministic",
          ix_fixtures.build_docx(3, 2, 1, 4, ["Calibri", "Arial"]) == docx.read_bytes())

    identical = ix_report.build_report(
        ix_extract.extract(docx), ix_extract.extract(docx),
        "source.docx", "converted.docx", True)
    check("identical conversion passes",
          identical["gate"]["passed"] and not identical["losses"]
          and not identical["gate"]["failures"])

    stripped = root / "stripped.docx"
    stripped.write_bytes(ix_fixtures.build_docx(0, 0, 0, 0, ["Calibri", "Arial"]))
    dropped = ix_report.build_report(
        ix_extract.extract(docx), ix_extract.extract(stripped),
        "source.docx", "stripped.docx", True)
    check("dropped count features fail gate",
          not dropped["gate"]["passed"]
          and len(dropped["gate"]["failures"]) == 4,
          f"failures={len(dropped['gate']['failures'])}")
    check("losses record dropped severity",
          {entry["severity"] for entry in dropped["losses"]
           if entry["type"] == "count"} == {"dropped"})

    partial_doc = root / "partial.docx"
    partial_doc.write_bytes(ix_fixtures.build_docx(1, 2, 1, 4, ["Calibri", "Arial"]))
    partial = ix_report.build_report(
        ix_extract.extract(docx), ix_extract.extract(partial_doc),
        "source.docx", "partial.docx", True)
    check("partial loss warns but passes",
          partial["gate"]["passed"]
          and any(item["reason"] == "partial-loss"
                  for item in partial["gate"]["warnings"]))

    calbri = root / "calibri.docx"
    calbri.write_bytes(ix_fixtures.build_docx(3, 2, 1, 4, ["Calibri"]))
    carlito = root / "carlito.docx"
    carlito.write_bytes(ix_fixtures.build_docx(3, 2, 1, 4, ["Carlito"]))
    substituted = ix_report.build_report(
        ix_extract.extract(calbri), ix_extract.extract(carlito),
        "calibri.docx", "carlito.docx", True)
    decision = substituted["font_decisions"][0]
    check("metric-compatible fallback decision",
          decision["status"] == "substituted"
          and decision["fallback"] == "Carlito"
          and decision["strategy"] == "metric-compatible"
          and decision["compatible"] is True
          and decision["warning"] is True,
          str(decision))
    check("documented substitution is a WARN not FAIL",
          substituted["gate"]["passed"]
          and any(item["reason"] == "documented-substitution"
                  for item in substituted["gate"]["warnings"]))

    wingdings = root / "wingdings.docx"
    wingdings.write_bytes(ix_fixtures.build_docx(3, 2, 1, 4, ["Wingdings"]))
    none_fonts = root / "none.docx"
    none_fonts.write_bytes(ix_fixtures.build_docx(3, 2, 1, 4, []))
    unresolved = ix_report.build_report(
        ix_extract.extract(wingdings), ix_extract.extract(none_fonts),
        "wingdings.docx", "none.docx", True)
    check("unresolved font drop fails gate",
          not unresolved["gate"]["passed"]
          and any(item["reason"] == "silent-drop"
                  for item in unresolved["gate"]["failures"]))

    verdana = root / "verdana.docx"
    verdana.write_bytes(ix_fixtures.build_docx(3, 2, 1, 4, ["Verdana"]))
    dejavu = root / "dejavu.docx"
    dejavu.write_bytes(ix_fixtures.build_docx(3, 2, 1, 4, ["DejaVu Sans"]))
    approx = ix_report.build_report(
        ix_extract.extract(verdana), ix_extract.extract(dejavu),
        "verdana.docx", "dejavu.docx", True)
    check("approximate fallback is documented WARN",
          approx["font_decisions"][0]["strategy"] == "approximate"
          and approx["font_decisions"][0]["compatible"] is False
          and approx["gate"]["passed"])

    again = ix_report.build_report(
        ix_extract.extract(docx), ix_extract.extract(docx),
        "source.docx", "converted.docx", True)
    check("report json is deterministic",
          ix_report.render_json(identical) == ix_report.render_json(again))

    ok_report = ix_report.validate(identical)
    ok_drop = ix_report.validate(dropped)
    check("validator accepts a valid report",
          ok_report[0] and ok_report[2]["passed"]
          and ix_report.validate(dropped)[0])
    tampered = json.loads(ix_report.render_json(identical))
    tampered["gate"]["passed"] = not tampered["gate"]["passed"]
    check("validator rejects tampered gate",
          not ix_report.validate(tampered)[0]
          and bool(ix_report.validate(tampered)[1]))
    bad = json.loads(ix_report.render_json(identical))
    bad["schema"] = "other/9"
    check("validator rejects bad schema", not ix_report.validate(bad)[0])

    check("stdlib-only imports", _stdlib_only())
    print(f"fixtures: {root}")
    return all(results)


def _stdlib_only() -> bool:
    """Verify every module imports only stdlib or sibling ix_* modules."""
    here = Path(__file__).resolve().parent
    for filename in MODULES:
        tree = ast.parse((here / filename).read_text(encoding="utf-8"))
        for node in ast.walk(tree):
            names: list[str] = []
            if isinstance(node, ast.Import):
                names = [alias.name.split(".", 1)[0] for alias in node.names]
            elif isinstance(node, ast.ImportFrom) and node.module:
                names = [node.module.split(".", 1)[0]]
            for name in names:
                if name in STDLIB_ROOTS or name.startswith("ix_"):
                    continue
                print(f"  non-stdlib import in {filename}: {name}")
                return False
    return True


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Generate/validate an Office conversion report (#2463).")
    parser.add_argument("--source", type=Path,
                        help="source OOXML file (DOCX/PPTX/XLSX)")
    parser.add_argument("--converted", type=Path,
                        help="GB-converted OOXML file of the same kind")
    parser.add_argument("--json", dest="json_path", type=Path,
                        help="write the deterministic report to this path")
    parser.add_argument("--validate", type=Path, metavar="REPORT",
                        help="validate a stored report and re-run its gate")
    parser.add_argument("--self-test", action="store_true",
                        help="run deterministic checks on synthetic fixtures")
    return parser


def _load_report(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise ValueError(f"cannot read report {path}: {exc}") from exc


def main(argv: list[str] | None = None) -> int:
    parser = _parser()
    args = parser.parse_args(argv)
    if args.self_test:
        return 0 if _self_test() else 1

    if args.validate is not None:
        try:
            report = _load_report(args.validate)
        except ValueError as exc:
            print(f"error: {exc}", file=sys.stderr)
            return 2
        shape_ok, errors, gate = ix_report.validate(report)
        if not shape_ok:
            for error in errors:
                print(f"error: {error}", file=sys.stderr)
            return 2
        assert gate is not None
        verdict = "PASS" if gate["passed"] else "FAIL"
        print(f"validation: OK; gate: {verdict} "
              f"(failures={len(gate['failures'])}, "
              f"warnings={len(gate['warnings'])})")
        return 0 if gate["passed"] else 1

    if args.source is None or args.converted is None:
        parser.error("--source and --converted are required "
                     "(or --validate / --self-test)")
    try:
        source_extract = ix_extract.extract(args.source)
        converted_extract = ix_extract.extract(args.converted)
    except ix_extract.ExtractError as exc:
        print(f"error: cannot extract features: {exc.code}", file=sys.stderr)
        return 2
    if source_extract["kind"] != converted_extract["kind"]:
        print("error: source and converted kinds differ "
              f"({source_extract['kind']} vs {converted_extract['kind']})",
              file=sys.stderr)
        return 2

    report = ix_report.build_report(
        source_extract, converted_extract,
        args.source.name, args.converted.name, False)
    text = ix_report.render_json(report)
    if args.json_path is not None:
        try:
            args.json_path.parent.mkdir(parents=True, exist_ok=True)
            args.json_path.write_text(text, encoding="utf-8")
        except OSError as exc:
            print(f"error: cannot write report: {exc}", file=sys.stderr)
            return 2
        _print_summary(report)
    else:
        print(text, end="")
    return 0 if report["gate"]["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
