/**
 * Browser-sweep pageerror regression guard (offline, no browser/server/cargo).
 *
 * Two exact root causes found by a real Chrome sweep, pinned at the source:
 *  (a) #2939 — the Attendant outside-click listener dereferenced
 *      document.getElementById("statusDropdown") after the window was closed;
 *      every later sidebar click threw "Cannot read properties of null
 *      (reading 'classList')". The lookup must be null-checked before the
 *      .classList.remove("show") call, with no behaviour change when present.
 *  (b) window-manager — registerHelp called resolveTarget(), which lived only
 *      inside 07_status_b_render.js and was never exported on
 *      window.__GBStatusInternal, so 07_status_c_events.js threw
 *      "ReferenceError: resolveTarget is not defined". Export it from the
 *      render module and import it in the events module; one definition only.
 *
 * GB_SWEEP86_PROBE_ROOT points at a /tmp tree holding mutated copies named
 * 01_init_queue.js / 07_status_b_render.js / 07_status_c_events.js, so a
 * mutation probe can prove the guard FAILS when either fix is removed.
 * The driver owns live UAT, issue comments/closure and deployment.
 */
"use strict";

var h = require("./00_harness");
var fs = require("fs");
var path = require("path");

var ATTENDANT = "botui/ui/suite/attendant/attendant-events/01_init_queue.js";
var RENDER = "botui/ui/suite/js/window-manager/07_status_b_render.js";
var EVENTS = "botui/ui/suite/js/window-manager/07_status_c_events.js";
var RUNNER = "botui/ui/suite/tests/webtest-office-fidelity.js";
var GUARD = "botui/ui/suite/tests/webtest-office-fidelity/86_sweep_null_dropdown_resolve.js";
var PROBE_ROOT = process.env.GB_SWEEP86_PROBE_ROOT || "";

function readSrc(rel) {
  var candidate = PROBE_ROOT ? path.join(PROBE_ROOT, path.basename(rel)) : "";
  if (candidate && fs.existsSync(candidate)) return fs.readFileSync(candidate, "utf8");
  return h.readRepo(rel);
}
function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1, (msg || "source must contain") + " <" + needle + ">");
}
function absent(src, needle, msg) {
  h.assert(src.indexOf(needle) === -1, (msg || "source must not contain") + " <" + needle + ">");
}
function countIn(src, needle) { return src.split(needle).length - 1; }
function span(src, startMarker, endMarker) {
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + ">");
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + ">");
  h.assert(end > start, "empty span between <" + startMarker + "> and <" + endMarker + ">");
  return src.slice(start, end);
}
/** Drop // comment lines so prose cannot satisfy or fail a code check. */
function codeOnly(src) {
  return src.split("\n").filter(function (line) {
    return line.trim().indexOf("//") !== 0;
  }).join("\n");
}

