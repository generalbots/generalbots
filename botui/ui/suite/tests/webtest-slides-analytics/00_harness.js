/**
 * Slides presentation analytics — offline test harness (#2489).
 * Run with: node botui/ui/suite/tests/webtest-slides-analytics.js
 *
 * Shared harness: counters, assertions, VM module loading and report.
 * Mirrors webtest-sheet-a11y/00_harness.js but is self-contained for plain Node.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");

// Load order matters: 66 defines the shared A namespace and base helpers that
// 67/68/69 extend. All four are IIFEs binding window.GBSlidesAnalytics.
var ANALYTICS_MODULES = [
  "slides/modules/66_analytics_state.js",
  "slides/modules/67_analytics_compute.js",
  "slides/modules/68_analytics_qa.js",
  "slides/modules/69_analytics_polls.js",
];

var PASS = 0;
var FAIL = 0;
var RESULTS = [];

function test(name, fn) {
  try {
    fn();
    PASS++;
    RESULTS.push({ name: name, status: "PASS" });
  } catch (e) {
    FAIL++;
    RESULTS.push({ name: name, status: "FAIL", error: e.message });
  }
}

function assert(condition, msg) {
  if (!condition) throw new Error(msg || "Assertion failed");
}

function readFile(relPath) {
  return fs.readFileSync(path.join(SUITE_DIR, relPath), "utf8");
}

// The four modules only read `window` at load time (they assign/extend
// window.GBSlidesAnalytics). `document` is stubbed to prove no DOM is needed;
// gbT is stubbed to return the caller's fallback so no locale catalog is read.
function loadAnalytics() {
  var sandbox = { window: {}, document: {}, console: { warn: function () {} } };
  sandbox.window.gbT = function (key, fallback) { return fallback; };
  var context = vm.createContext(sandbox);
  ANALYTICS_MODULES.forEach(function (rel) {
    vm.runInContext(readFile(rel), context, { filename: rel });
  });
  return sandbox.window.GBSlidesAnalytics;
}

var gbLog = globalThis.gbLog || { debug: function (line) { console.log(line); } };

function report() {
  var rule = "=".repeat(64);
  gbLog.debug("\n" + rule);
  gbLog.debug("SLIDES ANALYTICS — OFFLINE TEST RESULTS (#2489)");
  gbLog.debug(rule);
  RESULTS.forEach(function (r) {
    gbLog.debug("[" + r.status + "] " + r.name + (r.error ? " — " + r.error : ""));
  });
  gbLog.debug(rule);
  gbLog.debug("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  gbLog.debug("MODE: runtime (vm sandbox; stubbed window/document/gbT) + labelled static source checks");
  gbLog.debug(rule);
  process.exitCode = FAIL > 0 ? 1 : 0;
}

function finish() {
  report();
}

module.exports = {
  test: test,
  assert: assert,
  readFile: readFile,
  loadAnalytics: loadAnalytics,
  finish: finish,
  SUITE_DIR: SUITE_DIR,
  ANALYTICS_MODULES: ANALYTICS_MODULES,
};
