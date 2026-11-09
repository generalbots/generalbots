---
name: voice-tts
description: |
  Narrate document sections with TTS and natural pacing. Use for audio versions of documents.
version: 1.0.0
source: in-repo
---

# voice-tts

## Workflow

1. Split the text by section; keep sentences under ~25 words for natural pacing.
2. Use BotModels TTS with the requested voice; keep pronunciation hints for names/acronyms.
3. Insert short pauses between sections; avoid one giant utterance.
4. Match the narration language to the text, not the UI language.
5. Assemble the sections in order with consistent volume.
6. Verify the audio duration is plausible for the word count (~150 wpm).

## Quality checks

- No truncation at the end of a section.
- Pronunciation of proper nouns matches the provided hints.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
