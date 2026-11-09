---
name: image-editing
description: |
  Crop, blur or remove backgrounds, correct exposure and upscale images without artefacts. Use when an
  existing image must be adjusted for a deliverable.
version: 1.0.0
source: in-repo
---

# image-editing

## Workflow

1. Work on a copy; keep the original untouched in output/.
2. Crop to the target aspect ratio before resizing; never stretch.
3. Background removal: keep edges clean (feather 1-2 px), preserve hair/fine detail.
4. Colour/exposure: adjust in small steps and keep skin tones neutral.
5. Upscale no more than 2x with a real upscaler; otherwise report the limit.
6. Verify at 100%: no halo, no clipped highlights, no jpeg banding.

## Quality checks

- The result has the requested dimensions and colour space (sRGB).
- No visible artefacts at full resolution.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
