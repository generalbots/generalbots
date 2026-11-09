---
name: xlsx
description: |
  Create workbooks with formulas, charts, formatting, validation and freeze panes. Use for
  spreadsheets, budgets, trackers, dashboards and any .xlsx output.
version: 1.0.0
source: in-repo
---

# xlsx

## Workflow

1. Write with openpyxl (or rust_xlsxwriter): one header row, typed cells (numbers as numbers, not strings).
2. Formulas over hardcoded totals (=SUM(B2:B13), =AVERAGE(...)); name ranges used by charts.
3. Charts: reference a range that includes labels; title both axes; legend only when more than one series.
4. Formatting: number formats per column (currency, percent, date), bold header, column widths sized to content.
5. Freeze the header row and add a filter on the header range.
6. Validate: reopen the file, evaluate the summary cells and check the chart XML exists in xl/charts/.

## Quality checks

- Every numeric column is numeric, never text; no #REF! or #VALUE! on open.
- The chart series range matches the data range exactly (no off-by-one row).

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
