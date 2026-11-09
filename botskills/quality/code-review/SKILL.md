---
name: code-review
description: |
  Review for correctness, security and readability, with actionable comments. Use on any diff.
version: 1.0.0
source: in-repo
---

# code-review

## Workflow

1. Read the change against the requirement; does it do the whole job?
2. Correctness: edge cases, error paths, concurrency, data loss.
3. Security: input validation, authorization, secrets, injection.
4. Readability: naming, size, dead code, comments that explain why.
5. Check tests cover the new behaviour and the failure path.
6. Sort comments: blocking, should-fix, nit.

## Quality checks

- Every blocking comment names the concrete risk.
- No style bikeshedding without project rules behind it.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
