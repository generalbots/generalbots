---
name: news-briefing
description: |
  Produce a five-bullet briefing with sources and dates. Use for 'what happened' requests.
version: 1.0.0
source: in-repo
---

# news-briefing

## Workflow

1. Pick the 5 most consequential items for the user's context.
2. Each bullet: what happened, who it affects, why it matters.
3. Attach one source per bullet with its date.
4. Separate confirmed facts from reported/rumoured items.
5. Note what to watch next.
6. Keep the whole briefing under 200 words.

## Quality checks

- Every bullet has a dated source.
- Rumours are labelled as such.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
