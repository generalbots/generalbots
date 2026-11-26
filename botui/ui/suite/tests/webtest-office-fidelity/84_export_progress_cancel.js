/**
 * #2459 criterion 3 — slow exports show progress and can be cancelled.
 *
 * Offline proof only: static pins on the two shipped Docs export sources plus
 * a stubbed-DOM vm run of the real export menu IIFE (fast path paints nothing,
 * slow path paints a polite live region with Cancel, Cancel aborts and restores
 * idle). No server, browser, cargo or locale edit. The driver owns live UAT.
 *
 * GB_EXPORT_PROGRESS_PROBE_ROOT points at a /tmp tree holding mutated copies
 * named 02b_export.js / docs-export-helpers.js, so a mutation probe can prove
 * the guard fails when Cancel or progress is removed.
 */
"use strict";

var h = require("./00_harness");
var fs = require("fs");
var path = require("path");

var MENU = "botui/ui/suite/docs/modules/02b_export.js";
var HELPERS = "botui/ui/suite/docs/docs-export-helpers.js";
var RUNNER = "botui/ui/suite/tests/webtest-office-fidelity.js";
var LOCALES = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];
var KEYS = ["status-in-progress", "action-cancel"];
var PROBE_ROOT = process.env.GB_EXPORT_PROGRESS_PROBE_ROOT || "";

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
function span(src, startMarker, endMarker) {
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + ">");
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + ">");
  h.assert(end > start, "empty span between <" + startMarker + "> and <" + endMarker + ">");
  return src.slice(start, end);
}

h.staticTest("export status is a polite live region and progress is delayed", function () {
  var menu = readSrc(MENU);
  var region = span(menu, "function statusRegion()", "function clearStatus()");
  h.assert(region.length > 140, "non-vacuity: status region body must be substantial, got " + region.length);
  contains(region, 'setAttribute("role", "status")', "progress announces via a status region");
  contains(region, 'setAttribute("aria-live", "polite")', "polite live region");
  contains(region, 'setAttribute("aria-atomic", "true")', "atomic announcement");
  contains(menu, "PROGRESS_DELAY_MS = 200", "a real delay constant exists");
  contains(menu, "setTimeout(showProgress, PROGRESS_DELAY_MS)", "progress is delayed, not instant");
  contains(menu, 'docsT("status-in-progress", "In Progress")', "existing progress key is reused");
  contains(menu, 'docsT("action-cancel", "Cancel")', "existing cancel key is reused");
  var show = span(menu, "function showProgress()", "function clearProgress()");
  contains(show, 'querySelector(".docs-export-progress-text")', "progress text is a child node");
  contains(show, "progressControls();", "the Cancel control is installed with progress");
});

h.staticTest("Cancel aborts the in-flight fetch and every settle restores idle", function () {
  var menu = readSrc(MENU);
  var clear = span(menu, "function clearProgress()", "function cancelExport()");
  contains(clear, "clearTimeout(progressTimer)", "the delayed paint is cancelled");
  contains(clear, "progressTimer = null", "timer state cleared");
  contains(clear, "exportController = null", "controller state cleared");
  contains(clear, "clearStatus();", "live region returned to idle via the existing writer");
  var cancel = span(menu, "function cancelExport()", "function close()");
  contains(cancel, "controller.abort();", "Cancel aborts the request");
  contains(cancel, "clearProgress();", "Cancel restores idle immediately");
  var exportFn = span(menu, "function exportAs(format)", "function open(anchor)");
  contains(exportFn, "cancelExport();", "a superseded export is aborted first");
  contains(exportFn, "signal: controller ? controller.signal : null", "signal reaches the exporter");
  contains(exportFn, "onStart: function ()", "start hook arms progress");
  contains(exportFn, "onSettle: function (outcome)", "settle hook clears progress");
  contains(exportFn, "clearProgress();", "settle clears progress");
  contains(exportFn, 'outcome !== "cancelled"', "a cancelled export announces no degradation");
});

h.staticTest("GBDocsExport forwards the signal and never falls back after abort", function () {
  var helpers = readSrc(HELPERS);
  var fn = span(helpers, "function GBDocsExport(format, id, opts) {", "window.GBDocsExport = GBDocsExport;");
  h.assert(fn.length > 1500, "non-vacuity: exporter body must be substantial, got " + fn.length);
  contains(fn, "var options = opts || {};", "third argument is optional (fast path preserved)");
  contains(fn, "fetchOptions.signal = options.signal", "signal is passed to fetch");
  contains(fn, "options.onStart", "start hook is invoked");
  contains(fn, 'settle("success")', "success settles");
  contains(fn, "options.signal && options.signal.aborted", "abort is distinguished from transport failure");
  contains(fn, 'settle("cancelled")', "abort settles as cancelled");
  contains(fn, 'settle("fallback")', "local-save fallback settles too");
  var atAbort = fn.indexOf("options.signal && options.signal.aborted");
  var atFallback = fn.indexOf("var article = document.querySelector");
  h.assert(atAbort !== -1 && atFallback > atAbort,
    "abort gate must run before any local-save fallback");
});

