/**
 * Sheets print, reporting and scheduled delivery (#2438) — offline harness.
 * Run with: node botui/ui/suite/tests/webtest-sheet-reporting.js
 *
 * The six shipped reporting IIFEs (53..58) are read verbatim and executed in a
 * single shared `vm` context with stubbed window/document/localStorage/clock/
 * uid/gbT. No production logic is reimplemented: the namespaces under test are
 * the real ones the browser builds. DOM sentinels are code-like tokens so the
 * i18n hardcoded-literal scanner has nothing to report.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var MODULE_DIR = "sheet/js/modules-sheet-advanced/";
var SOURCES = {
  core: MODULE_DIR + "53_reporting_core.js",
  print: MODULE_DIR + "54_reporting_print.js",
  preview: MODULE_DIR + "55_reporting_preview.js",
  dashboard: MODULE_DIR + "56_reporting_dashboard.js",
  schedule: MODULE_DIR + "57_reporting_schedule.js",
  boot: MODULE_DIR + "58_reporting_boot.js",
};

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
  assert(actual === expected, (message || "values differ") + " — expected " + expected + ", got " + actual);
}

function deep(actual, expected, message) {
  var left = JSON.stringify(actual);
  var right = JSON.stringify(expected);
  assert(left === right, (message || "structures differ") + " — expected " + right + ", got " + left);
}

function readFile(relPath) {
  return fs.readFileSync(path.join(SUITE_DIR, relPath), "utf8");
}

function makeStorage() {
  var map = new Map();
  return {
    map: map,
    getItem: function (key) { return map.has(String(key)) ? map.get(String(key)) : null; },
    setItem: function (key, value) { map.set(String(key), String(value)); },
    removeItem: function (key) { map.delete(String(key)); },
    clear: function () { map.clear(); },
    key: function (index) { return Array.from(map.keys())[index] || null; },
    length: function () { return map.size; },
  };
}

/* Minimal DOM double: enough for createElement, appendChild, attribute reads,
 * focus, delegated listeners and the diagnostic selectors the modules use. */
function makeDom() {
  var allNodes = [];
  var documentEvents = {};
  var windowEvents = {};

  function register(node) {
    allNodes.push(node);
    return node;
  }

  function liveNodes() {
    return allNodes.filter(function (node) { return node.isConnected !== false; });
  }

  function querySelectorAll(selector) {
    if (selector === "[data-i18n]") {
      return liveNodes().filter(function (node) { return node.attributes["data-i18n"] != null; });
    }
    if (selector === "[data-i18n-placeholder]") {
      return liveNodes().filter(function (node) { return node.attributes["data-i18n-placeholder"] != null; });
    }
    if (selector.indexOf("button:not") >= 0) {
      return liveNodes().filter(function (node) {
        return /^(BUTTON|INPUT|SELECT)$/.test(node.tagName) && !node.disabled;
      });
    }
    return [];
  }

  function querySelector(selector) {
    if (selector.charAt(0) === "#") {
      var id = selector.slice(1);
      return liveNodes().filter(function (node) { return node.id === id; })[0] || null;
    }
    var matches = querySelectorAll(selector);
    return matches.length ? matches[0] : null;
  }

  function makeNode(tag) {
    var node = {
      tagName: String(tag || "").toUpperCase(),
      className: "",
      textContent: "",
      id: "",
      type: "",
      value: "",
      checked: false,
      placeholder: "",
      tabIndex: 0,
      disabled: false,
      focusCount: 0,
      style: {},
      dataset: {},
      attributes: {},
      children: [],
      handlers: {},
      isConnected: true,
      classList: {
        toggle: function () {},
        add: function () {},
        remove: function () {},
      },
      setAttribute: function (name, value) { node.attributes[name] = String(value); },
      getAttribute: function (name) {
        return Object.prototype.hasOwnProperty.call(node.attributes, name) ? node.attributes[name] : null;
      },
      appendChild: function (child) { node.children.push(child); return child; },
      removeChild: function (child) {
        node.children = node.children.filter(function (item) { return item !== child; });
        return child;
      },
      remove: function () { node.isConnected = false; },
      addEventListener: function (name, handler) {
        (node.handlers[name] = node.handlers[name] || []).push(handler);
      },
      removeEventListener: function (name, handler) {
        node.handlers[name] = (node.handlers[name] || []).filter(function (item) { return item !== handler; });
      },
      dispatch: function (name, event) {
        (node.handlers[name] || []).forEach(function (handler) {
          handler(Object.assign({ target: node, currentTarget: node, preventDefault: function () {} }, event));
        });
      },
      contains: function () { return true; },
      focus: function () { node.focusCount++; doc.activeElement = node; },
      querySelector: querySelector,
      querySelectorAll: querySelectorAll,
    };
    return register(node);
  }

  var doc = {
    readyState: "complete",
    title: "<doc>",
    activeElement: null,
    body: null,
    createElement: makeNode,
    createElementNS: function (namespace, tag) { return makeNode(tag); },
    getElementById: function (id) {
      return allNodes.filter(function (node) { return node.id === id; })[0] || null;
    },
    addEventListener: function (name, handler) {
      (documentEvents[name] = documentEvents[name] || []).push(handler);
    },
    removeEventListener: function (name, handler) {
      documentEvents[name] = (documentEvents[name] || []).filter(function (item) { return item !== handler; });
    },
    dispatch: function (name, event) {
      (documentEvents[name] || []).forEach(function (handler) {
        handler(Object.assign({ target: null, preventDefault: function () {} }, event));
      });
    },
    querySelector: querySelector,
    querySelectorAll: querySelectorAll,
  };
  doc.body = makeNode("body");
  doc.activeElement = makeNode("button");

  return {
    document: doc,
    makeNode: makeNode,
    allNodes: allNodes,
    documentEvents: documentEvents,
    windowEvents: windowEvents,
  };
}

