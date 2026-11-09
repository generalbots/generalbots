---
name: gif-creator
description: |
  Make short loops from frames or a video segment with a small file size. Use for animated previews.
version: 1.0.0
source: in-repo
---

# gif-creator

## Workflow

1. Choose the segment: 2-6 seconds, one action, loop-friendly start/end.
2. Resize to the target width (≤600 px for chat) and 10-15 fps.
3. Build a palette from the frames for clean colours.
4. Optimise with gifsicle; target <2 MB.
5. Check the loop seam: no jump at the loop point.
6. Deliver the file and the frame count/duration.

## Quality checks

- Loop is seamless.
- File size fits the destination limit.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
