/**
 * Offline harness for #2423 docs structured document model verification.
 * Run with: node botui/ui/suite/tests/webtest-docs-model.js
 *
 * The seven shipped IIFEs (68..74) are run in ONE shared vm context per case.
 * Only the environment is stubbed; no production logic is reimplemented.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var PASS = 0;
var FAIL = 0;
var RESULTS = [];
var NOTES = [];

function log(message) { console.log(message); }

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

function note(message) {
  NOTES.push(message);
  log("NOTE: " + message);
}

function assert(condition, msg) {
  if (!condition) throw new Error(msg || "Assertion failed");
}

function equal(actual, expected, msg) {
  if (actual !== expected) {
    throw new Error((msg || "equal") + ": expected <" + expected + "> got <" + actual + ">");
  }
}

function deepEqual(actual, expected, msg) {
  var a = JSON.stringify(actual);
  var b = JSON.stringify(expected);
  if (a !== b) throw new Error((msg || "deepEqual") + ": expected " + b + " got " + a);
}

function readFile(relPath) {
  return fs.readFileSync(path.join(SUITE_DIR, relPath), "utf8");
}

function readRepo(relPath) {
  return fs.readFileSync(path.resolve(SUITE_DIR, "..", "..", "..", relPath), "utf8");
}

function MODEL_MODULES() {
  return [
    "docs/modules/68_model_state.js",
    "docs/modules/69_model_schema.js",
    "docs/modules/70_model_ops.js",
    "docs/modules/71_model_projections.js",
    "docs/modules/72_model_dialog.js",
    "docs/modules/73_model_inspector.js",
    "docs/modules/74_model_init.js",
  ];
}

// ── fake DOM: just enough for the seven shipped IIFEs ──────────────────

function matchesSimple(node, token) {
  token = String(token || "").trim();
  if (!token) return false;
  if (token.indexOf(":not(") >= 0) {
    var cut = token.indexOf(":not(");
    if (!matchesSimple(node, token.slice(0, cut))) return false;
    var negated = token.slice(cut + 5).replace(/\)$/, "");
    return !matchesSimple(node, negated);
  }
  var tagMatch = /^[a-zA-Z][\w-]*/.exec(token);
  if (tagMatch) {
    if (String(node.tagName || "").toLowerCase() !== tagMatch[0].toLowerCase()) return false;
    token = token.slice(tagMatch[0].length);
  }
  while (token) {
    var cls = /^\.([A-Za-z][\w-]*)/.exec(token);
    if (cls) {
      if (String(node.className || "").split(/\s+/).indexOf(cls[1]) < 0) return false;
      token = token.slice(cls[0].length);
      continue;
    }
    var attr = /^\[([^\]="']+)(?:=["']?([^\]"']*)["']?)?\]/.exec(token);
    if (attr) {
      var present = Object.prototype.hasOwnProperty.call(node.attributes, attr[1]);
      if (!present) return false;
      if (typeof attr[2] !== "undefined" && node.attributes[attr[1]] !== attr[2]) return false;
      token = token.slice(attr[0].length);
      continue;
    }
    return false;
  }
  return true;
}

function matches(node, selector) {
  return String(selector || "").split(",").some(function (part) { return matchesSimple(node, part); });
}

