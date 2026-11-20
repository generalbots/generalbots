/**
 * Offline harness for #2426 docs long-document structure helper verification.
 * Run with: node botui/ui/suite/tests/webtest-docs-structure.js
 *
 * Mirrors webtest-docs-find/00_harness.js structure (ordered modules, counters,
 * assertions, vm loader, report) and adds a compact fake DOM. The four shipped
 * structure modules (42-45) are IIFEs bound onto window.DocsStructure; all of
 * their DOM access lives inside functions, so they load in a vm sandbox with a
 * stubbed window/document/gbT. Pure helpers are called directly; DOM-bound
 * helpers are exercised against the fake DOM and observed through the real
 * data attributes / label nodes they write.
 */
"use strict";

var fs = require("fs");
var path = require("path");
var vm = require("vm");

var SUITE_DIR = path.resolve(__dirname, "..", "..");
var TESTS_DIR = path.resolve(__dirname, "..");
var PASS = 0;
var FAIL = 0;
var RESULTS = [];

function log(message) {
  console.log(message);
}

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

function readFile(relPath) {
  return fs.readFileSync(path.join(SUITE_DIR, relPath), "utf8");
}

function fileExists(relPath) {
  return fs.existsSync(path.join(SUITE_DIR, relPath));
}

function loadScript(relPath, sandbox) {
  var context = vm.createContext(sandbox);
  vm.runInContext(readFile(relPath), context, { filename: relPath });
  return context;
}

// ── Compact fake DOM (only the surface used by modules 42-45) ─────────

function matches(node, selector) {
  selector = selector.trim();
  if (selector.charAt(0) === "[") {
    var m = /^\[([^\]=]+)(?:="([^"]*)")?\]$/.exec(selector);
    if (!m) return false;
    var name = m[1];
    if (!Object.prototype.hasOwnProperty.call(node.attributes, name)) return false;
    return typeof m[2] === "undefined" || node.attributes[name] === m[2];
  }
  if (selector.charAt(0) === ".") {
    var cls = selector.slice(1);
    return String(node.className || "").split(/\s+/).indexOf(cls) >= 0;
  }
  return String(node.tagName || "").toLowerCase() === selector.toLowerCase();
}

function queryAll(root, selector) {
  var parts = String(selector).split(",").map(function (s) { return s.trim(); });
  var out = [];
  function visit(node) {
    node.children.forEach(function (child) {
      if (child.nodeType !== 1) return;
      for (var i = 0; i < parts.length; i++) {
        if (matches(child, parts[i])) { out.push(child); break; }
      }
      visit(child);
    });
  }
  visit(root);
  return out;
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

  function clone(node, doc, deep) {
    if (node.nodeType === 3) return textNode(node.data);
    var copy = makeElement(node.tagName, doc);
    Object.keys(node.attributes).forEach(function (name) {
      copy.attributes[name] = node.attributes[name];
    });
    copy.className = node.className;
    if (deep) {
      node.children.forEach(function (child) {
        var childCopy = clone(child, doc, true);
        childCopy.parentNode = copy;
        copy.children.push(childCopy);
      });
    }
    return copy;
  }

  function makeElement(tag, doc) {
    return {
      nodeType: 1,
      tagName: String(tag || "").toUpperCase(),
      attributes: {},
      children: [],
      parentNode: null,
      ownerDocument: doc,
      className: "",
      getAttribute: function (name) {
        return Object.prototype.hasOwnProperty.call(this.attributes, name)
          ? this.attributes[name] : null;
      },
      setAttribute: function (name, value) { this.attributes[name] = String(value); },
      removeChild: function (child) {
        var i = this.children.indexOf(child);
        if (i >= 0) { this.children.splice(i, 1); child.parentNode = null; }
        return child;
      },
      insertBefore: function (node, ref) {
        node.parentNode = this;
        var i = ref ? this.children.indexOf(ref) : -1;
        if (i >= 0) this.children.splice(i, 0, node);
        else this.children.push(node);
        return node;
      },
      appendChild: function (node) { return this.insertBefore(node, null); },
      get firstChild() { return this.children[0] || null; },
      get nextSibling() {
        if (!this.parentNode) return null;
        var i = this.parentNode.children.indexOf(this);
        return i >= 0 ? (this.parentNode.children[i + 1] || null) : null;
      },
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
      querySelectorAll: function (selector) { return queryAll(this, selector); },
      querySelector: function (selector) { return queryAll(this, selector)[0] || null; },
      cloneNode: function (deep) { return clone(this, doc, deep); },
    };
  }

  var documentStub = {
    createElement: function (tag) { return makeElement(tag, documentStub); },
    getElementById: function () { return null; },
    querySelector: function () { return null; },
    execCommand: function () { return false; },
  };
  return { document: documentStub, makeElement: makeElement, textNode: textNode };
}

var STRUCTURE_MODULES = [
  "docs/modules/42_structure_core.js",
  "docs/modules/43_structure_numbering.js",
  "docs/modules/44_structure_fields.js",
  "docs/modules/45_structure_captions.js",
];

/** Load the four real shipped modules, in order, into one shared sandbox. */
function loadStructure(options) {
  var opts = options || {};
  var dom = createDom();
  var windowStub = opts.window || {};
  if (typeof windowStub.gbT !== "function") {
    windowStub.gbT = function (key, fallback) { return fallback; };
  }
  var sandbox = {
    window: windowStub,
    document: dom.document,
    console: console,
    crypto: opts.crypto === null ? undefined : (opts.crypto || {
      randomUUID: function () { return "12345678-abcd-efgh-ijkl-mnopqrstuvwx"; },
    }),
  };
  var context = vm.createContext(sandbox);
  STRUCTURE_MODULES.forEach(function (rel) {
    vm.runInContext(readFile(rel), context, { filename: rel });
  });
  return {
    context: context,
    NS: windowStub.DocsStructure,
    window: windowStub,
    doc: dom.document,
    dom: dom,
  };
}

function report() {
  log("\n" + "=".repeat(60));
  log("DOCS STRUCTURE OFFLINE RESULTS (#2426)");
  log("=".repeat(60));
  RESULTS.forEach(function (r) {
    log("[" + r.status + "] " + r.name + (r.error ? " -- " + r.error : ""));
  });
  log("=".repeat(60));
  log("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  log("=".repeat(60));
  process.exitCode = FAIL > 0 ? 1 : 0;
}

function finish() {
  report();
}

module.exports = {
  fs,
  path,
  vm,
  SUITE_DIR,
  TESTS_DIR,
  STRUCTURE_MODULES,
  test,
  assert,
  readFile,
  fileExists,
  loadScript,
  createDom,
  loadStructure,
  log,
  finish,
};
