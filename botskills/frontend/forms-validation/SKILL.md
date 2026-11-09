---
name: forms-validation
description: |
  Build forms with client validation that mirrors server rules and clear errors. Use for any input
  flow.
version: 1.0.0
source: in-repo
---

# forms-validation

## Workflow

1. Validate on blur and submit, not on every keystroke.
2. One error message per field, placed next to it, tied with aria-describedby.
3. Server is the source of truth: map its field errors back to inputs.
4. Disable submit only while submitting, never for invalid input (explain why instead).
5. Preserve input on error; never reset the form on a failed submit.
6. Success state is explicit: what happened and what comes next.

## Quality checks

- Keyboard-only submit works; errors are announced.
- Server validation error appears on the matching field.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
