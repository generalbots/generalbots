/**
 * #2957 — offline regression guard for the Browser toolbar open path.
 *
 * The Chrome sweep captured (botserver.log, 2026-11-26T07:13:27Z):
 *
 *   ReferenceError: resolveTarget is not defined
 *     at Object.registerHelp (.../window-manager/07_status_c_events.js?v=133:12:14)
 *     at browser-inline.js:415:37     <- the hover-help loop on normal open
 *     at Array.forEach (<anonymous>)
 *     at browser-inline.js:413:11
 *     at browser-inline.js:438:3      <- IIFE boot
 *
 * browser-inline.js publishes hover help for every toolbar control during a
 * normal open. registerHelp resolved its target through a bare `resolveTarget`
 * binding: the helper lived only inside 07_status_b_render.js and was never
 * exported on window.__GBStatusInternal, so 07_status_c_events.js threw on the
 * first control. #2939 landed the export/import pair; this guard proves that
 * exact toolbar path now completes, and that removing the import reproduces
 * the original ReferenceError (mutation probe, non-vacuous).
 *
 * The second half of #2957 — GET /api/browser/vm/<id>/tabs 401 — was fixed by
 * #2967. This module VERIFIES that fix is still present; it does not re-fix it.
 *
 * No server, browser, cargo or i18n change is involved; the driver owns live
 * UAT, issue closure and deployment.
 */
"use strict";

var h = require("./00_harness");

var BROWSER = "botui/ui/suite/browser/browser-inline.js";
var CORE = "botui/ui/suite/js/window-manager/07_status_a_core.js";
var RENDER = "botui/ui/suite/js/window-manager/07_status_b_render.js";
var EVENTS = "botui/ui/suite/js/window-manager/07_status_c_events.js";
var VM_TABS = "botui/ui/suite/browser/vm-tabs.js";
var DESKTOP_VM_TABS = "botui/ui/suite/desktop/vm-tabs.js";
var SHELL = "botui/ui/suite/desktop.html";

function src(rel) { return h.readRepo(rel); }
function count(text, needle) { return text.split(needle).length - 1; }
function contains(text, needle, msg) {
  h.assert(text.indexOf(needle) !== -1, (msg || "source must contain") + " <" + needle + ">");
}
function absent(text, needle, msg) {
  h.assert(text.indexOf(needle) === -1, (msg || "source must not contain") + " <" + needle + ">");
}

// ---------------------------------------------------------------------------
// Static: the toolbar path really goes through the shared help contract.
// ---------------------------------------------------------------------------
h.staticTest("#2957 browser toolbar publishes hover help through GBStatus on open", function () {
  var text = src(BROWSER);
  h.assert(text.length > 12000, "non-vacuity: browser-inline.js is the shipped module, not a stub");
  contains(text, "if (window.GBStatus && typeof window.GBStatus.registerHelp === 'function') {",
    "the hover-help loop is gated on the shared status contract");
  contains(text, "if (el) window.GBStatus.registerHelp(el, item[1]);",
    "each toolbar control is registered through GBStatus.registerHelp");
  contains(text, "['btn-refresh', 'Reload the current page']",
    "the Refresh control from the reproduction steps is covered");
  contains(text, "['browser-url', 'Enter a URL or search the web']",
    "the address bar (navigation/resolve control) is covered");
  h.assert(count(text, "window.GBStatus.registerHelp(el, item[1]);") === 1,
    "the toolbar uses one canonical registerHelp call");
});

