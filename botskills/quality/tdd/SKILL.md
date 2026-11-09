---
name: tdd
description: |
  Drive changes with red-green-refactor tests. Use for new behaviour or bug fixes.
version: 1.0.0
source: in-repo
---

# tdd

## Workflow

1. Write the smallest failing test that expresses the behaviour.
2. Run it and confirm it fails for the right reason.
3. Implement the minimum to pass; no speculative features.
4. Refactor with tests green; keep behaviour unchanged.
5. Add edge cases the bug exposed (empty, boundary, error path).
6. Commit tests with the change.

## Quality checks

- The test fails before the fix and passes after.
- No test asserts implementation details that will churn.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
