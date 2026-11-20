/**
 * Offline harness for #2425 docs enterprise review workflow verification.
 * Run with: node botui/ui/suite/tests/webtest-docs-review.js
 *
 * The suite lives in ordered modules under ./webtest-docs-review/:
 *   00_harness.js   counters, assertions, FakeNode/DOM/vm loader, report
 *   10_review.js    runtime tests of the real shipped review IIFEs (49..57)
 *   11_review_deferred.js  more runtime + labelled STATIC source assertions
 *
 * Every production module is loaded as its real, unmodified IIFE into ONE
 * shared Node vm sandbox with stubbed window/document/gbT/storage/clock/uid.
 * No production logic is reimplemented here: the fixtures only feed the real
 * helpers and observe what they return. DOM sentinels are code-like tokens
 * (angle brackets), never user-facing English literals (i18n gate #2096).
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

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

/** Load several ordered scripts into ONE shared vm context. */
function loadModules(relPaths, sandbox) {
  var context = vm.createContext(sandbox);
  relPaths.forEach(function (relPath) {
    vm.runInContext(readFile(relPath), context, { filename: relPath });
  });
  return context;
}

// ── Minimal but faithful DOM stub ─────────────────────────────────────
// Supports exactly the surface the shipped review modules touch: creation,
// attributes, dataset, classList, textContent, append/remove, contains,
// querySelector(All), closest, focus and listener dispatch.

function walk(root, fn) {
  fn(root);
  root.children.forEach(function (child) { walk(child, fn); });
}

function hasAttr(node, name) {
  return Object.prototype.hasOwnProperty.call(node._attrs, name);
}

function matches(node, selector) {
  selector = String(selector || "").trim();
  if (selector === "*") return true;
  var m = selector.match(/^([a-zA-Z]+)\.([\w-]+)$/);
  if (m) return node.tagName === m[1].toUpperCase() && node._classes.indexOf(m[2]) >= 0;
  m = selector.match(/^\.([\w-]+)$/);
  if (m) return node._classes.indexOf(m[1]) >= 0;
  m = selector.match(/^#([\w-]+)$/);
  if (m) return node.getAttribute("id") === m[1];
  m = selector.match(/^([a-zA-Z]+)\[([^=\]]+)="([^"]+)"\]$/);
  if (m) return node.tagName === m[1].toUpperCase() && node.getAttribute(m[2]) === m[3];
  m = selector.match(/^\[([^=\]]+)="([^"]+)"\]$/);
  if (m) return node.getAttribute(m[1]) === m[2];
  m = selector.match(/^([a-zA-Z]+)\[([^\]]+)\]$/);
  if (m) return node.tagName === m[1].toUpperCase() && hasAttr(node, m[2]);
  m = selector.match(/^\[([^\]=]+)\]$/);
  if (m) return hasAttr(node, m[1]);
  if (/^[a-zA-Z]+$/.test(selector)) return node.tagName === selector.toUpperCase();
  return false;
}

function FakeNode(tag, ownerDoc) {
  this.tagName = String(tag || "").toUpperCase();
  this.children = [];
  this.parentNode = null;
  this._attrs = {};
  this._classes = [];
  var self = this;
  // Mirror dataset writes into data-* attributes, as a real DOM element does,
  // so shipped code using querySelector('[data-review-anchor="..."]') works.
  this.dataset = new Proxy({}, {
    set: function (target, prop, value) {
      target[prop] = value;
      self._attrs["data-" + String(prop).replace(/[A-Z]/g, function (m) {
        return "-" + m.toLowerCase();
      })] = String(value);
      return true;
    },
  });
  this.style = {};
  this.listeners = {};
  this.value = "";
  this.hidden = false;
  this.disabled = false;
  this._text = "";
  this._ownerDoc = ownerDoc || null;
  this.classList = {
    add: function (c) { if (self._classes.indexOf(c) < 0) self._classes.push(c); },
    remove: function (c) {
      var i = self._classes.indexOf(c);
      if (i >= 0) self._classes.splice(i, 1);
    },
    contains: function (c) { return self._classes.indexOf(c) >= 0; },
  };
}

Object.defineProperty(FakeNode.prototype, "className", {
  get: function () { return this._classes.join(" "); },
  set: function (value) {
    this._classes = String(value || "").split(/\s+/).filter(function (c) { return !!c; });
  },
});

Object.defineProperty(FakeNode.prototype, "textContent", {
  get: function () {
    if (this.tagName === "#TEXT" || !this.children.length) return this._text;
    return this.children.map(function (c) { return c.textContent; }).join("");
  },
  set: function (value) {
    this.children = [];
    this._text = value == null ? "" : String(value);
  },
});

