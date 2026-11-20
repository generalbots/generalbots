/**
 * Slides diagram/flowchart authoring — offline test harness (#2488).
 * Run with: node botui/ui/suite/tests/webtest-slides-diagram.js
 *
 * Shared harness: counters, assertions, VM module loading and report.
 * Mirrors webtest-sheet-a11y/00_harness.js but is self-contained for plain Node.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");

// The four real shipped modules. They are plain scripts (not ES modules) whose
// top-level `var` declarations bind SDPresets/SDIcons/SDFlow/SDOrg plus their
// data arrays. Loading them into ONE context lets them resolve each other's
// globals exactly as they do when slides.html includes them in order.
var MODULE_RELS = [
  "slides/modules/31_diag_presets.js",
  "slides/modules/32_diag_icons.js",
  "slides/modules/33_diag_flow.js",
  "slides/modules/34_diag_org.js"
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

// Minimal ambient stubs. The pure helpers only need sdT (i18n lookup), sdUid
// (id generator), GBSlidesDiagrams.slideIndex() and the slide dimensions.
// sdUid is a deterministic counter rather than the production Date/random
// generator so a whole diagram JSON is byte-reproducible; call resetUid()
// between two builds to compare them. DOM helpers are never invoked at runtime
// here — their coverage is explicitly labelled STATIC in 10_diagram.js.
function loadModules() {
  var uid = 0;
  var sandbox = {
    console: { warn: function () {}, log: function () {} },
    SLIDE_W: 960,
    SLIDE_H: 540,
    sdT: function (key, fallback) { return fallback; },
    sdUid: function (prefix) { uid += 1; return prefix + "_" + uid; },
    GBSlidesDiagrams: { slideIndex: function () { return 0; } },
    window: {
      gbT: function (key, fallback) { return fallback; }
    },
    document: {}
  };
  var context = vm.createContext(sandbox);
  MODULE_RELS.forEach(function (rel) {
    vm.runInContext(readFile(rel), context, { filename: rel });
  });
  return {
    presets: sandbox.SDPresets,
    presetDefs: sandbox.SD_PRESETS,
    icons: sandbox.SDIcons,
    iconDefs: sandbox.SD_ICONS,
    flow: sandbox.SDFlow,
    org: sandbox.SDOrg,
    resetUid: function () { uid = 0; },
    source: function (rel) { return readFile(rel); }
  };
}

var gbLog = globalThis.gbLog || { debug: function (line) { console.log(line); } };

function report() {
  var rule = "=".repeat(64);
  gbLog.debug("\n" + rule);
  gbLog.debug("SLIDES DIAGRAM/FLOWCHART — OFFLINE TEST RESULTS");
  gbLog.debug(rule);
  RESULTS.forEach(function (r) {
    gbLog.debug("[" + r.status + "] " + r.name + (r.error ? " — " + r.error : ""));
  });
  gbLog.debug(rule);
  gbLog.debug("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  gbLog.debug("MODE: runtime (vm sandbox, real shipped modules, stubbed window/document/gbT)");
  gbLog.debug("      + STATIC source assertions for DOM-only render helpers");
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
