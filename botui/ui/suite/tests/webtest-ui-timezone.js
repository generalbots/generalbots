/**
 * Offline static-audit + runtime harness for issue #2389 (timezone support).
 * Run with: node botui/ui/suite/tests/webtest-ui-timezone.js
 *
 * STATIC: scans the SHIPPED Rust timezone helpers, dependency manifests,
 *         UTC-derived-date call sites and the suite timezone UI sources.
 * RUNTIME: loads the REAL shipped clock.js IANA helpers into a Node vm and
 *          executes the REAL settings timezoneList() helper there too.
 * Nothing is patched, no browser is launched and no baseline/ratchet file is
 * read or written. [BASELINE-OBSERVE] locks are characterizations, not gates
 * on a target state. [STATIC][GAP] / [KNOWN BUG] lines are reported only.
 *
 * Ordered modules under ./webtest-ui-timezone/:
 *   00_harness.js   counters, source walkers, vm engines, gap/bug report
 *   10_measure.js   [BASELINE-OBSERVE] measurements of today's tree
 *   20_runtime.js   RUNTIME probes of real shipped JS helpers
 *   30_criteria.js  [STATIC] source criteria, gaps and report-only bugs
 */
"use strict";

var h = require("./webtest-ui-timezone/00_harness");

require("./webtest-ui-timezone/10_measure");
require("./webtest-ui-timezone/20_runtime");
require("./webtest-ui-timezone/30_criteria");

h.finish();
