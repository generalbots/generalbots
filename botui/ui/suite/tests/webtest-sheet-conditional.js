/**
 * Sheets conditional formatting / cash-flow invariants — Offline Test Suite (#2260).
 * Run with: node botui/ui/suite/tests/webtest-sheet-conditional.js
 *
 * The two shipped advanced-sheet IIFEs (02_conditional.js and
 * 07_conditional_render.js) are loaded verbatim into one shared Node `vm`
 * sandbox (see ./webtest-sheet-conditional/00_harness.js). Real invariants —
 * rule model, negative-red/positive-green colour rules, add/edit/remove of
 * rules, range application and deterministic render output — are proved by
 * EXECUTING shipped code. Orchestration/backend-deferred aspects are covered
 * by clearly-labelled [STATIC] source assertions; observed bugs are captured
 * as [KNOWN BUG] characterization tests and reported, never patched here.
 *
 * Browser/CDP verification of the rendered grid and real backend persistence
 * remain separate gates.
 */
"use strict";

var harness = require("./webtest-sheet-conditional/00_harness");

require("./webtest-sheet-conditional/10_conditional");
require("./webtest-sheet-conditional/11_conditional_render");
require("./webtest-sheet-conditional/12_conditional_static");

harness.finish();
