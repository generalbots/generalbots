# Sheet — Desktop Spreadsheet Parity Plan 🟡 BETA

> **What is missing, why it matters, and the order in which it is being fixed.**

This page is the reference for anyone deciding whether to trust Sheet with a real workbook, and for anyone implementing the work. It is deliberately blunt: a spreadsheet that is almost right is worse than one that is obviously incomplete, because a wrong number looks exactly like a right one.

Tracking issue: [generalbots/generalbots#780](https://github.com/generalbots/generalbots/issues/780).

> **Progress (2026-08-10):** the frontend now ships a real `SheetAdvanced` module set —
> range selection, clipboard, undo/redo, fill handle, worksheet tab bar, live status bar,
> column-header sort, CSV/XLSX/Markdown export, conditional-format rendering, validation
> dots + list dropdown editor, and locale loading. Backend-heavy gaps (typed cells, parser,
> calc engine, number formats, xlsx round-trip, sessions, collab protocol) are unchanged.
>
> **Progress (2026-08-10, second pass):** the frontend now speaks A1 row/col addressing to
> the collab socket instead of the legacy `row*26+col` position encoding (gap 25), columns
> and rows resize interactively by dragging header edges with persistence through
> `/api/sheet/resize` (gap 11), and `Ctrl`+click / `Ctrl`+drag accumulate multi-ranges whose
> overlays render simultaneously (gap 12). The JS module test suite moved out of the served
> static tree (issue #809), then grew to 70 assertions covering core, clipboard, validation,
> widths, freeze, filters, formula fill, paste/i18n, undo, notes, charts, multi-range and
> drag-resize. The plan phases below mark what is genuinely shipped; gaps still open are
> labelled honestly.
>
> **Progress (2026-08-10, third pass):** the column count reached the desktop standard —
> 16,384 columns (`DEFAULT_TOTAL_COLS`) with full header/scroll virtualization, so only the
> visible column window is laid out and fetched (gap 10; canvas rendering remains the only
> unfinished part of issue 6). The collab protocol is now server-authoritative: `edit` and
> `cell_update` messages are applied to the session under one write guard, record an oplog
> entry via the session, and are stamped with a monotonically increasing `seq`; reconnecting
> clients replay the `GET /api/sheet/:id/ops?since=N` log through `onEdit` before trusting
> the live feed (gap 25 fully closed). Document listing is metadata-only: every save writes
> a `.meta.json` sidecar with ACLs, and the list endpoint reads sidecars instead of
> deserialising whole workbooks (gap 24). `.xlsx` import now maps workbook defined names and
> per-sheet protection into the model. The JS suite is at 76 assertions and the
> `botsheet-core` engine tests at 85, all green.
>
> **Progress (2026-08-15, large-sheet + fidelity pass):** the save-back path is zip-level
> preserve-and-passthrough (only `<c>` cells and the workbook sheet list are rewritten;
> charts/pivots/validation/images/macros are copied byte-for-byte), so the original `.xlsx`
> is never overwritten (E1–E5, E7 landed). Cell hyperlinks now import into the model. Two
> large-sheet fixes landed: (1) the import loop now iterates umya's actual-cell map instead
> of the `1..=max_row × 1..=max_col` bounding box — a sheet with a cell at row 1,000,000
> no longer scans a million empty rows — and (2) `source_bytes` serializes as base64 rather
> than a JSON number array. Calculation fidelity: `format_number` no longer saturates on
> `as i64` for magnitudes beyond i64, and typed operators now propagate the original error
> (`=1/0+5` → `#DIV/0!`, not `#VALUE!`); non-finite `^` results yield `#NUM!`. All file-size
> debt in `botsheet` + `botsheet-core` is cleared — `chart_read.rs` 701 → 431/161/134,
> `cell_ops.rs` 600 → 292/320, `crud.rs` 593 → 450/149, `requests.rs` 548 → 406/148,
> `websocket.rs` 457 → 391/72, `arrays.rs` 473 → 412/61. Every `.rs` is now ≤ 450 lines.
>
> **Progress (2026-08-15, rich text + dxf pass):** rich-text runs now import into
> the model by recovering per-run bold/italic/underline/colour/font/size from
> the raw `xl/sharedStrings.xml` (umya flattens them), keyed by `"row,col"` and
> attached as `Worksheet.rich_text`; conditional-format dxf styles now carry
> full font fidelity (family, size, weight, style, decoration) in addition to
> fill + colour.

---

## What already works well

- **Around 170 worksheet functions**, across arithmetic, statistics, text, dates, lookups, criteria aggregation, and the Microsoft 365 dynamic-array and lambda families (`FILTER`, `SORT`, `UNIQUE`, `SEQUENCE`, `LET`, `LAMBDA`, `MAP`, `REDUCE`, `BYROW`, `HSTACK`, `TEXTSPLIT`, `GROUPBY`, `PIVOTBY`), plus `BOT_AI_PROMPT` for AI-evaluated cells.
- **Over 70 HTTP endpoints** covering cells, ranges, formatting, worksheets, charts, validation, comments, protection, named ranges, external links and collaboration.
- **Per-cell OOXML number-format extraction** straight from the `.xlsx` XML, with the full ECMA-376 built-in format table.
- **Virtualised scrolling**, so a very large row count does not build a very large DOM.
- **Storage in Drive**, consistent with the rest of the platform.

The function library in particular is a genuine asset and is not being rewritten — it is being re-hosted on a real evaluator.

---

## The thirty gaps

Each row links to the issue that closes it.

### Data model

| # | Gap | Consequence |
|---|-----|-------------|
| 1 | ~~Cell values are `Option<String>`. No number, date, boolean or error type.~~ **Partially shipped (2026-08-09)** | New `engine::value::CellValue` gives typed numbers/text/booleans/errors in the formula engine; `=1/0` returns a typed `#DIV/0!`, comparisons are numeric. Storage remains string-cached. → issue 1 |
| 9 | Two incompatible cell-key conventions coexist (`"row,col"` and `"A1"`). | A workbook opened from Drive renders differently from one fetched through the range API. → issue 3 |

### Formula engine

| # | Gap | Consequence |
|---|-----|-------------|
| 2 | ~~No parser. Dispatch is a 170-arm match on the text before the first `(`.~~ **Partially shipped (2026-08-09)** | New `engine` module: lexer + Pratt parser → AST. `=SUM(A1:A3)+1`, `=A1&B1`, `=A1^2` and nested calls now evaluate with spreadsheet-correct precedence (`2^3^2 = 64`, `-2^2 = 4`). Legacy dispatcher remains as the function-call backend and fallback. → issue 2 |
| 3 | ~~The whole formula is upper-cased before evaluation.~~ **Partially shipped (2026-08-09)** | The typed parser does not uppercase string literals; `="Total: "&A1` preserves case. Legacy `CONCATENATE("…")` still routes through the old dispatcher (documented quirk). → issue 2 |
| 5 | ~~Recalculation stops after 1000 cells with no error and no warning.~~ **Partially shipped (2026-08-09)** | `recalc_cascade_typed` keeps the limit but logs and skips cycle members instead of silently stalling; `find_cycles` reports them. → issue 4 |
| 6 | ~~The dependency graph is rebuilt by re-scanning formula text on every keystroke.~~ **Shipped (2026-08-10)** | Per-session `DepGraph` caches the topology; `on_edit` replaces only the edited cell's edges and recalculation walks the cached dependents. Editing is O(formula), not O(workbook). → issue 4 |
| 7 | ~~The evaluator takes a single `&Worksheet`.~~ **Partially shipped (2026-08-09)** | `Reference.sheet` qualifiers parse and render (`Sheet2!A1`); evaluation returns a typed `#REF!` for cross-sheet refs the single-worksheet model cannot satisfy. → issue 3 |
| 8 | ~~`$` anchors are discarded during parsing.~~ **Partially shipped (2026-08-09)** | `Reference` keeps `col_absolute`/`row_absolute`; `translate(dr, dc)` shifts only relative parts. → issue 3 |

### Rendering

| # | Gap | Consequence |
|---|-----|-------------|
| 4 | Number formats are parsed, stored, and never applied. | `R$ 1.234,50` displays as `1234.5`; a date displays as `46096`. → issue 5 |
| 10 | ~~26 columns hardcoded; 1,200,000 rows against the desktop standard of 1,048,576.~~ **Partially shipped (2026-08-10)** | Row limit aligned to 1,048,576. Column count raised to the full 16,384 (`XFD`) with virtualised headers, cells and scrolling — only the visible column window is rendered and fetched; canvas rendering of the grid is the remaining piece. → issue 6 |
| 11 | ~~Column width and row height are constants.~~ **Shipped (2026-08-10)** | Widths and heights imported from `.xlsx` render (headers, cells, selection box, fill handle align), and interactive drag-resize on column headers and row-number gutter persists through `POST /api/sheet/resize`. → issue 6 |
| 12 | ~~Selection is one cell and one overlay element.~~ **Shipped (2026-08-10)** | Range drag-selection, Shift+click extension, `Ctrl+A` select-all, and row/column selection via header clicks. `Ctrl`+click and `Ctrl`+drag accumulate multi-ranges rendered as extra overlays. → issue 6 |
| 18 | ~~Frozen panes are imported with the axes swapped and never rendered.~~ **Shipped (2026-08-09)** | Sticky frozen top rows render as a pinned overlay (translates with the horizontal header); frozen left columns render as a virtualized pinned layer. Freeze/unfreeze is user-controllable via the toolbar and persisted through `/api/sheet/freeze`. → issue 6, issue 8 |
| 27 | ~~Conditional formatting, validation, charts and comments are stored and never rendered or enforced.~~ **Partially shipped (2026-08-09)** | Conditional-format rules render in the grid (`>`, `<`, `>=`, `<=`, between, text contains/starts-with/ends-with, duplicates, color scale). List validation renders a red marker dot + an in-cell dropdown; invalid single-cell edits are blocked with a toast. Column-header filters hide non-matching rows client-side. Charts render as SVG overlays (bar/line/pie). Cells with notes/comments show an orange corner marker, and a right-click context menu adds/edits/clears notes. The advanced JS module stubs (`01_core`…`19_notes`) are implemented. → issue 10 |

### Interaction

| # | Gap | Consequence |
|---|-----|-------------|
| 13 | ~~No clipboard.~~ **Shipped (2026-08-09)** | Copy/cut/paste/copy TSV with multi-cell block support and fallback textarea for non-secure contexts. Pasting a block from another spreadsheet works. → issue 7 |
| 14 | ~~No undo or redo. The buttons issue a request the server does not implement.~~ **Shipped (2026-08-09)** | Client-side 100-level undo/redo stack covering edits, paste, fill, cut and clear; toolbar buttons wired (`Ctrl+Z` / `Ctrl+Y`). → issue 7 |
| 15 | ~~No fill handle, no `Ctrl+D`/`Ctrl+R`, no series extrapolation.~~ **Shipped (2026-08-09)** | Drag fill handle with numeric series extrapolation and `$`-anchor-aware formula translation; `Ctrl+D`/`Ctrl+R` fill down/right. → issue 7 |
| 16 | ~~The merge button merges a cell with itself.~~ **Shipped (2026-08-09)** | The button merges the selected range; unmerge via `/api/sheet/unmerge`; merged regions render with the anchor spanning and covered cells hidden. → issue 7 |
| 17 | ~~No worksheet tab bar exists in the markup.~~ **Shipped (2026-08-09)** | Client-rendered tab bar with add/switch/delete/rename wired to the worksheet endpoints; only the first worksheet is no longer reachable. → issue 12 |

### File fidelity

| # | Gap | Consequence |
|---|-----|-------------|
| 19 | ~~Import drops defined names, validation, conditional formatting, charts, images, pivot tables, tables, autofilter, hidden rows, hyperlinks, rich text runs, protection, sheet visibility, print setup and external links.~~ **Partially shipped (2026-08-15)** | Import now maps workbook defined names, per-sheet protection, hidden rows/columns, sheet visibility, hyperlinks, tables, autofilter, images, data validation, conditional formatting (with dxf font/fill fidelity), rich-text runs (recovered from the raw `sharedStrings.xml`), cell comments/notes (recovered from `xl/commentsN.xml`), print setup, print areas/titles and external links into the model; charts are re-extracted from the raw package. Remaining: pivot tables are preserved byte-for-byte on save but not modelled, and none of these features are written back until an edit UI exists. → issue 8 |
| 20 | ~~Save writes the edited workbook back over the original `.xlsx`, from the lossy model, fire-and-forget.~~ **Shipped (2026-08-09, made non-destructive 2026-08-15)** | The save-back hook writes the edited xlsx BESIDE the original (`<name>.gbsheet.xlsx`) instead of overwriting it — the umya round-trip cannot preserve pivot tables, so the source is never touched until a zip-level preserve-and-passthrough lands. → issue 8, first commit |
| 21 | ~~`export_to_pdf_data` returns HTML bytes labelled as PDF.~~ **Shipped (2026-08-10)** | A real dependency-free PDF 1.4 writer (Helvetica, per-worksheet pages, repeated header row) replaces the HTML blob; the export handler serves `application/pdf`. → issue 8 |

### Infrastructure

| # | Gap | Consequence |
|---|-----|-------------|
| 22 | ~~Every request resolves to the user `"default-user"`.~~ **Shipped (2026-08-10)** | A `SheetUser` middleware derives the real identity from the platform bearer token before every sheet handler; anonymous traffic keeps the legacy `default-user` fallback so pre-auth clients still work. → issue 9 |
| 23 | ~~Whole-document read-modify-write per keystroke, serialised pretty-printed, with no concurrency control.~~ **Shipped (2026-08-10)** | Live sessions keep the workbook in memory, record an oplog, version every change, and persist through a debounced background task; keystrokes no longer touch Drive. → issue 9 |
| 24 | ~~Listing documents fully deserialises every one of them.~~ **Shipped (2026-08-10)** | Every save also writes a lightweight `.meta.json` sidecar; the list endpoint reads only the sidecars for ACL checks and metadata, falling back to a full load only for pre-sidecar documents. → issue 9 |
| 25 | ~~Collaboration encodes an address as `row * 26 + col`, last-write-wins, no sequencing, no reconnect recovery, no rendered presence.~~ **Shipped (2026-08-10)** | The collab socket speaks A1 row/col; every edit is applied server-side to the session, recorded in the oplog and stamped with a monotonically increasing `seq`; reconnecting clients replay `GET /api/sheet/:id/ops?since=N` before trusting the live feed; presence is rendered and swept. → issue 11 |
| 26 | ~~Collaboration state is five process-global statics with no eviction.~~ **Shipped (2026-08-10)** | A 20-second sweeper removes presence rows older than 75 s, typing indicators past 8 s, empty per-sheet maps and broadcast channels without live receivers; clean disconnects also remove their own rows. → issue 11 |

### Code health

| # | Gap | Consequence |
|---|-----|-------------|
| 28 | ~~~1,100 lines of dead CSS styling markup that does not exist; ~30 inline styles in the app HTML; ~180 inline styles in server-rendered fragments emitting classes no stylesheet defines.~~ **Partially shipped (2026-08-09)** | Audited all 162 class selectors in the sheet stylesheets against the sheet HTML, JS modules and server fragments repo-wide; removed 160 dead rules (1,263 lines) across the 8 stylesheets. Inline styles in HTML/fragments remain. → issue 12 |
| 29 | ~~Hardcoded Portuguese strings in the HTML and JS, while both locale catalogues sit unloaded.~~ **Partially shipped (2026-08-09)** | `locales/en.json` + `locales/pt-BR.json` (118 keys) are loaded and applied to the toolbar, sidebar tabs, panel buttons, sheet name and empty states via `data-i18n*` + `SheetI18n.t`; server-rendered fragment strings remain hardcoded. → issue 12 |
| 30 | ~~No tests for Sheet. Not one.~~ **Shipped (2026-08-10)** | `botui/tests/sheet_modules.test.js` covers core, clipboard, validation, widths, freeze, filters, formula fill, paste/i18n, undo, notes, charts, multi-range and drag-resize (70 assertions, green). Engine unit tests cover the typed evaluator and dependency graph. → issue 12 |

---

## The plan

Twelve issues in four phases. The ordering is a dependency order, not a priority order — each phase leaves the app strictly better, and nothing is built on a foundation that has to be torn out later.

### Phase 1 — Foundation

| Issue | Title | Closes | Status |
|---|---|---|---|
| [#781](https://github.com/generalbots/generalbots/issues/781) | Typed cell values — numbers, spreadsheet date serials, booleans, error values | 1 | 🟡 Partial — `engine::value::CellValue` shipped (2026-08-09); storage remains string-cached for legacy compatibility |
| [#782](https://github.com/generalbots/generalbots/issues/782) | Real formula parser — lexer, Pratt parser, AST, evaluator, function registry | 2, 3 | 🟢 Shipped — lexer + Pratt parser + AST + evaluator (2026-08-09) |
| [#783](https://github.com/generalbots/generalbots/issues/783) | Reference model — `$` anchors, sheet qualifiers, whole row/column, A1 keys, translation | 7, 8, 9 | 🟢 Shipped — `Reference` with anchors + translate; cross-sheet `Sheet2!A1` resolution (2026-08-09/10) |
| [#784](https://github.com/generalbots/generalbots/issues/784) | Calc engine — incremental dependency graph, topological recalc, cycle detection | 5, 6 | 🟢 Shipped — cached per-session `DepGraph`, `on_edit` incremental edges, `find_cycles` (2026-08-09/10) |

Phase 1 is unglamorous and unavoidable. While a cell is a `String` and a formula is a string match, everything above it is built on sand.

### Phase 2 — Presentation

| Issue | Title | Closes | Status |
|---|---|---|---|
| [#785](https://github.com/generalbots/generalbots/issues/785) | OOXML number format engine — currency, dates, accounting, scientific, fractions | 4 | 🟡 In progress — `engine::formats` renders currency/thousands/percent/dates (2026-08-09) |
| [#786](https://github.com/generalbots/generalbots/issues/786) | Canvas grid — 16,384 × 1,048,576, variable sizes, frozen panes, range selection | 10, 11, 12, 18 | 🟡 Partial — DOM grid renders variable widths/heights, frozen panes, range/multi-range/row/col selection, row limit aligned, interactive drag-resize, and the full 16,384-column range behind windowed headers/cells/scroll (2026-08-10); canvas rendering is the only remaining piece |

The grid moves from one absolutely positioned `<div>` per cell to canvas rendering with a DOM overlay for interactive chrome — the approach every serious web grid uses, because per-cell borders, fills, clipping and overflow cannot be composited at 60 fps in the DOM.

### Phase 3 — Interaction

| Issue | Title | Closes |
|---|---|---|
| [#787](https://github.com/generalbots/generalbots/issues/787) | Editing, clipboard, Paste Special, fill handle, undo/redo | 13, 14, 15, 16 |

The clipboard writes three flavours — TSV, styled HTML, and an internal JSON payload — so that a copy survives into other spreadsheet apps, and back again losslessly. Parsing competitor spreadsheets' `text/html` flavour is what makes "paste keeps my formatting" true.

### Phase 4 — Fidelity and infrastructure

| Issue | Title | Closes | Status |
|---|---|---|---|
| [#788](https://github.com/generalbots/generalbots/issues/788) | xlsx round-trip fidelity — stop the destructive save-back, preserve unmodelled parts | 19, 20, 21 | 🟡 Partial — `merge_into_original` preserves the untouched package and rewrites only edited cells; a real PDF 1.4 writer replaced the HTML-labelled-as-PDF export; import now maps defined names and sheet protection (2026-08-09/10). Validation, conditional formatting, charts, images, pivots and print setup are still not modelled on import |
| [#789](https://github.com/generalbots/generalbots/issues/789) | Document sessions — in-memory state, oplog, versioning, real identity and ACLs | 22, 23, 24 | 🟢 Shipped — sessions, oplog, versioning, debounced persistence, eviction, real identity via `SheetUser` and `can_read/can_write` enforced in every handler; metadata listing reads `.meta.json` sidecars; `GET /api/sheet/:id/ops?since=N` exposes oplog replay for reconnect recovery (2026-08-10) |
| [#790](https://github.com/generalbots/generalbots/issues/790) | Structured features — conditional formatting, validation, tables, filter/sort, charts, pivots | 27 | 🟡 Partial — CF rendering, validation dots + list dropdown, client filters, SVG charts, notes shipped end to end (2026-08-09); protection, external links and pivot rendering remain partial |
| [#791](https://github.com/generalbots/generalbots/issues/791) | Collaboration protocol — A1 addressing, server-authoritative sequencing, presence | 25, 26 | 🟢 Shipped — A1 addressing, presence, typing and selection wire up end-to-end; stale-state sweep added; edits are applied server-side, oplog-recorded and `seq`-stamped; reconnecting clients replay the oplog via `/api/sheet/:id/ops` (2026-08-10) |
| [#792](https://github.com/generalbots/generalbots/issues/792) | UI shell, i18n, dead code removal, and the test suite | 17, 28, 29, 30 | 🟡 Partial — tab bar, i18n and dead-CSS removal shipped; the JS test suite (76 assertions) lives in `botui/tests/sheet_modules.test.js`, `botsheet-core` engine tests at 85 and `botsheet` at 11, all green (2026-08-10); backend integration coverage for the parity flows remains thin |

**Issue 788 starts out of order.** Its first commit makes the `.xlsx` save-back non-destructive, because gap 20 is data loss reachable today and should not wait for its phase.

Round-trip fidelity is achieved by **preserve-and-passthrough**: retain the original `.xlsx` package, rewrite only the parts Sheet owns, and copy everything else verbatim. That is how a pivot table Sheet cannot render survives being edited around.

---

## Enterprise fidelity — open issues (2026-08-15)

The table below is the remaining distance between the current BETA and
"replace Excel / Google Sheets". It is ordered by user-facing risk: data safety
first, then visual fidelity, then scale. Each row is a ready-to-file issue with
a concrete acceptance bar. (The save-back path is now zip-level preserve-and-
passthrough — only `xl/worksheets/sheetN.xml` cell data is rewritten and every
other part is copied verbatim — and edited text cells reuse the shared-string
table when the value already exists, falling back to valid inline strings.)

| # | Issue | Builds on | Blocks enterprise use because |
|---|-------|-----------|-------------------------------|
| E1 | Shared-string **append** on save-back | #788 | **Shipped.** New text values append to `sharedStrings.xml` (updating `count`/`uniqueCount`) so text edits stay Excel-native `t="s"`; falls back to inline strings only when the workbook has no table. |
| E2 | Formula round-trip: shared/array/data-table formulas + cached `<v>` | #788 | **Partial.** Unchanged formula cells are reused verbatim — shared/array/data-table attributes and the cached `<v>` survive an unrelated edit. Remaining: formulas typed in the grid still write standalone `<f>` without shared-group refs. |
| E3 | Write-back of layout state: column widths, row heights, merged cells, frozen panes, hidden rows/cols | #788 | **Shipped.** Column widths, row heights, merged cells, frozen panes and hidden rows/cols import from the Drive-open path and write back into the exported xlsx (`xlsx_layout.rs`). |
| E4 | Sheet add / delete / rename round-trip | #788 | **Shipped (preserve-and-passthrough).** `xlsx_workbook.rs` reconciles the model against the original sheets (name-first, positional fallback for renames), rewrites `workbook.xml`/rels/`[Content_Types].xml`, and adds/removes worksheet parts; `xlsx_rename.rs` updates formula + defined-name references on rename. Remaining: no **reorder** UI/op exists, no stable per-sheet id (compound rename+delete in one save is ambiguous). |
| E5 | Error-value fidelity (`t="e"`) | #781 | **Shipped.** Canonical codes (`#DIV/0!`, `#NAME?`); the `#VALUE!!` double-bang and `NAME?.` typo are gone; error cells save as `t="e"` and `CellValue::parse` recognizes error literals on import. |
| E6 | Full import model: images, pivot tables, tables, autofilter, hyperlinks, rich text runs, sheet visibility, print setup, external links | #788, #790 | **Partial.** Hidden columns, sheet visibility, hyperlinks, structured tables, the autofilter range, images, data validation, conditional formatting (with dxf font/fill fidelity), external links, cell comments/notes and rich-text runs (recovered from the raw `sharedStrings.xml`) now import into the model; pivots survive via preserve-and-passthrough. Remaining: render hyperlinks/tables/autofilter/images/rich text in the grid, and write-back of these features once an edit UI exists. |
| E7 | Number formats: scientific, fractions, accounting alignment, locale rendering | #785 | **Partial.** Scientific (`0.00E+00`) and fraction (`# ?/?`, `# ??/??`) renderers landed with tests; accounting alignment and pt-BR `R$ 1.234,50` locale rendering remain. |
| E8 | Canvas grid | #786 | The DOM grid cannot sustain 60 fps at 16,384 × 1,048,576 with per-cell borders and fills. |
| E9 | `.xls` (BIFF8) / `.xlsb` import fidelity | #788 | **Partial.** `.xls`/`.xlsb` now import cell values + typed values via calamine (`import_binary.rs`); format detection distinguishes `.xlsb` from `.xlsx`. Remaining: styles, merges, charts and layout are not read from BIFF (calamine exposes values only). |
| E10 | Co-editing convergence (OT/CRDT) + cell-level locks | #791 | Server-authoritative last-write is safe but not Google-Sheets-grade concurrent editing. |
| E11 | Print / page-setup round-trip and PDF fidelity | #788, #791 | **Partial.** Page margins, header/footer text, `<pageSetup>`, manual page breaks, print areas and print titles (`_xlnm.Print_Area` / `_xlnm.Print_Titles`) now import into the model; the elements round-trip byte-for-byte through preserve-and-passthrough. Remaining: write-back once a print UI exists. |
| E12 | Performance budgets asserted in CI | all | The budgets below are aspirational until measured by a benchmark gate. |

Each lands the same way the shipped phases did: preserve-and-passthrough for
anything the model cannot represent, model expansion for the parts Sheet
renders, and canvas/CRDT only where the DOM and last-write-wins are provably
insufficient.

---

## Definition of done

- A real workbook — 40 sheets, pivot tables, nested `INDEX`/`MATCH`, conditional formatting, a chart — opens, renders, edits and saves without losing anything Sheet does not itself understand.
- Paste from another spreadsheet app keeps values, formulas, formats, fills and merges.
- Formulas are spreadsheet-correct: precedence, `2^3^2 = 64`, `-2^2 = 4`, error propagation, cross-sheet references, `$` anchors that survive a fill.
- 16,384 columns and 1,048,576 rows, scrolling above 55 fps.
- Undo works, 100 levels deep, one step per user action.
- Two people edit at once without either losing work.
- A viewer cannot write; a tenant cannot read another tenant's document.
- Every toolbar control does something. Every string is translatable.
- `cargo check` and `cargo clippy` clean. No `unwrap`/`expect`/`panic!` outside tests. No file over 450 lines. No `#[allow]`.
- Twenty browser scenarios pass through Chrome CDP, with screenshots.
- Performance budgets asserted, not merely observed:

| Metric | Budget |
|--------|--------|
| Initial paint, 100,000 cells | < 1 s |
| Sustained scroll | > 55 fps |
| Single-cell edit to repaint | < 16 ms |
| Recalc, 10,000 dependents | < 250 ms |
| Paste, 10,000 cells | < 500 ms |
| xlsx import, 1 MB | < 2 s |
| Server memory, 1,000,000 populated cells | < 500 MB |

---

## See Also

- [Sheet — Spreadsheets](./sheet.md) — user-facing feature reference with current status
- [Sheets API](../../08-rest-api-tools/sheets-api.md) — endpoint reference
- [AI Sheet Cache](../../03-knowledge-ai/ai-sheet-cache.md) — `BOT_AI_PROMPT` caching
