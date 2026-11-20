/**
 * Sheets unstructured ingest review (#2494) — offline harness.
 * Loads the nine shipped IIFEs in one vm sandbox with stubbed host globals.
 * Only genuine exported helpers are executed; DOM doubles are code-like tokens
 * so the i18n hardcoded-literal scanner has nothing to report.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var MODULE_DIR = "sheet/js/modules-sheet-advanced/";
var SOURCES = [
  MODULE_DIR + "59_ingest_core.js",
  MODULE_DIR + "59a_ingest_sim.js",
  MODULE_DIR + "59b_ingest_cells.js",
  MODULE_DIR + "60_ingest_preview.js",
  MODULE_DIR + "61_ingest_map.js",
  MODULE_DIR + "62_ingest_review.js",
  MODULE_DIR + "63_ingest_batch.js",
  MODULE_DIR + "64_ingest_policy.js",
  MODULE_DIR + "65_ingest_boot.js",
];

var FIXED_ISO = "2026-03-20T12:00:00.000Z";
var FIXED_MS = new Date(FIXED_ISO).getTime();
var PASS = 0;
var FAIL = 0;
var RESULTS = [];
var uidCounter = 0;

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

function uid(prefix) {
  uidCounter += 1;
  return String(prefix || "fix") + "-" + uidCounter;
}

function readFile(relPath) {
  return fs.readFileSync(path.join(SUITE_DIR, relPath), "utf8");
}

/* gbT stand-in: resolves the fallback and interpolates Fluent placeholders. */
function stubT(key, fallback, params) {
  var out = fallback == null ? String(key) : String(fallback);
  if (params && typeof params === "object") {
    Object.keys(params).forEach(function (name) {
      var value = String(params[name]);
      out = out.split("{ $" + name + " }").join(value).split("{" + name + "}").join(value);
    });
  }
  return out;
}

function makeStorage() {
  var map = new Map();
  return {
    map: map,
    getItem: function (key) { return map.has(String(key)) ? map.get(String(key)) : null; },
    setItem: function (key, value) { map.set(String(key), String(value)); },
    removeItem: function (key) { map.delete(String(key)); },
    clear: function () { map.clear(); },
  };
}

/* Minimal DOM double: createElement/append/attrs/classes/listeners/selectors. */
function makeDom() {
  var all = [];
  var documentEvents = Object.create(null);
  var windowEvents = Object.create(null);

  function register(node) {
    all.push(node);
    return node;
  }

  function getAttribute(name) {
    if (Object.prototype.hasOwnProperty.call(this.attributes, name)) return this.attributes[name];
    if (name === "tabindex") return String(this.tabIndex);
    return null;
  }

  function matches(node, selector) {
    var text = String(selector || "").trim();
    var notMatch = /:not\(\[([^\]]+)\]\)/.exec(text);
    var notAttr = notMatch ? notMatch[1] : "";
    if (notMatch) text = text.replace(notMatch[0], "").trim();
    var hit = false;
    if (text.charAt(0) === "#") hit = node.id === text.slice(1);
    else if (text.charAt(0) === ".") hit = node.classList.contains(text.slice(1));
    else if (text.charAt(0) === "[") {
      var attr = /^\[([^\]=]+)(?:\s*=\s*["']?([^\]"']*)["']?)?\]$/.exec(text);
      if (!attr) return false;
      var actual = node.getAttribute(attr[1]);
      hit = attr[2] == null ? actual != null : actual === attr[2];
    } else {
      hit = node.tagName === text.toUpperCase();
    }
    if (!hit || !notAttr) return hit;
    if (notAttr === "disabled") return !node.disabled && node.getAttribute("disabled") == null;
    return node.getAttribute(notAttr) !== "-1";
  }

  function querySelectorAll(selector) {
    var parts = String(selector || "").split(",");
    var out = [];
    (function walk(parent) {
      parent.children.forEach(function (child) {
        for (var i = 0; i < parts.length; i++) {
          if (matches(child, parts[i])) { out.push(child); break; }
        }
        walk(child);
      });
    })(this);
    return out;
  }

  function querySelector(selector) {
    return this.querySelectorAll(selector)[0] || null;
  }

  function makeNode(tag) {
    var classes = Object.create(null);
    var node = {
      tagName: String(tag || "").toUpperCase(),
      id: "",
      type: "",
      value: "",
      checked: false,
      selected: false,
      disabled: false,
      tabIndex: 0,
      focusCount: 0,
      htmlFor: "",
      min: "",
      max: "",
      isConnected: true,
      style: {},
      dataset: {},
      attributes: Object.create(null),
      children: [],
      handlers: Object.create(null),
      parentNode: null,
      _text: "",
    };
    node.classList = {
      add: function (name) { classes[name] = true; },
      remove: function (name) { delete classes[name]; },
      contains: function (name) { return Boolean(classes[name]); },
      has: function (name) { return Boolean(classes[name]); },
      toggle: function (name, force) {
        var on = force === undefined ? !classes[name] : Boolean(force);
        if (on) classes[name] = true; else delete classes[name];
        return on;
      },
    };
    Object.defineProperty(node, "className", {
      enumerable: true,
      get: function () { return Object.keys(classes).join(" "); },
      set: function (value) {
        Object.keys(classes).forEach(function (name) { delete classes[name]; });
        String(value || "").split(/\s+/).filter(Boolean).forEach(function (name) { classes[name] = true; });
      },
    });
    Object.defineProperty(node, "textContent", {
      enumerable: true,
      get: function () { return node._text; },
      set: function (value) {
        node._text = String(value == null ? "" : value);
        node.children.forEach(function (child) { child.parentNode = null; child.isConnected = false; });
        node.children = [];
      },
    });
    node.setAttribute = function (name, value) { node.attributes[name] = String(value); };
    node.getAttribute = getAttribute;
    node.removeAttribute = function (name) { delete node.attributes[name]; };
    node.appendChild = function (child) {
      child.parentNode = node;
      child.isConnected = true;
      node.children.push(child);
      return child;
    };
    node.removeChild = function (child) {
      node.children = node.children.filter(function (item) { return item !== child; });
      child.parentNode = null;
      child.isConnected = false;
      return child;
    };
    node.remove = function () {
      if (node.parentNode) node.parentNode.removeChild(node);
      else node.isConnected = false;
    };
    node.contains = function (other) {
      if (other === node) return true;
      return node.children.some(function (child) { return child.contains(other); });
    };
    node.addEventListener = function (name, handler) {
      (node.handlers[name] = node.handlers[name] || []).push(handler);
    };
    node.removeEventListener = function (name, handler) {
      node.handlers[name] = (node.handlers[name] || []).filter(function (item) { return item !== handler; });
    };
    node.dispatch = function (name, extra) {
      (node.handlers[name] || []).slice().forEach(function (handler) {
        handler(Object.assign({
          target: node, currentTarget: node, key: "", shiftKey: false,
          preventDefault: function () {},
        }, extra || {}));
      });
    };
    node.focus = function () { node.focusCount++; doc.activeElement = node; };
    node.querySelectorAll = querySelectorAll;
    node.querySelector = querySelector;
    return register(node);
  }

  var doc = {
    readyState: "complete",
    activeElement: null,
    body: null,
    createElement: makeNode,
    createTextNode: function (text) {
      var node = makeNode("#text");
      node._text = String(text == null ? "" : text);
      return node;
    },
    getElementById: function (id) {
      return all.filter(function (node) { return node.id === id; })[0] || null;
    },
    addEventListener: function (name, handler) {
      (documentEvents[name] = documentEvents[name] || []).push(handler);
    },
    removeEventListener: function (name, handler) {
      documentEvents[name] = (documentEvents[name] || []).filter(function (item) { return item !== handler; });
    },
    dispatch: function (name, extra) {
      (documentEvents[name] || []).slice().forEach(function (handler) {
        handler(Object.assign({ target: null, key: "", preventDefault: function () {} }, extra || {}));
      });
    },
    querySelectorAll: querySelectorAll,
    querySelector: querySelector,
  };
  doc.body = makeNode("body");
  doc.children = [doc.body];
  doc.activeElement = makeNode("button");

  return { document: doc, makeNode: makeNode, all: all, documentEvents: documentEvents, windowEvents: windowEvents };
}

