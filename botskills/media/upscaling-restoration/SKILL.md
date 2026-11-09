---
name: upscaling-restoration
description: |
  Enlarge or restore images without artefacts, within honest limits. Use for low-resolution assets.
version: 1.0.0
source: in-repo
---

# upscaling-restoration

## Workflow

1. Inspect the source: resolution, noise, compression artefacts, blur.
2. Upscale at most 2x per pass with a real upscaler; avoid naive resampling.
3. Denoise before sharpening, not after; keep skin/texture natural.
4. For text or line art, prefer vectorisation (SVG) over upscaling.
5. State the effective limit when the source is too low quality.
6. Compare 100% crops before/after and report what improved.

## Quality checks

- No halo or plastic skin at 100%.
- Claims match what the source can support.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