// ---------------------------------------------------------------------------
// (a) The Attendant outside-click handler is null-safe.
// ---------------------------------------------------------------------------
h.staticTest("#2939 the outside-click handler null-checks the dropdown before classList", function () {
  var src = codeOnly(readSrc(ATTENDANT));
  h.assertEqual(countIn(src, 'document.addEventListener("click",'), 1,
    "exactly one document-level click handler is installed");
  var handler = span(src, 'document.addEventListener("click", (e) => {', "\n                });");
  h.assert(handler.length > 200, "non-vacuity: click handler body must be substantial, got " + handler.length);

  h.assert(/\bdocument\s*\.\s*getElementById\(\s*"statusDropdown"\s*\)/.test(handler),
    "the dropdown is looked up by id inside the handler");
  h.assert(/\bif\s*\(\s*dropdown\s*\)\s*\{/.test(handler),
    "the lookup result is null-checked before any member access");
  h.assert(/\bdropdown\s*\.\s*classList\s*\.\s*remove\(\s*"show"\s*\)/.test(handler),
    "the existing hide behaviour is preserved when the element exists");

  var atLookup = handler.indexOf('getElementById("statusDropdown")');
  var atGuard = handler.indexOf("if (dropdown)");
  var atRemove = handler.indexOf('classList.remove("show")');
  h.assert(atLookup !== -1 && atGuard > atLookup && atRemove > atGuard,
    "order must be lookup → null-check → classList.remove (lookup=" + atLookup +
    " guard=" + atGuard + " remove=" + atRemove + ")");

  // Whitespace-stripped: the original unguarded lookup→classList chain must be gone.
  var flat = handler.replace(/\s/g, "");
  absent(flat, 'getElementById("statusDropdown").classList',
    "no direct getElementById(...).classList dereference remains");
  h.assertEqual(countIn(flat, 'getElementById("statusDropdown")'), 1,
    "the dropdown is looked up exactly once (no second unguarded lookup)");
});

// ---------------------------------------------------------------------------
// (b) resolveTarget has one definition, is exported, and is consumed once.
// ---------------------------------------------------------------------------
h.staticTest("resolveTarget is defined once and exported on __GBStatusInternal", function () {
  var src = codeOnly(readSrc(RENDER));
  h.assertEqual(countIn(src, "function resolveTarget(target) {"), 1,
    "exactly one resolveTarget definition, in the render module");
  h.assert(/\b_I\s*\.\s*resolveTarget\s*=\s*resolveTarget\s*;/.test(src),
    "resolveTarget is exported alongside the other status internals");
  var fn = span(src, "function resolveTarget(target) {", "\n  _I.renderStatus = renderStatus;");
  h.assert(fn.length > 250, "non-vacuity: resolveTarget body must be substantial, got " + fn.length);
  contains(fn, 'return document.querySelector(target);', "string selectors are still resolved");
  contains(fn, "if (target.nodeType === 1) return target;", "element targets are still accepted");
});

h.staticTest("the events module imports resolveTarget and never redefines it", function () {
  var src = codeOnly(readSrc(EVENTS));
  h.assert(/\bvar\s+resolveTarget\s*=\s*_I\s*\.\s*resolveTarget\s*;/.test(src),
    "the events module consumes the shared helper from __GBStatusInternal");
  h.assertEqual(countIn(src, "function resolveTarget("), 0,
    "no duplicate resolveTarget definition in the events module");
  var register = span(src, "function registerHelp(target, key, text) {", "function refreshHelp(root) {");
  h.assertEqual(countIn(register, "resolveTarget("), 1,
    "registerHelp calls the imported helper exactly once");
  contains(register, "var el = resolveTarget(target);", "registerHelp resolves its target first");
  contains(register, "if (!el) return null;", "an unresolved target is handled without throwing");
});

// ---------------------------------------------------------------------------
// Runtime proof (a): stubbed Attendant DOM, real shipped module in a vm.
// ---------------------------------------------------------------------------
function makeAttendant(dropdownExists) {
  var calls = { clickHandlers: [], chatListeners: [], removed: 0 };
  var dropdown = null;
  if (dropdownExists) {
    dropdown = {
      classList: {
        remove: function (name) { if (name === "show") calls.removed++; },
      },
    };
  }
  var chatInput = {
    style: {},
    addEventListener: function (type) { calls.chatListeners.push(type); },
  };
  var sandbox = {
    console: { error: function () {}, warn: function () {} },
    API_BASE: "",
    fetch: function () { return { then: function () { return this; }, catch: function () { return this; } }; },
    document: {
      readyState: "loading",
      addEventListener: function (type, fn) { if (type === "click") calls.clickHandlers.push(fn); },
      getElementById: function (id) {
        if (id === "chatInput") return chatInput;
        if (id === "statusDropdown") return dropdown;
        return null;
      },
    },
  };
  sandbox.window = sandbox;
  h.vm.createContext(sandbox);
  h.vm.runInContext(readSrc(ATTENDANT), sandbox, { filename: ATTENDANT });
  // readyState === "loading" defers the module's own init; drive the real
  // listener installer directly so only this handler is under test.
  sandbox.setupEventListeners();
  h.assertEqual(calls.clickHandlers.length, 1, "one document click handler installed");
  return { sandbox: sandbox, calls: calls };
}
function click(sandbox, calls, isInside) {
  var event = {
    target: {
      closest: function (selector) {
        h.assertEqual(selector, "#attendantStatus", "the handler probes the status anchor");
        return isInside ? { id: "attendantStatus" } : null;
      },
    },
  };
  calls.clickHandlers.forEach(function (fn) { fn(event); });
}

h.test("outside click with the dropdown absent does not throw and touches nothing", function () {
  var t = makeAttendant(false);
  click(t.sandbox, t.calls, false); // throws with the pre-fix direct dereference
  h.assertEqual(t.calls.removed, 0, "no classList call when the dropdown is missing");
});

h.test("outside click still hides the dropdown when it exists", function () {
  var t = makeAttendant(true);
  click(t.sandbox, t.calls, false);
  h.assertEqual(t.calls.removed, 1, "classList.remove('show') still runs when present");
});

h.test("a click inside the status anchor leaves the dropdown untouched", function () {
  var t = makeAttendant(true);
  click(t.sandbox, t.calls, true);
  h.assertEqual(t.calls.removed, 0, "the anchor click keeps the dropdown open");
});

// ---------------------------------------------------------------------------
// Runtime proof (b): both real status modules share one __GBStatusInternal.
// ---------------------------------------------------------------------------
function makeEl() {
  return {
    tagName: "BUTTON", attrs: {}, nodeType: 1,
    setAttribute: function (k, v) { this.attrs[k] = v; },
    getAttribute: function (k) { return this.attrs[k] === undefined ? null : this.attrs[k]; },
    hasAttribute: function (k) { return this.attrs[k] !== undefined; },
  };
}
function makeStatus() {
  var internal = {};
  var docListeners = {};
  var selectorTargets = {};
  var translate = function (key, fallback) { return fallback || key; };
  var sandbox = {
    console: { warn: function () {}, error: function () {} },
    Math: Math, JSON: JSON, Array: Array, Object: Object, String: String, Date: Date,
    document: {
      readyState: "loading",
      addEventListener: function (type, fn) { (docListeners[type] = docListeners[type] || []).push(fn); },
      dispatchEvent: function () {},
      querySelector: function (sel) { return selectorTargets[sel] || null; },
    },
  };
  sandbox.window = sandbox;
  sandbox.__GBStatusInternal = internal;
  sandbox.WM = {
    APPS_REGISTRY: [],
    defineMethods: function (proto, methods) { sandbox.definedMethods = methods; },
  };
  sandbox.WindowManager = { prototype: {}, activeWindowId: function () { return ""; }, openWindows: [] };
  sandbox.APPS_REGISTRY = [];
  internal.WM = sandbox.WM;
  internal.WindowManager = sandbox.WindowManager;
  internal.store = {};
  internal.listeners = {};
  internal.helpTexts = {};
  internal.translate = translate;
  internal.escapeHtml = function (value) { return String(value); };
  internal.appTitle = function () { return ""; };
  internal.appDescription = function () { return ""; };
  internal.normalizeState = function () {};
  internal.defaultPayload = function () {
    return { state: "ready", text: "", detail: "", helpKey: "", metrics: [], selection: null, connection: null };
  };
  internal.buildPayload = function (id, patch) {
    return Object.assign({}, internal.defaultPayload(id), patch || {});
  };
  internal.statusBarFor = function () { return null; };
  internal.applyHelp = function () {};
  internal.renderSlot = function () { return null; };
  h.vm.createContext(sandbox);
  h.vm.runInContext(readSrc(RENDER), sandbox, { filename: RENDER });
  h.vm.runInContext(readSrc(EVENTS), sandbox, { filename: EVENTS });
  return {
    sandbox: sandbox, internal: internal, docListeners: docListeners,
    selectorTargets: selectorTargets, helpTexts: internal.helpTexts,
  };
}

h.test("registerHelp resolves an element target through the exported helper", function () {
  var t = makeStatus();
  var el = makeEl();
  var returned = t.sandbox.GBStatus.registerHelp(el, "browser-url-help", "Enter a URL");
  h.assertEqual(returned, el, "the element target is returned unchanged");
  h.assertEqual(el.getAttribute("data-help"), "browser-url-help", "the help key is recorded");
  h.assertEqual(el.getAttribute("data-tooltip"), "Enter a URL", "the fallback is the tooltip text");
  h.assertEqual(t.helpTexts["browser-url-help"], "Enter a URL", "the help catalogue is populated");
});

h.test("registerHelp resolves selector strings and tolerates an unmatched one", function () {
  var t = makeStatus();
  var el = makeEl();
  t.selectorTargets["#toolbar"] = el;
  var returned = t.sandbox.GBStatus.registerHelp("#toolbar", "toolbar-help", "Toolbar tip");
  h.assertEqual(returned, el, "a selector string resolves to the shared element");
  h.assertEqual(el.getAttribute("title"), "Toolbar tip", "title is set from the resolved text");
  h.assertEqual(t.sandbox.GBStatus.registerHelp("#missing", "absent-help", "No target"), null,
    "an unmatched selector returns null instead of throwing");
});

h.test("the WindowManager bridge reaches the same resolveTarget path", function () {
  var t = makeStatus();
  h.assert(t.sandbox.definedMethods && typeof t.sandbox.definedMethods.registerStatusHelp === "function",
    "registerStatusHelp bridge is installed");
  var el = makeEl();
  var returned = t.sandbox.definedMethods.registerStatusHelp(el, "bridge-help", "Bridge tip");
  h.assertEqual(returned, el, "the bridge returns the resolved element");
  h.assertEqual(el.getAttribute("data-tooltip"), "Bridge tip", "the bridge sets the same tooltip");
});

// ---------------------------------------------------------------------------
// Guard hygiene and mutation non-vacuity.
// ---------------------------------------------------------------------------
h.staticTest("guard 86 is registered and its shipped sources stay focused", function () {
  [ATTENDANT, RENDER, EVENTS].forEach(function (rel) {
    var src = readSrc(rel);
    h.assert(src.split("\n").length < 450, rel + " must stay under 450 lines");
    h.assert(src.length > 500, rel + " must remain a shipped module");
  });
  var guardSrc = h.readRepo(GUARD);
  h.assert(guardSrc.split("\n").length < 450, "guard must stay under 450 lines");
  contains(h.readRepo(RUNNER), 'require("./webtest-office-fidelity/86_sweep_null_dropdown_resolve");',
    "guard 86 must be registered in the fidelity runner");
  var hooks = guardSrc.match(/=\s*process\.env\.GB_SWEEP86_PROBE_ROOT/g) || [];
  h.assertEqual(hooks.length, 1,
    "the guard exposes exactly one probe-root hook for mutation testing");
});

h.note("#sweep86: offline static + stubbed-vm proof only — no browser/server/cargo, no locale edit. Driver owns the live Chrome UAT and issue closure.");
