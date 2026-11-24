#!/usr/bin/env python3
"""Offline static audit of the office-interop / ODF import paths (#2463).

This is a *read-only* guard: it never builds botserver, never starts a browser
and never mutates the tree. It proves four things about the checked-out source:

  1. ROUTING   - ODF-family packages (ODS, and ODT as a refused ODF package)
                 are dispatched to the hardened ODF parser, never to the OOXML
                 (XLSX) one. The discriminator strings live in
                 ``storage/import/mod.rs`` (``detect_spreadsheet_format``) and
                 the drive-open override lives in ``storage/import/xlsx.rs``.
  2. MALWARE   - a failed or unavailable malware scan produces an explicit,
                 typed error (``error-file-upload-failed`` /
                 "the malware scan could not be completed") and the gate runs
                 *before* the parser, so an unscanned package never reaches it.
  3. TESTS     - the 7 ODF regression tests added with the hardened parser are
                 still present by name in ``storage/import/odf/tests.rs``.
  4. UI        - the interop-facing UI files (Sheets import page + office
                 fidelity/conformance harness) contain no CDN/unpkg reference.

Every assertion is non-vacuous: required files must exist and be non-empty,
required markers must be found at the expected place, and the malware ordering
is checked on source line numbers. Point ``--root`` at a modified copy of the
tree to verify that a single deliberate break produces a named FAIL.

Exit 0 on a clean audit, 1 on any violation.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import NamedTuple

DEFAULT_ROOT = Path(__file__).resolve().parent.parent

REL_ROUTING = "botserver/crates/botsheet/src/storage/import/mod.rs"
REL_XLSX = "botserver/crates/botsheet/src/storage/import/xlsx.rs"
REL_ODF = "botserver/crates/botsheet/src/storage/import/odf.rs"
REL_ODF_TESTS = "botserver/crates/botsheet/src/storage/import/odf/tests.rs"
# The brief mentioned botsheet/src/handlers/upload_hostile.rs; the gate actually
# lives in botsecurity-core. Audit the real source-of-truth paths.
REL_HOSTILE = "botserver/crates/botsecurity-core/src/upload_hostile.rs"
REL_UPLOAD_ERROR = "botserver/crates/botsecurity-core/src/upload_error.rs"
REL_IMPORT_HANDLER = "botserver/crates/botsheet/src/handlers/export_import.rs"

REL_SHEET_UI = "botui/ui/suite/sheet/sheet.html"
REL_FIDELITY_JS = "botui/ui/suite/tests/webtest-office-fidelity.js"
REL_FIDELITY_DIR = "botui/ui/suite/tests/webtest-office-fidelity"

ODF_REGRESSION_TESTS = (
    "refuses_a_package_with_too_many_entries",
    "refuses_a_package_exceeding_the_uncompressed_size_limit",
    "refuses_a_content_xml_exceeding_the_size_limit",
    "refuses_an_overlong_mimetype",
    "refuses_too_many_worksheets",
    "clamps_cell_text_to_the_text_limit",
    "refuses_a_self_closing_embedded_object_element",
)
MIN_TESTS = 5  # brief requires at least 5; all 7 are asserted below

# Only external *asset* hosts are forbidden. XML namespace URIs such as
# http://schemas.openxmlformats.org/... in the fidelity corpus are data, not
# CDN references, and the harness' own guard regex mentions the words.
CDN_HOST = re.compile(
    r"(?i:(?:https?:)?//[^/\s\"'()<>]+"
    r"(?:unpkg|jsdelivr|cdnjs|cloudflare|googleapis|jquery|bootstrapcdn"
    r"|esm\.sh|skypack|bootcdn|cdn)\b)"
    r"|(?i:\b(?:unpkg\.com|cdn\.jsdelivr\.net|cdnjs\.cloudflare\.com"
    r"|ajax\.googleapis\.com|code\.jquery\.com|cdn\.bootstrapcdn\.com"
    r"|esm\.sh|cdn\.skypack\.dev)\b)"
)


class AuditError(Exception):
    """An input file was missing, unreadable or structurally unexpected."""


class Check(NamedTuple):
    name: str
    ok: bool
    detail: str


def read(root: Path, rel: str) -> str:
    path = root / rel
    try:
        text = path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as exc:
        raise AuditError(f"cannot read {rel}: {exc}") from exc
    if not text.strip():
        raise AuditError(f"{rel} is empty - audit would be vacuous")
    return text


def line_of(text: str, needle: str) -> int | None:
    for number, line in enumerate(text.splitlines(), 1):
        if needle in line:
            return number
    return None


def masked(text: str) -> str:
    """Blank // and /* */ comments while preserving offsets and length."""
    out = list(text)
    i, n = 0, len(text)
    while i < n:
        if text[i] == '"':
            i += 1
            while i < n:
                if text[i] == "\\":
                    i += 2
                elif text[i] == '"':
                    i += 1
                    break
                else:
                    i += 1
            continue
        if text.startswith("//", i):
            while i < n and text[i] != "\n":
                out[i] = " "
                i += 1
            continue
        if text.startswith("/*", i):
            out[i] = out[i + 1] = " "
            i += 2
            depth = 1
            while i < n and depth:
                if text.startswith("/*", i):
                    out[i] = out[i + 1] = " "
                    i += 2
                    depth += 1
                elif text.startswith("*/", i):
                    out[i] = out[i + 1] = " "
                    i += 2
                    depth -= 1
                else:
                    if text[i] != "\n":
                        out[i] = " "
                    i += 1
            continue
        i += 1
    return "".join(out)


