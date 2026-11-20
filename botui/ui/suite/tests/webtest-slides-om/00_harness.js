/**
 * Slides canvas object model — offline test harness (#2430).
 *
 * Plain-Node vm sandbox. The seven shipped IIFEs are read from disk and run in
 * one shared context; window/document/storage/clock/uid are stubs only. No
 * production function is copied or reimplemented. Tests live in 10_om.js and
 * 11_om_ui.js; this file owns counters, the fake host and the report.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var MODULE_RELS = [
  "slides/modules/46_om_state.js",
  "slides/modules/47_om_ops.js",
  "slides/modules/47_om_ops_media.js",
  "slides/modules/48_om_canvas.js",
  "slides/modules/49_om_dialog.js",
  "slides/modules/49_om_dialog_media.js",
  "slides/modules/50_om_init.js"
];

var PASS = 0;
var FAIL = 0;
var RESULTS = [];
var activeDocument = null;

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
  assert(actual === expected, (msg || "values differ") + ": " + JSON.stringify(actual) + " != " + JSON.stringify(expected));
}

function source(rel) {
  return fs.readFileSync(path.join(SUITE_DIR, rel), "utf8");
}

function silentConsole() {
  return { log: function () {}, warn: function () {}, error: function () {}, debug: function () {} };
}

function specPart(node, part) {
  if (part === "*") return true;
  if (part.charAt(0) === ".") return node.classList.contains(part.slice(1));
  if (part.charAt(0) === "[") {
    var attr = /^\[([^=\]]+)(?:=["']?([^"'\]]+)["']?)?\]$/.exec(part);
    if (!attr) return false;
    var value = node.getAttribute(attr[1]);
    return attr[2] === undefined ? value !== null : value === attr[2];
  }
  return node.tagName === part.toUpperCase();
}

function matches(node, selector) {
  return String(selector).split(",").some(function (raw) {
    var part = raw.trim();
    var guarded = /^([\[\]\w-]+):not\(\[([^=\]]+)(?:=(["'])([^"']*)\3)?\]\)$/.exec(part);
    if (!guarded) return specPart(node, part);
    if (!specPart(node, guarded[1])) return false;
    var actual = node.getAttribute(guarded[2]);
    return guarded[4] === undefined ? actual === null : actual !== guarded[4];
  });
}

/* Minimal DOM node: only operations the shipped canvas/dialog modules call. */
function fakeNode(tag, attrs) {
  var node = {
    tagName: String(tag || "div").toUpperCase(),
    tag: tag || "div",
    className: "",
    attrs: Object.assign({}, attrs || {}),
    dataset: {},
    children: [],
    parentNode: null,
    nextSibling: null,
    style: {},
    _text: "",
    _listeners: {},
    tabIndex: 0,
    offsetParent: {},
    type: "",
    value: "",
    checked: false,
    disabled: false
  };

  Object.defineProperty(node, "firstChild", {
    get: function () { return node.children[0] || null; }
  });
  Object.defineProperty(node, "textContent", {
    get: function () { return node._text; },
    set: function (value) {
      node._text = value == null ? "" : String(value);
      if (node._text === "") node.children.slice().forEach(function (child) { node.removeChild(child); });
    }
  });

  function classes() { return node.className ? node.className.split(/\s+/) : []; }
  node.classList = {
    add: function (name) {
      var list = classes();
      if (list.indexOf(name) < 0) list.push(name);
      node.className = list.join(" ");
    },
    remove: function (name) {
      node.className = classes().filter(function (item) { return item !== name; }).join(" ");
    },
    contains: function (name) { return classes().indexOf(name) >= 0; },
    toggle: function (name, on) {
      if (on) node.classList.add(name); else node.classList.remove(name);
    }
  };

  node.getAttribute = function (key) {
    if (key === "class" && node.className) return node.className;
    if (Object.prototype.hasOwnProperty.call(node.attrs, key)) return node.attrs[key];
    if (key === "tabindex") return String(node.tabIndex);
    return null;
  };
  node.setAttribute = function (key, value) {
    node.attrs[key] = String(value);
    if (key === "class") node.className = String(value);
  };
  node.removeAttribute = function (key) { delete node.attrs[key]; };

  function resync() {
    node.children.forEach(function (child, index) {
      child.nextSibling = node.children[index + 1] || null;
    });
  }
  node.appendChild = function (child) {
    if (child.parentNode) child.parentNode.removeChild(child);
    node.children.push(child);
    child.parentNode = node;
    resync();
    return child;
  };
  node.insertBefore = function (child, ref) {
    if (child.parentNode) child.parentNode.removeChild(child);
    var at = ref ? node.children.indexOf(ref) : -1;
    if (at < 0) at = node.children.length;
    node.children.splice(at, 0, child);
    child.parentNode = node;
    resync();
    return child;
  };
  node.removeChild = function (child) {
    var at = node.children.indexOf(child);
    if (at >= 0) node.children.splice(at, 1);
    if (child.parentNode === node) child.parentNode = null;
    resync();
    return child;
  };
  node.remove = function () { if (node.parentNode) node.parentNode.removeChild(node); };

  node.querySelectorAll = function (selector) {
    var out = [];
    node.children.forEach(function (child) {
      if (matches(child, selector)) out.push(child);
      if (child.querySelectorAll) out = out.concat(child.querySelectorAll(selector));
    });
    return out;
  };
  node.querySelector = function (selector) { return node.querySelectorAll(selector)[0] || null; };
  node.closest = function (selector) {
    var cursor = node;
    while (cursor) {
      if (matches(cursor, selector)) return cursor;
      cursor = cursor.parentNode;
    }
    return null;
  };
  node.addEventListener = function (type, fn, capture) {
    node._listeners[type] = node._listeners[type] || [];
    node._listeners[type].push({ fn: fn, capture: !!capture });
  };
  node.removeEventListener = function (type, fn) {
    var list = node._listeners[type] || [];
    for (var i = list.length - 1; i >= 0; i--) {
      if (list[i].fn === fn) list.splice(i, 1);
    }
  };
  node.focus = function () {
    node._focused = true;
    if (activeDocument) activeDocument.activeElement = node;
  };
  node.getBoundingClientRect = function () { return { left: 0, top: 0, width: 400, height: 300 }; };
  return node;
}

