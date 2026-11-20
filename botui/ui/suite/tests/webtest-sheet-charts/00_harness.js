/**
 * Sheet charting helpers — offline test harness (#2490).
 * Run with: node botui/ui/suite/tests/webtest-sheet-charts.js
 *
 * The shipped charting modules are loaded unchanged into a vm sandbox with
 * stubbed window/document/sessionStorage/gbT/gbFormatNumber. Only the pure,
 * DOM-free helpers are exercised; renderers are called with hand-built ctx
 * objects and never touch a real DOM.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var MODULE_DIR = "sheet/js/modules-sheet-advanced/";
var DATA_REL = MODULE_DIR + "31_charting_data.js";
var PALETTE_REL = MODULE_DIR + "32_charting_palette.js";
var SVG_REL = MODULE_DIR + "33_charting_svg.js";
var HIERARCHY_REL = MODULE_DIR + "34b_charting_types_hierarchy.js";
var VARIANTS_REL = MODULE_DIR + "34c_charting_types_variants.js";

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

function equal(actual, expected, msg) {
  assert(actual === expected, (msg || "values differ") + " — expected " + expected + ", got " + actual);
}

function deep(actual, expected, msg) {
  var a = JSON.stringify(actual);
  var b = JSON.stringify(expected);
  assert(a === b, (msg || "structures differ") + " — expected " + b + ", got " + a);
}

function readFile(relPath) {
  return fs.readFileSync(path.join(SUITE_DIR, relPath), "utf8");
}

// Load the real modules in dependency order. 34b/34c grab the registry object
// at load time, so it must exist before they run. The palette is a genuine
// shipped, DOM-free dependency of both variant modules.
function loadModules() {
  var sandbox = {
    window: {},
    document: {},
    sessionStorage: { getItem: function () { return "0"; } },
    console: { log: function () {}, warn: function () {} },
  };
  sandbox.gbT = function (key, fallback) { return fallback == null ? key : fallback; };
  sandbox.window.gbT = sandbox.gbT;
  var context = vm.createContext(sandbox);

  function run(rel) {
    vm.runInContext(readFile(rel), context, { filename: rel });
  }

  run(DATA_REL);
  run(PALETTE_REL);
  run(SVG_REL);
  sandbox.window.SheetChartingTypes = { renderers: {} };
  run(HIERARCHY_REL);
  run(VARIANTS_REL);

  // Cross-realm Map must be constructed inside the vm for `instanceof Map`.
  vm.runInContext("window.__makeMap = function (pairs) { return new Map(pairs); };", context);

  return {
    sandbox: sandbox,
    window: sandbox.window,
    data: sandbox.window.SheetChartingData,
    palette: sandbox.window.SheetChartingPalette,
    svg: sandbox.window.SheetChartingSvg,
    renderers: sandbox.window.SheetChartingTypes.renderers,
    makeMap: sandbox.window.__makeMap,
    withFormat: function (fn, callback) {
      var previous = sandbox.window.gbFormatNumber;
      if (fn) sandbox.window.gbFormatNumber = fn; else delete sandbox.window.gbFormatNumber;
      try {
        return callback();
      } finally {
        if (previous === undefined) delete sandbox.window.gbFormatNumber;
        else sandbox.window.gbFormatNumber = previous;
      }
    },
  };
}

var gbLog = globalThis.gbLog || { debug: function (line) { console.log(line); } };

function report() {
  var rule = "=".repeat(64);
  gbLog.debug("\n" + rule);
  gbLog.debug("SHEET CHARTING HELPERS — OFFLINE TEST RESULTS (#2490)");
  gbLog.debug(rule);
  RESULTS.forEach(function (r) {
    gbLog.debug("[" + r.status + "] " + r.name + (r.error ? " — " + r.error : ""));
  });
  gbLog.debug(rule);
  gbLog.debug("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  gbLog.debug("MODE: runtime (vm sandbox; shipped modules, stubbed host globals) + labelled static source checks");
  gbLog.debug(rule);
  process.exitCode = FAIL > 0 ? 1 : 0;
}

function finish() {
  report();
}

module.exports = {
  test: test,
  assert: assert,
  equal: equal,
  deep: deep,
  readFile: readFile,
  loadModules: loadModules,
  finish: finish,
  SUITE_DIR: SUITE_DIR,
  DATA_REL: DATA_REL,
  PALETTE_REL: PALETTE_REL,
  SVG_REL: SVG_REL,
  HIERARCHY_REL: HIERARCHY_REL,
  VARIANTS_REL: VARIANTS_REL,
};
