# fidelity — OOXML round-trip feature comparator (#2457)

Test-only, stdlib-only tool that compares two corpus directories — an
**original** set of Office files and their **round-tripped** counterparts — and
emits a deterministic JSON or Markdown diff report with a pass/fail threshold.

This is a bounded slice of issue #2457's "round-trip harness". It is **not** the
full QA lab: it does not import/export through the running apps, rasterise
pages, build a coverage matrix, fuzz importers, drive a browser or wire a CI
job. It answers one question offline: *given two directories of DOCX/PPTX/XLSX
files, how much structural fidelity survived?* It complements
`bottest/tests/office_fidelity.rs` (byte-level per-part preservation) and
`bottest/tools/officestress` / `scripts/office_largefile_corpus.py` (synthetic
corpus generation).

## Usage

```bash
# Deterministic self-test on synthetic fixtures (writes only under /tmp/opencode).
python3 bottest/tools/fidelity/fidelity.py --self-test

# Compare two corpus directories; Markdown goes to stdout. Exit 0 PASS / 1 FAIL.
python3 bottest/tools/fidelity/fidelity.py \
    --original /tmp/original --roundtrip /tmp/roundtripped --threshold 0.95

# Machine-readable reports (byte-identical for identical inputs).
python3 bottest/tools/fidelity/fidelity.py \
    --original /tmp/original --roundtrip /tmp/roundtripped \
    --json /tmp/fidelity.json --md /tmp/fidelity.md
```

Exit codes: `0` PASS (or self-test passed) · `1` FAIL (score below threshold or
self-test failure) · `2` bad arguments, missing input directory or report write
error. CLI: `--threshold` in `[0,1]` (default `0.95`), `--text-weight` positive
(default `3.0`). Files are paired by relative path; the report never contains
absolute host paths from the corpus itself (only the two directory arguments
as given).

## What is extracted

Each package is opened as a ZIP; a fixed set of OOXML parts is parsed with
`xml.etree.ElementTree`. Elements are matched by **local name**, so namespace
prefixes do not matter. Features are a fixed schema per kind:

| Kind | Count features | Text |
|------|----------------|------|
| DOCX | paragraphs, tables, bold, italic, underline, hyperlinks, images, charts, footnotes, tracked_changes, comments, numbering | normalised `<w:t>` text |
| PPTX | slides, shapes, tables, bold, italic, underline, images, charts, masters | normalised `<a:t>` text on slides |
| XLSX | sheets, rows, cells, formulas, shared_strings, tables, bold, italic, underline, comments | normalised `<t>`/`<v>` text |

PPTX bold/italic/underline are read from `b`/`i`/`u` attributes on run
properties; DOCX and XLSX use `<w:b>`/`<font><b/>`-style elements. Parts outside
the list (themes, layouts, media, rels, custom XML) are ignored — byte-level
part preservation is covered by `office_fidelity.rs`.

## Scoring

* Count features: `1.0` when equal, otherwise `1 - |a-b| / max(a,b)` with status
  `dropped` (round-tripped lower) or `added` (higher). A flattened chart in a
  PPTX export therefore scores `0` and is reported as `dropped`, not hidden.
* Text: `1.0` when the normalised strings are identical; otherwise a
  `SequenceMatcher` ratio over the first 4000 chars with status `degraded`.
  Full-length SHA-256 digests are kept as evidence.
* Per-file score: weighted mean of its features (text weight 3.0 by default).
* Corpus score: unweighted mean of every file in the **union** of both trees.
  A file present only on one side (`missing`/`extra`) or an unreadable package
  scores `0`, so a corrupted export cannot silently pass.
* PASS when corpus score `>= --threshold`. An empty corpus scores `0` with note
  `no-office-files` and FAILs loudly.

## Output

JSON (`--json`) is `sort_keys`, indented, newline-terminated. Markdown (`--md`,
or stdout) has a summary table, a per-file table and a "Feature differences"
table listing only non-matching features and non-matched files. See the JSON
`schema_version` field for forward compatibility.

## Files

| File | Role |
|------|------|
| `fidelity.py` | CLI, self-test orchestration, exit codes |
| `fx_extract.py` | OOXML part parsing and feature extraction |
| `fx_compare.py` | pairing, scoring, threshold, report assembly |
| `fx_report.py` | deterministic JSON/Markdown rendering |
| `fx_fixtures.py` | synthetic stored-ZIP fixtures for `--self-test` |

Fixtures are structurally OOXML-like but not valid for Word/Excel; they are
generated from integer specs with fixed ZIP metadata, so the self-test is
reproducible on any Python 3 and OS. No real document text, tenant name or path
is embedded.
