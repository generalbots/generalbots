---
name: procurement
description: |
  Compare quotes and route approvals for purchases. Use for buying decisions.
version: 1.0.0
source: in-repo
---

# procurement

## Workflow

1. Define requirements and must-have vs nice-to-have first.
2. Collect at least three quotes with the same scope and validity date.
3. Normalise: taxes, freight, payment terms, warranty, total cost of ownership.
4. Score against weighted criteria; show the weights.
5. Route to the approver per policy; record the decision and rationale.
6. Keep the quotes attached to the decision.

## Quality checks

- Quotes are comparable (same scope, currency, taxes).
- The winning choice cites the scoring, not just the price.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
