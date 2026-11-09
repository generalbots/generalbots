---
name: svg-icons
description: |
  Design clean SVG icon sets with consistent grid, stroke and optical weight. Use when icons are
  needed for UI or documents.
version: 1.0.0
source: in-repo
---

# svg-icons

## Workflow

1. Draw on a 24x24 grid with 2 px stroke and round caps; keep a 1 px optical margin.
2. Use currentColor so icons inherit text colour; no embedded fills unless intentional.
3. Name icons by function (search, download), not by shape.
4. Provide an outlined set; add filled variants only when requested.
5. Optimise with svgo and strip editor metadata.
6. Verify at 16, 24 and 48 px: corners must not smear.

## Quality checks

- All icons share stroke width and grid.
- No raster images or base64 data inside the SVG.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
