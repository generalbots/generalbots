/**
 * Slides Animation/Transition Timeline — Offline Test Suite (#2431).
 * Run with: node botui/ui/suite/tests/webtest-slides-anim.js
 *
 * Pure, DOM-free helpers of the six shipped animation modules are loaded into
 * a single Node vm sandbox and verified without a browser. The modules live in
 * ./webtest-slides-anim/ (mirrors webtest-slides-diagram/):
 *   00_harness.js    counters, assertions, real-module loading, report
 *   10_anim.js       namespaces, catalogs, effect CRUD/reorder, trigger resolution
 *   11_anim_more.js  engine frames, motion, reduce-motion, transitions,
 *                    deterministic JSON and labelled STATIC render checks
 *
 * Present-mode playback, drag/drop in a real canvas and PPTX timing round-trips
 * remain browser/CI-only work and are not claimed by this suite.
 */
"use strict";

var harness = require("./webtest-slides-anim/00_harness");

require("./webtest-slides-anim/10_anim");
require("./webtest-slides-anim/11_anim_more");

harness.finish();
