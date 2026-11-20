/**
 * Sheet conditional formatting (#2260) — offline test harness.
 * Run with: node botui/ui/suite/tests/webtest-sheet-conditional.js
 *
 * The two shipped advanced-sheet IIFEs (02_conditional.js and
 * 07_conditional_render.js) are read verbatim and executed in ONE shared Node
 * `vm` context with stubbed window/document/localStorage/sessionStorage/gbT/
 * fixed clock/uid/fetch/timers. No production logic is reimplemented: every
 * namespace exercised below is the real one the browser builds. DOM sentinels
 * are code-like tokens so the i18n hardcoded-literal scanner has nothing to
 * report. Backend-deferred/missing-orchestration aspects are covered only by
 * labelled [STATIC] source assertions in 12_conditional_static.js.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var ROOT_DIR = path.resolve(SUITE_DIR, "..", "..", "..");
var MODULE_DIR = "sheet/js/modules-sheet-advanced/";
var SOURCES = {
  conditional: MODULE_DIR + "02_conditional.js",
  render: MODULE_DIR + "07_conditional_render.js",
};

var PASS = 0;
var FAIL = 0;
var RESULTS = [];
var PENDING = [];

function errorMessage(e) {
  return e && e.message ? e.message : String(e);
}

function record(name, status, error) {
  FAIL += status === "FAIL" ? 1 : 0;
  PASS += status === "PASS" ? 1 : 0;
  RESULTS.push({ name: name, status: status, error: error || null });
}

/* Every case file calls test(); async functions (returned promises) are
 * collected and awaited by finish() before the report is printed. */
function test(name, fn) {
  try {
    var result = fn();
    if (result && typeof result.then === "function") {
      PENDING.push(result.then(
        function () { record(name, "PASS"); },
        function (e) { record(name, "FAIL", errorMessage(e)); }
      ));
    } else {
      record(name, "PASS");
    }
  } catch (e) {
    record(name, "FAIL", errorMessage(e));
  }
}

function assert(condition, message) {
  if (!condition) throw new Error(message || "Assertion failed");
}

function equal(actual, expected, message) {
  assert(actual === expected,
    (message || "values differ") + " — expected <" + expected + ">, got <" + actual + ">");
}

function deep(actual, expected, message) {
  var a = JSON.stringify(actual);
  var b = JSON.stringify(expected);
  assert(a === b, (message || "structures differ") + " — expected " + b + ", got " + a);
}

function readSuite(rel) {
  return fs.readFileSync(path.join(SUITE_DIR, rel), "utf8");
}

function readRoot(rel) {
  return fs.readFileSync(path.join(ROOT_DIR, rel), "utf8");
}

function makeStorage() {
  var map = {};
  return {
    getItem: function (key) {
      return Object.prototype.hasOwnProperty.call(map, key) ? map[key] : null;
    },
    setItem: function (key, value) { map[key] = String(value); },
    removeItem: function (key) { delete map[key]; },
    clear: function () { map = {}; },
    keys: function () { return Object.keys(map); },
  };
}

function silentConsole() {
  return { log: function () {}, warn: function () {}, error: function () {}, debug: function () {} };
}

function makeFixedDate(fixedNow) {
  var RealDate = Date;
  function FixedDate() {
    if (!(this instanceof FixedDate)) return new RealDate(fixedNow).toString();
    if (arguments.length === 0) return new RealDate(fixedNow);
    return Reflect.construct(RealDate, Array.prototype.slice.call(arguments));
  }
  FixedDate.prototype = RealDate.prototype;
  FixedDate.now = function () { return fixedNow; };
  FixedDate.UTC = RealDate.UTC;
  FixedDate.parse = RealDate.parse;
  return FixedDate;
}

/* DOMStringMap semantics: dataset assignments are coerced to strings. */
function makeDataset() {
  return new Proxy({}, {
    set: function (target, key, value) {
      target[key] = String(value);
      return true;
    },
  });
}

