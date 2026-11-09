---
name: translation
description: |
  Localise meaning-first with a glossary and consistent terminology. Use when content must be
  translated.
version: 1.0.0
source: in-repo
---

# translation

## Workflow

1. Confirm target locale and formality (pt-BR vs pt-PT, tu/você).
2. Build a glossary for product terms and keep it applied across the text.
3. Translate meaning and intent; keep numbers, dates and units in local format.
4. Do not translate code, identifiers, URLs or brand names.
5. Back-translate a sample to verify meaning.
6. Flag puns/cultural references that need adaptation rather than literal translation.

## Quality checks

- Terminology matches the glossary everywhere.
- Dates/currency use the target locale format.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
