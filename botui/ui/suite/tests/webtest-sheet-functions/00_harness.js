/**
 * Sheet functions engine — offline test harness (#2434).
 * Run with: node botui/ui/suite/tests/webtest-sheet-functions.js
 *
 * Loads the eight REAL shipped IIFE modules into one vm sandbox with stubbed
 * window/document/gbT. Pure helpers are reached through their genuine exported
 * namespaces; nothing is reimplemented here. Mirrors webtest-sheet-a11y/00_harness.js.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var MODULES_REL = "sheet/js/modules-sheet-advanced/";
var ORDER = [
  "73_functions_errors.js",
  "74_functions_financial.js",
  "75_functions_date.js",
  "76a_functions_base.js",
  "76_functions_stat_text.js",
  "77_functions_catalog.js",
  "78a_functions_parser.js",
  "78_functions_evaluator.js",
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

function approx(actual, expected, tolerance, msg) {
  assert(typeof actual === "number" && Number.isFinite(actual),
    msg + ": expected a finite number, got " + String(actual));
  var delta = Math.abs(actual - expected);
  assert(delta <= (tolerance || 1e-9),
    msg + ": expected " + expected + ", got " + actual + " (delta " + delta + ")");
}

function readFile(relPath) {
  return fs.readFileSync(path.join(SUITE_DIR, relPath), "utf8");
}

function source(fileName) {
  return readFile(MODULES_REL + fileName);
}

function sourceHas(fileName, pattern, msg) {
  assert(new RegExp(pattern).test(source(fileName)), msg);
}

function sourceLacks(fileName, pattern, msg) {
  assert(!new RegExp(pattern).test(source(fileName)), msg);
}

/**
 * One fresh engine per suite run. The settings object is mutable so date-system
 * and separator tests can patch it and restore it without reloading modules.
 */
function loadEngine() {
  var state = {
    dateSystem: "1900",
    locale: "en",
    separators: { decimal: ".", group: "," },
    iterative: { enabled: false, maxIterations: 100, tolerance: 1e-7 },
    precisionAsDisplayed: false,
  };
  var sandbox = {
    console: { log: function () {}, warn: function () {}, error: function () {} },
  };
  sandbox.window = {};
  sandbox.document = {
    getElementById: function () { return null; },
    createElement: function () { return { style: {}, setAttribute: function () {}, appendChild: function () {} }; },
    addEventListener: function () {},
  };
  sandbox.gbT = function (key, fallback) { return fallback === undefined ? key : fallback; };
  sandbox.window.document = sandbox.document;
  sandbox.window.gbT = sandbox.gbT;
  sandbox.window.SheetFunctionsSettings = { state: function () { return state; } };
  var context = vm.createContext(sandbox);
  ORDER.forEach(function (fileName) {
    vm.runInContext(source(fileName), context, { filename: fileName });
  });
  var w = sandbox.window;
  /* Dates must be created inside the VM realm: the shipped date helper uses
   * `value instanceof Date`, which is false for a host-realm Date object. */
  function makeDate(year, monthIndex, day) {
    return vm.runInContext(
      "new Date(Date.UTC(" + year + "," + monthIndex + "," + day + "))", context);
  }
  function setSettings(patch) {
    var before = {};
    Object.keys(patch).forEach(function (key) { before[key] = state[key]; state[key] = patch[key]; });
    return function () {
      Object.keys(before).forEach(function (key) { state[key] = before[key]; });
    };
  }
  return {
    window: w,
    state: state,
    makeDate: makeDate,
    setSettings: setSettings,
    errors: w.SheetFormulaErrors,
    financial: w.SheetFunctionsFinancial,
    date: w.SheetFunctionsDate,
    base: w.SheetFunctionsBase,
    statText: w.SheetFunctionsStatText,
    catalog: w.SheetFunctionsCatalog,
    parser: w.SheetFunctionsParser,
    evaluator: w.SheetFunctionsEvaluator,
  };
}

var gbLog = globalThis.gbLog || { debug: function (line) { console.log(line); } };

function note(line) {
  gbLog.debug("    " + line);
}

function report() {
  var rule = "=".repeat(64);
  gbLog.debug("\n" + rule);
  gbLog.debug("SHEET FUNCTIONS ENGINE — OFFLINE TEST RESULTS (#2434)");
  gbLog.debug(rule);
  RESULTS.forEach(function (r) {
    gbLog.debug("[" + r.status + "] " + r.name + (r.error ? " — " + r.error : ""));
  });
  gbLog.debug(rule);
  gbLog.debug("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  gbLog.debug("MODE: runtime (vm sandbox; real shipped modules, stubbed window/document/gbT)");
  gbLog.debug("      + explicitly labelled STATIC source assertions");
  gbLog.debug(rule);
  process.exitCode = FAIL > 0 ? 1 : 0;
}

function finish() {
  report();
}

module.exports = {
  test: test,
  assert: assert,
  approx: approx,
  readFile: readFile,
  source: source,
  sourceHas: sourceHas,
  sourceLacks: sourceLacks,
  loadEngine: loadEngine,
  note: note,
  finish: finish,
  ORDER: ORDER,
  SUITE_DIR: SUITE_DIR,
};
