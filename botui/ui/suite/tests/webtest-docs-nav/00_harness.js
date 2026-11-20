/**
 * Offline harness for the #2485 docs NAV slice (nav pane, bookmarks, quick
 * parts, autotext). Run with: node botui/ui/suite/tests/webtest-docs-nav.js
 *
 * Loads the shipped IIFEs 19_nav_core, 20_nav_pane, 22_bookmarks,
 * 23_quickparts and 24_nav_init into ONE shared vm sandbox. The DOM stub in
 * ./01_dom implements exactly the surface those modules touch. Clock, uid,
 * storage, caret and translator are fixed for deterministic assertions.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");
var dom = require("./01_dom");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
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

function assertEqual(actual, expected, msg) {
  if (actual !== expected) {
    throw new Error((msg || "values differ") + " :: <" + actual + "> != <" + expected + ">");
  }
}

function readFile(relPath) {
  return fs.readFileSync(path.join(SUITE_DIR, relPath), "utf8");
}

function deepEqual(a, b) {
  return JSON.stringify(a) === JSON.stringify(b);
}

// Synchronous thenable: dialog promise chains resolve inside a test instead
// of leaking unresolved microtasks into the report.
function SyncPromise(executor) {
  var self = this;
  this._settled = false;
  this._value = undefined;
  this._callbacks = [];
  executor(function (value) {
    self._settled = true;
    self._value = value;
    self._callbacks.slice().forEach(function (cb) { cb(value); });
  });
}
SyncPromise.prototype.then = function (callback) {
  var self = this;
  return new SyncPromise(function (resolve) {
    var handler = function (value) { resolve(typeof callback === "function" ? callback(value) : value); };
    if (self._settled) handler(self._value);
    else self._callbacks.push(handler);
  });
};
SyncPromise.resolve = function (value) {
  return new SyncPromise(function (resolve) { resolve(value); });
};

var MODULES = [
  "docs/modules/19_nav_core.js",
  "docs/modules/20_nav_pane.js",
  "docs/modules/22_bookmarks.js",
  "docs/modules/23_quickparts.js",
  "docs/modules/24_nav_init.js",
];

function pad8(n) {
  var text = String(n);
  while (text.length < 8) text = "0" + text;
  return text;
}

function FixedDate() {}
FixedDate.now = function () { return 1700000000000; };
FixedDate.prototype.toISOString = function () { return "2023-11-14T22:13:20.000Z"; };

function fixture(options) {
  var opts = options || {};
  var store = {};
  var doc = dom.makeDocument(opts.readyState);
  var uuidCounter = 0;
  var trCalls = [];
  var selection = {
    rangeCount: 0,
    _ranges: [],
    getRangeAt: function (index) { return this._ranges[index]; },
    removeAllRanges: function () { this._ranges = []; this.rangeCount = 0; },
    addRange: function (range) { this._ranges.push(range); this.rangeCount = this._ranges.length; },
  };
  var sandbox = {
    window: {
      gbT: function (key, fallback) { return fallback; },
      localStorage: {
        getItem: function (key) { return Object.prototype.hasOwnProperty.call(store, key) ? store[key] : null; },
        setItem: function (key, value) { store[key] = String(value); },
        removeItem: function (key) { delete store[key]; },
        clear: function () { Object.keys(store).forEach(function (key) { delete store[key]; }); },
      },
      crypto: {
        randomUUID: function () {
          uuidCounter += 1;
          return pad8(uuidCounter) + "-0000-4000-8000-000000000000";
        },
      },
      getSelection: function () { return selection.rangeCount ? selection : null; },
      addEventListener: function (type, fn) { doc.addEventListener(type, fn); },
      removeEventListener: function (type, fn) { doc.removeEventListener(type, fn); },
      setTimeout: function () { return 0; },
      clearTimeout: function () {},
    },
    document: doc,
    crypto: {
      randomUUID: function () {
        uuidCounter += 1;
        return pad8(uuidCounter) + "-0000-4000-8000-000000000000";
      },
    },
    docsT: function (key, fallback, params) {
      trCalls.push({ key: key, fallback: fallback, params: params });
      return fallback;
    },
    getActiveArticle: function () { return sandbox.__article || null; },
    getCaretCharacterOffsetWithin: function () { return 7; },
    Date: FixedDate,
    Math: { random: function () { return 0.5; }, floor: Math.floor, ceil: Math.ceil, max: Math.max, min: Math.min },
    Promise: SyncPromise,
    console: console,
    setTimeout: function () { return 0; },
    clearTimeout: function () {},
    MutationObserver: function () { this.observe = function () {}; },
    fetch: function () { return SyncPromise.resolve({ ok: true, status: 204 }); },
    Event: function () {},
  };
  if (opts.noActiveArticle) delete sandbox.getActiveArticle;
  var context = vm.createContext(sandbox);
  MODULES.forEach(function (relPath) {
    vm.runInContext(readFile(relPath), context, { filename: relPath });
  });
  return {
    NS: context.window.DocsNav,
    sandbox: sandbox,
    doc: doc,
    store: store,
    trCalls: trCalls,
    selection: selection,
    setArticle: function (node) { sandbox.__article = node; },
    setCaret: function (node, offset) {
      selection._ranges = [{ startContainer: node, startOffset: offset }];
      selection.rangeCount = 1;
    },
    addHost: function (id) {
      var node = doc.createElement("div");
      node.setAttribute("id", id);
      doc.body.appendChild(node);
      return node;
    },
  };
}

function article(fx, docId) {
  var node = fx.doc.createElement("article");
  node.setAttribute("contenteditable", "true");
  node.dataset.docId = docId || "<a>";
  fx.doc.body.appendChild(node);
  fx.setArticle(node);
  return node;
}

function byClass(node, cls) {
  return node.querySelectorAll("*").filter(function (candidate) {
    return candidate._classes.indexOf(cls) >= 0;
  });
}

function report() {
  console.log("\n" + "=".repeat(64));
  console.log("DOCS NAV / QUICK PARTS OFFLINE RESULTS (#2485)");
  console.log("=".repeat(64));
  RESULTS.forEach(function (result) {
    console.log("[" + result.status + "] " + result.name + (result.error ? " -- " + result.error : ""));
  });
  console.log("=".repeat(64));
  console.log("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  console.log("=".repeat(64));
  process.exitCode = FAIL > 0 ? 1 : 0;
}

module.exports = {
  test: test,
  assert: assert,
  assertEqual: assertEqual,
  readFile: readFile,
  deepEqual: deepEqual,
  fixture: fixture,
  article: article,
  byClass: byClass,
  FakeNode: dom.FakeNode,
  SyncPromise: SyncPromise,
  report: report,
  finish: report,
};
