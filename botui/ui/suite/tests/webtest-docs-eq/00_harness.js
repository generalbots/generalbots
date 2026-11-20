/**
 * Offline harness for #2483 docs equations LaTeX → MathML verification.
 * Run with: node botui/ui/suite/tests/webtest-docs-eq.js
 *
 * Mirrors webtest-docs-find/00_harness.js (ordered modules, counters,
 * assertions, vm loader, report). A local logger is defined because the
 * browser-side gbLog global is not present in a bare Node process.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var TESTS_DIR = path.resolve(__dirname, "..");
var PASS = 0;
var FAIL = 0;
var RESULTS = [];

function log(message) {
  console.log(message);
}

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

function fileExists(relPath) {
  return fs.existsSync(path.join(SUITE_DIR, relPath));
}

function loadScript(relPath, sandbox) {
  var context = vm.createContext(sandbox);
  vm.runInContext(readFile(relPath), context, { filename: relPath });
  return context;
}

function report() {
  log("\n" + "=".repeat(60));
  log("DOCS EQUATIONS (LaTeX -> MathML) OFFLINE RESULTS (#2483)");
  log("=".repeat(60));
  RESULTS.forEach(function (r) {
    log("[" + r.status + "] " + r.name + (r.error ? " -- " + r.error : ""));
  });
  log("=".repeat(60));
  log("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  log("=".repeat(60));
  process.exitCode = FAIL > 0 ? 1 : 0;
}

function finish() {
  report();
}

module.exports = {
  fs,
  path,
  SUITE_DIR,
  TESTS_DIR,
  test,
  assert,
  readFile,
  fileExists,
  loadScript,
  log,
  finish,
};
