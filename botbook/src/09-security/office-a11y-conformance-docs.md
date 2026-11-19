# Office Accessibility Conformance Report (VPAT-style baseline) — Docs

**Status:** Provisional current-state baseline (not a formal audit)  
**Product:** General Bots — Docs editor (`botui/ui/suite/docs/`, `botserver/crates/botdocs`)  
**Scope:** Docs app UI/editor and related dialogs as shipped in current source state  
**Report date:** 2026-11-19 (offline documentation slice)  
**Standard referenced:** WCAG 2.2 Level A/AA (informational mapping only; not a conformance claim)  
**Evaluation method:** Source/documentation review of current repository state only. No browser/assistive-technology testing, no axe-core CI run, no keyboard-only end-to-end completion evidence, no PDF/UA validation, and no manual audit were performed for this baseline.

> This document describes **what the product does today** based on available repository/documentation evidence. It does **not** certify WCAG 2.2 AA conformance and does **not** replace a formal VPAT/ACR produced from audited testing.

## Summary

- **Overall conformance claim:** **Not Evaluated / Does Not Conform (no formal evidence)**.
- Docs is a preview/in-test editor with rich text, comments, track changes, templates/forms, export, and collaboration-related UI. Current public docs note collaboration presence depends on configured realtime endpoints and that some workflows are local/preview.
- Known gaps tracked for the Office suite include: lack of published audited accessibility conformance, no demonstrated CI axe gate for the trio, keyboard/focus/contrast/roles/states coverage not formally verified, tagged PDF/UA export not evidenced, and localization/i18n completeness not verified across all labels.

## WCAG 2.2 A/AA criteria mapping (baseline)

| Criterion area | Conformance (baseline) | Remarks (current-state / evidence limits) |
|---|---|---|
| 1.1 Text Alternatives | Not Evaluated | Editor supports inserted images/equations in UI; programmatic alt-text requirements and coverage across all content/dialogs not audited. |
| 1.3 Adaptable (structure, relationships, sequence) | Not Evaluated | Semantic heading/structure tools exist (TOC, headings, tables/captions in preview workspaces); DOM semantics, reading order, and form/control relationships not formally tested. |
| 1.4 Distinguishable (contrast, resize, reflow, text spacing) | Not Evaluated | No contrast/reflow/text-spacing audit evidence; theme/CSS behavior under zoom/reflow not verified. |
| 2.1 Keyboard Accessible | Not Evaluated | Toolbar/actions and dialogs exist; complete keyboard path for every command (including create→edit→comment→export) and absence of traps not demonstrated. |
| 2.4 Navigable (focus order, focus visible, headings, labels) | Not Evaluated | Focus management in modals/dialogs/panels and focus visibility not audited; some controls expose labels/ARIA in source but coverage is unverified. |
| 2.5 Input Modalities (target size, pointer gestures) | Not Evaluated | Touch target sizes and gesture alternatives not measured/audited in Docs editor. |
| 3.1/3.2 Readable, Predictable | Not Evaluated | Language/i18n support exists in suite; full en/pt-BR/es label coverage and locale-aware behavior for Docs not verified. No unexpected-context-change audit. |
| 3.3 Input Assistance | Not Evaluated | Validation/error identification/recovery in forms/dialogs not formally evaluated. |
| 4.1 Compatible (name, role, value, status messages) | Not Evaluated | Some ARIA/live-region patterns may exist; comprehensive name/role/value correctness and status-message behavior not audited. |

## Known gaps relevant to enterprise/a11y procurement (current)

- No formal WCAG 2.2 AA audit or published audited VPAT/ACR for Docs.
- No evidenced CI axe-core gate reporting zero critical issues for Docs.
- Keyboard-only completion of create→edit→comment→export not evidenced.
- Tagged PDF/UA export for Docs not evidenced as passing a validator.
- Full localization/hardcoded-string elimination for Docs UI not evidenced in this baseline.
- Review/collaboration features include local-preview behaviors; accessibility of all states not assessed.

## Evidence basis for this baseline

- `botbook/src/07-user-interface/apps/docs.md` (current shipped/preview behavior description)
- Issue #2454 “Today/Required/Acceptance criteria” (compliance, records, accessibility scope)
- Repository source presence: `botui/ui/suite/docs/`, `botserver/crates/botdocs`

## Deferred (not provided by this offline slice)

- Formal manual WCAG 2.2 AA audit and remediated conformance claims.
- Automated axe-core results/gates in CI and browser/runtime assistive-technology testing.
- PDF/UA validator evidence for exported Docs PDFs.
- Verified keyboard-only task completion evidence and screen-reader transcripts.
- i18n completeness verification (including es) across all Docs labels.
