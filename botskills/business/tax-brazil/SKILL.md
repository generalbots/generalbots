---
name: tax-brazil
description: |
  Calculate IRPJ/CSLL/PIS/COFINS/ISS with the regime and bases shown. Use for Brazilian tax questions.
version: 1.0.0
source: in-repo
---

# tax-brazil

## Workflow

1. Confirm the regime (Simples, presumido, real) and the period.
2. Compute each tax with its base and rate; show the calculation, not just the total.
3. Apply credits/reductions only when the regime allows them.
4. Respect periodicity (monthly/quarterly) and the correct due dates.
5. Flag values that need an accountant's confirmation.
6. Keep a line-by-line worksheet for audit.

## Quality checks

- Every tax line shows base × rate.
- Regime assumptions are stated up front.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
