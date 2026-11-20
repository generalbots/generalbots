/**
 * Sheet formula auditing offline harness (#2491).
 *
 * Loads the nine REAL shipped audit IIFEs from modules-sheet-advanced/ into ONE
 * Node vm sandbox with stubbed window/document/gbT/localStorage/fixed clock.
 * Nothing is reimplemented: every RUNTIME assertion executes the production
 * namespaces. Deferred browser/CI/backend criteria are labelled [STATIC] source
 * assertions; observed defects are labelled KNOWN BUG and never patched here.
 *
 * DOM sentinels are code-like tokens so the i18n literal scanner has no source
 * string to report. No dynamic i18n-key concatenation is used anywhere.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");
var dom = require("./dom");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var REPO_ROOT = path.resolve(SUITE_DIR, "..", "..", "..");
var MODULE_DIR = "sheet/js/modules-sheet-advanced/";
var FIXED_NOW = Date.UTC(2026, 0, 15, 12, 0, 0);
var FIXED_SHEET_ID = "sheet-audit-2491";

/* Exact order used by sheet.html (lines 303-311). 26_audit_eval defines the
 * parser 26a_audit_engine consumes, so it MUST precede 26a. */
var ORDER = [
  "25_audit_graph.js",
  "25a_audit_reports.js",
  "26_audit_eval.js",
  "26a_audit_engine.js",
  "27_audit_panel.js",
  "28_audit_sections.js",
  "28a_audit_sections2.js",
  "29_audit_overlay.js",
  "30_audit_boot.js",
];

function readSuite(rel) {
  return fs.readFileSync(path.join(SUITE_DIR, rel), "utf8");
}
function readRoot(rel) {
  return fs.readFileSync(path.join(REPO_ROOT, rel), "utf8");
}
function source(fileName) {
  return readSuite(MODULE_DIR + fileName);
}

/* ------------------------------ engine ----------------------------------- */

function makeBook() {
  return {
    name: "audit-book",
    calculation: {},
    worksheets: [
      { name: "Sheet1", data: {} },
      { name: "Sheet2", data: {} },
    ],
  };
}

function setCell(book, sheetIdx, row, col, data) {
  var ws = book.worksheets[sheetIdx];
  if (!ws) return null;
  var cell = { value: "", formula: "", comment: "" };
  Object.keys(data || {}).forEach(function (key) { cell[key] = data[key]; });
  ws.data[row + "," + col] = cell;
  return cell;
}