function makeEventTarget(target) {
  target._listeners = {};
  target.addEventListener = function (type, fn, capture) {
    target._listeners[type] = target._listeners[type] || [];
    target._listeners[type].push({ fn: fn, capture: !!capture });
  };
  target.removeEventListener = function (type, fn) {
    var list = target._listeners[type] || [];
    for (var i = list.length - 1; i >= 0; i--) {
      if (list[i].fn === fn) list.splice(i, 1);
    }
  };
  return target;
}

function dispatch(target, type, event) {
  var ev = Object.assign({
    target: target,
    key: "",
    button: 0,
    clientX: 0,
    clientY: 0,
    shiftKey: false,
    ctrlKey: false,
    metaKey: false,
    altKey: false,
    preventDefault: function () { ev.defaultPrevented = true; },
    stopImmediatePropagation: function () { ev.stopped = true; }
  }, event || {});
  var list = (target._listeners && target._listeners[type]) || [];
  list.slice().forEach(function (entry) {
    if (!ev.stopped) entry.fn(ev);
  });
  return ev;
}

function fakeDocument(canvasNode, toolbar) {
  var doc = makeEventTarget({
    readyState: "loading",
    body: fakeNode("body"),
    documentElement: fakeNode("html"),
    activeElement: null,
    getElementById: function (id) {
      if (id === "slide-canvas") return canvasNode;
      if (id === "slObjectModelBtn") return toolbar;
      return null;
    },
    createElement: function (tag) { return fakeNode(tag); },
    createElementNS: function (ns, tag) { return fakeNode(tag); }
  });
  doc.body.ownerDocument = doc;
  activeDocument = doc;
  doc.activeElement = fakeNode("body");
  return doc;
}

