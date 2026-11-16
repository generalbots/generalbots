/**
 * Shared harness for webtest-apps (#2418).
 * Node/vm-based checks: file existence, readyState-complete mount guards,
 * handler duplication guards, and hostile-input-as-text assertions.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var PASS = 0;
var FAIL = 0;
var RESULTS = [];
var PENDING = [];

function test(name, fn) {
  try {
    var output = fn();
    if (output && typeof output.then === "function") {
      PENDING.push(
        output.then(
          function () {
            PASS++;
            RESULTS.push({ name: name, status: "PASS" });
          },
          function (e) {
            FAIL++;
            RESULTS.push({ name: name, status: "FAIL", error: e.message });
          },
        ),
      );
      return;
    }
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

function fileExists(relPath) {
  return fs.existsSync(path.join(SUITE_DIR, relPath));
}

function makeElement(id, tag, elements) {
  var listeners = {};
  var registry = elements || null;
  function resolve(sel) {
    if (!sel || !registry) return null;
    if (sel[0] === "#") {
      var k = sel.slice(1);
      if (!registry[k]) registry[k] = makeElement(k, "div", registry);
      return registry[k];
    }
    var m = sel.match(/^\.([\w-]+)$/);
    if (m) return null;
    return null;
  }
  var el = {
    id: id || "",
    tagName: (tag || "div").toUpperCase(),
    style: {},
    dataset: {},
    className: "",
    innerHTML: "",
    textContent: "",
    value: "",
    children: [],
    hidden: false,
    disabled: false,
    checked: false,
    attributes: {},
    files: [],
    options: [],
    selectedIndex: 0,
    classList: {
      _set: new Set(),
      add: function (c) { this._set.add(c); },
      remove: function (c) { this._set.delete(c); },
      contains: function (c) { return this._set.has(c); },
      toggle: function (c, force) {
        if (force === undefined) { if (this._set.has(c)) this._set.delete(c); else this._set.add(c); }
        else if (force) this._set.add(c); else this._set.delete(c);
      },
    },
    addEventListener: function (type, fn) {
      if (!listeners[type]) listeners[type] = [];
      listeners[type].push(fn);
    },
    removeEventListener: function (type, fn) {
      if (!listeners[type]) return;
      listeners[type] = listeners[type].filter(function (f) { return f !== fn; });
    },
    dispatchEvent: function (evt) {
      var type = evt && evt.type ? evt.type : evt;
      (listeners[type] || []).slice().forEach(function (fn) { fn(evt || { type: type, target: el, preventDefault: function () {}, stopPropagation: function () {} }); });
    },
    click: function () { this.dispatchEvent({ type: "click", target: this, preventDefault: function () {}, stopPropagation: function () {} }); },
    appendChild: function (child) { this.children.push(child); if (child) child.parentElement = el; return child; },
    insertBefore: function (child) { this.children.push(child); if (child) child.parentElement = el; return child; },
    prepend: function (child) { this.children.unshift(child); if (child) child.parentElement = el; return child; },
    removeChild: function (child) { this.children = this.children.filter(function (c) { return c !== child; }); return child; },
    querySelector: function (sel) { return resolve(sel); },
    querySelectorAll: function () { return []; },
    setAttribute: function (k, v) { this.attributes[k] = String(v); if (k === "id" && registry) registry[v] = el; },
    getAttribute: function (k) { return this.attributes[k]; },
    removeAttribute: function (k) { delete this.attributes[k]; },
    focus: function () {},
    blur: function () {},
    scrollIntoView: function () {},
    closest: function () { return null; },
    getBoundingClientRect: function () { return { top: 0, left: 0, right: 0, bottom: 0, width: 100, height: 40 }; },
    parentElement: null,
  };
  Object.defineProperty(el, "listenerCount", {
    get: function () { return listeners; },
  });
  return el;
}

function makeDocument(opts) {
  opts = opts || {};
  var elements = {};
  var docListeners = {};
  function ensure(id, tag) {
    if (!elements[id]) elements[id] = makeElement(id, tag || "div", elements);
    return elements[id];
  }
  var body = ensure("body", "body");
  var doc = {
    readyState: opts.readyState || "complete",
    body: body,
    documentElement: makeElement("html", "html", elements),
    getElementById: function (id) { return ensure(id); },
    querySelector: function (sel) {
      if (!sel) return null;
      if (sel[0] === "#") return ensure(sel.slice(1));
      return null;
    },
    querySelectorAll: function () { return []; },
    createElement: function (tag) { return makeElement("", tag, elements); },
    addEventListener: function (type, fn) {
      if (!docListeners[type]) docListeners[type] = [];
      docListeners[type].push(fn);
    },
    removeEventListener: function () {},
    createTextNode: function (text) { return { nodeType: 3, textContent: text }; },
    dispatchEvent: function () {},
  };
  (opts.ids || []).forEach(function (id) { ensure(id); });
  doc.__elements = elements;
  doc.__docListeners = docListeners;
  return doc;
}

function loadScript(relPath, sandbox) {
  var code = readFile(relPath);
  var context = vm.createContext(sandbox);
  vm.runInContext(code, context, { filename: relPath });
  return context;
}

function countListeners(el, type) {
  if (!el || !el.listenerCount) return 0;
  return (el.listenerCount[type] || []).length;
}

function finish() {
  return Promise.all(PENDING).then(function () {
    console.log("\n=== webtest-apps (#2418) ===");
    RESULTS.forEach(function (r) {
      if (r.status === "PASS") console.log("  ✓ " + r.name);
      else console.log("  ✗ " + r.name + " — " + (r.error || "failed"));
    });
    console.log("\nTotal: " + (PASS + FAIL) + "  Pass: " + PASS + "  Fail: " + FAIL);
    if (FAIL > 0) process.exit(1);
  });
}

module.exports = {
  SUITE_DIR,
  test,
  assert,
  readFile,
  fileExists,
  makeElement,
  makeDocument,
  loadScript,
  countListeners,
  finish,
};
