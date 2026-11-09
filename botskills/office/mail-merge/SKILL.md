---
name: mail-merge
description: |
  Generate one document per recipient from a template and a data source (CSV/JSON). Use for
  certificates, invoices, letters and personalised reports.
version: 1.0.0
source: in-repo
---

# mail-merge

## Workflow

1. Validate the data first: required columns, row count, no blank keys, deduplicate recipients.
2. Template placeholders use clear tokens ({{name}}, {{amount}}) that cannot collide with document text.
3. Render one file per row into output/merge/, keeping the template untouched.
4. Name files deterministically (slug of the key, not the row index) to make re-runs idempotent.
5. Produce a manifest CSV with row → file and any skipped rows with the reason.
6. Spot-check 2-3 outputs and report counts rendered/skipped.

## Quality checks

- Every input row produces a file or a documented skip reason.
- No token remains in any rendered document.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
