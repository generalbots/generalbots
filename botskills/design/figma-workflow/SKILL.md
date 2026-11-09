---
name: figma-workflow
description: |
  Organise Figma frames, components and exports (.fig/png/svg). Use when the user references Figma or
  asks for a design file structure.
version: 1.0.0
source: in-repo
---

# figma-workflow

## Workflow

1. One page per flow; frames named by screen and state.
2. Build components with variants for state; no detached copies.
3. Use auto-layout for spacing; constraints for responsive frames.
4. Export at 1x/2x PNG plus SVG for icons; keep the .fig as the editable source.
5. Record a handoff note: frame → asset → owner.
6. Verify exports open and that icons remain crisp at 24 px.

## Quality checks

- No ungrouped floating layers in exported frames.
- Every exported asset has a matching frame name.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
