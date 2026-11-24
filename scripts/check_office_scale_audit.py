#!/usr/bin/env python3
"""Offline static audit of the office large-file / low-memory readiness work.

This is a read-only, CI-friendly guard for #2534. It never builds botserver,
never starts a browser and never mutates the tree. It asserts, non-vacuously:

  1. WINDOW  - ``window.rs`` pins the A1 window caps and ``parse_window_range``
               refuses oversized ranges; the handler routes through that guard.
  2. STREAM  - the XLSX importer runs a constant-memory streaming row pre-scan
               *before* building the umya DOM, enforces a per-sheet cell cap and
               is reachable from the shared import path.
  3. SCALE   - the pure budget model prices rows/cols/formulas per tier and has
               explicit over-budget refusal variants on every path; the shared
               ImportBudget rejects over-budget workbooks before parsing.
  4. ADMIT   - the per-request gate exposes SheetsImport/OfficeExport work
               classes, the routes call it, and refusals map to typed 413/507/429.
  5. CI      - the 20/20 office-memory self-test and the 57/57 mobile-office
               suite still exist, pass offline and are invoked by ci.yml.

The real 5-100 MB RSS soak needs a 4 GB cgroup and measured hardware; this
script deliberately does NOT fabricate RSS numbers. It guards the source-level
readiness only. Point ``--root`` at a modified copy for a non-vacuity probe.

Exit 0 when every assertion passes, 1 otherwise.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path
from typing import NamedTuple

DEFAULT_ROOT = Path(__file__).resolve().parent.parent

REL_WINDOW = "botserver/crates/botsheet/src/handlers/window.rs"
REL_XLSX = "botserver/crates/botsheet/src/storage/import/xlsx.rs"
REL_IMPORT_MOD = "botserver/crates/botsheet/src/storage/import/mod.rs"
REL_BUDGET = "botserver/crates/botsheet/src/scale/budget.rs"
REL_PLANNER = "botserver/crates/botsheet/src/scale/planner.rs"
REL_RECALC = "botserver/crates/botsheet/src/scale/recalc.rs"
REL_ADMISSION = "botserver/crates/botsheet/src/handlers/admission.rs"
REL_EXPORT_IMPORT = "botserver/crates/botsheet/src/handlers/export_import.rs"
REL_POLICY = "botserver/crates/botcore/src/shared/memory_admission/policy.rs"
REL_ERROR = "botserver/crates/botcore/src/shared/memory_admission/error.rs"
REL_GB_BUDGET = "botserver/crates/botcore/src/shared/memory_admission/budget.rs"
REL_OFFICE_MEMORY = "scripts/check_office_memory.py"
REL_MOBILE_JS = "botui/ui/suite/tests/webtest-ui-mobile-office.js"
REL_SLO = "ops/office-slo.yaml"
REL_CI = ".github/workflows/ci.yml"

SOURCES = (
    REL_WINDOW, REL_XLSX, REL_IMPORT_MOD, REL_BUDGET, REL_PLANNER, REL_RECALC,
    REL_ADMISSION, REL_EXPORT_IMPORT, REL_POLICY, REL_ERROR, REL_GB_BUDGET,
    REL_OFFICE_MEMORY, REL_MOBILE_JS, REL_SLO, REL_CI)


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


def masked(text: str) -> str:
    """Blank comments and string contents while preserving length/offsets."""
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


def body(text: str, signature: str) -> str:
    """Return the brace-balanced body of the item whose signature matches."""
    src = masked(text)
    at = src.find(signature)
    if at < 0:
        raise AuditError("item not found: {}".format(signature))
    opened = src.find("{", at)
    if opened < 0:
        raise AuditError("item has no body: {}".format(signature))
    depth, i, n = 0, opened, len(src)
    while i < n:
        if src[i] == "{":
            depth += 1
        elif src[i] == "}":
            depth -= 1
            if depth == 0:
                return src[opened : i + 1]
        i += 1
    raise AuditError("unbalanced body: {}".format(signature))


def int_const(text: str, name: str) -> int | None:
    match = re.search(r"\b" + re.escape(name) + r"\s*:\s*u32\s*=\s*([\d_]+)\s*;", text)
    if match is None:
        return None
    try:
        return int(match.group(1).replace("_", ""))
    except ValueError:
        return None


def run_offline(root: Path, argv: list[str]) -> tuple[int, str]:
    try:
        proc = subprocess.run(
            argv, cwd=str(root), capture_output=True, text=True, timeout=120, check=False
        )
    except (OSError, subprocess.SubprocessError) as exc:
        return -1, "cannot execute {}: {}".format(argv[0], exc)
    return proc.returncode, (proc.stdout + proc.stderr).strip()


def audit(root: Path) -> list[Check]:
    src = {rel: read(root, rel) for rel in SOURCES}
    window, xlsx, import_mod = src[REL_WINDOW], src[REL_XLSX], src[REL_IMPORT_MOD]
    budget_src, planner, recalc = src[REL_BUDGET], src[REL_PLANNER], src[REL_RECALC]
    admission, routes = src[REL_ADMISSION], src[REL_EXPORT_IMPORT]
    policy, error, gb_budget = src[REL_POLICY], src[REL_ERROR], src[REL_GB_BUDGET]
    memory_py, mobile_js, slo_yaml, ci = (
        src[REL_OFFICE_MEMORY], src[REL_MOBILE_JS], src[REL_SLO], src[REL_CI]
    )
    checks: list[Check] = []
    add = checks.append

    # 1. Windowed reads: hard caps + refuse oversized windows. ----------------
    rows_cap = int_const(window, "MAX_WINDOW_ROWS")
    cols_cap = int_const(window, "MAX_WINDOW_COLS")
    add(Check(
        "window.cap_constants",
        rows_cap == 1_000 and cols_cap == 256,
        "{} rows={} (want 1000), {} cols={} (want 256)".format(
            locator(REL_WINDOW, window, "MAX_WINDOW_ROWS"), rows_cap,
            locator(REL_WINDOW, window, "MAX_WINDOW_COLS"), cols_cap,
        ),
    ))
    range_body = body(window, "pub fn parse_window_range")
    add(Check(
        "window.refuses_oversized_window",
        "rows > MAX_WINDOW_ROWS" in range_body
        and "cols > MAX_WINDOW_COLS" in range_body
        and "return None" in range_body,
        "{} guard rows>cols caps inside parse_window_range".format(
            locator(REL_WINDOW, window, "pub fn parse_window_range")
        ),
    ))
    handler_body = body(window, "pub async fn handle_sheet_window")
    add(Check(
        "window.handler_uses_guard",
        "parse_window_range(&raw_range).ok_or_else(" in handler_body
        and "Invalid window range" in handler_body
        and "bounded_window_tiles(range)?" in handler_body,
        "{} handler rejects a failed parse and plans only bounded tiles".format(
            locator(REL_WINDOW, window, "pub async fn handle_sheet_window")
        ),
    ))

    # 2. Streaming import: no whole-workbook DOM before the budget check. ----
    scan_body = body(xlsx, "fn count_rows_streaming")
    add(Check(
        "streaming.count_rows_streaming_exists",
        "ZipArchive::new" in scan_body and "[0u8; 8192]" in scan_body
        and "entry.read(&mut buffer)" in scan_body and 'b"<row "' in scan_body,
        "{} bounded 8 KiB chunked row scan".format(
            locator(REL_XLSX, xlsx, "fn count_rows_streaming")
        ),
    ))
    scan_line = line_of(xlsx, "let rows = count_rows_streaming(bytes, budget.max_rows);")
    check_line = line_of(xlsx, "if let Err(error) = budget.check(ImportSize {")
    dom_line = line_of(xlsx, "umya_spreadsheet::reader::xlsx::read_reader(")
    add(Check(
        "streaming.scan_precedes_dom",
        scan_line is not None and check_line is not None and dom_line is not None
        and scan_line < check_line < dom_line,
        "{} streaming scan -> {} budget check -> {} DOM read".format(
            locator(REL_XLSX, xlsx, "let rows = count_rows_streaming"),
            locator(REL_XLSX, xlsx, "if let Err(error) = budget.check"),
            locator(REL_XLSX, xlsx, "read_reader("),
        ),
    ))
    cap_line = line_of(xlsx, "ensure_within_cell_cap(data.len(), sheet.get_name())?;")
    insert_line = line_of(xlsx, "data.insert(")
    cap_match = re.search(r"MAX_IMPORT_CELLS_PER_SHEET:\s*usize\s*=\s*([\d_]+)", xlsx)
    cap_value = int(cap_match.group(1).replace("_", "")) if cap_match else None
    add(Check(
        "streaming.per_sheet_cell_cap",
        cap_value == 250_000 and cap_line is not None and insert_line is not None
        and cap_line < insert_line,
        "{} cap={} before {} cell insert".format(
            locator(REL_XLSX, xlsx, "MAX_IMPORT_CELLS_PER_SHEET"), cap_value,
            locator(REL_XLSX, xlsx, "ensure_within_cell_cap(data.len()"),
        ),
    ))
    import_fn = body(import_mod, "pub fn import_spreadsheet_bytes")
    add(Check(
        "streaming.reachable_from_import_path",
        'parse_xlsx_to_worksheets(bytes, "xlsx")?' in import_fn
        and "pub use xlsx::parse_xlsx_to_worksheets;" in import_mod
        and "budget.check(ImportSize {" in import_fn,
        "{} import path dispatches to the streaming XLSX parser".format(
            locator(REL_IMPORT_MOD, import_mod, "pub fn import_spreadsheet_bytes")
        ),
    ))

    # 3. Scale planner: rows/cols/formulas priced, over-budget refused. ------
    budget_struct = body(budget_src, "pub struct MemoryBudget")
    classify_body = body(budget_src, "pub fn classify")
    tier_body = body(budget_src, "pub const fn for_tier")
    add(Check(
        "scale.budget_axes_rows_cols_formulas",
        all(field in budget_struct for field in (
            "max_cells", "max_formulas", "tile_rows", "tile_cols", "max_window_tiles"))
        and "budget.max_cells" in classify_body and "budget.max_formulas" in classify_body
        and "tile_rows:" in tier_body and "tile_cols:" in tier_body
        and "step_by(budget.tile_rows as usize)" in planner
        and "step_by(budget.tile_cols as usize)" in planner,
        "{} MemoryBudget axes; {} classify; {} tier geometry".format(
            locator(REL_BUDGET, budget_src, "pub struct MemoryBudget"),
            locator(REL_BUDGET, budget_src, "pub fn classify"),
            locator(REL_BUDGET, budget_src, "pub const fn for_tier"),
        ),
    ))
    within_body = body(budget_src, "pub fn within_cells_and_formulas")
    recalc_body = body(recalc, "pub fn estimate_recalc")
    plan_body = body(planner, "pub fn plan_window")
    error_body = body(budget_src, "pub enum BudgetError")
    add(Check(
        "scale.explicit_over_budget_refusals",
        "BudgetError::TooManyCells" in within_body
        and "BudgetError::TooManyFormulas" in within_body
        and all(name in error_body for name in (
            "InvalidBudget", "RecalcOverBudget", "TooManyCells",
            "TooManyFormulas", "TooManyTiles"))
        and "BudgetError::RecalcOverBudget" in recalc_body
        and "requested_tiles > budget.max_window_tiles" in plan_body
        and "BudgetError::TooManyTiles" in plan_body,
        "{} cells/formulas, {} recalc, {} tiles all fail closed".format(
            locator(REL_BUDGET, budget_src, "pub fn within_cells_and_formulas"),
            locator(REL_RECALC, recalc, "pub fn estimate_recalc"),
            locator(REL_PLANNER, planner, "pub fn plan_window"),
        ),
    ))
    gb_check = body(gb_budget, "pub fn check")
    add(Check(
        "scale.import_budget_rejects_over_budget",
        "size.bytes > self.max_bytes" in gb_check
        and "size.rows > self.max_rows" in gb_check
        and "size.pages > self.max_pages" in gb_check
        and all(name in gb_budget for name in (
            "TooLarge", "TooManyRows", "TooManyPages"))
        and "budget.check(ImportSize {" in import_mod
        and "budget.check(ImportSize {" in xlsx,
        "{} pure bytes/rows/pages gate used by {} and {}".format(
            locator(REL_GB_BUDGET, gb_budget, "pub fn check"), REL_IMPORT_MOD, REL_XLSX
        ),
    ))

    # 4. Admission: work classes, wired routes, typed refusals. --------------
    classes = body(policy, "pub enum WorkClass")
    add(Check(
        "admission.work_classes_exist",
        "SheetsImport" in classes and "OfficeExport" in classes
        and "const ALL" in policy and "Self::SheetsImport" in policy
        and "Self::OfficeExport" in policy,
        "{} declares SheetsImport and OfficeExport".format(
            locator(REL_POLICY, policy, "pub enum WorkClass")
        ),
    ))
    exports = re.findall(r"admit\(\s*WorkClass::OfficeExport", routes)
    imports = re.findall(r"admit\(WorkClass::SheetsImport", routes)
    add(Check(
        "admission.routes_gated",
        len(exports) >= 2 and len(imports) >= 1,
        "{} OfficeExport admits={}, SheetsImport admits={}".format(
            locator(REL_EXPORT_IMPORT, routes, "WorkClass::OfficeExport"),
            len(exports), len(imports),
        ),
    ))
    variants = body(error, "pub enum AdmissionError")
    statuses = body(error, "pub fn status_code")
    codes = body(error, "pub fn code")
    add(Check(
        "admission.typed_refusal_contract",
        all(name in variants for name in ("PayloadTooLarge", "BudgetExceeded", "Busy"))
        and "PAYLOAD_TOO_LARGE" in statuses and "INSUFFICIENT_STORAGE" in statuses
        and "TOO_MANY_REQUESTS" in statuses
        and all(code in codes for code in (
            "office_payload_too_large", "office_memory_budget_exceeded", "office_busy")),
        "{} variants + 413/507/429 + stable codes".format(
            locator(REL_ERROR, error, "pub enum AdmissionError")
        ),
    ))
    reject_body = body(admission, "fn reject(error: AdmissionError)")
    add(Check(
        "admission.reject_mapping_tested",
        "error.status_code()" in reject_body and "error.json_body()" in reject_body
        and "fn reject_maps_each_typed_error_to_413_507_429()" in admission
        and "office_memory_budget_exceeded" in admission and "office_busy" in admission,
        "{} maps typed errors; {} pins the table".format(
            locator(REL_ADMISSION, admission, "fn reject(error"),
            locator(REL_ADMISSION, admission, "fn reject_maps_each_typed_error"),
        ),
    ))

    # 5. CI guards: corpus soak (20) + mobile-office suite (57). -------------
    ci_commands = (
        "python3 scripts/office_largefile_corpus.py --self-test",
        "python3 scripts/check_office_memory.py --self-test",
        "python3 scripts/check_office_memory.py --check",
        "node botui/ui/suite/tests/webtest-ui-mobile-office.js",
    )
    missing_ci = [command for command in ci_commands if command not in ci]
    add(Check(
        "ci.guard_steps_present",
        not missing_ci and "Guard: Office large-file corpus and memory ceilings (#2530)" in ci
        and "Suite UI standards guards (#2335)" in ci,
        "{} missing: {}".format(locator(REL_CI, ci, "check_office_memory.py --check"),
                                ", ".join(missing_ci) if missing_ci else "none"),
    ))
    add(Check(
        "ci.corpus_declaration_has_20_cycles",
        'soak.get("cycles") != 20' in memory_py
        and "cgroup_reference_mb: 4096" in slo_yaml
        and "min_size_mb: 5" in slo_yaml and "max_size_mb: 100" in slo_yaml
        and "synthetic_only: true" in slo_yaml,
        "{} pins 20 soak cycles; {} declares 5-100 MB / 4 GB".format(
            locator(REL_OFFICE_MEMORY, memory_py, 'soak.get("cycles")'), REL_SLO
        ),
    ))
    rc, out = run_offline(root, [sys.executable, str(root / REL_OFFICE_MEMORY), "--self-test"])
    match = re.search(r"self-test:\s*(\d+)/\1 checks passed", out)
    count = int(match.group(1)) if match else -1
    add(Check(
        "ci.office_memory_selftest_20",
        rc == 0 and count == 20,
        "{} -> rc={} count={}/20; {}".format(
            REL_OFFICE_MEMORY, rc, count, out.splitlines()[-1] if out else "no output"
        ),
    ))
    add(Check(
        "ci.mobile_office_source_intact",
        all("require(\"./webtest-ui-mobile-office/" + module + "\")" in mobile_js
            for module in ("00_harness", "10_docs_runtime", "11_slides_runtime",
                           "12_desktop_gating", "13_sheet_overflow", "20_static_gaps"))
        and "harness.finish();" in mobile_js,
        "{} loads shipped mobile IIFE modules and finishes".format(
            locator(REL_MOBILE_JS, mobile_js, "harness.finish();")
        ),
    ))
    node = "node"
    rc, out = run_offline(root, [node, str(root / REL_MOBILE_JS)])
    totals = re.search(r"TOTAL:\s*(\d+)\s*\|\s*PASS:\s*(\d+)\s*\|\s*FAIL:\s*(\d+)", out)
    total = int(totals.group(1)) if totals else -1
    passed = int(totals.group(2)) if totals else -1
    failed = int(totals.group(3)) if totals else -1
    add(Check(
        "ci.mobile_office_selftest_57",
        rc == 0 and (total, passed, failed) == (57, 57, 0),
        "{} -> rc={} total={} pass={} fail={}".format(
            REL_MOBILE_JS, rc, total, passed, failed
        ),
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
        print("FAIL audit input error: {}".format(exc))
        return 1
    failed = [check for check in checks if not check.ok]
    print("=== Office large-file / low-memory readiness audit (#2534) ===")
    print("  root: {} | assertions: {} | passed: {}".format(
        args.root, len(checks), len(checks) - len(failed)))
    for check in checks:
        print("  {} {}: {}".format("PASS" if check.ok else "FAIL", check.name, check.detail))
    print("RESULT: {}".format("FAIL" if failed else "PASS"))
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
