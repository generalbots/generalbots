/**
 * Sheets Calculation Functions — Offline Test Suite (#2434).
 * Run with: node botui/ui/suite/tests/webtest-sheet-functions.js
 *
 * The eight shipped function IIFEs (73-78) are loaded from disk into a VM
 * sandbox and their genuine exported helpers are exercised without a browser.
 * Ordered modules under ./webtest-sheet-functions/ (mirrors webtest_gb7.js):
 *   00_harness.js  counters, assertions, real-module loader, report
 *   10_functions.js error semantics, financial/date/stat/text, parser, catalog
 *
 * XLSX round-trip, the function-wizard UI and browser/CDP UAT remain CI-only.
 */
"use strict";

var harness = require("./webtest-sheet-functions/00_harness");

require("./webtest-sheet-functions/10_functions");

harness.finish();