h.staticTest("the two reused keys are catalogued in all eight locales", function () {
  KEYS.forEach(function (key) {
    LOCALES.forEach(function (locale) {
      var dir = path.join(h.REPO_DIR, "botlib", "locales", locale);
      var all = h.fs.readdirSync(dir).filter(function (name) { return /\.ftl$/.test(name); })
        .map(function (name) { return h.fs.readFileSync(path.join(dir, name), "utf8"); }).join("\n");
      h.assert(new RegExp("^" + key + "\\s*=", "m").test(all), locale + " is missing key " + key);
    });
  });
});

h.staticTest("non-vacuity: small files, local assets only, guard wired", function () {
  [MENU, HELPERS].forEach(function (rel) {
    var src = readSrc(rel);
    h.assert(src.split("\n").length < 450, rel + " must stay under 450 lines");
    h.assert(src.length > 500, rel + " must remain a shipped module");
    h.assert(!/\bhttps?:\/\//i.test(src) && !/\b(?:cdn|unpkg|jsdelivr|cdnjs)\b/i.test(src),
      rel + " must not reference an external CDN");
  });
  contains(h.readRepo(RUNNER), 'require("./webtest-office-fidelity/84_export_progress_cancel")',
    "this guard must be required by the fidelity runner");
});

/** Minimal real-DOM-shaped sandbox for the shipped export menu IIFE. */
function makeEl(tag, id) {
  var el = {
    tagName: tag || "div", id: id || "", children: [], style: {}, dataset: {},
    attrs: {}, listeners: {}, className: "", type: "", parent: null, innerHTML: "", _text: "",
    setAttribute: function (k, v) { this.attrs[k] = v; },
    getAttribute: function (k) { return this.attrs[k] || null; },
    appendChild: function (c) { c.parent = this; this.children.push(c); return c; },
    insertBefore: function (c, ref) {
      c.parent = this;
      var at = ref ? this.children.indexOf(ref) : -1;
      if (at === -1) this.children.push(c); else this.children.splice(at, 0, c);
      return c;
    },
    remove: function () {
      if (this.parent) {
        var i = this.parent.children.indexOf(this);
        if (i !== -1) this.parent.children.splice(i, 1);
        this.parent = null;
      }
    },
    addEventListener: function (type, fn) {
      (this.listeners[type] = this.listeners[type] || []).push(fn);
    },
    click: function () {
      var event = { target: el, preventDefault: function () {}, stopPropagation: function () {} };
      (this.listeners.click || []).forEach(function (fn) { fn(event); });
    },
    getBoundingClientRect: function () { return { left: 0, bottom: 0 }; },
    querySelector: function (sel) { return find(this, sel); },
    querySelectorAll: function (sel) { return findAll(this, sel); },
  };
  Object.defineProperty(el, "textContent", {
    get: function () {
      return this._text || this.children.map(function (c) { return c.textContent; }).join("");
    },
    set: function (v) {
      this.children.forEach(function (c) { c.parent = null; });
      this.children = [];
      this._text = String(v);
    },
  });
  Object.defineProperty(el, "firstChild", {
    get: function () { return this.children[0] || null; },
  });
  return el;
}
function matches(el, sel) {
  if (sel.charAt(0) === ".") return el.className === sel.slice(1);
  if (sel.charAt(0) === "#") return el.id === sel.slice(1);
  return false;
}
function findAll(root, sel) {
  var out = [];
  (function walk(node) {
    node.children.forEach(function (child) {
      if (matches(child, sel)) out.push(child);
      walk(child);
    });
  })(root);
  return out;
}
function find(root, sel) { return findAll(root, sel)[0] || null; }