function loadModules() {
  var canvasNode = fakeNode("div");
  canvasNode.dataset.slideId = "0";
  var toolbar = fakeNode("button");
  var doc = fakeDocument(canvasNode, toolbar);
  var gbCalls = [];
  var store = {};
  var storage = {
    getItem: function (key) { return Object.prototype.hasOwnProperty.call(store, key) ? store[key] : null; },
    setItem: function (key, value) { store[key] = String(value); },
    removeItem: function (key) { delete store[key]; },
    clear: function () { Object.keys(store).forEach(function (key) { delete store[key]; }); }
  };
  var win = makeEventTarget({
    __prompts: 0,
    alert: function () { win.__prompts++; },
    confirm: function () { win.__prompts++; },
    prompt: function () { win.__prompts++; },
    gbT: function (key, fallback, params) {
      gbCalls.push({ key: key, fallback: fallback, params: params });
      return fallback;
    },
    localStorage: storage,
    sessionStorage: storage,
    getSlidesPresentationId: function () { return "offline-deck"; },
    SlideCanvas: { canvas: canvasNode, scale: 1 }
  });
  var sandbox = {
    window: win,
    document: doc,
    console: silentConsole(),
    setTimeout: function () { return 0; },
    clearTimeout: function () {}
  };
  var ctx = vm.createContext(sandbox);
  vm.runInContext(
    "Date.now = function () { return 1700000000000; };" +
    "Math.random = function () { window.__seq = (window.__seq || 0) + 1; return (window.__seq % 997) / 1001; };",
    ctx,
    { filename: "offline clock and uid" }
  );
  MODULE_RELS.forEach(function (rel) {
    vm.runInContext(source(rel), ctx, { filename: rel });
  });
  var api = {
    OM: win.GBSlidesOM,
    Ops: win.GBSlidesOMOps,
    helpers: win.GBSlidesOMOpsHelpers,
    Canvas: win.GBSlidesOMCanvas,
    Dialog: win.GBSlidesOMDialog,
    DialogUI: win.GBSlidesOMDialogUI,
    Init: win.GBSlidesObjectModel,
    window: win,
    document: doc,
    canvas: canvasNode,
    toolbar: toolbar,
    storage: storage,
    store: store,
    gbCalls: gbCalls,
    sandbox: sandbox,
    context: ctx
  };
  dispatch(doc, "DOMContentLoaded", {});
  return api;
}

var api = null;

function env() {
  if (!api) api = loadModules();
  return api;
}

function reset() {
  var e = env();
  if (e.Dialog.isOpen()) e.Dialog.close();
  e.OM.state = e.OM.fresh();
  e.OM.selection = [];
  e.OM.undoStack = [];
  e.OM.redoStack = [];
  e.OM._lastLabel = "";
  e.OM.penMode = false;
  e.OM.connectorMode = null;
  e.window.__seq = 0;
  e.window.__prompts = 0;
  e.storage.clear();
  e.gbCalls.length = 0;
  e.canvas.children.slice().forEach(function (child) { e.canvas.removeChild(child); });
  e.canvas.dataset.slideId = "0";
  e.document.activeElement = e.document.body;
}

function finish() {
  var rule = "=".repeat(66);
  var gbLog = globalThis.gbLog || { debug: function (line) { console.log(line); } };
  gbLog.debug("\n" + rule);
  gbLog.debug("SLIDES OBJECT MODEL — OFFLINE TEST RESULTS (#2430)");
  gbLog.debug(rule);
  RESULTS.forEach(function (result) {
    gbLog.debug("[" + result.status + "] " + result.name + (result.error ? " — " + result.error : ""));
  });
  gbLog.debug(rule);
  gbLog.debug("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  gbLog.debug("MODE: runtime (real shipped IIFEs in one vm sandbox; stubbed host) + labelled STATIC checks");
  gbLog.debug("NOT COVERED: browser pointer UAT, server persistence, real PPTX export");
  gbLog.debug(rule);
  process.exitCode = FAIL > 0 ? 1 : 0;
}

module.exports = {
  test: test, assert: assert, equal: equal, source: source,
  fakeNode: fakeNode, dispatch: dispatch, fakeDocument: fakeDocument,
  loadModules: loadModules, env: env, reset: reset, finish: finish,
  MODULE_RELS: MODULE_RELS, SUITE_DIR: SUITE_DIR
};
