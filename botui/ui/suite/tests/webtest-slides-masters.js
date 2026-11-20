/**
 * Slides Masters / Layouts / Theme Tokens / Brand Kit — Offline Test Suite (#2429).
 * Run with: node botui/ui/suite/tests/webtest-slides-masters.js
 *
 * The DOM-free helpers of the five shipped modules (60-64) are loaded into one
 * VM sandbox and verified without a browser. Ordered files live in
 * ./webtest-slides-masters/ (mirrors webtest-slides-diagram.js):
 *   00_harness.js  counters, deterministic module loading, report
 *   10_masters.js  cascade, placeholders, overrides, apply, brand kit,
 *                  templates + labelled STATIC checks for DOM-only helpers
 *
 * Real PPTX master/layout/theme parts, PPTX re-import, server persistence and
 * real-browser UAT remain CI/browser work and are not claimed by this suite.
 */
"use strict";

var harness = require("./webtest-slides-masters/00_harness");

require("./webtest-slides-masters/10_masters");

harness.finish();