def function_body(text: str, signature: str) -> str:
    """Return the brace-balanced body of the fn whose signature contains it."""
    src = masked(text)
    at = src.find(signature)
    if at < 0:
        raise AuditError(f"function not found: {signature}")
    open_at = src.find("{", at)
    if open_at < 0:
        raise AuditError(f"function has no body: {signature}")
    depth, i, n = 0, open_at, len(src)
    while i < n:
        if src[i] == '"':
            i += 1
            while i < n:
                if src[i] == "\\":
                    i += 2
                elif src[i] == '"':
                    i += 1
                    break
                else:
                    i += 1
            continue
        if src[i] == "{":
            depth += 1
        elif src[i] == "}":
            depth -= 1
            if depth == 0:
                return src[open_at : i + 1]
        i += 1
    raise AuditError(f"unbalanced function body: {signature}")


def has(text: str, needle: str) -> bool:
    return needle in text


def audit(root: Path) -> list[Check]:
    checks: list[Check] = []
    add = checks.append

    routing = read(root, REL_ROUTING)
    xlsx = read(root, REL_XLSX)
    odf = read(root, REL_ODF)
    tests = read(root, REL_ODF_TESTS)
    hostile = read(root, REL_HOSTILE)
    upload_error = read(root, REL_UPLOAD_ERROR)
    handler = read(root, REL_IMPORT_HANDLER)

    # -- 1. ODS / ODT routing -------------------------------------------------
    detect = function_body(routing, "pub fn detect_spreadsheet_format")
    add(Check(
        "routing.discriminator_ods",
        has(detect, '"ods"')
        and has(detect, "content.xml")
        and has(detect, "mimetype"),
        f"{REL_ROUTING}: detect_spreadsheet_format must mark ODF zips as \"ods\" "
        "via content.xml/mimetype",
    ))
    add(Check(
        "routing.discriminator_xlsx",
        has(detect, '"xlsx"') and has(detect, "xl/")
        and has(detect, "[Content_Types].xml"),
        f"{REL_ROUTING}: OOXML must be discriminated by xl/ / [Content_Types].xml",
    ))
    add(Check(
        "routing.ods_to_odf_parser",
        has(routing, '"ods" => odf::parse_ods_to_worksheets(bytes)?'),
        f"{REL_ROUTING}: detected ODS must dispatch to odf::parse_ods_to_worksheets, "
        "not parse_xlsx_to_worksheets",
    ))
    add(Check(
        "routing.ext_ods_fallback_to_odf",
        has(routing, 'ext == "ods"') and has(routing, "odf::parse_ods_to_worksheets(bytes)?"),
        f"{REL_ROUTING}: extension fallback for .ods must use the ODF parser",
    ))
    add(Check(
        "routing.drive_open_ods_to_odf",
        has(xlsx, 'if ext == "ods"')
        and has(xlsx, "return super::odf::parse_ods_to_worksheets(bytes);"),
        f"{REL_XLSX}: drive-open path must return through the ODF parser before "
        "the OOXML parse",
    ))
    add(Check(
        "routing.odf_mimetype_is_spreadsheet_only",
        has(odf, "application/vnd.oasis.opendocument.spreadsheet")
        and has(odf, "pub fn is_odf_spreadsheet")
        and has(odf, '"ODS import failed: not an ODF spreadsheet package"'),
        f"{REL_ODF}: ODT (non-spreadsheet mimetype) must be refused by the ODF "
        "path, never treated as a spreadsheet",
    ))
    add(Check(
        "routing.odt_refusal_test_present",
        has(tests, "fn refuses_a_non_spreadsheet_odf_package()")
        and has(tests, "!is_odf_spreadsheet(&bytes)")
        and has(tests, "not an ODF spreadsheet"),
        f"{REL_ODF_TESTS}: ODT refusal must be asserted with a named test",
    ))

    # -- 2. Malware gate: explicit error, before the parser -------------------
    add(Check(
        "malware.scan_failed_message",
        "the malware scan could not be completed" in upload_error
        and "Self::MalwareScanFailed" in upload_error,
        f"{REL_UPLOAD_ERROR}: MalwareScanFailed must emit the explicit "
        "\"the malware scan could not be completed\" message",
    ))
    add(Check(
        "malware.scanner_missing_message",
        "no malware scanner is configured" in upload_error
        and "Self::MalwareScannerMissing" in upload_error,
        f"{REL_UPLOAD_ERROR}: unavailable scanner must emit an explicit message",
    ))
    decision = function_body(hostile, "pub fn malware_decision")
    add(Check(
        "malware.failed_decision_is_error",
        has(decision, "MalwareScanDecision::Failed")
        and has(decision, "Err(UploadError::MalwareScanFailed)"),
        f"{REL_HOSTILE}: Failed decision must return Err(MalwareScanFailed)",
    ))
    add(Check(
        "malware.not_configured_is_error",
        has(decision, "MalwareScanDecision::NotConfigured if require_scanner")
        and has(decision, "Err(UploadError::MalwareScannerMissing)"),
        f"{REL_HOSTILE}: NotConfigured must fail closed with "
        "Err(MalwareScannerMissing)",
    ))
    add(Check(
        "malware.scanner_required_by_default",
        has(function_body(hostile, "impl Default for HostileFilePolicy"),
            "require_malware_scanner: true"),
        f"{REL_HOSTILE}: default policy must require a scanner (no opt-out)",
    ))
    inspect = function_body(hostile, "pub fn inspect_hostile_upload_with_scanner")
    decision_line = line_of(inspect, "malware_decision(&malware_scan, policy.require_malware_scanner)?;")
    report_line = line_of(inspect, "Ok(HostileFileReport {")
    add(Check(
        "malware.gate_fails_closed_before_report",
        decision_line is not None
        and report_line is not None
        and decision_line < report_line,
        f"{REL_HOSTILE}: malware_decision(...)? (line {decision_line}) must precede "
        f"Ok(HostileFileReport) (line {report_line}) - no success path without a scan",
    ))
    gate_line = line_of(handler, "audit_hostile_upload(")
    parser_line = line_of(handler, "import_spreadsheet_bytes(effective")
    add(Check(
        "malware.handler_gate_precedes_parser",
        gate_line is not None and parser_line is not None and gate_line < parser_line,
        f"{REL_IMPORT_HANDLER}: hostile-upload audit (line {gate_line}) must run "
        f"before import_spreadsheet_bytes (line {parser_line})",
    ))

    # -- 3. ODF regression tests ---------------------------------------------
    present = [name for name in ODF_REGRESSION_TESTS if has(tests, f"fn {name}()")]
    add(Check(
        "odf_tests.seven_regressions_present",
        len(present) >= MIN_TESTS and len(present) == len(ODF_REGRESSION_TESTS),
        f"{REL_ODF_TESTS}: expected all {len(ODF_REGRESSION_TESTS)} named regression "
        f"tests, found {len(present)}: {', '.join(present) or 'none'}",
    ))

    # -- 4. Interop UI: local assets only ------------------------------------
    sheet_ui = read(root, REL_SHEET_UI)
    add(Check(
        "ui.interop_import_marker",
        has(sheet_ui, 'id="sheetImportInput"')
        and has(sheet_ui, 'accept=".xlsx,.xls,.ods"'),
        f"{REL_SHEET_UI}: Sheets ODS import entry point must be present",
    ))

    fidelity_dir = root / REL_FIDELITY_DIR
    modules: list[Path] = []
    if fidelity_dir.is_dir():
        modules = sorted(p for p in fidelity_dir.glob("*.js") if p.stat().st_size > 0)
    add(Check(
        "ui.harness_files_present",
        len(modules) >= 5,
        f"{REL_FIDELITY_DIR}: expected >=5 non-empty interop fidelity modules, "
        f"found {len(modules)}",
    ))

    ui_files = [(REL_SHEET_UI, sheet_ui), (REL_FIDELITY_JS, read(root, REL_FIDELITY_JS))]
    ui_files.extend((str(p.relative_to(root)), p.read_text(encoding="utf-8")) for p in modules)
    cdn_hits: list[str] = []
    for rel, source in ui_files:
        for match in CDN_HOST.finditer(source):
            line = source.count("\n", 0, match.start()) + 1
            cdn_hits.append(f"{rel}:{line}:{match.group(0)[:80]}")
    add(Check(
        "ui.no_cdn_references",
        not cdn_hits and len(ui_files) >= 7,
        f"checked {len(ui_files)} interop UI files; CDN/unpkg hits: "
        + ("; ".join(cdn_hits[:5]) if cdn_hits else "none"),
    ))

    return checks


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=DEFAULT_ROOT,
                        help="workspace root (point at a copy for non-vacuity probes)")
    args = parser.parse_args()

    try:
        checks = audit(args.root)
    except AuditError as exc:
        print(f"FAIL audit input error: {exc}")
        return 1

    failed = [c for c in checks if not c.ok]
    print("=== Office interop static audit (#2463) ===")
    print(f"  root        : {args.root}")
    print(f"  assertions  : {len(checks)}")
    print(f"  passed      : {len(checks) - len(failed)}")
    for check in failed:
        print(f"  FAIL {check.name}: {check.detail}")
    print("RESULT: " + ("FAIL" if failed else "PASS"))
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
