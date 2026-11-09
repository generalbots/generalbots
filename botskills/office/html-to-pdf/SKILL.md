---
name: html-to-pdf
description: |
  Print HTML to PDF with headless Chromium, including headers, footers and page numbers. Use when the
  source is a web page or an HTML template.
version: 1.0.0
source: in-repo
---

# html-to-pdf

## Workflow

1. Render at A4 with 12 mm margins; set @page CSS for size, margins and page-break rules.
2. Use `--headless=new --print-to-pdf` with `--no-pdf-header-footer` unless a header/footer is requested.
3. Wait for fonts and images: `--virtual-time-budget=5000` or wait for network idle before printing.
4. Break pages deliberately: `break-inside: avoid` on cards and tables rows.
5. Screenshots for visual checks: `--screenshot --window-size=1200,1600`.
6. Verify: page count, embedded fonts, and that no table is cut across pages mid-row.

## Quality checks

- No missing glyphs (fonts installed: Noto + DejaVu in the office profile).
- Links are clickable and the document opens without a Chromium dialog.

## Output

Write the artefact to `output/` and report its path plus one line on what it contains. Never claim a result you did not open and check.
