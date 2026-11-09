---
name: docx
description: |
  Create and edit Word documents: styles, TOC, page setup, headers and footers, tracked changes. Use
  whenever the user asks for a .docx, a report, a letter, a résumé or a formatted document.
version: 1.0.0
source: in-repo
---

# docx

## Workflow

1. Generate with python-docx: set Normal/Heading styles before writing so the TOC and navigation pane work.
2. Page setup first: A4 (21 x 29.7 cm), 2.5 cm margins, then body text at 11 pt with 1.15 spacing.
3. Headings: exactly one H1 (document title), then H2/H3 in order; never skip a level.
4. Tables: header row bold with repeat-on-page-break; column widths set explicitly, not auto-fit.
5. Headers/footers: title and page X of Y via field codes; keep the footer outside the content area.
6. Save to output/ and verify the file opens (unzip word/document.xml) before reporting success.

## Quality checks

- Open the saved file and confirm heading order, page count and that the TOC field is present.
- No placeholder lorem text, no empty runs, no style overrides that break the theme.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
