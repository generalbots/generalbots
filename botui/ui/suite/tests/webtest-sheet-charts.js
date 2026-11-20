/**
 * Sheet Charting Helpers — Offline Test Suite (#2490).
 * Run with: node botui/ui/suite/tests/webtest-sheet-charts.js
 *
 * The pure, DOM-free helpers of the shipped charting modules are loaded into
 * a vm sandbox and verified without a browser. Ordered modules live in
 * ./webtest-sheet-charts/ (mirrors webtest_gb7.js and webtest-sheet-a11y.js):
 *   00_harness.js   counters, assertions, module loading, report
 *   10_charts.js    data/svg/variant/hierarchy invariants and static checks
 *
 * Waterfall/histogram/Pareto/trendline rendering, XLSX chart-part round-trip
 * and vector export remain browser/CI-only work and are not claimed here.
 */
"use strict";

var harness = require("./webtest-sheet-charts/00_harness");

require("./webtest-sheet-charts/10_charts");

harness.finish();
