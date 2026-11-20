/**
 * Offline verification for #2485 docs advanced find/replace helpers.
 * Run with: node botui/ui/suite/tests/webtest-docs-find.js
 *
 * The suite lives in ordered modules under ./webtest-docs-find/:
 *   00_harness.js  counters, assertions, vm loader, report
 *   10_find.js     matcher, regex guard, scope, undo and validation tests
 */
"use strict";

var harness = require("./webtest-docs-find/00_harness");

require("./webtest-docs-find/10_find");

harness.finish();
