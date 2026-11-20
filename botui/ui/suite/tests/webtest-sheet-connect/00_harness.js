/**
 * Sheets data connectivity and refresh — offline test harness (#2492).
 * Run with: node botui/ui/suite/tests/webtest-sheet-connect.js
 *
 * The ten shipped connectivity IIFEs are loaded UNCHANGED into one shared
 * Node `vm` sandbox with stubbed window/document/localStorage/gbT/clock/uid.
 * No production logic is reimplemented here. Genuine exported helpers run as
 * RUNTIME tests; private/DOM-only or backend-deferred behaviour is covered by
 * clearly-labelled STATIC source assertions in the case files.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var MODULE_PREFIX = "sheet/js/modules-sheet-advanced/";
var MODULES = [
  "66_connect_core.js",
  "66a_connect_feed.js",
  "66b_connect_steps.js",
  "66c_connect_ui.js",
  "67_connect_connectors.js",
  "68_connect_query.js",
  "69_connect_params.js",
  "70_connect_refresh.js",
  "71_connect_lineage.js",
  "72_connect_boot.js",
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
    RESULTS.push({ name: name, status: "FAIL", error: e && e.message ? e.message : String(e) });
  }
}

function assert(condition, msg) {
  if (!condition) throw new Error(msg || "Assertion failed");
}

function equal(actual, expected, msg) {
  assert(actual === expected, (msg || "values differ") + " — expected <" + expected + ">, got <" + actual + ">");
}

function deep(actual, expected, msg) {
  var a = JSON.stringify(actual);
  var b = JSON.stringify(expected);
  assert(a === b, (msg || "structures differ") + " — expected " + b + ", got " + a);
}

function readSource(name) {
  return fs.readFileSync(path.join(SUITE_DIR, MODULE_PREFIX + name), "utf8");
}

function makeStorage() {
  var map = {};
  return {
    getItem: function (key) { return Object.prototype.hasOwnProperty.call(map, key) ? map[key] : null; },
    setItem: function (key, value) { map[key] = String(value); },
    removeItem: function (key) { delete map[key]; },
    clear: function () { map = {}; },
    keys: function () { return Object.keys(map); },
  };
}

// A controllable Date: zero-argument construction is pinned to `fixedNow`,
// every other form delegates to the real Date so Date.UTC / parsing still work.
function makeDate(fixedNow) {
  var RealDate = Date;
  function FakeDate() {
    if (!(this instanceof FakeDate)) return new RealDate(fixedNow).toString();
    if (arguments.length === 0) return new RealDate(fixedNow);
    return Reflect.construct(RealDate, Array.prototype.slice.call(arguments));
  }
  FakeDate.prototype = RealDate.prototype;
  FakeDate.now = function () { return fixedNow; };
  FakeDate.UTC = RealDate.UTC;
  FakeDate.parse = RealDate.parse;
  return FakeDate;
}

// A compact DOM double: enough of the element API for the shipped element /
// dialog / table / tab helpers to execute as RUNTIME code, with attribute,
// dataset, listener and descendant-query support.
function makeDom() {
  function Node(tag) {
    this.tagName = String(tag || "").toUpperCase();
    this.children = [];
    this.attributes = {};
    this.dataset = {};
    this.style = {};
    this.listeners = {};
    this._text = "";
    this.parentNode = null;
    this.isConnected = true;
    this.tabIndex = 0;
    this.type = "";
    this.value = "";
    this.checked = false;
    this.disabled = false;
    this.id = "";
    this.className = "";
  }
  Object.defineProperty(Node.prototype, "textContent", {
    configurable: true,
    get: function () {
      if (this.children.length) {
        return this.children.map(function (child) { return child.textContent; }).join("");
      }
      return this._text;
    },
    set: function (value) {
      this._text = value == null ? "" : String(value);
      this.children.forEach(function (child) { child.parentNode = null; });
      this.children = [];
    },
  });
  Node.prototype.appendChild = function (child) {
    if (child.parentNode) child.parentNode.removeChild(child);
    child.parentNode = this;
    this.children.push(child);
    return child;
  };
  Node.prototype.removeChild = function (child) {
    var index = this.children.indexOf(child);
    if (index >= 0) this.children.splice(index, 1);
    child.parentNode = null;
    return child;
  };
  Node.prototype.remove = function () {
    if (this.parentNode) this.parentNode.removeChild(this);
    this.isConnected = false;
  };
  Node.prototype.contains = function (node) {
    var current = node;
    while (current) {
      if (current === this) return true;
      current = current.parentNode;
    }
    return false;
  };
  Node.prototype.setAttribute = function (name, value) {
    this.attributes[name] = String(value);
    if (name === "id") this.id = String(value);
  };
  Node.prototype.getAttribute = function (name) {
    return Object.prototype.hasOwnProperty.call(this.attributes, name) ? this.attributes[name] : null;
  };
  Node.prototype.removeAttribute = function (name) {
    delete this.attributes[name];
    if (name === "id") this.id = "";
  };
  Node.prototype.addEventListener = function (type, fn) {
    (this.listeners[type] = this.listeners[type] || []).push(fn);
  };
  Node.prototype.removeEventListener = function (type, fn) {
    var list = this.listeners[type] || [];
    var index = list.indexOf(fn);
    if (index >= 0) list.splice(index, 1);
  };
  Node.prototype.focus = function () { dom.activeElement = this; };
  Node.prototype.dispatch = function (type, ev) {
    ev = ev || {};
    ev.currentTarget = ev.currentTarget || this;
    ev.target = ev.target || this;
    ev.preventDefault = ev.preventDefault || function () {};
    (this.listeners[type] || []).slice().forEach(function (fn) { fn(ev); });
  };

  function descendants(node, out) {
    node.children.forEach(function (child) { out.push(child); descendants(child, out); });
    return out;
  }
  function attrValue(node, key) {
    if (key === "tabindex") return String(node.tabIndex);
    if (key === "disabled") return node.disabled ? "disabled" : undefined;
    return node.attributes[key];
  }
  function matches(node, selector) {
    selector = selector.trim();
    var nots = [];
    selector = selector.replace(/:not\(([^)]*)\)/g, function (_, inner) { nots.push(inner); return ""; });
    for (var n = 0; n < nots.length; n++) {
      var nm = nots[n].match(/^\[([^=]+)(?:=["']?([^"']*)["']?)?\]$/);
      if (!nm) continue;
      var gotNot = attrValue(node, nm[1].trim());
      if (nm[2] === undefined && gotNot !== undefined) return false;
      if (nm[2] !== undefined && gotNot === nm[2]) return false;
    }
    var attrs = [];
    selector = selector.replace(/\[([^\]]+)\]/g, function (_, inner) {
      var am = inner.match(/^([^=]+)(?:=["']?([^"']*)["']?)?$/);
      if (am) attrs.push([am[1].trim(), am[2]]);
      return "";
    });
    var tag = selector.match(/^[a-zA-Z][\w-]*/);
    if (tag && node.tagName !== tag[0].toUpperCase()) return false;
    var classes = selector.match(/\.[\w-]+/g) || [];
    for (var c = 0; c < classes.length; c++) {
      if ((node.className || "").split(/\s+/).indexOf(classes[c].slice(1)) < 0) return false;
    }
    var id = selector.match(/#[\w-]+/);
    if (id && node.id !== id[0].slice(1)) return false;
    for (var a = 0; a < attrs.length; a++) {
      var actual = attrValue(node, attrs[a][0]);
      if (actual === undefined) return false;
      if (attrs[a][1] !== undefined && actual !== attrs[a][1]) return false;
    }
    return true;
  }
  Node.prototype.querySelectorAll = function (selector) {
    var parts = selector.split(",").map(function (part) { return part.trim(); }).filter(Boolean);
    return descendants(this, []).filter(function (node) {
      return parts.some(function (part) { return matches(node, part); });
    });
  };
  Node.prototype.querySelector = function (selector) {
    return this.querySelectorAll(selector)[0] || null;
  };

  var dom = {
    Node: Node,
    activeElement: null,
    readyState: "complete",
    body: new Node("body"),
    toolbar: new Node("button"),
    listeners: {},
    createElement: function (tag) { return new Node(tag); },
    getElementById: function (id) { return id === "sheetConnectBtn" ? dom.toolbar : null; },
    addEventListener: function (type, fn) {
      (dom.listeners[type] = dom.listeners[type] || []).push(fn);
    },
    removeEventListener: function (type, fn) {
      var list = dom.listeners[type] || [];
      var index = list.indexOf(fn);
      if (index >= 0) list.splice(index, 1);
    },
    dispatch: function (type, ev) {
      ev = ev || {};
      ev.preventDefault = ev.preventDefault || function () {};
      (dom.listeners[type] || []).slice().forEach(function (fn) { fn(ev); });
    },
  };
  dom.querySelector = function (selector) { return dom.body.querySelector(selector); };
  dom.querySelectorAll = function (selector) { return dom.body.querySelectorAll(selector); };
  dom.toolbar.id = "sheetConnectBtn";
  dom.body.appendChild(dom.toolbar);
  return dom;
}

