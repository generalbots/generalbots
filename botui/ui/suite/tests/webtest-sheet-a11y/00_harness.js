/**
 * Sheet accessibility checker — offline test harness (#2493).
 * Run with: node botui/ui/suite/tests/webtest-sheet-a11y.js
 *
 * Shared harness: counters, assertions, VM module loading and report.
 * Mirrors webtest-gb7/00_harness.js but is self-contained for plain Node.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var CHECKER_REL = "sheet/js/modules-sheet-advanced/24_a11y_checker.js";

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

// The checker is an IIFE that only touches `window` at load time (it assigns
// window.SheetA11yChecker). `document` and gbT are referenced lazily by the
// render path, which these DOM-free helper tests never call.
function loadChecker() {
  var source = readFile(CHECKER_REL);
  var sandbox = { window: {}, document: {}, console: { warn: function () {} } };
  vm.runInContext(source, vm.createContext(sandbox), { filename: CHECKER_REL });
  return sandbox.window.SheetA11yChecker;
}

var gbLog = globalThis.gbLog || { debug: function (line) { console.log(line); } };

function report() {
  var rule = "=".repeat(64);
  gbLog.debug("\n" + rule);
  gbLog.debug("SHEET A11Y CHECKER — OFFLINE TEST RESULTS");
  gbLog.debug(rule);
  RESULTS.forEach(function (r) {
    gbLog.debug("[" + r.status + "] " + r.name + (r.error ? " — " + r.error : ""));
  });
  gbLog.debug(rule);
  gbLog.debug("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  gbLog.debug("MODE: runtime (vm sandbox; stubbed window/document/gbT)");
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
  loadChecker: loadChecker,
  finish: finish,
  SUITE_DIR: SUITE_DIR,
  CHECKER_REL: CHECKER_REL,
};
