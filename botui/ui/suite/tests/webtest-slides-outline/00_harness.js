/**
 * Slides review tools (#2433) — offline test harness.
 *
 * Loads the eight SHIPPED production IIFEs into ONE shared Node vm sandbox with
 * a deterministic stub for window/document/gbT/localStorage/clock/uid. The real
 * modules are never copied or reimplemented: every RUNTIME assertion drives the
 * exported namespaces (or their real DOM handlers) from this sandbox.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");
var dom = require("./00_dom");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var El = dom.El;
var DOC = dom.document;

var MODULES = [
  "slides/modules/20_outline_shared.js",
  "slides/modules/21_outline_view.js",
  "slides/modules/22_find_replace.js",
  "slides/modules/23_sections.js",
  "slides/modules/25_notes_workspace.js",
  "slides/modules/26_accessibility.js",
  "slides/modules/27_deck_properties.js",
  "slides/modules/28_outline_init.js"
];

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

function assert(condition, message) {
  if (!condition) throw new Error(message || "Assertion failed");
}

function equal(actual, expected, message) {
  assert(actual === expected,
    (message || "values differ") + " (got <" + actual + "> expected <" + expected + ">)");
}

function readFile(rel) {
  return fs.readFileSync(path.join(SUITE_DIR, rel), "utf8");
}

function noop() {}

// ── shared sandbox ──────────────────────────────────────────────────────
var CLOCK = 1700000000000;
var TIMERS = [];
var STORAGE = new Map();

var WIN = {
  gbT: function (key, fallback, params) {
    var text = String(fallback == null ? key : fallback);
    if (params) {
      text = text.replace(/\{(\w+)\}/g, function (full, name) {
        return Object.prototype.hasOwnProperty.call(params, name) ? String(params[name]) : full;
      });
    }
    return text;
  },
  localStorage: {
    getItem: function (key) { return STORAGE.has(String(key)) ? STORAGE.get(String(key)) : null; },
    setItem: function (key, value) { STORAGE.set(String(key), String(value)); },
    removeItem: function (key) { STORAGE.delete(String(key)); },
    clear: function () { STORAGE.clear(); }
  },
  setTimeout: function (fn, ms) { TIMERS.push({ fn: fn, ms: ms }); return TIMERS.length; },
  clearTimeout: noop,
  getSlidesPresentationId: function () { return "deck-A"; },
  getComputedStyle: function (el) { return { color: (el.style && el.style.color) || "" }; },
  refreshSlideView: function (index) { WIN.__jumps.push(index); },
  __jumps: [],
  __ajax: [],
  htmx: {
    process: noop,
    ajax: function (method, url, options) { WIN.__ajax.push([method, url, options]); }
  }
};

var CONTEXT = vm.createContext({
  window: WIN,
  document: DOC,
  console: { log: noop, warn: noop, error: noop, debug: noop },
  Date: { now: function () { return CLOCK; } }
});
MODULES.forEach(function (rel) {
  vm.runInContext(readFile(rel), CONTEXT, { filename: rel });
});

// ── fixture / state helpers ─────────────────────────────────────────────
function fixture() {
  DOC.body.textContent = "";
  WIN.__jumps.length = 0;
  WIN.__ajax.length = 0;
  var app = new El("div");
  app.id = "slides-app";
  DOC.body.appendChild(app);
  DOC.activeElement = null;
  return app;
}

function stackState() {
  var scope = WIN.GBSlidesOutline.scope();
  var undo = DOC.createElement("button");
  var redo = DOC.createElement("button");
  undo.className = "slides-tools-undo";
  redo.className = "slides-tools-redo";
  scope.appendChild(undo);
  scope.appendChild(redo);
  WIN.GBSlidesOutline.updateUndoUI();
  var state = { undo: undo.disabled, redo: redo.disabled };
  scope.removeChild(undo);
  scope.removeChild(redo);
  return state;
}

function resetUndo() {
  // Draining redo would push actions straight back onto the undo stack; the
  // next O.pushUndo clears redo on its own, so only the undo stack is drained.
  var guard = 0;
  while (!stackState().undo && guard++ < 200) WIN.GBSlidesOutline.undo();
}

function flushTimers() {
  var pending = TIMERS.splice(0, TIMERS.length);
  pending.forEach(function (timer) { timer.fn(); });
}

function byKey(key) {
  var all = DOC.querySelectorAll("*");
  for (var i = 0; i < all.length; i++) {
    if (all[i].getAttribute("data-i18n") === key) return all[i];
  }
  return null;
}

function gbLog(line) {
  (globalThis.gbLog && globalThis.gbLog.debug ? globalThis.gbLog.debug : function (text) {
    console.log(text);
  })(line);
}

function report() {
  var rule = "=".repeat(66);
  gbLog("\n" + rule);
  gbLog("SLIDES OUTLINE / REVIEW TOOLS — OFFLINE TEST RESULTS (#2433)");
  gbLog(rule);
  RESULTS.forEach(function (result) {
    gbLog("[" + result.status + "] " + result.name +
      (result.error ? " — " + result.error : ""));
  });
  gbLog(rule);
  gbLog("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  gbLog("MODE: RUNTIME (one shared vm sandbox, real shipped IIFEs) + labelled STATIC source checks");
  gbLog(rule);
  process.exitCode = FAIL > 0 ? 1 : 0;
}

module.exports = {
  test: test, assert: assert, equal: equal, readFile: readFile,
  finish: report, MODULES: MODULES, SUITE_DIR: SUITE_DIR,
  El: El, document: DOC, window: WIN,
  O: WIN.GBSlidesOutline,
  V: WIN.GBSlidesOutlineView,
  F: WIN.GBSlidesFindReplace,
  S: WIN.GBSlidesSections,
  N: WIN.GBSlidesNotes,
  A: WIN.GBSlidesAccessibility,
  P: WIN.GBSlidesProperties,
  R: WIN.GBSlidesReview,
  fixture: fixture, resetUndo: resetUndo, stackState: stackState,
  flushTimers: flushTimers, byKey: byKey,
  clearStorage: function () { STORAGE.clear(); },
  setClock: function (value) { CLOCK = value; },
  storageKeys: function () { return Array.from(STORAGE.keys()); }
};
