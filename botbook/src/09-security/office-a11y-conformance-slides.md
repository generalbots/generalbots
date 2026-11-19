# Office Accessibility Conformance Report (VPAT-style baseline) — Slides

**Status:** Provisional current-state baseline (not a formal audit)  
**Product:** General Bots — Slides app (`botui/ui/suite/slides/`, `botserver/crates/botslides`)  
**Scope:** Slides editor, presenter-related UI, and dialogs as shipped in current source state  
**Report date:** 2026-11-19 (offline documentation slice)  
**Standard referenced:** WCAG 2.2 Level A/AA (informational mapping only; not a conformance claim)  
**Evaluation method:** Source/documentation review of current repository state only. No browser/assistive-technology testing, no axe-core CI run, no keyboard-only end-to-end completion evidence, and no manual audit were performed for this baseline.

> This document describes **what the product does today** based on available repository/documentation evidence. It does **not** certify WCAG 2.2 AA conformance and does **not** replace a formal VPAT/ACR produced from audited testing.

## Summary

- **Overall conformance claim:** **Not Evaluated / Does Not Conform (no formal evidence)**.
- Slides provides presentation authoring/delivery UI (slides/elements/themes/transitions/animations model elements, presenter features, comments/versions/presence in codebase) but is desktop-first in many interactions; mobile/touch and accessibility conformance are not formally established.
- Known suite gaps: no published audited accessibility conformance per app, no demonstrated CI axe gate for the trio, keyboard/focus/contrast/roles/states coverage not formally verified, and localization/i18n completeness not verified across all labels.

## WCAG 2.2 A/AA criteria mapping (baseline)

| Criterion area | Conformance (baseline) | Remarks (current-state / evidence limits) |
|---|---|---|
| 1.1 Text Alternatives | Not Evaluated | Slide/media objects exist; alt-text authoring/enforcement and coverage in editor/presenter not audited. |
| 1.3 Adaptable (structure, relationships, sequence) | Not Evaluated | Slide/canvas structure and reading order for assistive tech not formally tested; fixed-canvas/layout behaviors may affect adaptability (not measured). |
| 1.4 Distinguishable (contrast, resize, reflow, text spacing) | Not Evaluated | No contrast/reflow/text-spacing audit evidence; desktop-first layouts and responsive behavior not verified for a11y. |
| 2.1 Keyboard Accessible | Not Evaluated | Commands/toolbars/dialogs exist; complete keyboard path for every command and create→edit→comment→export/present flows not demonstrated; no-trap behavior unverified. |
| 2.4 Navigable (focus order, focus visible, headings, labels) | Not Evaluated | Focus management and visible focus in canvas/toolbars/modals/panels not audited; control naming/label coverage unverified. |
| 2.5 Input Modalities (target size, pointer gestures) | Not Evaluated | Touch targets/gesture alternatives not measured; issue #2461 tracks mobile/touch requirements (not evidenced as met). |
| 3.1/3.2 Readable, Predictable | Not Evaluated | Suite i18n exists; full en/pt-BR/es label coverage and locale-aware behavior for Slides not verified. |
| 3.3 Input Assistance | Not Evaluated | Validation/error identification/recovery in dialogs and authoring flows not formally evaluated. |
| 4.1 Compatible (name, role, value, status messages) | Not Evaluated | ARIA/role/state correctness and status announcements (including presenter/collaboration states) not audited. |

## Known gaps relevant to enterprise/a11y procurement (current)

- No formal WCAG 2.2 AA audit or published audited VPAT/ACR for Slides.
- No evidenced CI axe-core gate reporting zero critical issues for Slides.
- Keyboard-only completion of key authoring/review/export tasks not evidenced.
- Responsive/touch accessibility and 44px target compliance not evidenced (tracked separately).
- Presenter/canvas interactions may rely on pointer/visual affordances; assistive-technology support not verified.
- Full localization/hardcoded-string elimination for Slides UI not evidenced in this baseline.

## Evidence basis for this baseline

- `botbook/src/07-user-interface/apps/slides.md` (current app behavior/preview context)
- Issue #2454 and #2461 (accessibility/compliance and mobile/touch scope and gaps)
- Repository source presence: `botui/ui/suite/slides/`, `botserver/crates/botslides`

## Deferred (not provided by this offline slice)

- Formal manual WCAG 2.2 AA audit and remediated conformance claims.
- Automated axe-core results/gates in CI and browser/runtime assistive-technology testing.
- Verified keyboard-only and screen-reader evidence for editor and presenter flows.
- Touch/mobile accessibility verification and responsive conformance evidence.
- i18n completeness verification (including es) across all Slides labels.
