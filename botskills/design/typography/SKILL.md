---
name: typography
description: |
  Choose and pair typefaces, set a scale and keep readable measure across deliverables. Use when
  text-heavy output looks cramped or inconsistent.
version: 1.0.0
source: in-repo
---

# typography

## Workflow

1. Pick one display and one text family; keep the pairing contrast obvious but safe.
2. Build a modular scale (1.25 or 1.333) with 6-8 steps; body 16-18 px web / 11 pt print.
3. Line height: 1.4-1.6 for body, 1.1-1.2 for display; measure 45-75 characters.
4. Use weight and size for hierarchy before colour.
5. Define tokens (font-family, sizes, line heights, letter spacing).
6. Check the longest heading and the smallest caption at real size.

## Quality checks

- No text below 9 pt print / 12 px web for body content.
- Headings do not overflow at the narrowest supported width.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
