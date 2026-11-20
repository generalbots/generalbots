/**
 * Offline verification for #2424 docs real layout/pagination preview helpers.
 * Run with: node botui/ui/suite/tests/webtest-docs-layout.js
 *
 * The suite lives in ordered modules under ./webtest-docs-layout/:
 *   00_harness.js  counters, assertions, shared-vm loader, report
 *   10_layout.js   pure geometry, rules, pagination and header/footer checks
 *
 * This exercises the shipped frontend PREVIEW model only. Real PDF fidelity,
 * PDF/UA and print byte-comparison are backend/renderer work and are not
 * claimed here.
 */
"use strict";

var harness = require("./webtest-docs-layout/00_harness");

require("./webtest-docs-layout/10_layout");

harness.finish();