function loadModules() {
  var store = makeStorage();
  var dom = makeDom();
  var fixedNow = Date.UTC(2026, 0, 15, 12, 0, 0);
  var windowListeners = {};
  var windowStub = {
    gbT: function (key) { return "{" + key + "}"; },
    addEventListener: function (type, fn) {
      (windowListeners[type] = windowListeners[type] || []).push(fn);
    },
    removeEventListener: function (type, fn) {
      var list = windowListeners[type] || [];
      var index = list.indexOf(fn);
      if (index >= 0) list.splice(index, 1);
    },
    dispatch: function (type, ev) {
      (windowListeners[type] || []).slice().forEach(function (fn) { fn(ev || {}); });
    },
  };
  var sandbox = {
    window: windowStub,
    document: dom,
    localStorage: store,
    console: { log: function () {}, warn: function () {}, error: function () {} },
    Date: makeDate(fixedNow),
  };
  var maths = Object.create(Math);
  maths.random = function () { return sandbox.__rand == null ? 0 : sandbox.__rand; };
  sandbox.Math = maths;
  sandbox.__rand = 0;
  var context = vm.createContext(sandbox);
  MODULES.forEach(function (name) {
    vm.runInContext(readSource(name), context, { filename: MODULE_PREFIX + name });
  });
  return {
    window: windowStub,
    document: dom,
    store: store,
    context: context,
    fixedNow: fixedNow,
    source: readSource,
    setRandom: function (value) { sandbox.__rand = value; },
    reset: function () { store.clear(); dom.activeElement = null; sandbox.__rand = 0; },
  };
}

var gbLog = globalThis.gbLog || { debug: function (line) { console.log(line); } };

function finish() {
  var rule = "=".repeat(66);
  gbLog.debug("\n" + rule);
  gbLog.debug("SHEET CONNECTIVITY + REFRESH — OFFLINE TEST RESULTS (#2492)");
  gbLog.debug(rule);
  RESULTS.forEach(function (result) {
    gbLog.debug("[" + result.status + "] " + result.name + (result.error ? " — " + result.error : ""));
  });
  gbLog.debug(rule);
  gbLog.debug("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  gbLog.debug("MODE: runtime (one vm sandbox, shipped IIFEs, stubbed host globals) + labelled static source checks");
  gbLog.debug(rule);
  process.exitCode = FAIL > 0 ? 1 : 0;
}

// One sandbox per test run: every case file shares this exact set of shipped
// module instances, so a mutation made through one facade is visible to all.
var env = loadModules();

module.exports = {
  test: test,
  assert: assert,
  equal: equal,
  deep: deep,
  readSource: readSource,
  loadModules: loadModules,
  finish: finish,
  env: env,
  MODULES: MODULES,
  SUITE_DIR: SUITE_DIR,
};
