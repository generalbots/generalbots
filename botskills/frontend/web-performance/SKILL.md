---
name: web-performance
description: |
  Hit Core Web Vitals with budgets for JS, images and fonts. Use when a page feels slow.
version: 1.0.0
source: in-repo
---

# web-performance

## Workflow

1. Measure first: LCP, CLS, INP on a production build (Lighthouse + real network).
2. Images: correct dimensions, modern formats, lazy-load below the fold, priority hint for the LCP image.
3. JS: split by route, defer non-critical work, remove duplicate libraries.
4. Fonts: subset, preload the one used above the fold, font-display: swap.
5. Cache: content-hashed assets with long max-age; HTML short-lived.
6. Re-measure after each change and record before/after.

## Quality checks

- LCP under 2.5 s, CLS under 0.1 on the target connection.
- No layout shift from images without dimensions.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
