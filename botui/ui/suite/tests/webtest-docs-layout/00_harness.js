/**
 * Offline harness for #2424 docs layout/pagination helper verification.
 * Run with: node botui/ui/suite/tests/webtest-docs-layout.js
 *
 * Mirrors webtest-docs-find/00_harness.js structure (ordered modules, counters,
 * assertions, file helpers, report). A local logger is defined because the
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

/**
 * Load several ordered scripts into ONE shared vm context. The five layout
 * modules are IIFEs that extend the same window.DocsLayout namespace, so they
 * must observe one another's exports at load time (76 needs 75's NS, 78 calls
 * 79's resolvePageChrome during paginate, etc.).
 */
function loadModules(relPaths, sandbox) {
  var context = vm.createContext(sandbox);
  relPaths.forEach(function (relPath) {
    vm.runInContext(readFile(relPath), context, { filename: relPath });
  });
  return context;
}

/** Deterministic structural equality for plain data returned by the helpers. */
function deepEqual(a, b) {
  return JSON.stringify(a) === JSON.stringify(b);
}

function report() {
  log("\n" + "=".repeat(60));
  log("DOCS LAYOUT/PAGINATION OFFLINE RESULTS (#2424)");
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
  loadModules,
  deepEqual,
  log,
  finish,
};
