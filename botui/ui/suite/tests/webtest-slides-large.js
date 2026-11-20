/**
 * Offline static-audit for #2529 slides large .pptx load path.
 * Run with: node botui/ui/suite/tests/webtest-slides-large.js
 *
 * This harness measures the CURRENT shipped state honestly. It scans the
 * shipped Slides modules (botui/ui/suite/slides/modules/*.js) and the slide
 * save/export Rust sources; it never edits production code, guards or baselines.
 *
 * Two assertion classes are reported:
 *   [BASELINE-OBSERVE] characterisation of what is shipped today (regression
 *                      detection), including real shipped JS run in a vm sandbox.
 *   [STATIC]           acceptance criteria that are NOT shipped; these document
 *                      scope only and never fabricate a runtime PASS.
 *
 * Ordered modules under ./webtest-slides-large/:
 *   00_harness.js          counters, assertions, source readers, report
 *   01_sources.js          shipped-source inventory + scan helpers
 *   02_sandbox.js          deterministic vm sandbox for real shipped modules
 *   10_media.js            media storage characterisation
 *   11_writes.js           per-edit write granularity characterisation
 *   12_render.js           render scope characterisation
 *   13_delta_vm.js         vm execution of 01_state.js and 13_delta.js
 *   14_static_deferred.js  [STATIC] criteria + size claims + findings
 *
 * Browser/CDP UAT and CI remain authoritative for runtime performance claims.
 */
"use strict";

var harness = require("./webtest-slides-large/00_harness");

require("./webtest-slides-large/01_sources");
require("./webtest-slides-large/02_sandbox");
require("./webtest-slides-large/10_media");
require("./webtest-slides-large/11_writes");
require("./webtest-slides-large/12_render");
require("./webtest-slides-large/13_delta_vm");
require("./webtest-slides-large/14_static_deferred");

harness.finish();
