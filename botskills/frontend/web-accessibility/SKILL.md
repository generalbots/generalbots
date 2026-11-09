---
name: web-accessibility
description: |
  Meet WCAG basics: semantics, focus order, labels, ARIA only when needed. Use for any user-facing UI.
version: 1.0.0
source: in-repo
---

# web-accessibility

## Workflow

1. Use native elements first (button, a, input, label) before ARIA roles.
2. Keyboard: every action reachable by Tab, visible focus, Esc closes overlays.
3. Forms: label tied to input, errors announced (aria-live), required marked in text.
4. Images: alt for informative content, empty alt for decorative.
5. Colour: contrast 4.5:1 body, 3:1 large/focus; never colour alone for meaning.
6. Check with the accessibility tree and a keyboard-only pass.

## Quality checks

- Full flow completes with keyboard only.
- No ARIA role that duplicates a native element.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
