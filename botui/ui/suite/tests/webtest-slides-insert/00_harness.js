/**
 * Slides Insert hub — durable offline test harness (#2428).
 * Run with: node botui/ui/suite/tests/webtest-slides-insert.js
 *
 * The four real shipped IIFEs are evaluated in ONE shared Node vm sandbox with
 * stubbed window/document/gbT/storage/clock/uid. No production logic is copied,
 * instrumented or modified: every runtime assertion calls a genuine function
 * loaded from botui/ui/suite/slides/modules/. DOM-only and backend-deferred
 * behaviour is labelled STATIC in the test files, never reported as runtime.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var REPO_DIR = path.resolve(SUITE_DIR, "..", "..", "..");
var MODULE_RELS = [
  "slides/modules/10_insert_catalog.js",
  "slides/modules/11_insert_dialog.js",
  "slides/modules/12_insert_actions.js",
  "slides/modules/13_insert_place.js",
];

var PASS = 0;
var FAIL = 0;
var RESULTS = [];
var PENDING = [];

function record(name, status, error) {
  RESULTS.push({ name: name, status: status, error: error || "" });
  if (status === "PASS") PASS++;
  else FAIL++;
}

function test(name, fn) {
  try {
    fn();
    record(name, "PASS");
  } catch (e) {
    record(name, "FAIL", e && e.message ? e.message : String(e));
  }
}

/* Promise tests MUST run one at a time: the shared sandbox has mutable
 * storage, fetch handlers and DOM. Their bodies are deferred to finish(). */
function asyncTest(name, fn) {
  RESULTS.push({ name: name, status: "PENDING", error: "" });
  PENDING.push({ name: name, fn: fn });
}

function assert(condition, msg) {
  if (!condition) throw new Error(msg || "Assertion failed");
}

function readRepo(rel) {
  return fs.readFileSync(path.join(REPO_DIR, rel), "utf8");
}

function readSuite(rel) {
  return fs.readFileSync(path.join(SUITE_DIR, rel), "utf8");
}

function silentConsole() {
  return { log: function () {}, warn: function () {}, error: function () {} };
}

// ── Minimal DOM (only operations the shipped insert modules actually call) ──
var ACTIVE_DOC = null;

function dataName(key) {
  return "data-" + String(key).replace(/[A-Z]/g, function (c) {
    return "-" + c.toLowerCase();
  });
}

function splitParts(selector) {
  var parts = [];
  var current = "";
  var quote = "";
  for (var i = 0; i < selector.length; i++) {
    var ch = selector[i];
    if (quote) {
      current += ch;
      if (ch === quote) quote = "";
    } else if (ch === '"' || ch === "'") {
      quote = ch;
      current += ch;
    } else if (/\s/.test(ch)) {
      if (current) parts.push(current);
      current = "";
    } else {
      current += ch;
    }
  }
  if (current) parts.push(current);
  return parts;
}

