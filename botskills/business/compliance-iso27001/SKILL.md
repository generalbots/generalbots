---
name: compliance-iso27001
description: |
  Map ISO 27001 controls to evidence and gaps. Use for certification readiness.
version: 1.0.0
source: in-repo
---

# compliance-iso27001

## Workflow

1. List the applicable Annex A controls for the scope.
2. For each control: current practice, owner, evidence, gap.
3. Classify gaps by risk and effort.
4. Collect dated evidence (policy, log, ticket, test).
5. Track remediation with owners and due dates.
6. Report readiness percentage per control family.

## Quality checks

- Every 'implemented' control has dated evidence.
- Gaps have an owner and a date.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
