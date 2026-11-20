/**
 * Issue #2509 — OCR / scan-to-document pipeline: durable OFFLINE harness.
 *
 * Loads the shipped confidence-review chain (59 core, 59a simulated parse,
 * 59b cells, 62 review) VERBATIM into one Node vm sandbox with stubbed host
 * globals. No server, no browser, no network, no boteval, no production edits.
 *
 * Every result is labelled:
 *   [RUNTIME]           shipped code executed; a real behavioural assertion
 *   [BUG]               defect proved by execution; report only, never patched
 *   [BASELINE-OBSERVE]  current absent/limited state measured from shipped code
 *   [STATIC]            source-text characterization only; no runtime claim
 *
 * A [BASELINE-OBSERVE]/[STATIC] PASS means "the characterization still holds",
 * NOT that an OCR feature is implemented or working.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var ROOT_DIR = path.resolve(SUITE_DIR, "..", "..", "..");
var MODULES = [
  "sheet/js/modules-sheet-advanced/59_ingest_core.js",
  "sheet/js/modules-sheet-advanced/59a_ingest_sim.js",
  "sheet/js/modules-sheet-advanced/59b_ingest_cells.js",
  "sheet/js/modules-sheet-advanced/62_ingest_review.js",
];
var FIXED_ISO = "2026-03-20T12:00:00.000Z";
var FIXED_MS = new Date(FIXED_ISO).getTime();
var LABELS = ["RUNTIME", "BUG", "BASELINE-OBSERVE", "STATIC"];
var PASS = 0;
var FAIL = 0;
var RESULTS = [];

function read(relPath) {
  return fs.readFileSync(path.join(SUITE_DIR, relPath), "utf8");
}

function exists(relPath) {
  return fs.existsSync(path.join(SUITE_DIR, relPath));
}

function readRepo(relPath) {
  return fs.readFileSync(path.join(ROOT_DIR, relPath), "utf8");
}

function assert(condition, message) {
  if (!condition) throw new Error(message || "Assertion failed");
}

function equal(actual, expected, message) {
  assert(actual === expected, (message || "values differ") +
    " — expected " + expected + ", got " + actual);
}

function deep(actual, expected, message) {
  var left = JSON.stringify(actual);
  var right = JSON.stringify(expected);
  assert(left === right, (message || "structures differ") +
    " — expected " + right + ", got " + left);
}

function capture(fn) {
  try {
    return { threw: false, value: fn() };
  } catch (error) {
    return { threw: true, message: error && error.message ? error.message : String(error) };
  }
}

function record(label, name, fn) {
  try {
    fn();
    PASS += 1;
    RESULTS.push({ label: label, name: name, status: "PASS" });
  } catch (error) {
    FAIL += 1;
    RESULTS.push({ label: label, name: name, status: "FAIL",
      error: error && error.message ? error.message : String(error) });
  }
}

function runtime(name, fn) { record("RUNTIME", name, fn); }
function bug(name, fn) { record("BUG", name, fn); }
function observe(name, fn) { record("BASELINE-OBSERVE", name, fn); }
function statik(name, fn) { record("STATIC", name, fn); }

/* gbT stand-in: interpolate Fluent placeholders from the shipped fallback. */
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

/* Lean DOM double: only the slots the two review IIFEs actually touch. */
function makeDom() {
  var all = [];

  function makeNode(tag) {
    var classes = Object.create(null);
    var node = {
      tagName: String(tag || "").toUpperCase(),
      id: "", type: "", value: "", checked: false, selected: false,
      disabled: false, tabIndex: 0, htmlFor: "", min: "", max: "",
      isConnected: true, style: {}, dataset: {},
      attributes: Object.create(null), children: [], handlers: Object.create(null),
      parentNode: null, _text: "",
    };

    Object.defineProperty(node, "className", {
      enumerable: true,
      get: function () { return Object.keys(classes).join(" "); },
      set: function (value) {
        Object.keys(classes).forEach(function (name) { delete classes[name]; });
        String(value == null ? "" : value).split(/\s+/).filter(Boolean).forEach(function (name) {
          classes[name] = true;
        });
      },
    });
    Object.defineProperty(node, "textContent", {
      enumerable: true,
      get: function () { return node._text; },
      set: function (value) {
        node._text = String(value == null ? "" : value);
        node.children.forEach(function (child) {
          child.parentNode = null;
          child.isConnected = false;
        });
        node.children = [];
      },
    });
    node.classList = {
      add: function (name) { classes[name] = true; },
      remove: function (name) { delete classes[name]; },
      contains: function (name) { return Boolean(classes[name]); },
      toggle: function (name, force) {
        var on = force === undefined ? !classes[name] : Boolean(force);
        if (on) classes[name] = true; else delete classes[name];
        return on;
      },
    };
    node.setAttribute = function (name, value) {
      node.attributes[name] = String(value);
    };
    node.getAttribute = function (name) {
      return Object.prototype.hasOwnProperty.call(node.attributes, name)
        ? node.attributes[name] : null;
    };
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
      node.handlers[name] = (node.handlers[name] || []).filter(function (item) {
        return item !== handler;
      });
    };
    node.dispatch = function (name, extra) {
      (node.handlers[name] || []).slice().forEach(function (handler) {
        handler(Object.assign({
          target: node, currentTarget: node, key: "", shiftKey: false,
          preventDefault: function () {},
        }, extra || {}));
      });
    };
    node.querySelectorAll = function (selector) {
      var wanted = String(selector || "").trim();
      var out = [];
      (function walk(parent) {
        parent.children.forEach(function (child) {
          var hit = wanted.charAt(0) === "."
            ? child.classList.contains(wanted.slice(1))
            : wanted.charAt(0) === "#"
              ? child.id === wanted.slice(1)
              : child.tagName === wanted.toUpperCase();
          if (hit) out.push(child);
          walk(child);
        });
      })(node);
      return out;
    };
    node.querySelector = function (selector) {
      return node.querySelectorAll(selector)[0] || null;
    };
    all.push(node);
    return node;
  }

  var document = {
    readyState: "complete",
    activeElement: null,
    createElement: makeNode,
    createTextNode: function (text) {
      var node = makeNode("#text");
      node._text = String(text == null ? "" : text);
      return node;
    },
    getElementById: function (id) {
      return all.filter(function (node) { return node.id === id; })[0] || null;
    },
  };
  document.body = makeNode("body");
  document.activeElement = makeNode("button");
  return { document: document, makeNode: makeNode, all: all };
}