h.staticTest("#2957 resolveTarget is exported once and consumed by the events module", function () {
  var render = src(RENDER);
  var events = src(EVENTS);
  h.assertEqual(count(render, "function resolveTarget(target) {"), 1,
    "exactly one resolveTarget definition, in the render module");
  contains(render, "_I.resolveTarget = resolveTarget;",
    "the helper is exported on __GBStatusInternal");
  h.assert(/\bvar\s+resolveTarget\s*=\s*_I\s*\.\s*resolveTarget\s*;/.test(events),
    "the events module imports the shared helper (the pre-fix bare binding is gone)");
  h.assertEqual(count(events, "function resolveTarget("), 0,
    "no duplicate resolveTarget definition in the events module");
  var start = events.indexOf("function registerHelp(target, key, text) {");
  var body = events.slice(start, events.indexOf("function refreshHelp(root) {", start));
  contains(body, "var el = resolveTarget(target);",
    "registerHelp resolves its target through the imported binding");
  contains(body, "if (!el) return null;", "an unresolved target fails closed");
});

h.staticTest("#2957 the desktop shell loads status modules before the Browser fragment", function () {
  var shell = src(SHELL);
  var core = shell.indexOf("07_status_a_core.js");
  var render = shell.indexOf("07_status_b_render.js");
  var events = shell.indexOf("07_status_c_events.js");
  h.assert(core !== -1 && render > core && events > render,
    "status a_core -> b_render -> c_events load in dependency order");
});

// ---------------------------------------------------------------------------
// Static: verify (do not re-fix) the #2967 tabs-poller 401 gates.
// ---------------------------------------------------------------------------
h.staticTest("#2957 the #2967 VM tabs 401 fix is still present", function () {
  var app = src(VM_TABS);
  var desktop = src(DESKTOP_VM_TABS);
  contains(app, "function hasSession()", "app poller uses the canonical session predicate");
  contains(app, "function browserAppActive()", "app poller re-checks its owning window");
  contains(app, "if (!hasSession()) return Promise.resolve(null);", "no session means no tabs fetch");
  absent(app, "return DEFAULT_VM;", "the old unconditional 'default' VM fallback is gone");
  absent(app, "setInterval(loadTabs, POLL_MS)", "no raw interval survives window close");
  contains(app, "GBAppLifecycle.interval", "the poller is lifecycle-owned");
  contains(desktop, "function hasSession()", "desktop twin uses the same session predicate");
  contains(desktop, "!hasSession()) return Promise.resolve(null);",
    "desktop twin drops unauthenticated tabs fetches");
});

// ---------------------------------------------------------------------------
// Runtime: the real modules in a stubbed DOM; invoke the toolbar open path.
// ---------------------------------------------------------------------------
function element(id) {
  return {
    id: id,
    nodeType: 1,
    value: "",
    textContent: "",
    innerHTML: "",
    style: {},
    dataset: {},
    attrs: {},
    setAttribute: function (k, v) { this.attrs[k] = String(v); },
    getAttribute: function (k) {
      return Object.prototype.hasOwnProperty.call(this.attrs, k) ? this.attrs[k] : null;
    },
    hasAttribute: function (k) { return Object.prototype.hasOwnProperty.call(this.attrs, k); },
    removeAttribute: function (k) { delete this.attrs[k]; },
    addEventListener: function () {},
    querySelector: function () { return null; },
    querySelectorAll: function () { return []; },
    appendChild: function () {},
    remove: function () {},
  };
}

var TOOLBAR_IDS = [
  "btn-back", "btn-forward", "btn-refresh", "btn-home", "browser-url",
  "btn-go", "btn-screenshot", "btn-bookmark", "btn-agentic",
];

