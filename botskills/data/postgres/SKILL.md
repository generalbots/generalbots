---
name: postgres
description: |
  Design schemas, indexes and migrations for PostgreSQL, and read EXPLAIN. Use for database modelling
  or slow queries.
version: 1.0.0
source: in-repo
---

# postgres

## Workflow

1. Model normalised first (3NF), denormalise only for measured read paths.
2. Primary keys: UUID or bigint sequence on purpose; every FK indexed.
3. Indexes: btree for equality/range, partial for filtered queries, covering with INCLUDE when it removes heap reads.
4. Read EXPLAIN (ANALYZE, BUFFERS) and fix the node with the highest actual time.
5. Migrations: additive first (add nullable column, backfill, then enforce); never rewrite a table in place.
6. Use transactions and lock timeouts for data changes.

## Quality checks

- Query plan shows an index scan where expected, no sequential scan on large tables.
- Migration is reversible and safe to run online.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