function matchesCompound(node, compound) {
  if (!node || typeof node.getAttribute !== "function") return false;
  var tokens = compound.match(/([#.]?[\w-]+)|(\[[^\]]+\])|(:not\([^)]*\))/g) || [];
  return tokens.every(function (token) {
    if (token[0] === "#") return node.getAttribute("id") === token.slice(1);
    if (token[0] === ".") return String(node.className || "").split(/\s+/).indexOf(token.slice(1)) >= 0;
    if (token[0] === "[") {
      var body = token.slice(1, -1);
      var eq = body.indexOf("=");
      var name = eq < 0 ? body : body.slice(0, eq);
      var raw = eq < 0 ? null : body.slice(eq + 1).trim().replace(/^["']|["']$/g, "");
      var value = node.getAttribute(name);
      return value !== null && (eq < 0 || value === raw);
    }
    if (token.indexOf(":not(") === 0) {
      var inner = token.slice(5, -1);
      return !matchesCompound(node, inner);
    }
    return String(node.tag || "").toLowerCase() === token.toLowerCase();
  });
}

function descendants(node, out) {
  node.children.forEach(function (child) {
    out.push(child);
    descendants(child, out);
  });
  return out;
}

function queryAll(root, selector) {
  var wanted = selector.split(",").map(function (s) { return splitParts(s.trim()); });
  return descendants(root, []).filter(function (node) {
    return wanted.some(function (parts) {
      if (!parts.length || !matchesCompound(node, parts[parts.length - 1])) return false;
      var ancestor = node.parentNode;
      for (var i = parts.length - 2; i >= 0; i--) {
        while (ancestor && !matchesCompound(ancestor, parts[i])) ancestor = ancestor.parentNode;
        if (!ancestor) return false;
        ancestor = ancestor.parentNode;
      }
      return true;
    });
  });
}

function dispatch(node, type, event) {
  event = event || {};
  event.target = event.target || node;
  if (typeof event.preventDefault !== "function") {
    event.preventDefault = function () { event.defaultPrevented = true; };
  }
  if (typeof event.stopPropagation !== "function") event.stopPropagation = function () {};
  var list = (node._listeners && node._listeners[type]) || [];
  list.slice().forEach(function (fn) { fn(event); });
  return event;
}

function fakeNode(tag) {
  var classes = {};
  var node = {
    tag: String(tag || ""),
    attrs: {},
    children: [],
    parentNode: null,
    _listeners: {},
    className: "",
    _text: "",
    style: {},
    value: "",
    tabIndex: -1,
    disabled: false,
    draggable: false,
    offsetParent: {},
    getAttribute: function (name) {
      return Object.prototype.hasOwnProperty.call(node.attrs, name) ? node.attrs[name] : null;
    },
    setAttribute: function (name, value) { node.attrs[name] = String(value); },
    removeAttribute: function (name) { delete node.attrs[name]; },
    hasAttribute: function (name) { return node.getAttribute(name) !== null; },
    appendChild: function (child) {
      if (child.parentNode) child.parentNode.removeChild(child);
      node.children.push(child);
      child.parentNode = node;
      return child;
    },
    removeChild: function (child) {
      var at = node.children.indexOf(child);
      if (at >= 0) node.children.splice(at, 1);
      if (child.parentNode === node) child.parentNode = null;
      return child;
    },
    remove: function () {
      if (node.parentNode) node.parentNode.removeChild(node);
    },
    addEventListener: function (type, fn) {
      node._listeners[type] = node._listeners[type] || [];
      node._listeners[type].push(fn);
    },
    querySelector: function (selector) { return queryAll(node, selector)[0] || null; },
    querySelectorAll: function (selector) { return queryAll(node, selector); },
    closest: function (selector) {
      var part = splitParts(selector)[0];
      var current = node;
      while (current) {
        if (matchesCompound(current, part)) return current;
        current = current.parentNode;
      }
      return null;
    },
    getBoundingClientRect: function () { return { left: 0, top: 0, width: 960, height: 540 }; },
    focus: function () { if (ACTIVE_DOC) ACTIVE_DOC.activeElement = node; },
    classList: {
      add: function (name) { classes[name] = true; },
      remove: function (name) { delete classes[name]; },
      contains: function (name) { return !!classes[name]; },
      toggle: function (name, on) {
        if (on) classes[name] = true;
        else delete classes[name];
        return !!classes[name];
      },
    },
  };
  Object.defineProperty(node, "textContent", {
    get: function () { return node._text; },
    set: function (value) {
      node._text = String(value);
      if (value === "") node.children.slice().forEach(function (child) { node.removeChild(child); });
    },
  });
  Object.defineProperty(node, "id", {
    get: function () { return node.getAttribute("id") || ""; },
    set: function (value) { node.setAttribute("id", value); },
  });
  node.dataset = new Proxy({}, {
    get: function (_t, key) { return typeof key === "string" ? node.getAttribute(dataName(key)) : undefined; },
    set: function (_t, key, value) { node.setAttribute(dataName(key), String(value)); return true; },
    has: function (_t, key) { return node.hasAttribute(dataName(key)); },
  });
  return node;
}

function fakeDocument() {
  var body = fakeNode("body");
  var doc = {
    readyState: "loading",
    activeElement: null,
    _listeners: {},
    children: [body],
    parentNode: null,
    body: body,
    addEventListener: function (type, fn) {
      doc._listeners[type] = doc._listeners[type] || [];
      doc._listeners[type].push(fn);
    },
    createElement: function (tag) { return fakeNode(tag); },
    createElementNS: function (_ns, tag) { return fakeNode(tag); },
    getElementById: function (id) { return queryAll(doc, "#" + id)[0] || null; },
    querySelector: function (selector) { return queryAll(doc, selector)[0] || null; },
    querySelectorAll: function (selector) { return queryAll(doc, selector); },
  };
  body.parentNode = doc;
  ACTIVE_DOC = doc;
  return doc;
}

function memoryStorage() {
  var map = new Map();
  return {
    getItem: function (key) { return map.has(key) ? map.get(key) : null; },
    setItem: function (key, value) { map.set(key, String(value)); },
    removeItem: function (key) { map.delete(key); },
    clear: function () { map.clear(); },
  };
}

// ── One shared sandbox for all shipped modules ────────────────────────────
function loadModules() {
  var doc = fakeDocument();
  var statusBar = doc.createElement("div");
  statusBar.className = "sl-status-bar";
  var statusWrap = doc.createElement("span");
  var statusDot = doc.createElement("span");
  statusWrap.appendChild(statusDot);
  statusBar.appendChild(statusWrap);
  doc.body.appendChild(statusBar);
  var fetchCalls = [];
  var refreshCalls = [];
  var toasts = [];
  var fetchHandler = function () {
    return Promise.resolve({ ok: true, status: 200, json: function () { return Promise.resolve({}); } });
  };
  function gbT(key, fallback, params) {
    var out = "<" + key + ">";
    if (params) {
      Object.keys(params).forEach(function (name) {
        out = out.split("{" + name + "}").join(String(params[name]));
      });
    }
    return out;
  }
  var win = {
    gbT: gbT,
    localStorage: memoryStorage(),
    sessionStorage: memoryStorage(),
    location: { origin: "http://localhost" },
    SlideCanvas: { scale: 1 },
    GBToasts: { show: function (title, message, kind) { toasts.push({ title: title, message: message, kind: kind }); } },
    getComputedStyle: function () { return { color: "rgb(1, 2, 3)" }; },
    gbFormatDate: function () { return "<date>"; },
  };
  var sandbox = {
    window: win,
    document: doc,
    console: silentConsole(),
    SLIDE_W: 960,
    SLIDE_H: 540,
    fetch: function (url, init) { fetchCalls.push({ url: url, init: init }); return fetchHandler(url, init); },
    getGBAccessToken: function () { return "<token>"; },
    getSlidesPresentationId: function () { return "<deck>"; },
    gbStoreGet: function (key, fallback) {
      var raw = win.localStorage.getItem(key);
      if (!raw) return fallback;
      try { return JSON.parse(raw); } catch (e) { return fallback; }
    },
    refreshSlideView: function (index) { refreshCalls.push(index); },
  };
  var context = vm.createContext(sandbox);
  MODULE_RELS.forEach(function (rel) {
    vm.runInContext(readSuite(rel), context, { filename: rel });
  });
  var uid = 0;
  sandbox.SlidesInsertPlace.newId = function () { uid += 1; return "<uid" + uid + ">"; };
  function reset() {
    sandbox.SlidesInsertHub.close();
    win.localStorage.clear();
    win.sessionStorage.clear();
    sandbox.SlidesInsertHub.category = "recent";
    sandbox.SlidesInsertHub.query = "";
    sandbox.SlidesInsertHub.selectedId = null;
    sandbox.SlidesInsertHub.reuseItems = [];
    sandbox.SlidesInsertHub.optionCache = null;
    sandbox.SlidesInsertPlace._cascade = 0;
    sandbox.SlidesInsertPlace.pendingImages = {};
    win.SlideCanvas.scale = 1;
    fetchCalls.length = 0;
    refreshCalls.length = 0;
    toasts.length = 0;
    uid = 0;
  }
  return {
    sandbox: sandbox,
    win: win,
    doc: doc,
    hub: sandbox.SlidesInsertHub,
    place: sandbox.SlidesInsertPlace,
    reset: reset,
    resetUid: function () { uid = 0; },
    setFetchHandler: function (fn) { fetchHandler = fn; },
    fetchCalls: fetchCalls,
    refreshCalls: refreshCalls,
    toasts: toasts,
    runInit: function () { (doc._listeners.DOMContentLoaded || []).slice().forEach(function (fn) { fn(); }); },
    gbT: gbT,
  };
}

var env = loadModules();

function text(node) { return node ? node.textContent : ""; }
function setText(node, value) { node.textContent = value; }
function fire(node, type, event) { return dispatch(node, type, event); }

var gbLog = globalThis.gbLog || { debug: function (line) { console.log(line); } };

function report() {
  var rule = "=".repeat(64);
  gbLog.debug("\n" + rule);
  gbLog.debug("SLIDES INSERT HUB — OFFLINE TEST RESULTS (#2428)");
  gbLog.debug(rule);
  RESULTS.forEach(function (result) {
    gbLog.debug("[" + result.status + "] " + result.name + (result.error ? " — " + result.error : ""));
  });
  gbLog.debug(rule);
  gbLog.debug("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  gbLog.debug("MODE: RUNTIME (one vm sandbox; real shipped IIFEs, stubbed host)" +
    " + STATIC (source/catalog assertions only)");
  gbLog.debug("NOT COVERED: browser paint, real drag/drop dataTransfer, server-side legacy modals");
  gbLog.debug(rule);
  process.exitCode = FAIL > 0 ? 1 : 0;
}

function mark(name, status, error) {
  RESULTS.forEach(function (r) {
    if (r.name === name) {
      r.status = status;
      r.error = error || "";
    }
  });
  if (status === "PASS") PASS++;
  else FAIL++;
}

function finish() {
  return PENDING.reduce(function (chain, item) {
    return chain.then(function () {
      var out;
      try {
        out = item.fn();
      } catch (e) {
        mark(item.name, "FAIL", e && e.message ? e.message : String(e));
        return;
      }
      return Promise.resolve(out).then(function () {
        mark(item.name, "PASS");
      }, function (error) {
        mark(item.name, "FAIL", error && error.message ? error.message : String(error));
      });
    });
  }, Promise.resolve()).then(report, function (error) {
    record("async harness", "FAIL", error && error.message ? error.message : String(error));
    report();
  });
}

module.exports = {
  test: test,
  asyncTest: asyncTest,
  assert: assert,
  readRepo: readRepo,
  readSuite: readSuite,
  env: env,
  MODULE_RELS: MODULE_RELS,
  SUITE_DIR: SUITE_DIR,
  REPO_DIR: REPO_DIR,
  text: text,
  setText: setText,
  fire: fire,
  queryAll: queryAll,
  finish: finish,
};
