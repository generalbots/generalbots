# interop — Office conversion-report generator + gate (#2463)

Offline, stdlib-only generator and validator for a deterministic "what did GB's
conversion lose?" report. Given a source Office file (DOCX/PPTX/XLSX) and the
GB-converted output, it emits a JSON report of structural losses and a
machine-readable metric-compatible font-fallback decision, then applies a gate
that fails when a required feature is **silently** dropped. Companion to
`tools/fidelity` (#2457) and `tools/officestress` (#2531); no server, browser,
network or third-party package is used.

## Scope

In scope (offline, this slice):

* extract five structural features — `headings`, `tables`, `images`,
  `formulas`, `fonts` — from OOXML DOCX/PPTX/XLSX packages;
* diff source vs converted into an ordered `losses` list;
* produce one `font_decisions` record per source font with a
  metric-compatible fallback decision;
* gate on silent drops; validate a stored report by re-deriving everything.

**Out of scope** (require a running server, explicitly not implemented here):
WOPI / Office Online host + connector, Google Workspace import/export, OLE
embedded-object open/download, ODF 1.3 conformance validation, PDF form fill,
and the cross-format compare UI. Those remain tracked by #2463.

## Usage

Run from the repository root (the tool imports its sibling `ix_*` modules by
script directory, so no package install is needed):

```bash
# Offline deterministic self-test on synthetic OOXML fixtures.
python3 bottest/tools/interop/interop.py --self-test

# Generate a report for a real source + GB-converted pair.
python3 bottest/tools/interop/interop.py \
  --source /tmp/source.docx --converted /tmp/converted.docx \
  --json /tmp/opencode/conversion-report.json

# Print the report JSON to stdout instead of writing it.
python3 bottest/tools/interop/interop.py \
  --source /tmp/source.xlsx --converted /tmp/converted.xlsx

# Validate a stored report: re-derive losses/decisions/gate and fail closed.
python3 bottest/tools/interop/interop.py \
  --validate /tmp/opencode/conversion-report.json
```

Exit codes: `0` PASS/self-test ok, `1` gate FAIL, `2` bad args / bad input /
I/O. Output and fixtures default to `/tmp/opencode/` — never the repo root.

## Report schema

`schema` is `gb-interop-conversion-report/1`; JSON is `sort_keys`, two-space
indented, ASCII-escaped and newline-terminated.

| Field | Meaning |
|---|---|
| `source`, `converted` | `name`, `kind` and the five-feature `features` vector (`fonts` is a sorted name list) |
| `losses` | count losses (before > after) and missing fonts; count losses carry `severity` `dropped`/`partial`; font losses carry `fallback`, `strategy`, `documented` |
| `font_decisions` | one record per unique source font: `status` (`available`/`substituted`/`unresolved`), `fallback`, `strategy` (`none`/`metric-compatible`/`approximate`), `warning`, `compatible`, `reason` |
| `gate` | `passed`, `required_features`, `failures`, `warnings`, `checks` |

### Font fallback decision

Known Croscore pairs are treated as metric-compatible (Calibri→Carlito,
Cambria→Caladea, Arial/Helvetica→Liberation Sans, Times New Roman→Liberation
Serif, Courier New→Liberation Mono, Arial Narrow→Liberation Sans Narrow).
Visually similar but width-different pairs (Verdana/Tahoma→DejaVu Sans,
Georgia→Liberation Serif, Comic Sans MS→Comic Neue) are `approximate` — warned
but **not** claimed metric-compatible. Symbol fonts (Wingdings, Symbol, …) and
unknown fonts are `unresolved`. GB's shipped inventory is listed in
`ix_fonts.AVAILABLE_FONTS`.

### Gate semantics

* **FAIL** — a required count feature (`headings`/`tables`/`images`/`formulas`)
  goes from >0 to 0 with no replacement (silent drop), or a source font
  disappears with no fallback actually present in the converted output.
* **WARN (pass)** — a count feature is reduced but not eliminated, or a source
  font is replaced by a recorded fallback that *is* present in the converted
  output (documented substitution).
* **OK** — no loss.

The validator re-computes `losses`, `font_decisions` and `gate` from the stored
feature vectors and compares them with the stored fields; any mismatch (or bad
schema) is reported as invalid (exit 2), so a hand-edited report cannot smuggle
a PASS.

## Files

| File | Role |
|---|---|
| `interop.py` | CLI, self-test, report writing and validation entry point |
| `ix_extract.py` | namespace-agnostic OOXML structural feature extraction |
| `ix_fonts.py` | font inventory + metric-compatible fallback decisions |
| `ix_report.py` | report build, loss/gate computation, validator |
| `ix_fixtures.py` | deterministic synthetic OOXML fixture builders |

All modules are kept under 450 lines and import only the Python standard
library (verified by the self-test's `stdlib-only imports` check).