/* Node double: enough of the element API for the shipped renderer to create
 * validation/comment markers and to locate grid cells by data-row/data-col. */
function makeNode(tag) {
  var node = {
    tagName: String(tag || "<node>").toUpperCase(),
    className: "",
    textContent: "",
    title: "",
    style: {},
    dataset: makeDataset(),
    attributes: {},
    children: [],
    isConnected: true,
    parentNode: null,
    appendChild: function (child) {
      child.parentNode = node;
      node.children.push(child);
      return child;
    },
    removeChild: function (child) {
      var i = node.children.indexOf(child);
      if (i >= 0) node.children.splice(i, 1);
      child.parentNode = null;
      return child;
    },
    setAttribute: function (name, value) { node.attributes[name] = String(value); },
    getAttribute: function (name) {
      return Object.prototype.hasOwnProperty.call(node.attributes, name) ? node.attributes[name] : null;
    },
    querySelector: function (selector) { return descendants(node, selector)[0] || null; },
    querySelectorAll: function (selector) { return descendants(node, selector); },
  };
  return node;
}

function descendants(node, selector) {
  var out = [];
  (function walk(parent) {
    parent.children.forEach(function (child) { out.push(child); walk(child); });
  })(node);
  return out.filter(function (child) { return matches(child, selector); });
}

function matches(node, selector) {
  selector = String(selector || "").trim();
  var coord = selector.match(/^\[data-row="(\d+)"\]\[data-col="(\d+)"\]$/);
  if (coord) {
    return String(node.dataset.row) === coord[1] && String(node.dataset.col) === coord[2];
  }
  if (selector.charAt(0) === ".") {
    return (" " + (node.className || "") + " ").indexOf(" " + selector.slice(1) + " ") >= 0;
  }
  return false;
}

function makeDoc() {
  return {
    readyState: "complete",
    body: makeNode("<body>"),
    createElement: function (tag) { return makeNode(tag); },
    createElementNS: function (namespace, tag) { return makeNode(tag); },
    getElementById: function () { return null; },
    querySelector: function () { return null; },
    querySelectorAll: function () { return []; },
    addEventListener: function () {},
    removeEventListener: function () {},
  };
}

/* A visibleRowRange end is EXCLUSIVE: rows 0 .. end-1 are rendered. */
function makeGrid(rows, cols, visibleEnd) {
  var g = {
    cells: new Map(),
    bodyInner: makeNode("<grid>"),
    totalCols: cols,
    renderCount: 0,
    cellNodes: {},
  };
  g.visibleRowRange = function () { return { start: 0, end: visibleEnd == null ? rows : visibleEnd }; };
  g.visibleColRange = function () { return { start: 0, end: cols }; };
  g.render = function () { g.renderCount++; };
  g.set = function (r, c, value, extra) {
    var data = { value: value };
    if (extra) Object.keys(extra).forEach(function (k) { data[k] = extra[k]; });
    g.cells.set(r + "," + c, data);
    return data;
  };
  g.node = function (r, c) { return g.cellNodes[r + "," + c] || null; };
  for (var r = 0; r < rows; r++) {
    for (var c = 0; c < cols; c++) {
      var cell = makeNode("<cell>");
      cell.dataset.row = String(r);
      cell.dataset.col = String(c);
      g.bodyInner.appendChild(cell);
      g.cellNodes[r + "," + c] = cell;
    }
  }
  return g;
}

