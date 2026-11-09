---
name: crm-actions
description: |
  Create and update contacts, deals and pipeline stages through the CRM API. Use for sales operations.
version: 1.0.0
source: in-repo
---

# crm-actions

## Workflow

1. Search before create: match by email/phone to avoid duplicates.
2. Create with the required fields and a source tag.
3. Move deals through valid stages only; record the reason for closed-lost.
4. Log activities (call, email, note) against the right record.
5. Respect tenant scoping: only the caller's org data.
6. Report the record id and what changed.

## Quality checks

- No duplicate contact created.
- Stage transitions are valid for the pipeline.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
