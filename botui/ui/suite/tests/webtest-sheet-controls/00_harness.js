/**
 * Sheets enterprise controls — offline test harness (#2437).
 * Run with: node botui/ui/suite/tests/webtest-sheet-controls.js
 *
 * Loads the eight REAL shipped control modules into one vm sandbox with stubbed
 * window/document/localStorage/gbT. No production file is modified or copied.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var MODULE_DIR = "sheet/js/modules-sheet-advanced/";
var CONTROL_MODULES = [
  "44_controls_core.js",
  "45_controls_protection.js",
  "46_controls_inputs.js",
  "47_controls_acl.js",
  "48_controls_classification.js",
  "49_controls_links.js",
  "50_controls_audit.js",
  "51_controls_snapshot.js",
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

function moduleSource(fileName) {
  return readFile(MODULE_DIR + fileName);
}

/* Deterministic stand-in for gbT: interpolate catalog-style fallbacks so that
 * level/banner helpers are observable without hardcoding translated text. */
function gbT(key, fallback, params) {
  var text = String(fallback == null ? key : fallback);
  if (params) {
    Object.keys(params).forEach(function (name) {
      text = text.split("{" + name + "}").join(String(params[name]));
    });
  }
  return text;
}

function makeStorage() {
  var store = Object.create(null);
  return {
    getItem: function (key) {
      return Object.prototype.hasOwnProperty.call(store, key) ? store[key] : null;
    },
    setItem: function (key, value) { store[key] = String(value); },
    removeItem: function (key) { delete store[key]; },
    clear: function () { Object.keys(store).forEach(function (key) { delete store[key]; }); },
  };
}

function makeDocument() {
  return {
    body: null,
    activeElement: null,
    createElement: function () {
      return { setAttribute: function () {}, appendChild: function () {}, addEventListener: function () {} };
    },
    getElementById: function () { return null; },
    querySelectorAll: function () { return []; },
    addEventListener: function () {},
    removeEventListener: function () {},
  };
}

function loadControls() {
  var storage = makeStorage();
  var document = makeDocument();
  var window = { gbT: gbT };
  var noop = function () {};
  var sandbox = {
    window: window,
    document: document,
    localStorage: storage,
    console: { log: noop, warn: noop, error: noop, debug: noop },
  };
  var context = vm.createContext(sandbox);
  CONTROL_MODULES.forEach(function (fileName) {
    var rel = MODULE_DIR + fileName;
    vm.runInContext(readFile(rel), context, { filename: rel });
  });
  var core = window.SheetControls;
  return {
    window: window,
    document: document,
    storage: storage,
    core: core,
    protection: window.SheetControlsProtection,
    inputs: window.SheetControlsInputs,
    acl: window.SheetControlsAcl,
    classification: window.SheetControlsClassification,
    links: window.SheetControlsLinks,
    audit: window.SheetControlsAudit,
    snapshot: window.SheetControlsSnapshot,
    reset: function () { storage.clear(); },
    seed: function (mutator) {
      var next = core.state();
      mutator(next);
      core.save(next);
      return next;
    },
  };
}

var gbLog = globalThis.gbLog || { debug: function (line) { console.log(line); } };

function report() {
  var rule = "=".repeat(64);
  gbLog.debug("\n" + rule);
  gbLog.debug("SHEET ENTERPRISE CONTROLS — OFFLINE TEST RESULTS");
  gbLog.debug(rule);
  RESULTS.forEach(function (r) {
    gbLog.debug("[" + r.status + "] " + r.name + (r.error ? " — " + r.error : ""));
  });
  gbLog.debug(rule);
  gbLog.debug("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  gbLog.debug("MODE: runtime (vm sandbox; stubbed window/document/localStorage/gbT) + labelled STATIC checks");
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
  moduleSource: moduleSource,
  loadControls: loadControls,
  finish: finish,
  CONTROL_MODULES: CONTROL_MODULES,
  SUITE_DIR: SUITE_DIR,
};
