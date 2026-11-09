---
name: canvas-design
description: |
  Create posters and social art on the canvas with a clear grid, hierarchy and export sizes. Use for
  visual assets that are not slide decks.
version: 1.0.0
source: in-repo
---

# canvas-design

## Workflow

1. Choose the target format first (1:1, 4:5, 16:9) and set the artboard to the exact pixel size.
2. Establish a 12-column grid and one focal point; keep 8 px spacing increments.
3. Hierarchy: headline (largest), supporting line, then details; maximum two typefaces.
4. Colour: one dominant, one accent, neutrals for text; check contrast.
5. Export PNG at 2x and SVG for vector reuse; name files by campaign/format.
6. Verify at thumbnail size: the headline must still be readable.

## Quality checks

- No element touches the safe margin; no clipped text.
- Every export has the requested dimensions.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