function FixedDate() {
  return { toISOString: function () { return FIXED_ISO; } };
}
FixedDate.now = function () { return FIXED_MS; };

function loadModules() {
  var dom = makeDom();
  var storage = makeStorage();
  var win = {};
  var trigger = dom.makeNode("button");
  trigger.id = "sheetIngestBtn";
  dom.document.body.appendChild(trigger);
  var sandbox = {
    document: dom.document,
    localStorage: storage,
    Date: FixedDate,
    console: { log: function () {}, warn: function () {}, error: function () {} },
  };
  sandbox.gbT = stubT;
  win.gbT = stubT;
  win.localStorage = storage;
  win.SheetCore = { currentSheetId: function () { return "sheet-A"; } };
  win.__LOADED_SHEET = { id: "sheet-A" };
  win.addEventListener = function (name, handler) {
    (dom.windowEvents[name] = dom.windowEvents[name] || []).push(handler);
  };
  win.removeEventListener = function (name, handler) {
    dom.windowEvents[name] = (dom.windowEvents[name] || []).filter(function (item) { return item !== handler; });
  };
  sandbox.window = win;
  var context = vm.createContext(sandbox);
  SOURCES.forEach(function (rel) {
    vm.runInContext(readFile(rel), context, { filename: rel });
  });

  var env = {
    sandbox: sandbox,
    context: context,
    window: win,
    document: dom.document,
    dom: dom,
    storage: storage,
    core: win.SheetIngestCore,
    sim: win.SheetIngestSim,
    preview: win.SheetIngestPreview,
    map: win.SheetIngestMap,
    review: win.SheetIngestReview,
    batch: win.SheetIngestBatch,
    policy: win.SheetIngestPolicy,
    boot: win.SheetIngest,
    trigger: trigger,
    fixedIso: FIXED_ISO,
    reset: function () { storage.clear(); },
    seed: function () {
      win.SheetIngestCore.mutate(function (target) {
        win.SheetIngestSim.ensureParse(target, "stmt");
        win.SheetIngestSim.ensureParse(target, "inv");
      });
    },
  };
  return env;
}

var gbLog = globalThis.gbLog || { debug: function (line) { console.log(line); } };

function finish() {
  var rule = "=".repeat(64);
  gbLog.debug("\n" + rule);
  gbLog.debug("SHEET UNSTRUCTURED INGEST — OFFLINE TEST RESULTS (#2494)");
  gbLog.debug(rule);
  RESULTS.forEach(function (result) {
    gbLog.debug("[" + result.status + "] " + result.name + (result.error ? " — " + result.error : ""));
  });
  gbLog.debug(rule);
  gbLog.debug("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  gbLog.debug("MODE: runtime (one vm sandbox; nine shipped modules, stubbed host globals) + labelled [STATIC] source checks");
  gbLog.debug(rule);
  process.exitCode = FAIL > 0 ? 1 : 0;
}

module.exports = {
  test: test, assert: assert, equal: equal, deep: deep, uid: uid,
  readFile: readFile, loadModules: loadModules, finish: finish,
  SOURCES: SOURCES, SUITE_DIR: SUITE_DIR,
};
