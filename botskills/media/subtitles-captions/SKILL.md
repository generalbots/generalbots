---
name: subtitles-captions
description: |
  Produce SRT/VTT captions from audio with timing and speaker labels. Use for video accessibility.
version: 1.0.0
source: in-repo
---

# subtitles-captions

## Workflow

1. Transcribe with word timestamps, then align to sentences (max 2 lines, 42 chars/line).
2. Split at natural pauses; never split mid-phrase.
3. Keep reading speed 15-20 chars/second.
4. Mark speakers when more than one; use VTT cues for positioning only if needed.
5. Validate the file parses (SRT/VTT) and timings are monotonic.
6. Deliver the caption file and mention the language.

## Quality checks

- No cue overlaps the next.
- Captions stay within the safe area.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
