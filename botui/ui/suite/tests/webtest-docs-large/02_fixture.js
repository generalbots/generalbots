/**
 * Deterministic sandbox factory for #2533 shipped docs-large IIFEs.
 *
 * Modules 84..91 are loaded UNMODIFIED into one shared vm context. Stubs cover
 * window/document/gbT/localStorage plus a fixed clock and deterministic uid.
 * No production logic is reimplemented: fixtures only feed the real helpers and
 * observe the values they return.
 */
"use strict";

var h = require("./00_harness");
var dom = require("./01_dom");

var MODULES = [
  "docs/modules/84_large_state.js",
  "docs/modules/85_large_fixture.js",
  "docs/modules/86_large_window.js",
  "docs/modules/87_large_save.js",
  "docs/modules/88_large_degradation.js",
  "docs/modules/89_large_baseline.js",
  "docs/modules/90_large_inspector.js",
  "docs/modules/91_large_init.js",
];

function FixedDate() {}
FixedDate.now = function () { return 1700000000000; };
FixedDate.prototype.toISOString = function () { return "2023-11-14T22:13:20.000Z"; };

function fixedMath() {
  var math = {};
  Object.getOwnPropertyNames(Math).forEach(function (name) { math[name] = Math[name]; });
  math.random = function () { return 0.5; };
  return math;
}

function pad8(n) {
  var text = String(n);
  while (text.length < 8) text = "0" + text;
  return text;
}

/** Fresh sandbox: state, storage, clock and uid are deterministic per fixture. */
function fixture() {
  var store = {};
  var doc = dom.makeDocument();
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
        clear: function () { Object.keys(store).forEach(function (key) { delete store[key]; }); },
      },
      crypto: {
        randomUUID: function () {
          uuidCounter += 1;
          return pad8(uuidCounter) + "-0000-4000-8000-000000000000";
        },
      },
    },
    document: doc,
    Date: FixedDate,
    Math: fixedMath(),
    console: console,
    setTimeout: function () { return 0; },
    clearTimeout: function () {},
  };
  var context = h.loadModules(MODULES, sandbox);
  return {
    NS: context.window.DocsLarge,
    sandbox: sandbox,
    store: store,
    doc: doc,
    ctx: context,
  };
}

/** Register an empty contenteditable article with a stable document id. */
function article(fx, docId) {
  var node = fx.doc.createElement("article");
  node.setAttribute("contenteditable", "true");
  node.dataset.docId = docId || "<a>";
  fx.doc.body.appendChild(node);
  return node;
}

function registerNode(fx, id, node) {
  fx.doc._nodes[id] = node;
  return node;
}

/** Register a #docTitle input (used by the shipped docId resolver). */
function titleInput(fx, value) {
  var node = fx.doc.createElement("input");
  node.setAttribute("id", "docTitle");
  node.value = value;
  return registerNode(fx, "docTitle", node);
}

/** Fire the deferred boot hook so module 91 wires its delegated listeners. */
function boot(fx) {
  fx.doc.dispatch("DOMContentLoaded", {});
}

function i18nKey(node) {
  return node.getAttribute("data-i18n");
}

module.exports = {
  MODULES,
  fixture,
  article,
  registerNode,
  titleInput,
  boot,
  i18nKey,
  FixedDate,
  fixedMath,
};
