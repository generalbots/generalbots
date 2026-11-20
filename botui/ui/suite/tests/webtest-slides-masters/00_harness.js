/**
 * Slides masters / layouts / theme tokens / brand kit — offline test harness (#2429).
 * Run with: node botui/ui/suite/tests/webtest-slides-masters.js
 *
 * Shared harness: counters, assertions, deterministic VM loading of the five
 * real shipped modules (60-64) and a report. Mirrors the style of
 * webtest-slides-diagram/00_harness.js without editing that suite.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");

// Plain scripts (not ES modules): each IIFE attaches its helpers to the single
// window.GBSlidesMasters namespace. Load order matters — 62/63/64 capture
// M.tr from module 60 at load time. Module 65 (DOM wiring) is deliberately not
// loaded; its assembly needs a real document and is a browser/CI concern.
var MODULE_RELS = [
  "slides/modules/60_masters_state.js",
  "slides/modules/61_masters_cascade.js",
  "slides/modules/62_masters_dialog.js",
  "slides/modules/63_masters_panels_a.js",
  "slides/modules/64_masters_panels_b.js"
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

// Deterministic stand-ins. These replace only time, id generation, storage and
// i18n lookup — never a shipped helper's result. uuids are a counter (like the
// diagram suite's sdUid stub) so layout/theme/template ids are reproducible.
function loadModules() {
  var uidSeq = 0;
  var store = {};
  var storage = {
    getItem: function (key) {
      return Object.prototype.hasOwnProperty.call(store, key) ? store[key] : null;
    },
    setItem: function (key, value) { store[key] = String(value); },
    removeItem: function (key) { delete store[key]; },
    clear: function () { store = {}; }
  };
  function FixedDate() {}
  FixedDate.now = function () { return 1767225600000; };
  FixedDate.prototype.toISOString = function () { return "2026-01-01T00:00:00.000Z"; };

  var sandbox = {
    console: { log: function () {}, warn: function () {}, error: function () {} },
    Date: FixedDate,
    document: {
      querySelectorAll: function () { return []; },
      getElementById: function () { return null; },
      addEventListener: function () {},
      body: {}
    },
    window: {
      gbT: function (key, fallback) { return fallback; },
      __SLIDES_PRESENTATION_ID: "masters-offline-deck",
      localStorage: storage,
      getSlidesPresentationId: function () { return null; }
    }
  };
  var context = vm.createContext(sandbox);
  MODULE_RELS.forEach(function (rel) {
    vm.runInContext(readFile(rel), context, { filename: rel });
  });

  var M = sandbox.window.GBSlidesMasters;
  M.uid = function (prefix) { uidSeq += 1; return (prefix || "id") + "_" + uidSeq; };

  function resetState() {
    M.state = M.fresh();
    M.undoStack.length = 0;
    M.redoStack.length = 0;
    M.listeners = {};
    uidSeq = 0;
    storage.clear();
    return M.state;
  }

  return {
    masters: M,
    sandbox: sandbox,
    storage: storage,
    resetState: resetState,
    resetUid: function () { uidSeq = 0; },
    source: function (rel) { return readFile(rel); }
  };
}

var gbLog = globalThis.gbLog || { debug: function (line) { console.log(line); } };

function report() {
  var rule = "=".repeat(64);
  gbLog.debug("\n" + rule);
  gbLog.debug("SLIDES MASTERS / LAYOUTS / BRAND KIT — OFFLINE TEST RESULTS");
  gbLog.debug(rule);
  RESULTS.forEach(function (r) {
    gbLog.debug("[" + r.status + "] " + r.name + (r.error ? " — " + r.error : ""));
  });
  gbLog.debug(rule);
  gbLog.debug("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  gbLog.debug("MODE: runtime (vm sandbox, real shipped modules 60-64, stubbed window/document/gbT/clock/uid)");
  gbLog.debug("      + STATIC source assertions for DOM-bound dialog/panel helpers (never executed)");
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
  loadModules: loadModules,
  finish: finish,
  SUITE_DIR: SUITE_DIR,
  MODULE_RELS: MODULE_RELS
};
