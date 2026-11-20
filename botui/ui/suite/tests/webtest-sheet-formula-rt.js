/**
 * Sheets: XLSX formula round-trip and server-side locale/calc persistence (#2572).
 * Run with: node botui/ui/suite/tests/webtest-sheet-formula-rt.js
 *
 * One vm sandbox loads the nine REAL shipped function IIFEs from
 * sheet/js/modules-sheet-advanced/ (73, 74, 75, 76a, 76, 77, 78a, 78, 79).
 * Ordered modules under ./webtest-sheet-formula-rt/:
 *   00_harness.js          counters, vm loader, stubs, report
 *   10_roundtrip.js        parse -> evaluate -> format invariants, locale/separators
 *   20_errors_persistence.js error semantics, [STATIC] deferred criteria, KNOWN BUGs
 *
 * No production file is modified. Rust codec/persistence/CI criteria are
 * checked as labelled [STATIC] source assertions, never as runtime PASS.
 */
"use strict";

var harness = require("./webtest-sheet-formula-rt/00_harness");

require("./webtest-sheet-formula-rt/10_roundtrip");
require("./webtest-sheet-formula-rt/20_errors_persistence");

harness.finish();
