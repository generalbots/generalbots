/**
 * Offline verification for #2483 docs equations LaTeX → MathML converter.
 * Run with: node botui/ui/suite/tests/webtest-docs-eq.js
 *
 * The suite lives in ordered modules under ./webtest-docs-eq/:
 *   00_harness.js  counters, assertions, vm loader, report
 *   10_eq.js       converter, escaping, balance, budgets and static guards
 */
"use strict";

var harness = require("./webtest-docs-eq/00_harness");

require("./webtest-docs-eq/10_eq");

harness.finish();
