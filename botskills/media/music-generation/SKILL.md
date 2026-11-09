---
name: music-generation
description: |
  Generate music with explicit mood, tempo, key and length parameters. Use for background tracks.
version: 1.0.0
source: in-repo
---

# music-generation

## Workflow

1. Write the brief: genre, mood, tempo (BPM), instrumentation, length, vocals/no vocals.
2. Keep the prompt free of artist names; describe the sound instead.
3. Generate, then check the tempo and that the ending is not abrupt.
4. For loops, request a seamless loop or trim on a beat.
5. Transcode to the delivery format.
6. Name the file with mood/tempo for reuse.

## Quality checks

- Duration and tempo match the brief.
- No copyrighted melody is recognisable.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
