/**
 * Offline verification for #2482 docs rich-table helpers.
 * Run with: node botui/ui/suite/tests/webtest-docs-table.js
 *
 * The suite lives in ordered modules under ./webtest-docs-table/:
 *   00_harness.js  counters, fake DOM, vm loader, report
 *   10_table.js    real-module runtime invariants + source contracts
 */
"use strict";

var harness = require("./webtest-docs-table/00_harness");

require("./webtest-docs-table/10_table");

harness.finish();