FakeNode.prototype.appendChild = function (child) {
  child.parentNode = this;
  this.children.push(child);
  return child;
};
FakeNode.prototype.removeChild = function (child) {
  var i = this.children.indexOf(child);
  if (i >= 0) this.children.splice(i, 1);
  child.parentNode = null;
  return child;
};
FakeNode.prototype.setAttribute = function (name, value) { this._attrs[name] = String(value); };
FakeNode.prototype.getAttribute = function (name) {
  return hasAttr(this, name) ? this._attrs[name] : null;
};
FakeNode.prototype.hasAttribute = function (name) { return hasAttr(this, name); };
FakeNode.prototype.addEventListener = function (type, fn) {
  (this.listeners[type] || (this.listeners[type] = [])).push(fn);
};
FakeNode.prototype.removeEventListener = function (type, fn) {
  var list = this.listeners[type] || [];
  var i = list.indexOf(fn);
  if (i >= 0) list.splice(i, 1);
};
FakeNode.prototype.dispatch = function (type, event) {
  (this.listeners[type] || []).slice().forEach(function (fn) { fn(event); });
};
FakeNode.prototype.contains = function (node) {
  var found = false;
  walk(this, function (candidate) { if (candidate === node) found = true; });
  return found;
};
FakeNode.prototype.querySelector = function (selector) {
  var found = null;
  var self = this;
  walk(this, function (candidate) {
    if (found || candidate === self) return;
    if (matches(candidate, selector)) found = candidate;
  });
  return found;
};
FakeNode.prototype.querySelectorAll = function (selector) {
  var self = this;
  var parts = String(selector).split(",").map(function (s) { return s.trim(); });
  var out = [];
  walk(this, function (candidate) {
    if (candidate === self) return;
    parts.forEach(function (raw) {
      var part = raw;
      var negated = part.indexOf(":not(") >= 0;
      if (negated) part = part.slice(0, part.indexOf(":not("));
      if (!matches(candidate, part)) return;
      if (negated && candidate.getAttribute("tabindex") === "-1") return;
      if (out.indexOf(candidate) < 0) out.push(candidate);
    });
  });
  return out;
};
FakeNode.prototype.closest = function (selector) {
  var node = this;
  while (node) {
    if (matches(node, selector)) return node;
    node = node.parentNode;
  }
  return null;
};
FakeNode.prototype.focus = function () {
  if (this._ownerDoc) this._ownerDoc.activeElement = this;
  return this;
};

function makeDocument() {
  var doc = {
    readyState: "loading",
    body: new FakeNode("body", null),
    activeElement: null,
    _nodes: {},
    _listeners: {},
    createElement: function (tag) { return new FakeNode(tag, doc); },
    createTextNode: function (text) {
      var node = new FakeNode("#text", doc);
      node._text = String(text);
      return node;
    },
    getElementById: function (id) {
      return Object.prototype.hasOwnProperty.call(doc._nodes, id) ? doc._nodes[id] : null;
    },
    querySelector: function (selector) { return doc.body.querySelector(selector); },
    addEventListener: function (type, fn) {
      (doc._listeners[type] || (doc._listeners[type] = [])).push(fn);
    },
    removeEventListener: function (type, fn) {
      var list = doc._listeners[type] || [];
      var i = list.indexOf(fn);
      if (i >= 0) list.splice(i, 1);
    },
    dispatch: function (type, event) {
      (doc._listeners[type] || []).slice().forEach(function (fn) { fn(event); });
    },
  };
  doc.body._ownerDoc = doc;
  return doc;
}

function pad8(n) {
  var text = String(n);
  while (text.length < 8) text = "0" + text;
  return text;
}

function FixedDate() {}
FixedDate.now = function () { return 1700000000000; };
FixedDate.prototype.toISOString = function () { return "2023-11-14T22:13:20.000Z"; };

var MODULES = [
  "docs/modules/08_comments.js",
  "docs/modules/11_history_server.js",
  "docs/modules/49_review_state.js",
  "docs/modules/50_review_model.js",
  "docs/modules/51_review_marks.js",
  "docs/modules/52_review_comments.js",
  "docs/modules/53_review_compare.js",
  "docs/modules/54_review_dashboard.js",
  "docs/modules/55_review_permissions.js",
  "docs/modules/56_review_dialogs.js",
  "docs/modules/57_review_init.js",
];

/** Fresh sandbox with deterministic storage, clock and uid generator. */
function fixture() {
  var store = {};
  var doc = makeDocument();
  var uuidCounter = 0;
  var sandbox = {
    window: {
      gbT: function (key, fallback) { return fallback; },
      localStorage: {
        getItem: function (key) {
          return Object.prototype.hasOwnProperty.call(store, key) ? store[key] : null;
        },
        setItem: function (key, value) { store[key] = String(value); },
        removeItem: function (key) { delete store[key]; },
        clear: function () { Object.keys(store).forEach(function (k) { delete store[k]; }); },
      },
      crypto: {
        randomUUID: function () {
          uuidCounter += 1;
          return pad8(uuidCounter) + "-0000-4000-8000-000000000000";
        },
      },
      getSelection: function () { return null; },
    },
    document: doc,
    Date: FixedDate,
    console: console,
    setTimeout: function () { return 0; },
    clearTimeout: function () {},
    MutationObserver: function () { this.observe = function () {}; },
    fetch: function () { return Promise.resolve({ ok: true, status: 204 }); },
    Event: function () {},
  };
  var ctx = loadModules(MODULES, sandbox);
  return {
    NS: ctx.window.DocsReview,
    sandbox: sandbox,
    store: store,
    doc: doc,
    ctx: ctx,
  };
}

/** Register an empty contenteditable article with a stable doc id. */
function article(fx, docId) {
  var node = fx.doc.createElement("article");
  node.setAttribute("contenteditable", "true");
  node.dataset.docId = docId || "<a>";
  fx.doc.body.appendChild(node);
  return node;
}

/** Collect descendants whose class list contains <cls>. */
function byClass(node, cls) {
  return node.querySelectorAll("*").filter(function (n) {
    return n._classes.indexOf(cls) >= 0;
  });
}

function report() {
  console.log("\n" + "=".repeat(64));
  console.log("DOCS ENTERPRISE REVIEW OFFLINE RESULTS (#2425)");
  console.log("=".repeat(64));
  RESULTS.forEach(function (r) {
    console.log("[" + r.status + "] " + r.name + (r.error ? " -- " + r.error : ""));
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
  loadModules: loadModules,
  fixture: fixture,
  article: article,
  byClass: byClass,
  FakeNode: FakeNode,
  report: report,
};
