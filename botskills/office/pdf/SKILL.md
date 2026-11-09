---
name: pdf
description: |
  Read, merge, split, rotate, fill forms and extract tables or text from PDFs. Use for any PDF
  manipulation or extraction request.
version: 1.0.0
source: in-repo
---

# pdf

## Workflow

1. Inspect with pypdf: page count, page size, encryption and text layer presence.
2. Extraction: pdftotext -layout or pdfplumber for tables; keep the table structure instead of flat lines.
3. Merge/split: preserve bookmarks and page order; write to a new file, never overwrite the source.
4. Forms: list field names first, fill by name, then flatten only when the user asks.
5. OCR only when there is no text layer; say so explicitly and keep the OCR text beside the original page number.
6. Verify: page counts before/after, spot-check the first and last page of the result.

## Quality checks

- Source files stay untouched; output is a new file in output/.
- A scanned PDF is reported as scanned rather than silently returning empty text.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