function makeSandbox(eventsSource) {
  var nodes = {};
  TOOLBAR_IDS.concat([
    "browser-viewport", "browser-tabs-bar", "browser-status-text",
    "browser-loading-indicator", "browser-page-info", "browser-security-info",
    "protocol-icon", "browser-search", "browser-search-input",
  ]).forEach(function (id) { nodes[id] = element(id); });
  // The first tab's page must exist for updateTabs() at boot.
  var firstPage = element("browser-page-0");
  nodes["browser-viewport"].querySelector = function () { return firstPage; };

  var documentStub = {
    readyState: "complete",
    hidden: false,
    getElementById: function (id) { return nodes[id] || null; },
    querySelector: function () { return null; },
    querySelectorAll: function () { return []; },
    createElement: function () { return element("created"); },
    addEventListener: function () {},
    dispatchEvent: function () { return true; },
  };

  function WindowManager() {}
  WindowManager.prototype = {};
  WindowManager.prototype.activeWindowId = "browser";
  WindowManager.openWindows = [];

  var sandbox = {
    console: { log: function () {}, error: function () {}, warn: function () {} },
    document: documentStub,
    location: { search: "", protocol: "http:", host: "localhost:3000" },
    URL: URL,
    URLSearchParams: URLSearchParams,
    CustomEvent: function () {},
    fetch: function () { return Promise.resolve({ ok: false, status: 401 }); },
    setInterval: function () { return 0; },
    clearInterval: function () {},
    setTimeout: function () { return 0; },
    clearTimeout: function () {},
    localStorage: { getItem: function () { return null; }, setItem: function () {} },
    sessionStorage: { getItem: function () { return null; }, setItem: function () {} },
    navigator: { mediaDevices: null },
    WebSocket: function () {},
    APPS_REGISTRY: [],
    WindowManager: WindowManager,
    __gbAppParams__: {},
    getGBAccessToken: function () { return null; },
    gbStoreGet: function (_key, fallback) { return fallback; },
  };
  sandbox.window = sandbox;
  sandbox.__GBWindowManager = {
    WindowManager: WindowManager,
    APPS_REGISTRY: [],
    defineMethods: function (proto, methods) {
      Object.keys(methods).forEach(function (k) { proto[k] = methods[k]; });
    },
  };

  var context = h.vm.createContext(sandbox);
  h.vm.runInContext(src(CORE), context, { filename: CORE });
  h.vm.runInContext(src(RENDER), context, { filename: RENDER });
  h.vm.runInContext(eventsSource, context, { filename: EVENTS });
  return { context: context, sandbox: sandbox, nodes: nodes };
}

h.test("#2957 Browser toolbar open completes with no resolveTarget ReferenceError", function () {
  var setup = makeSandbox(src(EVENTS));
  h.assertEqual(typeof setup.sandbox.GBStatus, "object", "the shared status contract is installed");
  h.assertEqual(typeof setup.sandbox.GBStatus.registerHelp, "function",
    "GBStatus.registerHelp is available to the toolbar");
  h.assertEqual(typeof setup.sandbox.__GBStatusInternal.resolveTarget, "function",
    "the helper is exported before the Browser fragment runs");

  // This is the exact line that threw in the sweep: browser-inline.js boot
  // iterates the toolbar help list and calls GBStatus.registerHelp per control.
  h.vm.runInContext(src(BROWSER), setup.context, { filename: BROWSER });

  TOOLBAR_IDS.forEach(function (id) {
    h.assert(setup.nodes[id].hasAttribute("data-help"),
      "toolbar control #" + id + " received hover help without throwing");
  });
  h.assert(setup.nodes["btn-refresh"].getAttribute("data-help").length > 0,
    "Refresh control help text is non-empty");
});

h.test("#2957 mutation probe: removing the import reproduces the exact ReferenceError", function () {
  var events = src(EVENTS);
  var marker = "  var resolveTarget = _I.resolveTarget;";
  h.assert(events.indexOf(marker) !== -1, "the import line exists and can be mutated");
  var mutated = events.replace(marker, "");
  h.assert(mutated !== events && mutated.indexOf(marker) === -1,
    "mutation removed the import from the events module");

  var setup = makeSandbox(mutated);
  h.assertEqual(typeof setup.sandbox.GBStatus.registerHelp, "function",
    "the mutated module still installs registerHelp");
  h.assertThrows(function () {
    h.vm.runInContext(src(BROWSER), setup.context, { filename: BROWSER });
  }, /resolveTarget is not defined/,
    "the original bare binding throws the exact swept ReferenceError on toolbar open");
});