function createDom() {
  function textNode(data) {
    return {
      nodeType: 3,
      data: String(data),
      parentNode: null,
      get textContent() { return this.data; },
      set textContent(value) { this.data = String(value); },
    };
  }

  function all(root, selector) {
    var out = [];
    function visit(node) {
      node.children.forEach(function (child) {
        if (child.nodeType !== 1) return;
        if (matches(child, selector)) out.push(child);
        visit(child);
      });
    }
    if (root === documentStub.body && matches(root, selector)) out.push(root);
    visit(root);
    return out;
  }

  function makeElement(tag, doc) {
    var node = {
      nodeType: 1,
      tagName: String(tag || "").toUpperCase(),
      attributes: {},
      children: [],
      parentNode: null,
      ownerDocument: doc,
      className: "",
      id: "",
      value: "",
      placeholder: "",
      disabled: false,
      dataset: {},
      listeners: {},
      focused: false,
      getAttribute: function (name) {
        return Object.prototype.hasOwnProperty.call(this.attributes, name)
          ? this.attributes[name] : null;
      },
      setAttribute: function (name, value) {
        this.attributes[name] = String(value);
        if (name === "id") this.id = String(value);
      },
      removeAttribute: function (name) {
        delete this.attributes[name];
        if (name === "id") this.id = "";
      },
      appendChild: function (child) { return this.insertBefore(child, null); },
      insertBefore: function (child, ref) {
        child.parentNode = this;
        var at = ref ? this.children.indexOf(ref) : -1;
        if (at >= 0) this.children.splice(at, 0, child);
        else this.children.push(child);
        return child;
      },
      removeChild: function (child) {
        var at = this.children.indexOf(child);
        if (at >= 0) { this.children.splice(at, 1); child.parentNode = null; }
        return child;
      },
      addEventListener: function (type, fn) {
        if (!this.listeners[type]) this.listeners[type] = [];
        this.listeners[type].push(fn);
      },
      removeEventListener: function (type, fn) {
        var list = this.listeners[type] || [];
        var at = list.indexOf(fn);
        if (at >= 0) list.splice(at, 1);
      },
      dispatch: function (type, event) {
        (this.listeners[type] || []).slice().forEach(function (fn) { fn(event); });
      },
      focus: function () { doc.activeElement = this; this.focused = true; },
      closest: function (selector) {
        var cur = this;
        while (cur && cur.nodeType === 1) {
          if (matches(cur, selector)) return cur;
          cur = cur.parentNode;
        }
        return null;
      },
      querySelectorAll: function (selector) { return all(this, selector); },
      querySelector: function (selector) { return all(this, selector)[0] || null; },
      get firstChild() { return this.children[0] || null; },
      get textContent() {
        return this.children.map(function (child) { return child.textContent; }).join("");
      },
      set textContent(value) {
        this.children.forEach(function (child) { child.parentNode = null; });
        this.children = [];
        if (value !== "" && value !== null && typeof value !== "undefined") {
          this.appendChild(textNode(value));
        }
      },
    };
    return node;
  }

  var documentStub = {
    body: null,
    activeElement: null,
    readyState: "complete",
    listeners: {},
    createElement: function (tag) { return makeElement(tag, documentStub); },
    getElementById: function (id) {
      var byAttribute = all(documentStub.body, "[id=\"" + id + "\"]")[0];
      if (byAttribute) return byAttribute;
      var found = null;
      (function visit(node) {
        node.children.forEach(function (child) {
          if (found) return;
          if (child.nodeType !== 1) return;
          if (child.id === id) { found = child; return; }
          visit(child);
        });
      }(documentStub.body));
      return found;
    },
    querySelector: function (selector) { return all(documentStub.body, selector)[0] || null; },
    querySelectorAll: function (selector) { return all(documentStub.body, selector); },
    addEventListener: function (type, fn) {
      if (!this.listeners[type]) this.listeners[type] = [];
      this.listeners[type].push(fn);
    },
    removeEventListener: function (type, fn) {
      var list = this.listeners[type] || [];
      var at = list.indexOf(fn);
      if (at >= 0) list.splice(at, 1);
    },
    dispatch: function (type, event) {
      (this.listeners[type] || []).slice().forEach(function (fn) { fn(event); });
    },
  };
  documentStub.body = makeElement("body", documentStub);
  documentStub.activeElement = documentStub.body;
  documentStub.body.focus = function () { documentStub.activeElement = this; };
  return { document: documentStub, makeElement: makeElement, textNode: textNode };
}

// ── deterministic clock / ids / storage / i18n ─────────────────────────

function makeDate(clock) {
  var RealDate = Date;
  function GBDate(value) {
    if (!arguments.length) return new RealDate(clock.now());
    return new RealDate(value);
  }
  GBDate.now = function () { return clock.now(); };
  GBDate.parse = RealDate.parse;
  GBDate.UTC = RealDate.UTC;
  GBDate.prototype = RealDate.prototype;
  return GBDate;
}

function loadModel(options) {
  var opts = options || {};
  var dom = createDom();
  var clock = {
    value: typeof opts.now === "number" ? opts.now : 1700000000000,
    now: function () { return this.value; },
    set: function (value) { this.value = value; },
  };
  var ids = { value: 0 };
  var store = {};
  var gbCalls = [];
  var windowStub = {
    crypto: {
      randomUUID: function () {
        ids.value += 1;
        var hex = ids.value.toString(16);
        while (hex.length < 8) hex = "0" + hex;
        return hex + "-0000-4000-8000-000000000000";
      },
    },
    localStorage: {
      getItem: function (key) { return Object.prototype.hasOwnProperty.call(store, key) ? store[key] : null; },
      setItem: function (key, value) { store[key] = String(value); },
      removeItem: function (key) { delete store[key]; },
      keys: function () { return Object.keys(store); },
    },
    gbT: function (key, fallback, params) {
      gbCalls.push({ key: key, fallback: fallback, params: params });
      return fallback;
    },
  };
  var sandbox = {
    window: windowStub,
    document: dom.document,
    console: console,
    Date: makeDate(clock),
  };
  var context = vm.createContext(sandbox);
  MODEL_MODULES().forEach(function (rel) {
    vm.runInContext(readFile(rel), context, { filename: rel });
  });
  return {
    NS: windowStub.DocsModel,
    window: windowStub,
    doc: dom.document,
    dom: dom,
    context: context,
    clock: clock,
    store: store,
    ids: ids,
    gbCalls: gbCalls,
  };
}

function event(options) {
  var opts = options || {};
  return {
    key: opts.key || "",
    ctrlKey: !!opts.ctrlKey,
    altKey: !!opts.altKey,
    shiftKey: !!opts.shiftKey,
    target: opts.target || null,
    prevented: false,
    stopped: false,
    preventDefault: function () { this.prevented = true; },
    stopPropagation: function () { this.stopped = true; },
  };
}

function finish() {
  log("\n" + "=".repeat(64));
  log("DOCS STRUCTURED MODEL OFFLINE RESULTS (#2423)");
  log("=".repeat(64));
  RESULTS.forEach(function (r) {
    log("[" + r.status + "] " + r.name + (r.error ? " -- " + r.error : ""));
  });
  log("=".repeat(64));
  log("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  log("=".repeat(64));
  process.exitCode = FAIL > 0 ? 1 : 0;
}

module.exports = {
  SUITE_DIR,
  MODEL_MODULES,
  test,
  note,
  assert,
  equal,
  deepEqual,
  readFile,
  readRepo,
  loadModel,
  createDom,
  event,
  matches,
  finish,
};
