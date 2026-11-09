---
name: migrations
description: |
  Write reversible, zero-downtime schema migrations. Use before changing a shared database.
version: 1.0.0
source: in-repo
---

# migrations

## Workflow

1. Expand → migrate → contract: add new shape, dual-write/backfill, switch reads, then remove old.
2. Add nullable columns or defaults that do not rewrite the table.
3. Create indexes CONCURRENTLY; avoid long locks.
4. Backfill in batches with checkpoints; never one huge transaction.
5. Each migration has a tested down path or a documented reason it cannot be reversed.
6. Deploy order: migration before code that needs it; removal after the code stops using it.

## Quality checks

- Rolling deploy never sees a missing column.
- Rollback is possible without data loss.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
