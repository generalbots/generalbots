---
name: pptx
description: |
  Create presentations with layouts, speaker notes and a consistent theme. Use for decks, pitches,
  training material and any .pptx output.
version: 1.0.0
source: in-repo
---

# pptx

## Workflow

1. Pick one 16:9 master and 3-4 layouts (title, section, content, image+caption); reuse them for every slide.
2. One idea per slide, max 6 bullets and 12 words per bullet; move detail into speaker notes.
3. Images: full-bleed on image slides, aspect ratio preserved, no stretching.
4. Speaker notes on every slide: what to say, not what is written.
5. Consistent typography: title 32-40 pt, body 18-24 pt, footer 10 pt.
6. Verify: reopen the deck, count slides, confirm notes exist and no text box overflows its placeholder.

## Quality checks

- No slide has unreadable text at thumbnail size; no lorem ipsum.
- Theme colours come from one palette; no per-shape hardcoded colours outside it.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
