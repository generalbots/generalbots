/**
 * Offline static test-floor audit for issue #2544.
 * Run with: node botui/ui/suite/tests/webtest-test-floor.js
 *
 * This is a STATIC audit: it lexically scans the workspace Rust crates and CI
 * configuration. It never runs cargo, never launches a browser and never
 * fabricates a runtime PASS. It writes no baseline/ratchet file.
 *
 * Ordered modules under ./webtest-test-floor/:
 *   00_harness.js   counters, crate scanner, assertions, report
 *   10_measure.js   [BASELINE-OBSERVE] characterizations of today's tree
 *   20_criteria.js  [STATIC] gaps, real bugs and [CI-ONLY] deferred criteria
 */
"use strict";

var h = require("./webtest-test-floor/00_harness");

require("./webtest-test-floor/10_measure");
require("./webtest-test-floor/20_criteria");

h.finish();
