/**
 * Mobile native-intake offline static audit — issue #2510.
 * Run with: node botui/ui/suite/tests/webtest-ui-intake.js
 *
 * Read-only scan of the shipped Drive intake surface: drive.html file inputs,
 * 07_events.js capability/bindings, the PWA manifest and its share_target,
 * sw.js share interception, the share-intake page, the shell registration and
 * the intake catalog keys. Nothing is patched, no guard/baseline is touched.
 *
 * [BASELINE-OBSERVE] locks today's observed state as regression detection.
 * [STATIC] criteria that are not shippable offline are reported as gaps, never
 * as runtime PASS. Observed bugs are reported with file:line evidence only.
 *
 * Ordered modules under ./webtest-ui-intake/:
 *   00_harness.js  file access, counters, assertions, report
 *   10_scan.js     source scan + [OBSERVE] mechanical invariants
 *   20_assert.js   [BASELINE-OBSERVE] locks, [STATIC] criteria, bugs and gaps
 */
"use strict";

var h = require("./webtest-ui-intake/00_harness");

require("./webtest-ui-intake/10_scan");
require("./webtest-ui-intake/20_assert");

h.finish();