function loadModules() {
  var fixedNow = Date.UTC(2026, 0, 15, 12, 0, 0);
  var currentSelection = null;
  var grid = null;
  var i18nCalls = [];
  var fetchCalls = [];
  var timers = [];
  var fetchImpl = function () {
    return Promise.resolve({ json: function () { return Promise.resolve({ success: true }); } });
  };

  var store = makeStorage();
  var session = makeStorage();
  var doc = makeDoc();

  function gbT(key, fallback, params) {
    var out = fallback == null ? String(key) : String(fallback);
    if (params && typeof params === "object") {
      Object.keys(params).forEach(function (name) {
        out = out.split("{" + name + "}").join(String(params[name]));
      });
    }
    return out;
  }

  var win = {
    gbT: gbT,
    localStorage: store,
    sessionStorage: session,
    __SHEET_INITIAL_ID: "sheet-cashflow-2026",
    __LOADED_SHEET: null,
    SheetAdvanced: { getSelection: function () { return currentSelection; } },
    SheetCore: {},
    SheetI18n: {
      t: function (key, fallback) {
        i18nCalls.push({ key: key, fallback: fallback });
        return fallback;
      },
    },
    addEventListener: function () {},
    removeEventListener: function () {},
  };
  win.SheetCore.getGrid = function () { return grid; };

  var sandbox = {
    window: win,
    document: doc,
    localStorage: store,
    sessionStorage: session,
    gbT: gbT,
    console: silentConsole(),
    fetch: function (url, init) {
      fetchCalls.push({ url: url, init: init });
      return fetchImpl(url, init);
    },
    setTimeout: function (fn, delay) {
      timers.push({ fn: fn, delay: delay });
      return timers.length;
    },
    clearTimeout: function () {},
    Date: makeFixedDate(fixedNow),
  };
  var maths = Object.create(Math);
  maths.random = function () { return 0.5; };
  sandbox.Math = maths;

  var context = vm.createContext(sandbox);
  Object.keys(SOURCES).forEach(function (key) {
    vm.runInContext(readSuite(SOURCES[key]), context, { filename: SOURCES[key] });
  });

  return {
    window: win,
    document: doc,
    context: context,
    sandbox: sandbox,
    fetchCalls: fetchCalls,
    timers: timers,
    i18nCalls: i18nCalls,
    setSelection: function (sel) { currentSelection = sel; },
    setGrid: function (g) { grid = g; },
    setSheet: function (sheet) { win.__LOADED_SHEET = sheet; },
    setActiveIndex: function (i) { session.setItem("sheet_active_index", String(i)); },
    setFetchResponse: function (j) {
      fetchImpl = function () {
        return Promise.resolve({ json: function () { return Promise.resolve(j); } });
      };
    },
    setFetchFailure: function () {
      fetchImpl = function () { return Promise.reject(new Error("<network>")); };
    },
    clearFetch: function () { fetchCalls.length = 0; },
    flushTimers: function () {
      var list = timers.splice(0);
      list.forEach(function (t) { t.fn(); });
    },
    makeGrid: makeGrid,
    reset: function () {
      currentSelection = null;
      grid = null;
      win.__LOADED_SHEET = null;
      fetchImpl = function () {
        return Promise.resolve({ json: function () { return Promise.resolve({ success: true }); } });
      };
      fetchCalls.length = 0;
      timers.length = 0;
      i18nCalls.length = 0;
      session.clear();
      store.clear();
    },
  };
}

var gbLog = globalThis.gbLog || { debug: function (line) { console.log(line); } };

async function finish() {
  if (PENDING.length) await Promise.all(PENDING);
  var rule = "=".repeat(66);
  gbLog.debug("\n" + rule);
  gbLog.debug("SHEET CONDITIONAL FORMATTING — OFFLINE TEST RESULTS (#2260)");
  gbLog.debug(rule);
  RESULTS.forEach(function (result) {
    gbLog.debug("[" + result.status + "] " + result.name + (result.error ? " — " + result.error : ""));
  });
  gbLog.debug(rule);
  gbLog.debug("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  gbLog.debug("MODE: runtime (one vm sandbox per case file; shipped IIFEs, stubbed host globals) + labelled static source checks");
  gbLog.debug(rule);
  process.exitCode = FAIL > 0 ? 1 : 0;
}

module.exports = {
  test: test,
  assert: assert,
  equal: equal,
  deep: deep,
  readSuite: readSuite,
  readRoot: readRoot,
  loadModules: loadModules,
  finish: finish,
  SOURCES: SOURCES,
  SUITE_DIR: SUITE_DIR,
  ROOT_DIR: ROOT_DIR,
};
