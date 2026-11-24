# Current limitations

The office apps are in test. This page records what the shipped preview does **not** do today, so you can decide whether a task is safe to put in these editors. It is a description of current behaviour, not a commitment about future work.

## Co-editing is not a supported workflow

You can see other people in a document, spreadsheet or deck: the shared collaboration client connects to app-specific realtime routes, shows presence and cursors, and a conflict panel can surface divergent edits from the server's operation log. That is **presence and review**, not concurrent editing you can rely on.

- Concurrent spreadsheet cell edits are still applied with transport-level last-write-wins: whichever message arrives last overwrites the cell, and a client that reconnects can lose another author's edit depending on delivery order. A deterministic last-writer-wins register model exists in the spreadsheet crate and is used by the local undo/offline building blocks, but it is not the transport's write path.
- The document editor's structured model, review state and permissions matrix are stored in browser-scoped storage and are labelled "Local only — not saved to the server". The legacy editor still writes HTML; DOCX/ODT round-trip, server-side operation validation and co-editing merge are not in place.
- There is no stable-anchor, per-user undo workflow for parallel document review, and no offline merge you can depend on for a workbook or deck.

Treat the editors as single-author tools with visibility aids today. If two people need to edit the same file, coordinate who is writing rather than relying on the app to merge.

## Enterprise controls are previews

The **Enterprise controls** workspace in Sheet — protection targets, input ranges, range-to-role access maps, classification labels, export gating, external-link allowlist, audit trail and snapshot badge — is a local policy preview stored in the browser. It states plainly that there is no server enforcement, signing or persistence. The same applies to the print-and-report workspace: print areas, page setup, dashboards and scheduled reports are rendered locally; there is no real PDF export with engine parity and no cron or e-mail delivery.

Consequences you should plan around:

- Marking a range as protected in the browser is not a server-enforced write restriction.
- A classification label produces a banner and watermark preview; it does not watermark an exported file.
- The cell-change audit trail and snapshot hash are local conveniences, not reconstructable retention or signed evidence.
- Credentials for data connectors are represented as `vault://` references in a local simulation; no Vault resolution or network fetch happens.

## File-format fidelity is partial

DOCX, PPTX and XLSX round-trips are not lossless. In the spreadsheet, pivot tables are dropped from the modelled workbook (the original package bytes can survive a save-back, but the feature is not modelled), charts and rich text are re-extracted from the raw package, and some BIFF formats expose values and formulas only. In Docs, equation OMML round-trip, print-quality layout and tagged PDF are not implemented; the Pages and layout workspace is a deterministic local pagination preview, not a PDF rendering engine.

Export what you need and open the exported file in a desktop office application before treating it as final. The per-app chapters and the [Office Fidelity Contract](./../07-user-interface/apps/office-fidelity.md) list the individual gaps.

## Large-file and long-document paths are experimental

Docs offers an experimental large-document load path with viewport windowing and incremental save, but it is a fixture-driven preview: streaming DOCX import, real server body limits, real ETag enforcement and measured performance budgets are not implemented. Very large documents may open read-only with an explicit enable-editing-anyway action. Sheet's virtual grid scrolls over large row counts, but its local-only formula engine and advanced chart previews do not guarantee parity with the server evaluator or with XLSX chart parts.

## Validation and sorting have local scope

Sheet's data-validation manager marks invalid cells in the browser and carries an error message, but it does not block a write on the server. Sorting a range moves values; formulas referring to moved cells are not rewritten. Advanced analysis panels — formula auditing, function wizard, sparklines, slicers, Goal Seek and scenarios — read the browser's in-memory cells and are labelled local-only.

## Accessibility and localisation

The apps ship English and pt-BR catalogs and keyboard-accessible dialogs with focus handling and status regions, but a WCAG 2.2 AA conformance claim cannot be made for the preview. Localised catalogue keys are required for new UI; do not rely on text that has not been through the catalogs.

## See Also

- [Creating office files from chat](./from-chat.md)
- [Working in the editors](./editors.md)
- [Office Fidelity Contract](./../07-user-interface/apps/office-fidelity.md)
- [Sheet - Spreadsheets](./../07-user-interface/apps/sheet.md)
- [Docs - AI Writing](./../07-user-interface/apps/docs.md)
