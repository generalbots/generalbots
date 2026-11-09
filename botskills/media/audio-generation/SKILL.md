---
name: audio-generation
description: |
  Generate music and ambience through BotModels jukebox with duration and mood parameters. Use for
  background audio.
version: 1.0.0
source: in-repo
---

# audio-generation

## Workflow

1. Specify mood, tempo, duration and whether vocals are allowed.
2. Call the jukebox endpoint and poll the job until it finishes.
3. Transcode to the target format/bitrate with the format endpoint when needed.
4. Check duration matches the requested length.
5. Deliver the file plus a one-line description of what was generated.
6. Do not bake copyrighted artist references into the prompt.

## Quality checks

- The audio file plays and its duration matches the request.
- Format/bitrate match the deliverable.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
