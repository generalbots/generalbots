/**
 * Slides Canvas Object Model — Offline Test Suite (#2430).
 * Run with: node botui/ui/suite/tests/webtest-slides-om.js
 *
 * The seven shipped object-model IIFEs (46-50) are read from disk and run in one
 * shared vm sandbox with a stubbed window/document/storage/clock/uid host. No
 * production function is copied or reimplemented. Ordered files live in
 * ./webtest-slides-om/ (mirrors webtest-slides-masters.js):
 *   00_harness.js  counters, shared sandbox module loading, fake host, report
 *   10_om.js       state/storage/undo, selection/hierarchy, z-order, align,
 *                  distribute, group/ungroup, deterministic state
 *   11_om_ui.js    media ops, canvas/dialog boot wiring + labelled STATIC checks
 *
 * Real-browser pointer UAT, server persistence and native PPTX round-trips
 * remain CI/browser work and are NOT claimed by this suite. Known production
 * bugs are asserted as such (labelled KNOWN-BUG) rather than silently masked.
 */
"use strict";

var harness = require("./webtest-slides-om/00_harness");

require("./webtest-slides-om/10_om");
require("./webtest-slides-om/11_om_ui");

harness.finish();
