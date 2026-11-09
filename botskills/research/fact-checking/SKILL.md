---
name: fact-checking
description: |
  Turn a claim into evidence and a verdict (supported, contested, unsupported). Use before repeating a
  statistic.
version: 1.0.0
source: in-repo
---

# fact-checking

## Workflow

1. Restate the claim precisely (who, what, when, how much).
2. Find the primary source: paper, filing, official dataset, not a summary.
3. Check the date and the population the number applies to.
4. Look for contradicting primary sources.
5. Verdict: supported / contested / unsupported, with one line of reasoning.
6. If no primary source exists, say so and do not repeat the claim as fact.

## Quality checks

- Verdict is backed by at least one primary source or explicitly marked unsupported.
- Numbers keep their original units and date.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
