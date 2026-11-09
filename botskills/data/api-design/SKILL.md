---
name: api-design
description: |
  Design REST/JSON contracts with stable versioning, pagination and honest errors. Use when exposing
  or consuming an API.
version: 1.0.0
source: in-repo
---

# api-design

## Workflow

1. Resources are nouns, actions are HTTP verbs; status codes match the outcome.
2. Errors: machine-readable code, human message, field details; never a bare 500 for validation.
3. Pagination: cursor for large/ordered sets, limit+offset only for small stable sets.
4. Versioning: additive changes first; breaking changes behind a new version.
5. Idempotency keys for unsafe retries (POST create, payments).
6. Document request/response examples and the error table.

## Quality checks

- Every failure path has a documented code.
- Retrying a request with the same idempotency key is safe.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
