---
name: prisma
description: |
  Model schemas and data access with Prisma. Use for TypeScript projects using Prisma.
version: 1.0.0
source: in-repo
---

# prisma

## Workflow

1. Schema first: relations, indexes and unique constraints explicit.
2. Use select/include to fetch only needed fields; avoid over-fetching.
3. Transactions for multi-step writes (prisma.$transaction).
4. Migrations are versioned and reviewed; never db push in production.
5. Guard N+1 with relation load strategy or batched queries.
6. Seed data idempotent (upsert by natural key).

## Quality checks

- No query returns fields the caller does not use.
- Migration history is linear and applies cleanly to an empty DB.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
