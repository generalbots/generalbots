# Sheet — Desktop Spreadsheet Parity Plan 🟡 BETA

> **What is missing, why it matters, and the order in which it is being fixed.**

This page is the reference for anyone deciding whether to trust Sheet with a real workbook, and for anyone implementing the work. It is deliberately blunt: a spreadsheet that is almost right is worse than one that is obviously incomplete, because a wrong number looks exactly like a right one.

Tracking issue: [generalbots/generalbots#780](https://github.com/generalbots/generalbots/issues/780).

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
| 1 | Cell values are `Option<String>`. No number, date, boolean or error type. | `=SUM()` over `1,234.50` returns `0`. Dates cannot be subtracted. Numbers cannot be right-aligned. ZIP codes lose leading zeros. → issue 1 |
| 9 | Two incompatible cell-key conventions coexist (`"row,col"` and `"A1"`). | A workbook opened from Drive renders differently from one fetched through the range API. → issue 3 |

### Formula engine

| # | Gap | Consequence |
|---|-----|-------------|
| 2 | No parser. Dispatch is a 170-arm match on the text before the first `(`. | `=SUM(A1:A3)+1`, `=A1&B1`, `=A1^2` and every nested call return `#ERROR!`. → issue 2 |
| 3 | The whole formula is upper-cased before evaluation. | `=CONCATENATE("Total: ",A1)` returns `TOTAL: 7`. Data corruption, not cosmetics. → issue 2 |
| 5 | Recalculation stops after 1000 cells with no error and no warning. | A financial model displays stale numbers that look final. → issue 4 |
| 6 | The dependency graph is rebuilt by re-scanning formula text on every keystroke. | Editing is O(workbook) instead of O(dependents). → issue 4 |
| 7 | The evaluator takes a single `&Worksheet`. | `=Sheet2!A1` is unrepresentable, despite the model holding many worksheets. → issue 3 |
| 8 | `$` anchors are discarded during parsing. | No copy, fill or paste can translate references correctly. → issue 3 |

### Rendering

| # | Gap | Consequence |
|---|-----|-------------|
| 4 | Number formats are parsed, stored, and never applied. | `R$ 1.234,50` displays as `1234.5`; a date displays as `46096`. → issue 5 |
| 10 | 26 columns hardcoded; 1,200,000 rows against the desktop standard of 1,048,576. | Most of the address space is unreachable, and the row count is wrong in the other direction. → issue 6 |
| 11 | Column width and row height are constants. | Widths and heights imported from `.xlsx` cannot be displayed; resize is impossible by construction. → issue 6 |
| 12 | Selection is one cell and one overlay element. | No range, multi-range, row, column or whole-sheet selection. → issue 6 |
| 18 | Frozen panes are imported with the axes swapped and never rendered. | A frozen header row does not stay put. → issue 6, issue 8 |
| 27 | Conditional formatting, validation, charts and comments are stored and never rendered or enforced. | Four toolbar buttons open modals whose results vanish. Three advanced JS modules are empty stubs that are nonetheless loaded. → issue 10 |

### Interaction

| # | Gap | Consequence |
|---|-----|-------------|
| 13 | No clipboard. | Pasting a block from another spreadsheet — the most common spreadsheet operation there is — does nothing. → issue 7 |
| 14 | No undo or redo. The buttons issue a request the server does not implement. | One mistake is unrecoverable. → issue 7 |
| 15 | No fill handle, no `Ctrl+D`/`Ctrl+R`, no series extrapolation. | Every repeated formula must be typed. → issue 7 |
| 16 | The merge button merges a cell with itself. | A no-op that appears to work. → issue 7 |
| 17 | No worksheet tab bar exists in the markup. | Only the first worksheet of a workbook is reachable. → issue 12 |

### File fidelity

| # | Gap | Consequence |
|---|-----|-------------|
| 19 | Import drops defined names, validation, conditional formatting, charts, images, pivot tables, tables, autofilter, hidden rows, hyperlinks, rich text runs, protection, sheet visibility, print setup and external links. | Most of a real workbook does not arrive. → issue 8 |
| 20 | Save writes the edited workbook back over the original `.xlsx`, from the lossy model, fire-and-forget. | Opening a real workbook and typing one character can strip its charts, pivots, validation and conditional formatting. **This is the most serious item on this page.** → issue 8, first commit |
| 21 | `export_to_pdf_data` returns HTML bytes labelled as PDF. | Any client trusting the content type gets a corrupt file. → issue 8 |

### Infrastructure

| # | Gap | Consequence |
|---|-----|-------------|
| 22 | Every request resolves to the user `"default-user"`. | All documents share one namespace; any client can read or overwrite any document by id. In a multi-tenant SaaS this is a security hole. → issue 9 |
| 23 | Whole-document read-modify-write per keystroke, serialised pretty-printed, with no concurrency control. | Tens of megabytes per edit on a large workbook, and two clients editing different cells lose one edit silently. → issue 9 |
| 24 | Listing documents fully deserialises every one of them. | The document list degrades linearly with library size. → issue 9 |
| 25 | Collaboration encodes an address as `row * 26 + col`, last-write-wins, no sequencing, no reconnect recovery, no rendered presence. | Any grid wider than 26 columns lands every remote edit on the wrong cell. → issue 11 |
| 26 | Collaboration state is five process-global statics with no eviction. | Cannot work behind more than one server instance; leaks. → issue 11 |

### Code health

| # | Gap | Consequence |
|---|-----|-------------|
| 28 | ~1,100 lines of dead CSS styling markup that does not exist; ~30 inline styles in the app HTML; ~180 inline styles in server-rendered fragments emitting classes no stylesheet defines. | Nothing is themeable and the real rules are hard to find. → issue 12 |
| 29 | Hardcoded Portuguese strings in the HTML and JS, while both locale catalogues sit unloaded. | The app cannot be localised. → issue 12 |
| 30 | No tests for Sheet. Not one. | Every regression reaches a user first. → issue 12 |

---

## The plan

Twelve issues in four phases. The ordering is a dependency order, not a priority order — each phase leaves the app strictly better, and nothing is built on a foundation that has to be torn out later.

### Phase 1 — Foundation

| Issue | Title | Closes |
|---|---|---|
| [#781](https://github.com/generalbots/generalbots/issues/781) | Typed cell values — numbers, spreadsheet date serials, booleans, error values | 1 |
| [#782](https://github.com/generalbots/generalbots/issues/782) | Real formula parser — lexer, Pratt parser, AST, evaluator, function registry | 2, 3 |
| [#783](https://github.com/generalbots/generalbots/issues/783) | Reference model — `$` anchors, sheet qualifiers, whole row/column, A1 keys, translation | 7, 8, 9 |
| [#784](https://github.com/generalbots/generalbots/issues/784) | Calc engine — incremental dependency graph, topological recalc, cycle detection | 5, 6 |

Phase 1 is unglamorous and unavoidable. While a cell is a `String` and a formula is a string match, everything above it is built on sand.

### Phase 2 — Presentation

| Issue | Title | Closes |
|---|---|---|
| [#785](https://github.com/generalbots/generalbots/issues/785) | OOXML number format engine — currency, dates, accounting, scientific, fractions | 4 |
| [#786](https://github.com/generalbots/generalbots/issues/786) | Canvas grid — 16,384 × 1,048,576, variable sizes, frozen panes, range selection | 10, 11, 12, 18 |

The grid moves from one absolutely positioned `<div>` per cell to canvas rendering with a DOM overlay for interactive chrome — the approach every serious web grid uses, because per-cell borders, fills, clipping and overflow cannot be composited at 60 fps in the DOM.

### Phase 3 — Interaction

| Issue | Title | Closes |
|---|---|---|
| [#787](https://github.com/generalbots/generalbots/issues/787) | Editing, clipboard, Paste Special, fill handle, undo/redo | 13, 14, 15, 16 |

The clipboard writes three flavours — TSV, styled HTML, and an internal JSON payload — so that a copy survives into other spreadsheet apps, and back again losslessly. Parsing competitor spreadsheets' `text/html` flavour is what makes "paste keeps my formatting" true.

### Phase 4 — Fidelity and infrastructure

| Issue | Title | Closes |
|---|---|---|
| [#788](https://github.com/generalbots/generalbots/issues/788) | xlsx round-trip fidelity — stop the destructive save-back, preserve unmodelled parts | 19, 20, 21 |
| [#789](https://github.com/generalbots/generalbots/issues/789) | Document sessions — in-memory state, oplog, versioning, real identity and ACLs | 22, 23, 24 |
| [#790](https://github.com/generalbots/generalbots/issues/790) | Structured features — conditional formatting, validation, tables, filter/sort, charts, pivots | 27 |
| [#791](https://github.com/generalbots/generalbots/issues/791) | Collaboration protocol — A1 addressing, server-authoritative sequencing, presence | 25, 26 |
| [#792](https://github.com/generalbots/generalbots/issues/792) | UI shell, i18n, dead code removal, and the test suite | 17, 28, 29, 30 |

**Issue 788 starts out of order.** Its first commit makes the `.xlsx` save-back non-destructive, because gap 20 is data loss reachable today and should not wait for its phase.

Round-trip fidelity is achieved by **preserve-and-passthrough**: retain the original `.xlsx` package, rewrite only the parts Sheet owns, and copy everything else verbatim. That is how a pivot table Sheet cannot render survives being edited around.

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
