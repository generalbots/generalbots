/**
 * Core harness for #2533 docs large .docx load path verification.
 *
 * Mirrors webtest-docs-review/00_harness.js structure (ordered modules,
 * counters, assertions, shared-vm loader, report). A local logger is used
 * because the browser-side gbLog global is absent in a bare Node process.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var REPO_DIR = path.resolve(__dirname, "..", "..", "..", "..", "..");
var SUITE_DIR = path.resolve(__dirname, "..", "..");
var PASS = 0;
var FAIL = 0;
var RESULTS = [];

function test(name, fn) {
  try {
    fn();
    PASS++;
    RESULTS.push({ name: name, status: "PASS" });
  } catch (error) {
    FAIL++;
    RESULTS.push({ name: name, status: "FAIL", error: error.message });
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

function readFile(relPath) {
  return fs.readFileSync(path.join(SUITE_DIR, relPath), "utf8");
}

function readRepo(relPath) {
  return fs.readFileSync(path.join(REPO_DIR, relPath), "utf8");
}

function fileExists(relPath) {
  return fs.existsSync(path.join(SUITE_DIR, relPath));
}

function deepEqual(a, b) {
  return JSON.stringify(a) === JSON.stringify(b);
}

/** Load several ordered scripts into ONE shared vm context. */
function loadModules(relPaths, sandbox) {
  var context = vm.createContext(sandbox);
  relPaths.forEach(function (relPath) {
    vm.runInContext(readFile(relPath), context, { filename: relPath });
  });
  return context;
}

/** Record an informational line (deferred scope) without counting it as a test. */
function note(text) {
  RESULTS.push({ name: "[NOTE] " + text, status: "NOTE" });
}

function report() {
  console.log("\n" + "=".repeat(64));
  console.log("DOCS LARGE .DOCX LOAD PATH OFFLINE RESULTS (#2533)");
  console.log("=".repeat(64));
  RESULTS.forEach(function (result) {
    console.log("[" + result.status + "] " + result.name + (result.error ? " -- " + result.error : ""));
  });
  console.log("=".repeat(64));
  console.log("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  console.log("=".repeat(64));
  process.exitCode = FAIL > 0 ? 1 : 0;
}

function finish() {
  report();
}

module.exports = {
  fs,
  path,
  vm,
  REPO_DIR,
  SUITE_DIR,
  test,
  assert,
  assertEqual,
  readFile,
  readRepo,
  fileExists,
  deepEqual,
  loadModules,
  note,
  finish,
};
