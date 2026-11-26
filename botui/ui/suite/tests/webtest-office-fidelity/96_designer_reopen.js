/**
 * #2956 — Designer produced three uncaught exceptions when the app window was
 * re-opened (its classic scripts are re-evaluated by HTMX on every open):
 *   SyntaxError: Identifier 'state' has already been declared
 *   ReferenceError: initDesigner is not defined
 *   TypeError: Cannot read properties of undefined (reading 'size')
 *
 * This guard pins the shipped fix:
 *   1. shared state/templates live on uniquely named, guarded window objects
 *      (`GBDesignerState`, `GBDesignerTemplates`) — no top-level const/let;
 *   2. `initDesigner` is published by its owning script and the bootstrap
 *      calls the published function, never a bare identifier;
 *   3. node counts go through `gbDesignerNodeCount()` which tolerates a
 *      missing nodes collection;
 *   4. document listeners are bound once per window.
 *
 * The RUNTIME case evaluates all four shipped scripts TWICE in ONE vm context
 * with a small DOM stub — that is the double-injection reproducer — and checks
 * state identity, listener counts and that initDesigner still runs.
 *
 * No server, browser, cargo or commit is involved; the driver owns closure.
 */
"use strict";

var h = require("./00_harness");
var fs = require("fs");
var path = require("path");

var STATE = "botui/ui/suite/designer-inline-state.js";
var NODES = "botui/ui/suite/designer-inline-nodes.js";
var CONTEXT = "botui/ui/suite/designer-inline-context-menu.js";
var SEARCH = "botui/ui/suite/designer-inline-search.js";
var HTML = "botui/ui/suite/designer.html";
var RUNNER = "botui/ui/suite/tests/webtest-office-fidelity.js";
var PROBE_ROOT = process.env.GB_DESIGNER_REOPEN_PROBE_ROOT || "";
var FILES = [STATE, NODES, CONTEXT, SEARCH];

function read(rel) {
  var candidate = PROBE_ROOT ? path.join(PROBE_ROOT, path.basename(rel)) : "";
  if (candidate && fs.existsSync(candidate)) return fs.readFileSync(candidate, "utf8");
  return h.readRepo(rel);
}
function codeOnly(src) {
  return src.replace(/\/\*[\s\S]*?\*\//g, "")
    .split("\n").filter(function (line) { return line.trim().indexOf("//") !== 0; })
    .join("\n");
}
function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1, (msg || "source must contain") + " <" + needle + ">");
}
function absent(src, needle, msg) {
  h.assert(src.indexOf(needle) === -1, (msg || "source must not contain") + " <" + needle + ">");
}
function count(src, needle) { return src.split(needle).length - 1; }
function lines(src) { return src.split("\n").length; }

/** Minimal DOM sufficient for the four scripts' load-time + init paths. */
function stubElement() {
  return {
    style: {}, dataset: {}, textContent: "", value: "", innerHTML: "",
    addEventListener: function () {}, removeEventListener: function () {},
    querySelectorAll: function () { return []; }, querySelector: function () { return null; },
    getBoundingClientRect: function () { return { left: 0, top: 0 }; },
    classList: { add: function () {}, remove: function () {}, toggle: function () {} },
    appendChild: function () {}, remove: function () {}, closest: function () { return null; },
    setPointerCapture: function () {}, releasePointerCapture: function () {}
  };
}

function makeHost() {
  var listeners = { body: {}, document: {} };
  function bump(bucket, type) { listeners[bucket][type] = (listeners[bucket][type] || 0) + 1; }
  var sandbox = {
    console: { warn: function () {}, error: function () {}, log: function () {} },
    URLSearchParams: URLSearchParams,
    setTimeout: function () { return 0; },
    fetch: function () { return Promise.resolve({ ok: false }); },
    gbLog: { debug: function () {} },
    location: { search: "", hash: "", href: "" },
    document: {
      readyState: "loading",
      currentScript: null,
      location: { search: "", hash: "", href: "" },
      body: { addEventListener: function (type) { bump("body", type); } },
      addEventListener: function (type) { bump("document", type); },
      getElementById: function () { return stubElement(); },
      querySelector: function () { return null; },
      querySelectorAll: function () { return []; },
      createElement: function () { return stubElement(); }
    }
  };
  sandbox.window = sandbox;
  return { sandbox: sandbox, listeners: listeners };
}

function evaluateAll(context, rels) {
  rels.forEach(function (rel) {
    h.vm.runInContext(read(rel), context, { filename: rel });
  });
}

h.staticTest("#2956 no top-level const/let can collide on Designer re-injection", function () {
  FILES.forEach(function (rel) {
    var src = codeOnly(read(rel));
    h.assert(lines(src) > 100, "non-vacuity: " + rel + " is no longer a shipped module");
    h.assert(!/^(?:const|let)\s/m.test(src), rel + " still declares a top-level const/let");
    absent(src, "const GBDesignerState", rel + " redeclares the state namespace");
    absent(src, "const state", rel + " redeclares a global state");
  });
  var state = codeOnly(read(STATE));
  contains(state, "if (!window.GBDesignerState) {", "state namespace creation is guarded");
  contains(state, "var GBDesignerState = window.GBDesignerState;", "alias is a re-declarable var");
  contains(state, "if (!window.GBDesignerTemplates) {", "node template creation is guarded");
  contains(state, "var GBDesignerTemplates = window.GBDesignerTemplates;", "template alias is a var");
});

