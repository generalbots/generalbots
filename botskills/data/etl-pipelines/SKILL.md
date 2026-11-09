---
name: etl-pipelines
description: |
  Build extract/transform/load jobs with validation, checkpoints and idempotency. Use for data
  movement.
version: 1.0.0
source: in-repo
---

# etl-pipelines

## Workflow

1. Extract with a stable cursor/watermark, not 'everything every time'.
2. Validate schema and ranges on arrival; quarantine bad rows with reasons.
3. Transform is pure and tested on samples; no hidden side effects.
4. Load with upsert by natural key inside a transaction per batch.
5. Checkpoint progress so a rerun resumes instead of duplicating.
6. Report counts: read, valid, rejected, upserted, skipped.

## Quality checks

- Re-running a completed batch is a no-op.
- Row counts reconcile: read = loaded + rejected.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
