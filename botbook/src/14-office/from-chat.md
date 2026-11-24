# Creating office files from chat

The assistant in Chat (and on the other chat channels) can create office files by calling verbs from the command catalog. A verb is a named action with named parameters; the server maps it to a registered API route, performs the action and returns a result that includes a deep link to the relevant app. You do not need to know the verb names to use them — describe what you want in ordinary language. The names below are useful when reading the assistant's reply or when troubleshooting why a file did not open.

## A spreadsheet from a request

A request such as "create a spreadsheet with monthly revenue and a total row" is served by the sheet verbs. The most common ones are:

| Verb | What it does | Parameters you will see in the call |
|---|---|---|
| `sheet.create` | Creates a new spreadsheet from rows and opens it in Sheet | `title`, `rows` (a JSON array of rows; values or `=formulas`), optional `styles`, `formats`, `conditional`, `chart` |
| `sheet.cells.set` | Writes values or formulas into a range of an existing spreadsheet | `file_id`, `range`, `values`, plus the same optional styling parameters |
| `sheet.cell.set` / `sheet.range.set` / `sheet.formula.set` | Write one value, a block of values or a formula into a cell | `file_id`, `cell` or `range`, `value` / `values` / `formula` |
| `sheet.chart.create` | Adds a bar, line or pie chart from a range | `file_id`, `range`, `type` |
| `sheet.import` | Turns a CSV/XLSX/ODS file from Drive (for example an attached `inbox/data.csv`) into an openable spreadsheet | `path` |

Formatting is applied in the **same call** as the values: `styles` maps an A1 reference to attributes such as font weight, colour and alignment; `formats` maps a reference to a number-format code; `conditional` carries rules such as "negative values in red". A balance row should be written as a formula so later edits recalculate. When a chart is requested, the chart description is passed with the values rather than inserted separately.

For an existing workbook, the assistant can also sort a range (`sheet.sort`), apply a filter (`sheet.filter.apply`), set data-validation rules (`sheet.data_validation.set`), apply conditional formatting (`sheet.conditional_format.apply`) and build a pivot table from a source range that includes the header row (`sheet.pivot.create`).

## A letter or other document

Two verbs cover documents:

- `docs.create` creates a document from a title and a markdown body — useful when the content of the letter or report is already known.
- `docs.new` starts from a template. The template names are `blank`, `letter`, `meeting` and `report`. The **letter** starter contains the usual placeholders — your name and address, date, recipient name and address, salutation, opening/body/closing paragraphs and signature — ready to fill in the editor.

The document verbs also include `docs.get` / `docs.save` / `docs.update` to read and write content, `docs.toc.generate` and `docs.outline` for structure, `docs.comment.add` for anchored comments, `docs.ai` for an AI edit, and `docs.export` for a downloadable file.

`docs.open_from_drive` takes a Drive object key **and** its `bucket`; the Docs loader needs both. A path on its own is not enough.

## A presentation

`slides.create` takes a deck title and an optional JSON array of slides with `title` and `body`. `slides.slide.add` inserts a slide at a position with a `title`, `content` or `blank` layout. Media and theme changes have their own verbs, and the Present action starts the deck full screen in the Slides editor.

## The "open" buttons in the reply

Every office verb declares a deep link:

- `docs.create` → `app://docs`
- `docs.new` → `app://docs`
- `sheet.create` → `app://sheet`
- `sheet.cells.set` → `app://sheet?file_id=…`
- `slides.create` → `app://slides`
- `slides.slide.add` → `app://slides?file=…`

When a command runs, the server fills in the placeholders from the real result (for example, the new spreadsheet id) and attaches the link to the response. The chat renderer turns that link into a button that opens the app window in the desktop shell with that record loaded. A few safeguards apply:

- Only apps present in the published app catalog, with at least one of their declared record keys, become buttons. An invented link in a reply stays plain text.
- Links inside code blocks are not converted, so examples in a code block remain readable.
- A bare `app://docs` or `app://sheet` is enough to open the app; an id is only required to open a specific file.
- Deep links also work when the target app is already open: the shell dispatches the request to the existing window and the app loads the new file there.

If a button is missing, the most likely cause is that the command did not return a real id, or that the link used a key the app does not declare.

## Exporting from the conversation

Export verbs return a file rather than opening an editor:

| Verb | Formats | Notes |
|---|---|---|
| `docs.export` | PDF, DOCX, ODT, Markdown (md), HTML, text (txt) | Docs has a separate route per format; an unsupported format is rejected with the list of valid ones |
| `sheet.export` | XLSX, CSV, PDF | The chat verb uses the spreadsheet's id and the requested format |
| `slides.export` | Presentation export through the Slides API | Available as a catalog verb on the Slides app |

The export menus in Docs and Sheet do the same job for a file you already have open; see [Working in the editors](./editors.md). Exported XLSX and DOCX files are subject to the fidelity limits described in [Office Fidelity Contract](./../07-user-interface/apps/office-fidelity.md).

## A typical exchange

> **You:** Create a cash-flow spreadsheet with January, February and March, a total row and a bar chart.
>
> **Assistant:** creates the workbook with the three months as columns, a total row written as `=SUM(...)`, a bold header, a number format on the value cells, a red fill on negative values and a chart from the data range. The reply includes an **Open Sheet** button.
>
> **You:** Add April and update the chart.
>
> **Assistant:** calls `sheet.cells.set` on the existing file id (or a range write), then regenerates the chart, and the reply points back to the same workbook.
>
> **You:** Export it as PDF.
>
> **Assistant:** calls `sheet.export` with the file id and `pdf`.

The assistant may ask for clarification before creating the file if the data or the title is ambiguous. Chat-created files are not magic documents: they are ordinary Docs, Sheet and Slides records and can be edited in the app afterwards.

## See Also

- [Working in the editors](./editors.md)
- [Current limitations](./current-limitations.md)
- [AI Sheet Cache](./../03-knowledge-ai/ai-sheet-cache.md) — the `BOT_AI_PROMPT` cell function
- [Sheet - Spreadsheets](./../07-user-interface/apps/sheet.md)
