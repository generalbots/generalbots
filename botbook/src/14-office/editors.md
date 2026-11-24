# Working in the editors

This page covers the parts of the office editors you use after a file exists — whether it was created in chat, opened from Drive or started from the app menu. The full capability lists for each app are in the app chapters linked at the end.

## Opening a generated file

All three editors are Suite apps that run inside the desktop shell. Open them from the app menu (Preview mode must be on), click the open button in a chat reply, or open a file from Drive. They all accept the same deep-link mechanism, so the chat and Drive routes converge on the same editor.

### Docs

A Docs window reads its target from the URL, from `window.__gbAppParams__` or from a `gb:deep-link` event sent by the shell. The useful parameters are:

- `file` — the document id or Drive object key.
- `bucket` — the Drive bucket that contains the file. Supply it together with `file` when opening a Drive object.
- `vmConvId` and `vmPath` — an agent-workspace file. In this mode the server converts the file to a preview, the editor sanitizes the HTML before displaying it and uses the file name as the document title.

When none of these are present, Docs opens a blank document.

### Sheet

A Sheet window accepts:

- `file_id` (or `id`) — loads that workbook; an optional `cell` jumps straight to an A1 cell. This is the form used by chat deep links.
- `bucket` **and** `file` — asks `/api/sheet/load-from-drive` to load the Drive object, then opens the returned workbook.

If a deep link names no workbook, or the Drive load fails, the editor reports the problem instead of silently leaving an unrelated workbook on screen.

### Slides

A Slides deep link opens the deck in the editor. The **Present** control starts the full-screen presentation; the presenter notes, timer and audience panels are part of the deck workflow.

## Editing cells

In Sheet, select a cell and double-click it, press `F2`, or just start typing to replace the selection. The cell enters edit mode with its caret at the end; if the cell holds a formula, the formula text is shown for editing rather than the computed value. `Enter` commits and moves down, `Tab` commits and moves right, and `Esc` abandons the edit. Arrow keys move the selection, `Ctrl+C`/`Ctrl+V`/`Ctrl+X` copy, paste and cut, `Ctrl+Z`/`Ctrl+Y` undo and redo, and `Ctrl+D`/`Ctrl+R` fill down or right. Formatting applies to the whole selected range.

The same write paths are available from chat for automation: `sheet.cell.set`, `sheet.range.set` and `sheet.formula.set` write values and formulas, while `sheet.format.apply`, `sheet.merge` / `sheet.unmerge` and `sheet.conditional_format.apply` change presentation. Values and styling sent together through `sheet.create` or `sheet.cells.set` are applied in one operation, which is the reliable way to produce a formatted table from a conversation.

XLSX files keep their original package on save-back: only edited cells are rewritten, so features the workbook model cannot represent are not destroyed by an edit round-trip.

## Data validation

Sheet ships a data-validation rule manager, reachable from the **Data Validation** toolbar button and from the **Validations** panel in the sidebar.

To add a rule, enter an A1 range (for example `A1:C10`), choose a type — **whole number**, **decimal**, **list** or **date** — and an operator: any value, between, greater/less than (with or without equality), equal or not equal. Supply the value and, for *between*, the second bound; for a list, type the allowed values separated by commas; and optionally enter an error message. Use **Add** to keep the rule, **Apply** to re-evaluate the sheet, **Clear** to remove the highlighting and **Remove** to delete a rule from the list.

Rules are listed with their id, range and type. Cells that violate a rule are marked on the grid, and a marker stays with the cell as the grid scrolls and re-renders. The chat verb `sheet.data_validation.set` defines the same kind of rule on a range when you ask for it in a conversation.

> **Current limitation:** validation in the browser is a local aid. It highlights values and carries an error message, but it is not a server-enforced write restriction.

## Outlines and sorting

### Document outline

Docs has a navigation pane with the document's heading tree, plus bookmarks and drag-to-reorder. The `docs.outline` verb returns the headings, and `docs.toc.generate` builds a table of contents from them. The editor's Document structure workspace also handles automatic numbering, captions and cross-reference fields.

### Slide sorter

Slides has a slide sorter panel. It lists the deck's slides with stable ids and titles, and reorders them by pointer drag or with the **↑** and **↓** buttons on each row — the buttons are the reliable fallback when dragging is awkward. Rows renumber after a move, the thumbnail rail is updated to match the panel order, and the currently selected slide stays selected. The panel follows the Slides window when the desktop re-targets it.

The sorter is a view-level tool: reordering slides here changes the editor's slide order and thumbnails, not a server-side document. The structural outline, sections, find/replace and speaker notes live in the Slides app itself.

### Sorting a range

In Sheet, the column-header sort menu sorts ascending (A→Z) or descending (Z→A) over the selection or the populated region. The `sheet.sort` verb does the same by range, column and direction. **Current limitation:** formulas that reference cells moved by a sort are not rewritten.

## Exporting from the toolbar

### Docs

The **Export** menu downloads the active document as DOCX, ODT, PDF, HTML, Markdown or plain text. If the document has no stored copy yet, the editor content is exported instead.

### Sheet

The **Export** menu offers CSV, XLSX, Markdown, ODS, PDF, HTML and JSON. CSV output contains unformatted values. PDF export produces a page set per worksheet with a repeated header row.

### Slides

Presentation export is available through the Slides API and the catalog's `slides.export` verb. The editor's present/print workflow stays inside the deck rather than producing another office format.

## See Also

- [Creating office files from chat](./from-chat.md)
- [Current limitations](./current-limitations.md)
- [Docs - AI Writing](./../07-user-interface/apps/docs.md)
- [Sheet - Spreadsheets](./../07-user-interface/apps/sheet.md)
- [Slides - Presentations](./../07-user-interface/apps/slides.md)
