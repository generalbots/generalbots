---
name: office-convert
description: |
  Round-trip between CSV/JSON/Markdown and docx/xlsx/pptx while preserving structure. Use for format
  conversion requests.
version: 1.0.0
source: in-repo
---

# office-convert

## Workflow

1. Identify the source structure (table, sections, slides) before choosing a target mapping.
2. CSV/JSON → xlsx: one sheet per table, header row, typed columns.
3. Markdown → docx: headings, lists, code blocks and tables mapped to real styles.
4. → pptx: one slide per H2, bullets from lists, notes from blockquotes.
5. Preserve encoding (UTF-8) and line breaks; never flatten a table into prose.
6. Verify by parsing the output back into a data structure and comparing row/column counts.

## Quality checks

- Round-trip preserves counts and values; lossy fields are listed explicitly.
- No mojibake and no lost leading zeros in identifier columns.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
