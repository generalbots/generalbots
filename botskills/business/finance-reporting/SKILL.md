---
name: finance-reporting
description: |
  Produce P&L, cash-flow and KPI reports from the ledger. Use for monthly/quarterly reporting.
version: 1.0.0
source: in-repo
---

# finance-reporting

## Workflow

1. Reconcile source data first (balances tie to the ledger).
2. P&L by period with comparatives and variance explanations.
3. Cash flow: operating/investing/financing, with the runway calculation.
4. KPIs with definitions and the same formula every period.
5. Flag one-off items separately from recurring.
6. Deliver numbers plus a short narrative of what changed and why.

## Quality checks

- Totals reconcile to the source ledger.
- Every KPI cites its formula and period.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