h.staticTest("#2956 initDesigner is published by its owner and called via window", function () {
  var state = codeOnly(read(STATE));
  var search = codeOnly(read(SEARCH));
  contains(state, "function initDesigner()", "initializer is defined in the state module");
  contains(state, "window.initDesigner = initDesigner;", "owner publishes the initializer");
  var definedAt = state.indexOf("function initDesigner()");
  var publishedAt = state.indexOf("window.initDesigner = initDesigner;");
  h.assert(definedAt !== -1 && publishedAt > definedAt, "publication follows definition");
  contains(search, "function gbDesignerBoot()", "bootstrap uses a guarded wrapper");
  contains(search, "typeof window.initDesigner === 'function'", "bootstrap gates on publication");
  contains(search, "window.initDesigner();", "bootstrap calls the published function");
  absent(search, "DOMContentLoaded, initDesigner", "no bare identifier on DOMContentLoaded");
  absent(search, "\n            initDesigner();", "no bare identifier in the ready branch");
  contains(search, "__gbDesignerDomReadyBound", "DOM-ready listener is bound once");
});

h.staticTest("#2956 every .size read is guarded and listeners bind once", function () {
  var state = codeOnly(read(STATE));
  var context = codeOnly(read(CONTEXT));
  contains(state, "function gbDesignerNodeCount()", "a guarded node-count accessor exists");
  contains(state, "window.gbDesignerNodeCount = gbDesignerNodeCount;", "accessor is published");
  contains(state, "window.GBDesignerState.nodes.size", "accessor reads size off the namespace");
  absent(context, "GBDesignerState.nodes.size", "no bare .size dereference in the UI module");
  contains(context, "window.gbDesignerNodeCount()", "status bar uses the guarded accessor");
  contains(context, "__gbDesignerKeysBound", "document keydown listener is bound once");
  contains(state, "__gbDesignerSwapBound", "htmx:afterSwap listener is bound once");
});

h.staticTest("#2956 shipped wiring and 450-line ceiling", function () {
  var html = read(HTML);
  var order = [STATE, NODES, CONTEXT, SEARCH].map(function (rel) {
    return "/suite/" + path.basename(rel);
  });
  var at = order.map(function (url) { return html.indexOf(url); });
  h.assert(at[0] !== -1 && at[0] < at[1] && at[1] < at[2] && at[2] < at[3],
    "designer.html must load state → nodes → context → search in order");
  order.forEach(function (url) {
    h.assert(count(html, url) === 1, url + " must be injected exactly once");
  });
  FILES.forEach(function (rel) {
    var src = read(rel);
    h.assert(lines(src) <= 450, rel + " exceeds the 450-line ceiling");
    h.assert(!/\bhttps?:\/\//i.test(src), rel + " must not reference an external CDN");
  });
  contains(h.readRepo(RUNNER), 'require("./webtest-office-fidelity/96_designer_reopen")',
    "this guard must be registered in the fidelity runner");
});

h.test("#2956 double script injection in one window raises no exception", function () {
  var host = makeHost();
  var context = h.vm.createContext(host.sandbox);
  evaluateAll(context, FILES); // first open
  h.assert(typeof host.sandbox.initDesigner === "function", "initDesigner published on first open");
  h.assert(host.sandbox.GBDesignerState && host.sandbox.GBDesignerState.nodes &&
    typeof host.sandbox.GBDesignerState.nodes.set === "function",
    "state namespace created on first open");
  var firstState = host.sandbox.GBDesignerState;
  host.sandbox.GBDesignerState.nodes.set("node-1", { id: "node-1" });

  evaluateAll(context, FILES); // re-open — used to throw the duplicate declaration
  h.assert(host.sandbox.GBDesignerState === firstState, "re-open must reuse the live state object");
  h.assert(host.sandbox.GBDesignerState.nodes.size === 1, "re-open must not clear live nodes");
  h.assertEqual(typeof host.sandbox.initDesigner, "function", "initDesigner survives re-open");
  h.assertEqual(host.listeners.body["htmx:afterSwap"], 1, "swap listener bound once");
  h.assertEqual(host.listeners.document.keydown, 1, "keydown listener bound once");
  h.assertEqual(host.listeners.document.DOMContentLoaded, 1, "DOM-ready listener bound once");
});

h.test("#2956 init runs after re-open and .size deref is null-safe", function () {
  var host = makeHost();
  var context = h.vm.createContext(host.sandbox);
  evaluateAll(context, FILES);
  evaluateAll(context, FILES);
  host.sandbox.initDesigner(); // must be callable, with no missing-function exception
  host.sandbox.GBDesignerState.nodes = undefined; // simulate a partially built state
  h.assertEqual(host.sandbox.gbDesignerNodeCount(), 0, "node count is safe without a nodes collection");
  vmCall(context, "updateStatusBar"); // must not throw on the undefined collection
});

function vmCall(context, name) {
  h.vm.runInContext(name + "();", context, { filename: "self-test" });
}
