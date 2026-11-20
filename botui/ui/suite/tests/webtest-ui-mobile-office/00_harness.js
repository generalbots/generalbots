/**
 * Core harness for the #2515 mobile-office offline suite.
 *
 * Ordered modules, counters, assertions, shared-vm loader and report. A local
 * logger is used because the browser-side gbLog global is absent in Node.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var REPO_DIR = path.resolve(__dirname, "..", "..", "..", "..", "..");
var PASS = 0;
var FAIL = 0;
var RESULTS = [];

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
function knownBug(name, fn) { record("KNOWN BUG — " + name, "KNOWN BUG", fn); }

function assert(condition, msg) {
  if (!condition) throw new Error(msg || "Assertion failed");
}

function assertEqual(actual, expected, msg) {
  if (actual !== expected) {
    throw new Error((msg || "values differ") + " :: <" + actual + "> != <" + expected + ">");
  }
}

function readFile(relPath) {
  return fs.readFileSync(path.join(SUITE_DIR, relPath), "utf8");
}

function readRepo(relPath) {
  return fs.readFileSync(path.join(REPO_DIR, relPath), "utf8");
}

function fileExists(relPath) {
  return fs.existsSync(path.join(SUITE_DIR, relPath));
}

function sourceHas(relPath, pattern, msg) {
  assert(new RegExp(pattern).test(readFile(relPath)), msg || relPath + " must contain " + pattern);
}

function sourceLacks(relPath, pattern, msg) {
  assert(!new RegExp(pattern).test(readFile(relPath)), msg || relPath + " must not contain " + pattern);
}

/** Load several ordered scripts into ONE fresh vm context. */
function loadModules(relPaths, sandbox) {
  var context = vm.createContext(sandbox);
  relPaths.forEach(function (relPath) {
    vm.runInContext(readFile(relPath), context, { filename: relPath });
  });
  return context;
}

function note(line) {
  RESULTS.push({ name: "[NOTE] " + line, status: "NOTE", kind: "NOTE" });
}

function finish() {
  var rule = "=".repeat(70);
  var counts = { RUNTIME: 0, STATIC: 0, "KNOWN BUG": 0 };
  RESULTS.forEach(function (result) {
    if (result.status === "PASS" && counts[result.kind] !== undefined) counts[result.kind]++;
  });
  console.log("\n" + rule);
  console.log("MOBILE OFFICE (#2515) — OFFLINE TEST RESULTS");
  console.log(rule);
  RESULTS.forEach(function (result) {
    console.log("[" + result.status + "] " + result.name + (result.error ? " — " + result.error : ""));
  });
  console.log(rule);
  console.log("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  console.log("RUNTIME PASS: " + counts.RUNTIME + " | STATIC PASS: " + counts.STATIC +
    " | KNOWN BUG (documented): " + counts["KNOWN BUG"]);
  console.log("MODE: one vm sandbox per fixture; two real shipped mobile IIFEs; stubbed " +
    "window/document/gbT/localStorage/PointerEvent/visualViewport/fixed clock/fixed uid");
  console.log("      [STATIC] = source assertions only, no runtime claim");
  console.log(rule);
  process.exitCode = FAIL > 0 ? 1 : 0;
}

module.exports = {
  fs: fs,
  path: path,
  vm: vm,
  SUITE_DIR: SUITE_DIR,
  REPO_DIR: REPO_DIR,
  test: test,
  staticTest: staticTest,
  knownBug: knownBug,
  assert: assert,
  assertEqual: assertEqual,
  readFile: readFile,
  readRepo: readRepo,
  fileExists: fileExists,
  sourceHas: sourceHas,
  sourceLacks: sourceLacks,
  loadModules: loadModules,
  note: note,
  finish: finish,
};
