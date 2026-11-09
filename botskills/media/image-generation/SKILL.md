---
name: image-generation
description: |
  Generate or edit images from a prompt with the right aspect ratio and style. Use for illustrations
  and visuals.
version: 1.0.0
source: in-repo
---

# image-generation

## Workflow

1. Write the prompt with subject, style, composition, lighting and aspect ratio.
2. Use BotModels image generation; prefer the provider over local models.
3. For edits, keep the subject identity and change only the requested part.
4. Check text rendering: generated text is unreliable, add text in a vector/design tool instead.
5. Export at the size the deliverable needs; keep the original.
6. Verify the result has no artefacts, extra limbs or watermark.

## Quality checks

- Aspect ratio matches the requested format.
- No garbled text baked into the image.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
