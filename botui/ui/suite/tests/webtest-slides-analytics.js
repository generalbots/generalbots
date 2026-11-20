/**
 * Slides Presentation Analytics — Offline Test Suite (#2489).
 * Run with: node botui/ui/suite/tests/webtest-slides-analytics.js
 *
 * Pure, DOM-free helpers of the four shipped analytics layers are loaded into
 * a minimal VM sandbox and verified without a browser. The ordered modules
 * live in ./webtest-slides-analytics/ (mirrors webtest_gb7.js):
 *   00_harness.js   counters, assertions, module loading, report
 *   10_analytics.js state shape, aggregation, percentages, polls, Q&A, exports
 *
 * Live Q&A, real poll voting and follow-along UAT remain browser/CI-only work.
 */
"use strict";

var harness = require("./webtest-slides-analytics/00_harness");

require("./webtest-slides-analytics/10_analytics");

harness.finish();
