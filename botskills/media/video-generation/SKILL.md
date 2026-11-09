---
name: video-generation
description: |
  Build slideshow/caption videos from images or slides, with timing and narration. Use for short
  narrated videos.
version: 1.0.0
source: in-repo
---

# video-generation

## Workflow

1. Collect the assets in order and the script per segment.
2. Set per-image duration from the narration length, not a fixed 3 s.
3. Burn captions only when requested; otherwise ship a sidecar SRT.
4. Use consistent transitions and no zoom/pan on text-heavy slides.
5. Render to MP4 H.264 + AAC, 1080p unless specified.
6. Verify duration, audio sync and that the first frame is not black.

## Quality checks

- Audio and captions align at the segment boundaries.
- The video plays in a browser without a plugin.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