/* gbT stand-in: resolves the fallback and interpolates {name} placeholders so
 * fieldText/lineFor results are meaningful without shipping locale catalogs. */
function gbT(key, fallback, params) {
  var out = fallback == null ? String(key) : String(fallback);
  if (params && typeof params === "object") {
    Object.keys(params).forEach(function (name) {
      out = out.split("{" + name + "}").join(String(params[name]));
    });
  }
  return out;
}

function loadModules() {
  var dom = makeDom();
  var storage = makeStorage();
  var win = {};
  var sandbox = {
    window: win,
    document: dom.document,
    localStorage: storage,
    console: { log: function () {}, warn: function () {}, error: function () {} },
  };
  sandbox.gbT = gbT;
  win.gbT = gbT;
  win.localStorage = storage;
  win.SheetCore = {
    currentSheetId: function () { return "sheet-A"; },
    currentSheetName: function () { return "Budget"; },
  };
  win.__LOADED_SHEET = { id: "sheet-A", name: "Budget" };
  win.addEventListener = function (name, handler) {
    (dom.windowEvents[name] = dom.windowEvents[name] || []).push(handler);
  };
  win.removeEventListener = function (name, handler) {
    dom.windowEvents[name] = (dom.windowEvents[name] || []).filter(function (item) { return item !== handler; });
  };

  var context = vm.createContext(sandbox);
  Object.keys(SOURCES).forEach(function (key) {
    vm.runInContext(readFile(SOURCES[key]), context, { filename: SOURCES[key] });
  });
  vm.runInContext([
    "window.__gbOrigRandom = Math.random;",
    "window.__gbOrigNow = Date.now;",
    "window.__setFixedIds = function (on) {",
    "  if (on) {",
    "    Math.random = function () { return 0.5; };",
    "    Date.now = function () { return 1767225600000; };",
    "  } else {",
    "    Math.random = window.__gbOrigRandom;",
    "    Date.now = window.__gbOrigNow;",
    "  }",
    "};",
    "window.__makeDate = function (y, m, d, h, min) {",
    "  return new Date(y, m, d, h == null ? 0 : h, min == null ? 0 : min);",
    "};",
  ].join("\n"), context);

  return {
    sandbox: sandbox,
    context: context,
    window: win,
    document: dom.document,
    dom: dom,
    storage: storage,
    core: win.SheetReportingCore,
    print: win.SheetReportingPrint,
    preview: win.SheetReportingPreview,
    dashboard: win.SheetReportingDashboard,
    schedule: win.SheetReportingSchedule,
    boot: win.SheetReporting,
    resetState: function () { storage.clear(); },
    makeDate: win.__makeDate,
    withFixedIds: function (fn) {
      win.__setFixedIds(true);
      try {
        return fn();
      } finally {
        win.__setFixedIds(false);
      }
    },
  };
}

var gbLog = globalThis.gbLog || { debug: function (line) { console.log(line); } };

function finish() {
  var rule = "=".repeat(64);
  gbLog.debug("\n" + rule);
  gbLog.debug("SHEET REPORTING — OFFLINE TEST RESULTS (#2438)");
  gbLog.debug(rule);
  RESULTS.forEach(function (result) {
    gbLog.debug("[" + result.status + "] " + result.name + (result.error ? " — " + result.error : ""));
  });
  gbLog.debug(rule);
  gbLog.debug("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  gbLog.debug("MODE: runtime (one vm sandbox; shipped modules, stubbed host globals) + labelled static source checks");
  gbLog.debug(rule);
  process.exitCode = FAIL > 0 ? 1 : 0;
}

module.exports = {
  test: test,
  assert: assert,
  equal: equal,
  deep: deep,
  readFile: readFile,
  loadModules: loadModules,
  finish: finish,
  SOURCES: SOURCES,
  SUITE_DIR: SUITE_DIR,
};
