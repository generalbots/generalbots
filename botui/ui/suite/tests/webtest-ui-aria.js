/**
 * Suite UI assistive-technology static audit — issue #2331.
 * Run with: node botui/ui/suite/tests/webtest-ui-aria.js
 *
 * Offline, read-only characterization of the SHIPPED suite sources: HTML and
 * JS under botui/ui/suite with the tests subtree excluded. It counts live
 * regions, status/alert roles, i18n coverage and dynamic text writes, then
 * locks the measured numbers as [BASELINE-OBSERVE] characterizations.
 *
 * The ordered modules live in ./webtest-ui-aria/ (same layout as the other
 * webtest runners):
 *   00_harness.js  counters, file walk, assertions, report
 *   10_scan.js     scanner + [OBSERVE] invariants
 *   20_assert.js   [BASELINE-OBSERVE] locks + [STATIC] criteria and gaps
 *   30_wcag.js     [STATIC][WCAG-*] source-level WCAG 2.2 AA proxies
 *
 * Nothing here patches production code, guard files or baselines. Screen-reader
 * and browser UAT remain the driver's job; see [STATIC][GAP] in the report.
 */
"use strict";

var harness = require("./webtest-ui-aria/00_harness");

require("./webtest-ui-aria/10_scan");
require("./webtest-ui-aria/20_assert");
require("./webtest-ui-aria/30_wcag");

harness.finish();
