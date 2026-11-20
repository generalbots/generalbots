/**
 * Sheets: formula auditing and evaluation — offline test suite (#2491).
 * Run with: node botui/ui/suite/tests/webtest-sheet-audit.js
 *
 * One vm sandbox loads the nine REAL shipped audit IIFEs from
 * sheet/js/modules-sheet-advanced/ (25, 25a, 26, 26a, 27, 28, 28a, 29, 30)
 * with stubbed window/document/gbT/localStorage/MutationObserver/fixed clock.
 *
 * Ordered modules under ./webtest-sheet-audit/:
 *   dom.js               minimal DOM doubles (no production logic reimplemented)
 *   00_harness.js        vm loader, stubs, counters, report
 *   10_graph.js          dependency graph, reference parsing, tracing
 *   20_eval.js           tokenizer, parser, step trace, nested IF(SUMIF(...))
 *   30_reports.js        cycles, error classification, formula text, maps
 *   40_ui.js             panel, watch list, overlay, show-formulas, boot
 *   50_static.js         [STATIC] deferred criteria and KNOWN BUG reports
 *
 * No production file is modified. Deferred browser/CI/backend criteria are
 * labelled [STATIC] source assertions, never runtime PASS. Observed defects
 * are labelled KNOWN BUG and reported only, never patched here.
 */
"use strict";

var harness = require("./webtest-sheet-audit/00_harness");

require("./webtest-sheet-audit/10_graph");
require("./webtest-sheet-audit/20_eval");
require("./webtest-sheet-audit/30_reports");
require("./webtest-sheet-audit/40_ui");
require("./webtest-sheet-audit/50_static");

harness.finish();
