/**
 * Offline static audit for #2528 — large .xlsx load path.
 * Run with: node botui/ui/suite/tests/webtest-sheet-large.js
 *
 * Scans shipped Sheet JS and botserver XLSX import/load/save Rust, then executes
 * safe shipped IIFEs in a vm sandbox to prove local request/window behaviour.
 * [BASELINE-OBSERVE] locks today's measured state, [STATIC] documents gaps and
 * [VM] proves deterministic local behaviour. No runtime, RSS, timing, transfer
 * size or browser evidence is fabricated; those remain browser/CI gates.
 *
 * Ordered modules under ./webtest-sheet-large/:
 *   00_harness.js       counters, scanners, cached source reads, report
 *   10_measure_js.js    [BASELINE-OBSERVE] shipped Sheet UI counts and paths
 *   11_measure_rust.js  [BASELINE-OBSERVE] shipped XLSX import/load/save facts
 *   20_criteria.js      [STATIC] gap / partial / deferred criteria
 *   30_vm.js            [VM] shipped IIFEs under stubbed window/fetch
 */
"use strict";

var harness = require("./webtest-sheet-large/00_harness");

require("./webtest-sheet-large/10_measure_js");
require("./webtest-sheet-large/11_measure_rust");
require("./webtest-sheet-large/20_criteria");
require("./webtest-sheet-large/30_vm");

harness.finish();