function FixedDate() {
  return { toISOString: function () { return FIXED_ISO; } };
}
FixedDate.now = function () { return FIXED_MS; };

/* One sandbox for the whole run; storage is cleared between cases. */
function loadModules() {
  var dom = makeDom();
  var storage = makeStorage();
  var statuses = [];
  var counters = { rerender: 0, clearSelection: 0, selectCell: 0 };
  var win = {
    gbT: stubT,
    localStorage: storage,
    SheetCore: { currentSheetId: function () { return "ocr-sheet"; } },
    addEventListener: function () {},
    removeEventListener: function () {},
  };
  var sandbox = {
    window: win,
    document: dom.document,
    localStorage: storage,
    Date: FixedDate,
    console: { log: function () {}, warn: function () {}, error: function () {} },
  };
  var context = vm.createContext(sandbox);
  MODULES.forEach(function (rel) {
    vm.runInContext(read(rel), context, { filename: rel });
  });
  var env = {
    sandbox: sandbox, context: context, window: win, document: dom.document,
    dom: dom, storage: storage, statuses: statuses, counters: counters,
    core: win.SheetIngestCore,
    sim: win.SheetIngestSim,
    review: win.SheetIngestReview,
    fixedIso: FIXED_ISO,
    ctx: {
      state: function () { return win.SheetIngestCore.state(); },
      setStatus: function (message, isError) {
        statuses.push({ message: message, isError: Boolean(isError) });
      },
      rerender: function () { counters.rerender += 1; },
      clearSelection: function () { counters.clearSelection += 1; },
      selectCell: function () { counters.selectCell += 1; },
    },
    reset: function () {
      storage.clear();
      statuses.length = 0;
      counters.rerender = 0;
      counters.clearSelection = 0;
      counters.selectCell = 0;
    },
    seed: function () {
      win.SheetIngestCore.mutate(function (target) {
        win.SheetIngestSim.ensureParse(target, "stmt");
        win.SheetIngestSim.ensureParse(target, "inv");
      });
    },
  };
  return env;
}

var log = (globalThis.gbLog && globalThis.gbLog.debug) || console.log;

function finish() {
  var rule = "=".repeat(72);
  log("\n" + rule);
  log("SHEET OCR / SCAN-TO-DOCUMENT PIPELINE — OFFLINE CHARACTERIZATION (#2509)");
  log(rule);
  LABELS.forEach(function (label) {
    var rows = RESULTS.filter(function (result) { return result.label === label; });
    if (!rows.length) return;
    var passed = rows.filter(function (result) { return result.status === "PASS"; }).length;
    log("-- " + label + ": " + passed + "/" + rows.length + " --");
    rows.forEach(function (result) {
      log("[" + result.status + "] [" + label + "] " + result.name +
        (result.error ? " — " + result.error : ""));
    });
  });
  log(rule);
  log("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  log("RUNTIME = shipped code executed; BUG = defect proof (report only); " +
    "BASELINE-OBSERVE/STATIC = current absence, not feature PASS.");
  log(rule);
  process.exitCode = FAIL > 0 ? 1 : 0;
}

module.exports = {
  SUITE_DIR: SUITE_DIR, ROOT_DIR: ROOT_DIR, MODULES: MODULES, FIXED_ISO: FIXED_ISO,
  read: read, readRepo: readRepo, exists: exists, assert: assert, equal: equal, deep: deep,
  capture: capture, runtime: runtime, bug: bug, observe: observe, statik: statik,
  loadModules: loadModules, finish: finish,
};