function makeMenu() {
  var timers = {};
  var timerSeq = 0;
  var calls = { exports: [], aborts: 0, toasts: 0, timers: timers };
  var bar = makeEl("div");
  bar.className = "docs-status-bar";
  var body = makeEl("body");
  body.appendChild(bar);
  var article = makeEl("article");
  article.dataset.docId = "doc-1";
  article.querySelectorAll = function () { return []; };
  var documentStub = {
    readyState: "loading",
    body: body,
    addEventListener: function () {},
    getElementById: function (id) { return find(body, "#" + id); },
    querySelector: function (sel) {
      if (sel === ".docs-status-bar") return bar;
      if (sel.indexOf("article[contenteditable]") !== -1) return article;
      return null;
    },
    createElement: function (tag) { return makeEl(tag); },
    createTextNode: function (text) { var node = makeEl("#text"); node._text = String(text); return node; },
  };
  var sandbox = {
    console: console, document: documentStub,
    Date: Date, Math: Math, JSON: JSON, Array: Array, Object: Object, String: String,
    encodeURIComponent: encodeURIComponent,
    setTimeout: function (fn, ms) { var id = ++timerSeq; timers[id] = { fn: fn, ms: ms }; return id; },
    clearTimeout: function (id) { delete timers[id]; },
    docsT: function (key, fallback) { return fallback; },
    AbortController: function () {
      this.signal = { aborted: false };
      this.abort = function () { this.signal.aborted = true; calls.aborts++; };
    },
  };
  sandbox.window = sandbox;
  sandbox.innerWidth = 1024;
  sandbox.GBDocsExport = function (format, id, opts) { calls.exports.push({ format: format, id: id, opts: opts }); };
  sandbox.GBToasts = { show: function () { calls.toasts++; } };
  h.vm.createContext(sandbox);
  h.vm.runInContext(readSrc(MENU), sandbox, { filename: MENU });
  var anchor = makeEl("button");
  sandbox.DocsExportMenu.open(anchor);
  var menu = find(body, ".dc-export-menu");
  h.assert(menu, "the export menu was built");
  var pdf = menu.children.filter(function (b) { return b.textContent === "PDF"; })[0];
  h.assert(pdf, "a PDF export action exists");
  pdf.click();
  h.assertEqual(calls.exports.length, 1, "one export was started");
  return { sandbox: sandbox, bar: bar, body: body, calls: calls, opts: calls.exports[0].opts };
}

function regionState(bar) {
  var region = find(bar, "#docsExportStatus");
  return {
    exists: !!region,
    role: region ? region.getAttribute("role") : null,
    live: region ? region.getAttribute("aria-live") : null,
    text: region ? find(region, ".docs-export-progress-text") : null,
    cancel: region ? find(region, ".docs-export-cancel") : null,
    raw: region ? region.textContent : "",
  };
}
function runTimers(calls) {
  Object.keys(calls.timers).forEach(function (id) { calls.timers[id].fn(); });
}

h.test("a fast export settles before the delay, so no progress UI is painted", function () {
  var t = makeMenu();
  h.assert(t.opts && typeof t.opts.onStart === "function" && typeof t.opts.onSettle === "function",
    "real lifecycle hooks were passed to GBDocsExport");
  h.assert(t.opts.signal, "an AbortSignal was passed to GBDocsExport");
  t.opts.onStart();
  h.assertEqual(Object.keys(t.calls.timers).length, 1, "the delayed paint is armed");
  t.opts.onSettle("success");
  h.assertEqual(Object.keys(t.calls.timers).length, 0, "settle clears the delayed paint");
  runTimers(t.calls);
  var state = regionState(t.bar);
  h.assert(!state.text && !state.cancel, "fast export paints no progress text and no Cancel button");
  h.assertEqual(t.calls.toasts, 0, "no spurious toast on the fast path");
});

h.test("a slow export paints a polite status with Cancel, and Cancel aborts + restores idle", function () {
  var t = makeMenu();
  t.opts.onStart();
  runTimers(t.calls);
  var state = regionState(t.bar);
  h.assert(state.exists, "a status region was created for the slow export");
  h.assertEqual(state.role, "status", "role=status");
  h.assertEqual(state.live, "polite", "aria-live=polite");
  h.assert(state.text && state.text.textContent === "In Progress", "localized progress text is visible");
  h.assert(state.cancel && state.cancel.textContent === "Cancel", "a localized Cancel button is visible");
  state.cancel.click();
  h.assertEqual(t.calls.aborts, 1, "Cancel aborted the in-flight request");
  h.assertEqual(t.opts.signal.aborted, true, "the exporter's signal is aborted");
  var idle = regionState(t.bar);
  h.assert(!idle.text && !idle.cancel && idle.raw === "", "Cancel restored the region to idle");
  t.opts.onSettle("cancelled");
  h.assertEqual(t.calls.toasts, 0, "a cancelled export announces no degraded-export toast");
  h.assert(regionState(t.bar).raw === "", "the cancelled settle leaves the region idle");
});

h.note("#export-progress-cancel: offline static + stubbed-DOM proof only — no browser/server/cargo, no locale edit. Driver owns the live Chrome UAT and issue closure.");
