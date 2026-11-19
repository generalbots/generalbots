# =============================================================================
# General Bots — en — Sheets grid accessibility (#2493)
# English is the source of truth: every key must be mirrored in all locales.
# Placeholders ({value}, {range}, ...) are preserved by the FTL loader.
# =============================================================================

# Grid semantics
sheet-a11y-grid-label = Spreadsheet grid
sheet-a11y-rowgroup-label = Grid rows

# Formula bar and name box
sheet-a11y-formula-bar-label = Formula bar
sheet-a11y-name-box-label = Name box, enter a cell reference
sheet-a11y-name-box-placeholder = A1
sheet-a11y-invalid-reference = Invalid cell reference: {value}

# Cell labels and live announcements
sheet-a11y-cell-label = Cell {address}: {value}
sheet-a11y-cell-formula-label = Formula {formula}
sheet-a11y-live-selection = {range}, {rows} rows by {cols} columns, sum {sum}
sheet-a11y-live-edited = Cell {address} updated to {value}
sheet-a11y-clear-too-large = Selection too large to clear with the keyboard

# Accessibility checker
sheet-a11y-checker-title = Accessibility check
sheet-a11y-checker-close = Close accessibility check
sheet-a11y-checker-run = Check accessibility
sheet-a11y-checker-summary = {count} accessibility issues found
sheet-a11y-checker-empty = No accessibility issues found.
sheet-a11y-checker-sheet = Sheet
sheet-a11y-checker-sheet-fallback = Sheet {index}
sheet-a11y-severity-warning = Warning
sheet-a11y-finding-merged = Merged cells at {range} can confuse screen readers; avoid merging in accessible workbooks.
sheet-a11y-finding-color-only = Conditional format at {range} uses colour only; add an icon, label or pattern.
sheet-a11y-finding-no-alt = Image {name} has no alternative text.
sheet-a11y-unnamed-image = unnamed
sheet-a11y-finding-no-header = Worksheet {sheet} has data but no header row.
