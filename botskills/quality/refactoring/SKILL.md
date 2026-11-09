---
name: refactoring
description: |
  Change structure in small, behaviour-preserving steps. Use before adding features to tangled code.
version: 1.0.0
source: in-repo
---

# refactoring

## Workflow

1. Pin behaviour with tests (or characterization tests) first.
2. Make one mechanical change at a time; run tests after each.
3. Extract functions/modules by responsibility, not by line count.
4. Rename to domain language; update call sites in the same change.
5. Delete dead code instead of commenting it out.
6. Stop when the structure is clear; avoid a rewrite in disguise.

## Quality checks

- Test suite green after every step.
- No behaviour change mixed into a refactor commit.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
