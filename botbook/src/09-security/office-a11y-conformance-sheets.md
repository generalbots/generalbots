# Office Accessibility Conformance Report (VPAT-style baseline) — Sheets

**Status:** Provisional current-state baseline (not a formal audit)  
**Product:** General Bots — Sheets app (`botui/ui/suite/sheet/`, `botserver/crates/botsheet`, `botserver/crates/botsheet-core`)  
**Scope:** Sheets grid/editor UI and dialogs as shipped in current source state  
**Report date:** 2026-11-19 (offline documentation slice)  
**Standard referenced:** WCAG 2.2 Level A/AA (informational mapping only; not a conformance claim)  
**Evaluation method:** Source/documentation review of current repository state only. No browser/assistive-technology testing, no axe-core CI run, no keyboard-only end-to-end completion evidence, no PDF/UA validation, and no manual audit were performed for this baseline.

> This document describes **what the product does today** based on available repository/documentation evidence. It does **not** certify WCAG 2.2 AA conformance and does **not** replace a formal VPAT/ACR produced from audited testing.

## Summary

- **Overall conformance claim:** **Not Evaluated / Does Not Conform (no formal evidence)**.
- Sheets provides a virtualized grid and spreadsheet features (formulas, charts, import/export paths, etc. in evolving preview state). Grid accessibility, keyboard completeness, assistive-technology semantics, and export accessibility are not formally established.
- Known suite gaps: no published audited accessibility conformance per app, no demonstrated CI axe gate for the trio, keyboard/focus/contrast/roles/states coverage not formally verified, tagged PDF/UA export not evidenced, and localization/i18n completeness not verified across all labels.

## WCAG 2.2 A/AA criteria mapping (baseline)

| Criterion area | Conformance (baseline) | Remarks (current-state / evidence limits) |
|---|---|---|
| 1.1 Text Alternatives | Not Evaluated | Charts/images/visual elements may exist; text alternatives and non-text content accessibility not audited. |
| 1.3 Adaptable (structure, relationships, sequence) | Not Evaluated | Grid/table semantics, header associations, and programmatic relationships for assistive tech not formally tested. |
| 1.4 Distinguishable (contrast, resize, reflow, text spacing) | Not Evaluated | No contrast/reflow/text-spacing audit evidence; grid/canvas rendering behavior under zoom/reflow not verified. |
| 2.1 Keyboard Accessible | Not Evaluated | Grid navigation/editing and toolbar commands exist in UI; complete keyboard path for every command and create→edit→comment→export flows not demonstrated; no-trap behavior unverified. |
| 2.4 Navigable (focus order, focus visible, headings, labels) | Not Evaluated | Focus management between grid, formula bar, panels, and dialogs not audited; visible focus and control naming coverage unverified. |
| 2.5 Input Modalities (target size, pointer gestures) | Not Evaluated | Pointer/selection interactions common in spreadsheets; target sizes and non-pointer alternatives not measured/audited (mobile/touch tracked separately). |
| 3.1/3.2 Readable, Predictable | Not Evaluated | Suite i18n exists; full en/pt-BR/es label coverage and locale-aware number/date/currency behavior in UI not verified as complete for Sheets. |
| 3.3 Input Assistance | Not Evaluated | Formula/input validation, error identification, and correction assistance accessibility not formally evaluated. |
| 4.1 Compatible (name, role, value, status messages) | Not Evaluated | Grid/cell roles, live updates (recalc/status), and control name/role/value correctness not audited with assistive technologies. |

## Known gaps relevant to enterprise/a11y procurement (current)

- No formal WCAG 2.2 AA audit or published audited VPAT/ACR for Sheets.
- No evidenced CI axe-core gate reporting zero critical issues for Sheets.
- Keyboard-only completion of create→edit→comment→export not evidenced.
- Tagged PDF/UA export for Sheets not evidenced as passing a validator.
- Grid accessibility (semantics, focus, announcements) not formally verified (tracked in part by related grid/accessibility work).
- Full localization/hardcoded-string elimination for Sheets UI not evidenced in this baseline.

## Evidence basis for this baseline

- `botbook/src/07-user-interface/apps/sheet.md` and related Sheets docs (current app behavior/preview context)
- Issue #2454 and #2493 (compliance/a11y scope and grid accessibility gaps)
- Repository source presence: `botui/ui/suite/sheet/`, `botserver/crates/botsheet`, `botserver/crates/botsheet-core`

## Deferred (not provided by this offline slice)

- Formal manual WCAG 2.2 AA audit and remediated conformance claims.
- Automated axe-core results/gates in CI and browser/runtime assistive-technology testing.
- PDF/UA validator evidence for exported Sheets PDFs.
- Verified keyboard-only and screen-reader evidence for grid/editor workflows.
- i18n completeness verification (including es) across all Sheets labels.
