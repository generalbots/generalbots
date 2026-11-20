/**
 * Sheets Enterprise Controls — Offline Test Suite (#2437).
 * Run with: node botui/ui/suite/tests/webtest-sheet-controls.js
 *
 * The eight REAL shipped control modules (44-51) are loaded into one vm
 * sandbox with stubbed window/document/localStorage/gbT, and their exported
 * pure helpers are verified offline. Source-bound promises are labelled STATIC.
 *
 * Ordered modules in ./webtest-sheet-controls/:
 *   00_harness.js   counters, assertions, module loading, report
 *   10_controls.js  protection/ACL/classification/link/audit/snapshot invariants
 *
 * Server enforcement, watermarked exports, Vault credentials, one-year
 * reconstructable history and browser/CDP UAT remain CI/browser-only work.
 */
"use strict";

var harness = require("./webtest-sheet-controls/00_harness");

require("./webtest-sheet-controls/10_controls");

harness.finish();
