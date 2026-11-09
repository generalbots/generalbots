---
name: customer-support
description: |
  Triage, answer with macros and escalate with context. Use for support queues.
version: 1.0.0
source: in-repo
---

# customer-support

## Workflow

1. Classify the request: question, bug, billing, access, churn risk.
2. Answer with the documented macro; personalise the first and last line.
3. If a bug: collect reproduction steps, environment, IDs and screenshots.
4. Set severity by impact and urgency; escalate with the collected context.
5. Follow up at the promised time, even if there is no resolution yet.
6. Close with what was done and what the customer should do next.

## Quality checks

- Every escalation has reproduction details.
- No promise is made that the team cannot keep.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
