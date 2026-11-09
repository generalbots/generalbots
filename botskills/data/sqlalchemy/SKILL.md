---
name: sqlalchemy
description: |
  Use SQLAlchemy sessions, models and queries without N+1 or stale state. Use for Python database
  work.
version: 1.0.0
source: in-repo
---

# sqlalchemy

## Workflow

1. One session per request/task; close it in a finally or context manager.
2. Eager-load relationships used by the response (selectinload/joinedload).
3. Bulk operations with bulk_insert_mappings/execute, not per-row ORM calls.
4. Keep business logic out of models; use repositories/services.
5. Use migrations (Alembic) for schema, never create_all in production.
6. Log slow queries and check for N+1 in the request path.

## Quality checks

- No lazy load inside a loop.
- Session rollback on error; no leaked connections.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
