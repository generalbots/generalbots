/**
 * Sheet Accessibility Checker — Offline Test Suite (#2493).
 * Run with: node botui/ui/suite/tests/webtest-sheet-a11y.js
 *
 * Pure, DOM-free helpers of the production checker are loaded into a minimal
 * VM sandbox and verified without a browser. The ordered modules live in
 * ./webtest-sheet-a11y/ (mirrors webtest_gb7.js):
 *   00_harness.js   counters, assertions, module loading, report
 *   10_checker.js   report-category invariants and determinism checks
 *
 * Screen-reader and visual UAT remain browser/CI-only work.
 */
"use strict";

var harness = require("./webtest-sheet-a11y/00_harness");

require("./webtest-sheet-a11y/10_checker");

harness.finish();