function createEngine() {
  var book = makeBook();
  var index = 0;
  var selection = null;
  var events = [];
  var docEvents = {};
  var winEvents = {};
  var rafQueue = [];
  var observers = [];
  var setRangeCalls = [];
  var switchCalls = [];
  var i18nCalls = [];

  var doc = {
    readyState: "complete",
    documentElement: { getAttribute: function () { return "en"; }, lang: "en" },
    body: dom.makeNode("<body>"),
    activeElement: null,
    byId: {},
    createElement: function (tag) { return dom.makeNode(tag); },
    createElementNS: function (namespace, tag) { return dom.makeNode(tag); },
    getElementById: function (id) {
      if (doc.byId[id]) return doc.byId[id];
      return doc.body.querySelector("#" + id);
    },
    addEventListener: function (type, handler) {
      (docEvents[type] || (docEvents[type] = [])).push(handler);
    },
  };
  var auditButton = dom.makeNode("button");
  auditButton.id = "sheetAuditBtn";
  doc.byId.sheetAuditBtn = auditButton;

  var grid = {
    cells: new Map(),
    bodyInner: dom.makeNode("<grid>"),
    editingCell: null,
    onCellValueChanged: null,
  };

  function gbT(key, fallback, params) {
    i18nCalls.push({ key: key, fallback: fallback });
    var out = fallback == null ? String(key) : String(fallback);
    if (params && typeof params === "object") {
      Object.keys(params).forEach(function (name) {
        out = out.split("{" + name + "}").join(String(params[name]));
      });
    }
    return out;
  }

  var inst = {
    loadedSheet: book,
    worksheetIndex: index,
    host: doc.body,
    grid: grid,
  };

  var win = {
    document: doc,
    gbT: gbT,
    localStorage: dom.makeStorage(),
    CustomEvent: function (type, options) {
      this.type = type;
      this.detail = options && options.detail;
      this.bubbles = options && options.bubbles;
    },
    SheetDefaults: {
      WORKSHEET_INDEX: 0,
      getActiveSheetInstance: function () { return inst; },
    },
    SheetVirtualGrid: grid,
    SheetCore: {
      currentSheetId: function () { return FIXED_SHEET_ID; },
      wsIndex: function () { return index; },
      colX: function (c) { return 48 + c * 96; },
      colWidth: function () { return 96; },
      rowTop: function (r) { return r * 24; },
      rowHeight: function () { return 24; },
    },
    SheetAdvanced: {
      getSelection: function () { return selection; },
      setRange: function () {
        setRangeCalls.push(Array.prototype.slice.call(arguments));
      },
    },
    SheetTabs: {
      switchWorksheet: function (next) { switchCalls.push(next); },
    },
    SheetI18n: {
      getLocale: function () { return "en"; },
      t: gbT,
      localize: function () {},
    },
    gbFormatTime: function () { return "12:00:00"; },
    dispatchEvent: function (event) {
      events.push(event);
      (winEvents[event.type] || []).forEach(function (handler) { handler(event); });
    },
    addEventListener: function (type, handler) {
      (winEvents[type] || (winEvents[type] = [])).push(handler);
    },
  };

  function MutationObserver(callback) {
    this.callback = callback;
    this.targets = [];
    this.disconnected = false;
    observers.push(this);
  }
  MutationObserver.prototype.observe = function (target, options) {
    this.targets.push({ target: target, options: options });
  };
  MutationObserver.prototype.disconnect = function () { this.disconnected = true; };
  MutationObserver.prototype.fire = function () {
    if (!this.disconnected) this.callback([], this);
  };

  var sandbox = {
    window: win,
    document: doc,
    localStorage: win.localStorage,
    navigator: { clipboard: { writeText: function () { return Promise.resolve(); } } },
    MutationObserver: MutationObserver,
    CustomEvent: win.CustomEvent,
    console: { log: function () {}, warn: function () {}, error: function () {}, debug: function () {} },
    Date: dom.makeFixedDate(FIXED_NOW),
    requestAnimationFrame: function (cb) { rafQueue.push(cb); return rafQueue.length; },
    setTimeout: function (cb) { rafQueue.push(cb); return rafQueue.length; },
  };
  sandbox.globalThis = sandbox;
  var context = vm.createContext(sandbox);
  ORDER.forEach(function (fileName) {
    vm.runInContext(source(fileName), context, { filename: fileName });
  });

  function flushRaf() {
    var queued = rafQueue.splice(0);
    queued.forEach(function (cb) { cb(); });
    return queued.length;
  }
  function triggerMutation() {
    observers.forEach(function (observer) { observer.fire(); });
    return observers.length;
  }
  function setIndex(next) {
    index = next;
    inst.worksheetIndex = next;
  }
  function setSelection(value) {
    selection = value;
  }
  function setBook(next) {
    book = next;
    inst.loadedSheet = book;
    win.__LOADED_SHEET = book;
  }
  function dispatchDoc(type) {
    (docEvents[type] || []).forEach(function (handler) { handler({ type: type }); });
  }

  return {
    context: context, sandbox: sandbox, window: win, doc: doc, grid: grid,
    inst: inst, auditButton: auditButton, events: events, docEvents: docEvents,
    winEvents: winEvents, observers: observers, i18nCalls: i18nCalls,
    setRangeCalls: setRangeCalls, switchCalls: switchCalls,
    book: function () { return book; },
    index: function () { return index; },
    selection: function () { return selection; },
    setIndex: setIndex, setSelection: setSelection, setBook: setBook,
    setCell: function (s, r, c, data) { return setCell(book, s, r, c, data); },
    flushRaf: flushRaf, triggerMutation: triggerMutation, dispatchDoc: dispatchDoc,
    graph: win.SheetAuditGraph,
    eval: win.SheetAuditEval,
    evalInternals: win.__SheetAuditEvalInternals,
    graphInternals: win.__SheetAuditGraphInternals,
    ui: win.SheetAuditUI,
    sections: win.SheetAuditSections,
    overlay: win.SheetAuditOverlay,
    advanced: win.SheetAdvancedAudit,
    tr: win.SheetAuditT,
    fixedSheetId: FIXED_SHEET_ID,
  };
}

