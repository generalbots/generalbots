---
name: tailwind
description: |
  Style with Tailwind utilities and design tokens without ad-hoc overrides. Use for utility-first UI
  work.
version: 1.0.0
source: in-repo
---

# tailwind

## Workflow

1. Define tokens (colours, spacing, radii, fonts) in the theme; use them instead of arbitrary values.
2. Extract repeated class clusters into components, not @apply soup.
3. Responsive: mobile-first breakpoints; test the 360 px and 1440 px widths.
4. Dark mode via the same semantic tokens (bg-surface, text-muted), never a second palette.
5. Keep focus-visible styles; never remove outlines without a replacement.
6. Production build must purge unused classes; verify the CSS size.

## Quality checks

- No hardcoded hex colours in markup.
- Every interactive element has hover, focus and disabled styles.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
