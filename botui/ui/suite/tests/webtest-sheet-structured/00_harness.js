/**
 * Sheets structured data & what-if — offline test harness (#2436).
 * Run with: node botui/ui/suite/tests/webtest-sheet-structured.js
 *
 * Loads the five shipped IIFE modules in one vm sandbox with stubbed
 * window/document/gbT/localStorage/sessionStorage. Nothing in production is
 * modified or duplicated: only genuine exported helpers are invoked at
 * runtime; DOM-bound or private helpers are exercised through exported
 * effects or labelled STATIC source assertions.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var CORE_REL = "sheet/js/modules-sheet-advanced/38_structured_core.js";
var MODULE_RELS = [
  CORE_REL,
  "sheet/js/modules-sheet-advanced/39_structured_sparklines.js",
  "sheet/js/modules-sheet-advanced/40_structured_slicer.js",
  "sheet/js/modules-sheet-advanced/41_structured_goalseek.js",
  "sheet/js/modules-sheet-advanced/42_structured_scenarios.js",
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

function makeStorage() {
  var data = Object.create(null);
  return {
    getItem: function (key) { return Object.prototype.hasOwnProperty.call(data, key) ? data[key] : null; },
    setItem: function (key, value) { data[key] = String(value); },
    removeItem: function (key) { delete data[key]; },
    clear: function () { data = Object.create(null); },
  };
}

function makeElement(tag) {
  var node = {
    tagName: String(tag || "").toUpperCase(),
    className: "",
    children: [],
    attributes: Object.create(null),
    listeners: Object.create(null),
    style: {},
    dataset: {},
    classList: { toggle: function () {} },
    textContent: "",
    innerHTML: "",
    value: "",
    type: "",
    id: "",
    disabled: false,
    tabIndex: 0,
    isConnected: false,
    parentNode: null,
    setAttribute: function (name, value) { this.attributes[name] = String(value); },
    getAttribute: function (name) {
      return Object.prototype.hasOwnProperty.call(this.attributes, name) ? this.attributes[name] : null;
    },
    removeAttribute: function (name) { delete this.attributes[name]; },
    addEventListener: function (type, fn) {
      if (!this.listeners[type]) this.listeners[type] = [];
      this.listeners[type].push(fn);
    },
    removeEventListener: function (type, fn) {
      var list = this.listeners[type] || [];
      var at = list.indexOf(fn);
      if (at >= 0) list.splice(at, 1);
    },
    appendChild: function (child) {
      child.parentNode = this;
      child.isConnected = true;
      this.children.push(child);
      return child;
    },
    removeChild: function (child) {
      var at = this.children.indexOf(child);
      if (at >= 0) this.children.splice(at, 1);
      child.isConnected = false;
      return child;
    },
    remove: function () {
      if (this.parentNode) this.parentNode.removeChild(this);
      else this.isConnected = false;
    },
    contains: function (other) {
      if (other === this) return true;
      return this.children.some(function (child) { return child.contains(other); });
    },
    querySelector: function () { return null; },
    querySelectorAll: function () { return []; },
    focus: function () { doc.activeElement = this; },
  };
  return node;
}

function fire(node, type, extra) {
  var event = {
    target: node,
    key: "",
    shiftKey: false,
    preventDefault: function () {},
  };
  Object.keys(extra || {}).forEach(function (key) { event[key] = extra[key]; });
  (node.listeners[type] || []).slice().forEach(function (fn) { fn(event); });
}

function findAll(root, predicate) {
  var found = [];
  (function walk(node) {
    node.children.forEach(function (child) {
      if (predicate(child)) found.push(child);
      walk(child);
    });
  })(root);
  return found;
}

var doc = makeElement("document");
doc.body = makeElement("body");
doc.body.isConnected = true;
doc.activeElement = null;
doc.createElement = makeElement;
doc.getElementById = function () { return null; };

function loadModules() {
  var local = makeStorage();
  var session = makeStorage();
  var sandbox = {
    console: { warn: function () {}, error: function () {}, log: function () {} },
    document: doc,
    localStorage: local,
    sessionStorage: session,
    gbT: function (key, fallback) { return fallback; },
  };
  var context = vm.createContext(sandbox);
  sandbox.window = sandbox;
  sandbox.sheetNotify = null;
  sandbox.SheetUndo = null;
  MODULE_RELS.forEach(function (rel) {
    vm.runInContext(readFile(rel), context, { filename: rel });
  });
  return {
    window: sandbox,
    context: context,
    document: doc,
    localStorage: local,
    sessionStorage: session,
    S: sandbox.SheetStructured,
    Spark: sandbox.SheetStructuredSparklines,
    Slicer: sandbox.SheetStructuredSlicer,
    Goal: sandbox.SheetStructuredGoalSeek,
    Scenarios: sandbox.SheetStructuredScenarios,
  };
}

// Maps must be built inside the vm realm for the modules' `instanceof Map`.
function newCellMap(env) {
  return vm.runInContext("new Map()", env.context);
}

function setCell(env, ref, data) {
  var pos = env.S.parseRef(ref);
  if (!pos) throw new Error("bad fixture ref: " + ref);
  env.window.SheetVirtualGrid.cells.set(pos.row + "," + pos.col, data);
}

function attachGrid(env) {
  env.window.SheetVirtualGrid = { cells: newCellMap(env) };
  return env.window.SheetVirtualGrid;
}

var gbLog = globalThis.gbLog || { debug: function (line) { console.log(line); } };

function report() {
  var rule = "=".repeat(64);
  gbLog.debug("\n" + rule);
  gbLog.debug("SHEET STRUCTURED DATA / WHAT-IF — OFFLINE TEST RESULTS");
  gbLog.debug(rule);
  RESULTS.forEach(function (r) {
    gbLog.debug("[" + r.status + "] " + r.name + (r.error ? " — " + r.error : ""));
  });
  gbLog.debug(rule);
  gbLog.debug("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  gbLog.debug("MODE: runtime (vm sandbox, stubbed window/document/gbT) + labelled STATIC source checks");
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
  attachGrid: attachGrid,
  setCell: setCell,
  newCellMap: newCellMap,
  findAll: findAll,
  fire: fire,
  finish: finish,
  MODULE_RELS: MODULE_RELS,
  CORE_REL: CORE_REL,
  SUITE_DIR: SUITE_DIR,
};