/* --------------------------- test bookkeeping ---------------------------- */

var PASS = 0;
var FAIL = 0;
var RESULTS = [];

function errorMessage(error) {
  return error && error.message ? error.message : String(error);
}
function record(name, kind, status, error) {
  if (status === "PASS") PASS++;
  else FAIL++;
  RESULTS.push({ name: name, kind: kind, status: status, error: error || null });
}
function run(name, kind, fn) {
  try {
    fn();
    record(name, kind, "PASS");
  } catch (error) {
    record(name, kind, "FAIL", errorMessage(error));
  }
}
function test(name, fn) { run(name, "RUNTIME", fn); }
function staticTest(name, fn) { run("[STATIC] " + name, "STATIC", fn); }
function knownBug(name, fn) { run("KNOWN BUG \u2014 " + name, "KNOWN BUG", fn); }

function assert(condition, message) {
  if (!condition) throw new Error(message || "Assertion failed");
}
function equal(actual, expected, message) {
  assert(actual === expected, (message || "values differ") +
    " \u2014 expected <" + expected + ">, got <" + actual + ">");
}
function deep(actual, expected, message) {
  var a = JSON.stringify(actual);
  var b = JSON.stringify(expected);
  assert(a === b, (message || "structures differ") + " \u2014 expected " + b + ", got " + a);
}
function throws(fn, message) {
  var raised = false;
  try { fn(); } catch (error) { raised = true; }
  assert(raised, message || "expected the call to throw");
}

function sourceHas(fileName, pattern, message) {
  assert(new RegExp(pattern).test(source(fileName)), message || "source pattern missing");
}
function sourceLacks(fileName, pattern, message) {
  assert(!new RegExp(pattern).test(source(fileName)), message || "unexpected source pattern");
}
function rootHas(rel, pattern, message) {
  assert(new RegExp(pattern).test(readRoot(rel)), message || "root source pattern missing");
}

function finish() {
  var line = "=".repeat(68);
  var counts = { RUNTIME: 0, STATIC: 0, "KNOWN BUG": 0 };
  RESULTS.forEach(function (result) {
    if (result.status === "PASS" && counts[result.kind] !== undefined) counts[result.kind]++;
  });
  var out = globalThis.gbLog || { debug: function (s) { console.log(s); } };
  out.debug("\n" + line);
  out.debug("SHEET FORMULA AUDITING \u2014 OFFLINE TEST RESULTS (#2491)");
  out.debug(line);
  RESULTS.forEach(function (result) {
    out.debug("[" + result.status + "] " + result.name +
      (result.error ? " \u2014 " + result.error : ""));
  });
  out.debug(line);
  out.debug("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  out.debug("RUNTIME PASS: " + counts.RUNTIME + " | STATIC PASS: " + counts.STATIC +
    " | KNOWN BUG (documented): " + counts["KNOWN BUG"]);
  out.debug("MODE: one vm sandbox; nine real shipped IIFEs; stubbed window/document/" +
    "gbT/localStorage/MutationObserver/fixed clock/fixed uid");
  out.debug("      [STATIC] = source assertions only, no runtime claim");
  out.debug(line);
  process.exitCode = FAIL > 0 ? 1 : 0;
}

module.exports = {
  SUITE_DIR: SUITE_DIR, REPO_ROOT: REPO_ROOT, MODULE_DIR: MODULE_DIR,
  ORDER: ORDER, FIXED_NOW: FIXED_NOW, FIXED_SHEET_ID: FIXED_SHEET_ID,
  makeBook: makeBook, setCell: setCell, makeNode: dom.makeNode,
  createEngine: createEngine,
  readSuite: readSuite, readRoot: readRoot, source: source,
  test: test, staticTest: staticTest, knownBug: knownBug,
  assert: assert, equal: equal, deep: deep, throws: throws,
  sourceHas: sourceHas, sourceLacks: sourceLacks, rootHas: rootHas,
  finish: finish,
};
