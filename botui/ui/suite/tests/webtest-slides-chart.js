/**
 * Slides Chart Helpers — Offline Test Suite (#2486).
 * Run with: node botui/ui/suite/tests/webtest-slides-chart.js
 *
 * The pure, DOM-free helpers of three shipped chart modules are loaded into a
 * minimal VM sandbox and verified without a browser. Ordered modules live in
 * ./webtest-slides-chart/ (mirrors webtest_gb7.js and webtest-sheet-a11y.js):
 *   00_harness.js   counters, assertions, module loading, report
 *   10_chart.js     state/style/linking invariants + labelled static checks
 *
 * Native PPTX chart round-trip, live Sheet refresh and real-slide UAT remain
 * CI/browser-only work and are NOT claimed by this offline harness.
 */
"use strict";

var harness = require("./webtest-slides-chart/00_harness");

require("./webtest-slides-chart/10_chart");

harness.finish();
