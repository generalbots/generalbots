/**
 * Core harness for the #2457 offline Office-fidelity harness.
 *
 * Ordered modules, counters, assertions, shared-vm loader and a report that
 * also prints the feature-coverage matrix derived from recorded test results.
 * A local logger is used because the browser-side gbLog global is absent in
 * Node. No file outside this directory is written by this suite.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var REPO_DIR = path.resolve(__dirname, "..", "..", "..", "..", "..");
var PASS = 0;
var FAIL = 0;
var DEGRADED = 0;
var RESULTS = [];
var COVERAGE = [];

function record(name, kind, fn) {
  try {
    fn();
    PASS++;
    RESULTS.push({ name: name, kind: kind, status: "PASS" });
  } catch (error) {
    FAIL++;
    RESULTS.push({ name: name, kind: kind, status: "FAIL", error: error.message });
  }
}

function test(name, fn) { record(name, "RUNTIME", fn); }
function staticTest(name, fn) { record("[STATIC] " + name, "STATIC", fn); }

/** A confirmed, documented degradation: fn must ASSERT the degraded shape. */
function knownBug(name, fn) {
  try {
    fn();
    DEGRADED++;
    RESULTS.push({ name: name, kind: "KNOWN BUG", status: "DEGRADED" });
  } catch (error) {
    FAIL++;
    RESULTS.push({ name: name, kind: "KNOWN BUG", status: "FAIL", error: error.message });
  }
}

function assert(condition, msg) {
  if (!condition) throw new Error(msg || "Assertion failed");
}

function assertEqual(actual, expected, msg) {
  if (actual !== expected) {
    throw new Error((msg || "values differ") + " :: <" + actual + "> != <" + expected + ">");
  }
}

function assertGreater(actual, floor, msg) {
  if (!(actual > floor)) {
    throw new Error((msg || "expected value above floor") + " :: <" + actual + "> <= <" + floor + ">");
  }
}

function assertThrows(fn, pattern, msg) {
  var threw = false;
  var message = "";
  try { fn(); } catch (error) { threw = true; message = error.message; }
  assert(threw, msg || "expected function to throw");
  if (pattern) assert(new RegExp(pattern).test(message),
    (msg || "wrong error") + " :: <" + message + "> !~ /" + pattern + "/");
}

function readSuite(relPath) {
  return fs.readFileSync(path.join(SUITE_DIR, relPath), "utf8");
}
function readRepo(relPath) {
  return fs.readFileSync(path.join(REPO_DIR, relPath), "utf8");
}
function fileExistsRepo(relPath) {
  return fs.existsSync(path.join(REPO_DIR, relPath));
}
function sourceHas(relPath, pattern, msg) {
  assert(new RegExp(pattern).test(readRepo(relPath)), msg || relPath + " must contain " + pattern);
}

/** Load several ordered scripts into ONE fresh vm context. */
function loadModules(relPaths, sandbox) {
  var context = vm.createContext(sandbox);
  relPaths.forEach(function (relPath) {
    vm.runInContext(readSuite(relPath), context, { filename: relPath });
  });
  return context;
}

function note(line) {
  RESULTS.push({ name: line, status: "NOTE", kind: "NOTE" });
}
function setCoverage(rows) { COVERAGE = rows || []; }
function results() { return RESULTS.slice(); }

function printCoverage() {
  if (!COVERAGE.length) return;
  var rule = "-".repeat(70);
  console.log("\n" + rule);
  console.log("FEATURE COVERAGE MATRIX (derived from PASS/DEGRADED/FAIL results)");
  console.log(rule);
  console.log("app      feature            status      evidence");
  COVERAGE.forEach(function (row) {
    console.log(
      String(row.app).padEnd(9) +
      String(row.feature).padEnd(19) +
      String(row.status).padEnd(12) +
      String(row.evidence || "").slice(0, 60)
    );
  });
  console.log(rule);
}

function finish() {
  var rule = "=".repeat(70);
  var counts = { RUNTIME: 0, STATIC: 0, "KNOWN BUG": 0 };
  RESULTS.forEach(function (result) {
    if ((result.status === "PASS" || result.status === "DEGRADED") && counts[result.kind] !== undefined) {
      counts[result.kind]++;
    }
  });
  console.log("\n" + rule);
  console.log("OFFICE FIDELITY (#2457) — OFFLINE TEST RESULTS");
  console.log(rule);
  RESULTS.forEach(function (result) {
    console.log("[" + result.status + "] " + result.name + (result.error ? " — " + result.error : ""));
  });
  printCoverage();
  console.log(rule);
  console.log("TOTAL: " + (PASS + FAIL + DEGRADED) + " | PASS: " + PASS +
    " | DEGRADED (documented): " + DEGRADED + " | FAIL: " + FAIL);
  console.log("RUNTIME PASS: " + counts.RUNTIME + " | STATIC PASS: " + counts.STATIC +
    " | KNOWN BUG confirmed: " + counts["KNOWN BUG"]);
  console.log("MODE: real shipped export/preview IIFEs in a stubbed Node vm; " +
    "hand-built + generated OOXML corpus in a store-only ZIP; no server, no browser, no cargo.");
  console.log("      [STATIC] = source assertions only; DEGRADED = measured, disclosed loss");
  console.log("      KNOWN BUG tests characterise present defects; update them when a fix lands.");
  console.log(rule);
  process.exitCode = FAIL > 0 ? 1 : 0;
}

module.exports = {
  fs: fs, path: path, vm: vm,
  SUITE_DIR: SUITE_DIR, REPO_DIR: REPO_DIR,
  test: test, staticTest: staticTest, knownBug: knownBug,
  assert: assert, assertEqual: assertEqual, assertGreater: assertGreater, assertThrows: assertThrows,
  readSuite: readSuite, readRepo: readRepo, fileExistsRepo: fileExistsRepo, sourceHas: sourceHas,
  loadModules: loadModules, note: note,
  setCoverage: setCoverage, results: results, finish: finish,
};
