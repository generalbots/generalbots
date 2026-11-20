/**
 * Offline static boot-budget audit for issue #2495.
 * Run with: node botui/ui/suite/tests/webtest-ui-bootbudget.js
 *
 * This is a STATIC audit: it lexically scans the shipped suite HTML and the
 * shipped boot-budget.js loader. It does not launch a browser, does not run
 * Lighthouse and does not fabricate timing numbers.
 *
 * Ordered modules under ./webtest-ui-bootbudget/:
 *   00_harness.js   counters, assertions, HTML scanner, report
 *   10_measure.js   [BASELINE-OBSERVE] characterizations of today's counts
 *   20_criteria.js  [STATIC] gap characterizations for unshipped criteria
 */
"use strict";

var h = require("./webtest-ui-bootbudget/00_harness");

require("./webtest-ui-bootbudget/10_measure");
require("./webtest-ui-bootbudget/20_criteria");

h.finish();
