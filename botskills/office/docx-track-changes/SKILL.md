---
name: docx-track-changes
description: |
  Review, accept or reject tracked changes in a Word document and produce a clean copy. Use for
  contract redlines and edited drafts.
version: 1.0.0
source: in-repo
---

# docx-track-changes

## Workflow

1. Extract the revisions from word/document.xml (w:ins / w:del) and list author, date and text per change.
2. Summarise by author and clause so the reviewer can decide in one pass.
3. Apply decisions by keeping or dropping the revision nodes; never delete surrounding runs.
4. Write the clean copy and a change log (who, what, accepted/rejected).
5. Verify: the clean document has no remaining w:ins/w:del and the text matches the accepted set.

## Quality checks

- Nothing is silently accepted: every change is listed with its decision.
- Original file remains untouched; output names the version.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
