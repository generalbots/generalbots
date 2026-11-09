---
name: spreadsheet-modeling
description: |
  Build financial models with assumptions, scenario tables, pivots and checks. Use for budgets,
  forecasts, unit economics and any model that must stay auditable.
version: 1.0.0
source: in-repo
---

# spreadsheet-modeling

## Workflow

1. Separate sheets: Assumptions, Drivers, Model, Outputs, Checks. Only Assumptions contains hardcoded numbers.
2. Colour code: blue inputs, black formulas, green cross-sheet links; document the legend on Assumptions.
3. Scenario table with a selector (base/best/worst) feeding the model through named ranges.
4. Add a Checks sheet: totals tie, no hardcoded formula except inputs, balance rows zero out.
5. Pivot/summary on Outputs only; charts reference named ranges.
6. Verify with a recalc (LibreOffice headless or formulas re-evaluated) and report the checks' results.

## Quality checks

- Changing one assumption changes the outputs and keeps Checks green.
- No circular references; every external number has a source cell.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
