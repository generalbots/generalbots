/**
 * Sheets connectivity and refresh — OFFLINE test suite (#2492).
 * Run with: node botui/ui/suite/tests/webtest-sheet-connect.js
 *
 * The ten shipped connectivity IIFEs are loaded once into a single shared Node
 * `vm` sandbox with stubbed window/document/localStorage/gbT/clock/uid. Ordered
 * case files live in ./webtest-sheet-connect/ (mirrors webtest-sheet-charts.js):
 *   00_harness.js        shared sandbox, DOM/storage doubles, assertions, report
 *   10_connect.js        core state, feed substitution/refresh, query-step chain
 *   11_sheets_connect.js connector registry + query facade runtime cases
 *   12_sheets_connect_more.js params/refresh/lineage/boot runtime + STATIC cases
 *
 * Browser-only rendering, real dialog focus trapping and backend refresh
 * scheduling remain browser/CI work and are not claimed here.
 */
"use strict";

var harness = require("./webtest-sheet-connect/00_harness");

require("./webtest-sheet-connect/10_connect");
require("./webtest-sheet-connect/11_sheets_connect");
require("./webtest-sheet-connect/12_sheets_connect_more");

harness.finish();
