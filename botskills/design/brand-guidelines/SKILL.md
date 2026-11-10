---
name: brand-guidelines
description: |
  Apply brand colours, typography, spacing and logo rules consistently across outputs. Use when the
  user names a brand or provides brand assets.
version: 1.0.0
source: in-repo
---

# brand-guidelines

## Workflow

1. Extract the design values: primary/secondary colours, type scale, spacing unit, logo clear space.
2. Map each token to the target tool (CSS variables, docx styles, pptx theme colours).
3. Logo: minimum size, clear space, never stretch, never recolour outside the allowed variants.
4. Document the tokens used at the end of the artefact or in a companion file.
5. Check contrast and legibility at final size.
6. If a token is missing, ask instead of inventing a colour.

## Quality checks

- One palette across every page/slide; no off-brand accent appears.
- Logo usage matches the clear-space rule.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
