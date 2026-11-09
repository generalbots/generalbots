---
name: color-systems
description: |
  Build palettes, design tokens and contrast-safe pairs for UI and documents. Use for theming and
  accessibility-driven colour decisions.
version: 1.0.0
source: in-repo
---

# color-systems

## Workflow

1. Pick a base hue and build a 9-step scale (50-900) with consistent lightness deltas.
2. Define semantic tokens (surface, text, muted, accent, danger) on top of the scale.
3. Check pairs: body text ≥4.5:1, large text ≥3:1, focus ring ≥3:1 against both surfaces.
4. Provide light and dark variants derived from the same semantic names.
5. Output tokens as CSS variables and a JSON map.
6. Verify with a contrast calculator and report the ratios.

## Quality checks

- Every semantic pair passes its contrast target.
- No token name encodes a colour value (use semantic names).

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
