/**
 * Offline verification for #2426 docs long-document structure helpers.
 * Run with: node botui/ui/suite/tests/webtest-docs-structure.js
 *
 * The suite lives in ordered modules under ./webtest-docs-structure/:
 *   00_harness.js  counters, assertions, vm loader, fake DOM, report
 *   10_structure.js  numbering, fields, captions and deferred-scope checks
 */
"use strict";

var harness = require("./webtest-docs-structure/00_harness");

require("./webtest-docs-structure/10_structure");

harness.finish();
