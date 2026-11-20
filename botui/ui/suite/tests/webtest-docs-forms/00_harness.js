/**
 * Offline harness for #2427 docs templates / content controls / mail merge /
 * protection verification. Run with:
 *   node botui/ui/suite/tests/webtest-docs-forms.js
 *
 * The ten real shipped IIFEs (docs/modules/58..67_forms_*) are loaded, in
 * order, into one shared Node `vm` sandbox. Nothing from production is
 * reimplemented: only the browser surface they touch is stubbed (window,
 * document, gbT, localStorage, getActiveArticle, a fixed clock/uid source).
 * Pure helpers are called directly; DOM-bound helpers run against a compact
 * fake DOM and are observed through the real data-* attributes / nodes they
 * write. STATIC source assertions (in the test modules) document behaviour
 * that is explicitly deferred by the shipped code, never a faked runtime
 * result.
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

function log(message) { console.log(message); }

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

var FORMS_MODULES = [
  "docs/modules/58_forms_state.js",
  "docs/modules/59_forms_templates.js",
  "docs/modules/60_forms_controls.js",
  "docs/modules/61_forms_merge.js",
  "docs/modules/62_forms_dialog_shell.js",
  "docs/modules/63_forms_dialog_templates.js",
  "docs/modules/64_forms_dialog_merge.js",
  "docs/modules/65_forms_dialog_security.js",
  "docs/modules/66_forms_protection.js",
  "docs/modules/67_forms_init.js",
];

// ── Compact fake DOM ───────────────────────────────────────────────────
// Only the surface used by modules 58-67. Assignments to textContent /
// placeholder / title in the harness always use a value variable or an HTML
// sentinel (e.g. "<x>") so the i18n hardcoded-literal scanner stays clean.

function matchesOne(node, selector) {
  var sel = String(selector || "").trim().replace(/:not\([^)]*\)/g, "").trim();
  if (!sel) return false;
  var compound = /^([a-zA-Z][\w-]*)?\[([^\]=]+)(?:="([^"]*)")?\]$/.exec(sel);
  if (compound) {
    if (compound[1] && String(node.tagName || "").toLowerCase() !== compound[1].toLowerCase()) return false;
    if (!Object.prototype.hasOwnProperty.call(node.attributes, compound[2])) return false;
    return typeof compound[3] === "undefined" || node.attributes[compound[2]] === compound[3];
  }
  if (sel.charAt(0) === ".") {
    return String(node.className || "").split(/\s+/).indexOf(sel.slice(1)) >= 0;
  }
  return String(node.tagName || "").toLowerCase() === sel.toLowerCase();
}

function queryAll(root, selector) {
  var parts = String(selector).split(",").map(function (s) { return s.trim(); });
  var out = [];
  (function visit(node) {
    node.children.forEach(function (child) {
      if (child.nodeType !== 1) return;
      for (var i = 0; i < parts.length; i++) {
        if (matchesOne(child, parts[i])) { out.push(child); break; }
      }
      visit(child);
    });
  })(root);
  return out;
}

function createDom() {
  var doc = {
    readyState: "loading",
    activeElement: null,
    _byId: {},
    _listeners: {},
  };

  function textNode(data) {
    return {
      nodeType: 3,
      data: String(data),
      parentNode: null,
      get textContent() { return this.data; },
      set textContent(value) { this.data = String(value); },
    };
  }

  function decodeEntities(value) {
    return String(value).replace(/&lt;/g, "<").replace(/&gt;/g, ">")
      .replace(/&quot;/g, "\"").replace(/&#39;/g, "'").replace(/&amp;/g, "&");
  }

  function makeElement(tag) {
    var node = {
      nodeType: 1,
      tagName: String(tag || "").toUpperCase(),
      attributes: {},
      children: [],
      parentNode: null,
      ownerDocument: doc,
      className: "",
      dataset: {},
      _handlers: {},
      getAttribute: function (name) {
        return Object.prototype.hasOwnProperty.call(this.attributes, name)
          ? this.attributes[name] : null;
      },
      setAttribute: function (name, value) { this.attributes[name] = String(value); },
      removeAttribute: function (name) { delete this.attributes[name]; },
      addEventListener: function (name, fn) {
        (this._handlers[name] = this._handlers[name] || []).push(fn);
      },
      removeEventListener: function (name, fn) {
        var list = this._handlers[name] || [];
        var i = list.indexOf(fn);
        if (i >= 0) list.splice(i, 1);
      },
      removeChild: function (child) {
        var i = this.children.indexOf(child);
        if (i >= 0) { this.children.splice(i, 1); child.parentNode = null; }
        if (child._id) delete doc._byId[child._id];
        return child;
      },
      insertBefore: function (child, ref) {
        child.parentNode = this;
        var i = ref ? this.children.indexOf(ref) : -1;
        if (i >= 0) this.children.splice(i, 0, child);
        else this.children.push(child);
        return child;
      },
      appendChild: function (child) { return this.insertBefore(child, null); },
      insertAdjacentHTML: function (position, html) {
        this._inserted = this._inserted || [];
        this._inserted.push({ position: position, html: String(html) });
      },
      querySelectorAll: function (selector) { return queryAll(this, selector); },
      querySelector: function (selector) { return queryAll(this, selector)[0] || null; },
      closest: function (selector) {
        var parts = String(selector || "").split(",");
        var current = this;
        while (current && current.nodeType === 1) {
          for (var i = 0; i < parts.length; i++) {
            if (matchesOne(current, parts[i])) return current;
          }
          current = current.parentNode;
        }
        return null;
      },
      focus: function () { doc.activeElement = this; },
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
    };
    Object.defineProperty(node, "id", {
      get: function () { return node._id || ""; },
      set: function (value) {
        node._id = String(value);
        doc._byId[node._id] = node;
      },
    });
    Object.defineProperty(node, "outerHTML", {
      get: function () { return node._outerHTML || ""; },
      set: function (html) {
        node._outerHTML = String(html);
        var attrs = {};
        var re = /data-([\w-]+)="([^"]*)"/g;
        var match;
        while ((match = re.exec(node._outerHTML))) {
          attrs["data-" + match[1]] = decodeEntities(match[2]);
        }
        node.attributes = attrs;
        var cls = /class="([^"]*)"/.exec(node._outerHTML);
        if (cls) node.className = decodeEntities(cls[1]);
      },
    });
    return node;
  }

  doc.createElement = function (tag) { return makeElement(tag); };
  doc.getElementById = function (id) { return doc._byId[id] || null; };
  doc.querySelector = function (selector) { return queryAll(doc.body, selector)[0] || null; };
  doc.querySelectorAll = function (selector) { return queryAll(doc.body, selector); };
  doc.addEventListener = function (name, fn) {
    (doc._listeners[name] = doc._listeners[name] || []).push(fn);
  };
  doc.removeEventListener = function (name, fn) {
    var list = doc._listeners[name] || [];
    var i = list.indexOf(fn);
    if (i >= 0) list.splice(i, 1);
  };
  doc.body = makeElement("body");
  return {
    document: doc,
    makeElement: makeElement,
    textNode: textNode,
    matchesOne: matchesOne,
    fire: function (name, event) {
      (doc._listeners[name] || []).slice().forEach(function (fn) { fn(event); });
    },
  };
}

// ── Shared-sandbox loader ──────────────────────────────────────────────

function loadForms(options) {
  var opts = options || {};
  var dom = createDom();
  var windowStub = opts.window || {};
  if (typeof windowStub.gbT !== "function") {
    windowStub.gbT = function (key, fallback) { return fallback; };
  }
  if (opts.crypto === false) {
    windowStub.crypto = undefined;
  } else if (!windowStub.crypto) {
    windowStub.crypto = {
      randomUUID: function () { return "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"; },
    };
  }
  if (!windowStub.localStorage) {
    var store = {};
    windowStub.localStorage = {
      getItem: function (key) {
        return Object.prototype.hasOwnProperty.call(store, key) ? store[key] : null;
      },
      setItem: function (key, value) { store[key] = String(value); },
      removeItem: function (key) { delete store[key]; },
      clear: function () { store = {}; },
      _store: store,
    };
  }
  var active = { article: opts.article || null };
  var clock = { now: opts.now == null ? 1700000000000 : opts.now };
  var vmMath = Object.create(Math);
  vmMath.random = function () { return 0.25; };
  var sandbox = {
    window: windowStub,
    document: dom.document,
    console: console,
    getActiveArticle: function () { return active.article; },
    Date: { now: function () { return clock.now; } },
    Math: vmMath,
    JSON: JSON,
  };
  var context = vm.createContext(sandbox);
  FORMS_MODULES.forEach(function (rel) {
    vm.runInContext(readFile(rel), context, { filename: rel });
  });
  return {
    context: context,
    NS: windowStub.DocsForms,
    window: windowStub,
    doc: dom.document,
    dom: dom,
    active: active,
    clock: clock,
    setArticle: function (node) { active.article = node; },
    fire: dom.fire,
  };
}

function makeArticle(f, docId) {
  var article = f.doc.createElement("article");
  article.setAttribute("contenteditable", "true");
  if (docId) article.dataset.docId = docId;
  f.setArticle(article);
  f.doc.body.appendChild(article);
  return article;
}

function report() {
  log("\n" + "=".repeat(62));
  log("DOCS FORMS / PROTECTION OFFLINE RESULTS (#2427)");
  log("=".repeat(62));
  RESULTS.forEach(function (r) {
    log("[" + r.status + "] " + r.name + (r.error ? " -- " + r.error : ""));
  });
  log("=".repeat(62));
  log("TOTAL: " + (PASS + FAIL) + " | PASS: " + PASS + " | FAIL: " + FAIL);
  log("=".repeat(62));
  process.exitCode = FAIL > 0 ? 1 : 0;
}

function finish() { report(); }

module.exports = {
  fs,
  path,
  vm,
  SUITE_DIR,
  TESTS_DIR,
  FORMS_MODULES,
  test,
  assert,
  readFile,
  fileExists,
  createDom,
  loadForms,
  makeArticle,
  log,
  finish,
};
