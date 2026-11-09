---
name: document-a11y
description: |
  Make documents accessible: heading structure, alt text, table headers, contrast and reading order.
  Use when the output must meet WCAG/PDF-UA expectations.
version: 1.0.0
source: in-repo
---

# document-a11y

## Workflow

1. Headings: real heading styles in order; no fake headings made bold.
2. Images: descriptive alt text (what it shows, not 'image'); decorative images marked as decorative.
3. Tables: header row flagged, no merged cells used for layout, captions where helpful.
4. Contrast: body text at least 4.5:1, large text 3:1; check with the palette values.
5. PDF: tag the document (pdf with structure) and set the language attribute.
6. Report the checks performed and any item that could not be fixed.

## Quality checks

- A screen reader can navigate by headings and tables.
- No information conveyed by colour alone.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
