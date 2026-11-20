/**
 * Slides Table & Media — Offline Test Suite (#2487).
 * Run with: node botui/ui/suite/tests/webtest-slides-table.js
 *
 * Pure, DOM-free table/media helpers that already ship are loaded from the
 * real modules into a minimal vm sandbox with a fake host and verified without
 * a browser. The ordered modules live in ./webtest-slides-table/ (mirrors
 * webtest-sheet-a11y.js):
 *   00_harness.js   counters, assertions, module loading, fake host, report
 *   10_table.js     model invariants, cell commit, table export-facing helpers,
 *                   bulk selection/order/move/duplicate/remove invariants
 *
 * NATIVE PPTX table export (a:tbl + import), video/audio trim, caption loading
 * and real-slide browser UAT remain unimplemented/unverified and are NOT
 * claimed by this harness.
 */
"use strict";

var harness = require("./webtest-slides-table/00_harness");

require("./webtest-slides-table/10_table");

harness.finish();
