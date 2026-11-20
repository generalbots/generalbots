/**
 * Slides chart helpers — offline test harness (#2486).
 * Run with: node botui/ui/suite/tests/webtest-slides-chart.js
 *
 * Shared harness: counters, assertions, VM module loading and report. Mirrors
 * webtest-sheet-a11y/00_harness.js (added for #2493). The three shipped chart
 * modules are IIFEs that only touch `window` at load time (they bind
 * window.GBSlidesCharts and its style/link sub-namespaces); `document` and gbT
 * are referenced lazily by render paths these DOM-free tests never call.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var MODULE_RELS = [
  "slides/modules/75_charts_state.js",
  "slides/modules/76_charts_style.js",
  "slides/modules/80_charts_linking.js",
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

// One sandbox, three modules: they cooperate through window.GBSlidesCharts.
// localStorage is an in-memory stub so save/load/reset behave deterministically
// without touching the host filesystem.
function loadCharts() {
  var memory = {};
  var sandbox = {
    console: console,
    document: {},
    window: {
      gbT: function (key, fallback) { return fallback; },
      localStorage: {
        getItem: function (key) {
          return Object.prototype.hasOwnProperty.call(memory, key) ? memory[key] : null;
        },
        setItem: function (key, value) { memory[key] = String(value); },
        removeItem: function (key) { delete memory[key]; },
      },
    },
  };
  var context = vm.createContext(sandbox);
  MODULE_RELS.forEach(function (rel) {
    vm.runInContext(readFile(rel), context, { filename: rel });
  });
  return { C: sandbox.window.GBSlidesCharts, sandbox: sandbox, memory: memory };
}

var loaded = loadCharts();
var C = loaded.C;

// Start each stateful case from a clean deck-local chart state.
function reset() {
  C.reset();
  return C;
}

var gbLog = globalThis.gbLog || { debug: function (line) { console.log(line); } };

function report() {
  var rule = "=".repeat(64);
  gbLog.debug("\n" + rule);
  gbLog.debug("SLIDES CHART HELPERS — OFFLINE TEST RESULTS");
  gbLog.debug(rule);
  RESULTS.forEach(function (r) {
    gbLog.debug("[" + r.status + "] " + r.name + (r.error ? " — " + r.error : ""));
  });
  gbLog.debug(rule);
  gbLog.debug("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  gbLog.debug("MODE: runtime (vm sandbox; stubbed window/document/gbT) + labelled static source assertions");
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
  C: C,
  sandbox: loaded.sandbox,
  reset: reset,
  finish: finish,
  SUITE_DIR: SUITE_DIR,
  MODULE_RELS: MODULE_RELS,
};
