/**
 * [STATIC] deterministic guards for the #2459 office performance-mark dispatchers.
 * Every case reads real shipped source as text and FAILS if a hook, an allowlist,
 * the afterApply ordering or a report field is removed: no runtime claim, no
 * server, no browser, no cargo, no PII introduced here.
 *
 * Covered contracts:
 *   (a) gbPerfSignal maps save/export/slide/recalc to document CustomEvents and
 *       carries only a normalized app label (+ a known export format) — no ids
 *   (b) every shipped UI hook calls gbPerfSignal with the correct app/stage,
 *       guarded and placed on the success path only
 *   (c) the sheet recalc callback runs only after cells are applied and rendered
 *       inside a live range response; scroll/resize/format/sort pass no callback
 *   (d) report() exposes openToSave/openToExport/openToSlide/openToRecalc with
 *       noPii = true and no document/user fields
 *
 * All assertions are literal source substrings plus span/ordering checks.
 */
"use strict";

var h = require("./00_harness");

var MARKS = "botui/ui/suite/js/office-perf-marks.js";
var DOCS_SAVE = "botui/ui/suite/docs/modules/02_save.js";
var SHEET_SAVE = "botui/ui/suite/sheet/sheet-inline/01_sidebar_b.js";
var SHEET_EXPORT = "botui/ui/suite/sheet/js/modules-sheet-advanced/12_export.js";
var SLIDES_EVENTS = "botui/ui/suite/slides/modules/04_events.js";
var GRID_GEOMETRY = "botui/ui/suite/sheet/js/constants/02_grid_geometry.js";
var GRID_EDITING = "botui/ui/suite/sheet/js/constants/04_grid_editing.js";
var GRID_BOOT = "botui/ui/suite/sheet/js/constants/06_boot.js";
var SHEET_SORT = "botui/ui/suite/sheet/js/modules-sheet-advanced/11_sort.js";

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(relPath, needle, msg) {
  h.assert(
    h.readRepo(relPath).indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath
  );
}

/** Exact source span between two markers (exclusive of the end marker). */
function between(relPath, startMarker, endMarker) {
  var src = h.readRepo(relPath);
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + "> in " + relPath);
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + "> in " + relPath);
  var span = src.slice(start, end);
  h.assert(span.length > 0, "empty source span between <" + startMarker + "> and <" + endMarker + ">");
  return span;
}

/** A field that must never reach a CustomEvent detail or a report entry. */
var PII_FIELDS = ["docId", "resourceId", "token", "email", "userId", "tenantId", "fileName", "content:"];

function absent(span, needle, msg) {
  h.assert(span.indexOf(needle) === -1,
    (msg || "source must not contain") + " <" + needle + "> :: <" + span.slice(0, 200) + ">");
}

function countOf(text, needle) { return text.split(needle).length - 1; }

// (a) + (d) — one dispatcher contract plus its report surface, both shipped in
// office-perf-marks.js. The closed allowlists guarantee a normalized label; the
// detail literal and the report object must carry no document/user identity.
h.staticTest("gbPerfSignal dispatch and report expose only normalized PII-free fields", function () {
  var src = h.readRepo(MARKS);

  // Closed allowlists: only a known app label and a known format can be emitted.
  contains(MARKS, 'var NO_LABEL = "unknown";', "unknown fallback label exists");
  contains(MARKS, "var KNOWN_APPS = (", "app allowlist is declared");
  contains(MARKS, "var KNOWN_FORMATS = (", "format allowlist is declared");
  contains(MARKS, "if (KNOWN_APPS.indexOf(lower) !== -1) return lower.slice(0, maxLength || 32);",
    "label() only returns allowlisted app ids");
  contains(MARKS, "return KNOWN_FORMATS.indexOf(lower) !== -1 ? lower : NO_LABEL;",
    "formatLabel() only returns allowlisted formats");

  // The four production stages map to document CustomEvents via a fixed table.
  var dispatch = between(MARKS, "global.gbPerfSignal = function (appId, stage, format) {",
    "global.GBOfficePerf = api;");
  var names = between(MARKS, "var names = {", "var name = names[stage];");
  [
    'save: "gb:office-saved", exported: "gb:office-exported",',
    'export: "gb:office-exported", slide: "gb:office-slide", recalc: "gb:office-recalc"'
  ].forEach(function (pair) {
    h.assert(names.indexOf(pair) !== -1, "stage-to-event map must carry <" + pair + "> :: <" + names + ">");
  });
  h.assertEqual(countOf(names, ': "'), 5, "the stage map must have exactly five entries");

  // Only the normalized label, the stage and (for exports) a normalized format
  // are dispatched, through one CustomEvent on document.
  [
    "var detail = { appId: label(appId, 32), stage: stage };",
    'if (stage === "export" || stage === "exported") detail.format = formatLabel(format);',
    "global.document.dispatchEvent(new global.CustomEvent(name, { detail: detail }));"
  ].forEach(function (line) {
    h.assert(dispatch.indexOf(line) !== -1, "dispatcher must contain <" + line + ">");
  });
  h.assertEqual(countOf(dispatch, "var detail ="), 1, "dispatcher must build exactly one detail object");
  var detail = between(MARKS, "var detail = { appId: label(appId, 32), stage: stage };",
    "} catch (e) { /* instrumentation must never break the app */ }");
  PII_FIELDS.forEach(function (field) {
    absent(detail, field, "CustomEvent detail must not carry " + field);
  });

  // Ordering: label() exists before the dispatcher, which precedes the exposed
  // API object; inside, the names table precedes the guarded dispatch.
  var labelPos = src.indexOf("function label(value, maxLength) {");
  var dispatchPos = src.indexOf("global.gbPerfSignal = function (appId, stage, format) {");
  var apiPos = src.indexOf("global.GBOfficePerf = api;");
  h.assert(labelPos !== -1 && dispatchPos !== -1 && apiPos !== -1 &&
    labelPos < dispatchPos && dispatchPos < apiPos,
    "label() -> gbPerfSignal -> GBOfficePerf must appear in that order");
  var mapPos = dispatch.indexOf("var names = {");
  var detailPos = dispatch.indexOf("var detail =");
  var sendPos = dispatch.indexOf("global.document.dispatchEvent(");
  h.assert(mapPos !== -1 && detailPos !== -1 && sendPos !== -1 && mapPos < detailPos && detailPos < sendPos,
    "names table -> detail -> dispatchEvent must appear in that order");

  // report(): the four required durations plus the no-PII declaration, and no
  // document/user field in the whole report span.
  var report = between(MARKS, "function report() {", "var api = {");
  h.assert(report.indexOf("var result = { noPii: true, apps: {} };") !== -1,
    "report must declare noPii: true :: <" + report.slice(0, 200) + ">");
  [
    "openToSave: duration(open, save),",
    "openToExport: duration(open, exported),",
    "openToSlide: duration(open, marks.slide || null),",
    "openToRecalc: duration(open, marks.recalc || null)"
  ].forEach(function (field) {
    h.assert(report.indexOf(field) !== -1, "report must expose <" + field + ">");
  });
  h.assertEqual(countOf(report, "duration("), 7, "report must expose exactly seven duration fields");
  PII_FIELDS.forEach(function (field) {
    absent(report, field, "report must not carry " + field);
  });
});

// (b) — every shipped dispatcher, exactly once, guarded, inside its success
// path. Removing any one of these five hooks makes this case FAIL.
h.staticTest("Every shipped office UI hook dispatches the correct app and stage", function () {
  var guard = 'if (typeof window.gbPerfSignal === "function")';
  var hooks = [
    { path: DOCS_SAVE, label: "docs save",
      line: guard + ' window.gbPerfSignal("docs", "save");',
      fn: "function scheduleSave(id, content) {", end: "/* Restore a local draft",
      success: 'setSaveStatus("All changes saved", false);', failure: ".catch(function () {" },
    { path: SHEET_SAVE, label: "sheet save",
      line: guard + ' window.gbPerfSignal("sheet", "save");',
      fn: "function saveSheet(arg) {", end: "document.addEventListener('keydown', function(e) {",
      success: "return j;", failure: ".catch(function(err) {" },
    { path: SHEET_EXPORT, label: "sheet export",
      line: guard + ' window.gbPerfSignal("sheet", "export", format);',
      fn: "function exportFormat(format, ext) {", end: "function showToast(msg, ok) {",
      success: "download(blob, filename);", failure: ".catch(function (err) {" },
    { path: SLIDES_EVENTS, label: "slides transition",
      line: guard + ' window.gbPerfSignal("slides", "slide");',
      fn: "function gbSlidesMarkTransition() {", end: "window.refreshSlideView = function (slideIndex) {" },
    { path: GRID_GEOMETRY, label: "sheet recalc",
      line: guard + ' window.gbPerfSignal("sheet", "recalc");',
      fn: "function markSheetRecalc() {", end: "D.markSheetRecalc = markSheetRecalc;" }
  ];

  hooks.forEach(function (hook) {
    var src = h.readRepo(hook.path);
    h.assertEqual(countOf(src, "window.gbPerfSignal(\""), 1,
      hook.label + " must have exactly one dispatcher call");
    contains(hook.path, hook.line, hook.label + " must dispatch its exact app/stage");
    var span = between(hook.path, hook.fn, hook.end);
    h.assert(span.indexOf(hook.line) !== -1,
      hook.label + " hook must live inside its handler :: <" + span.slice(0, 200) + ">");
  });

  // The three async hooks are fail-closed: success marker < dispatch < catch.
  hooks.slice(0, 3).forEach(function (hook) {
    var span = between(hook.path, hook.fn, hook.end);
    var success = span.indexOf(hook.success);
    var dispatch = span.indexOf(hook.line);
    var failure = span.indexOf(hook.failure);
    h.assert(success !== -1 && dispatch !== -1 && failure !== -1 &&
      success < dispatch && dispatch < failure,
      hook.label + " must dispatch only after a successful response :: <" + span.slice(0, 260) + ">");
  });

  // The two synchronous signals are invoked from a real UI transition/edit path.
  contains(GRID_GEOMETRY, "D.markSheetRecalc = markSheetRecalc;",
    "the recalc mark helper is exported for the grid edit paths");
  var refresh = between(SLIDES_EVENTS, "window.refreshSlideView = function (slideIndex) {",
    'document.addEventListener("click", function (e) {');
  var markCall = refresh.indexOf("gbSlidesMarkTransition();");
  var ajax = refresh.indexOf("htmx.ajax(");
  h.assert(markCall !== -1 && ajax !== -1 && markCall < ajax,
    "the slide mark must run when a slide view (including thumbnails) refreshes");
});

// (c) — the recalc mark is a callback on a live range response: cells applied,
// render done, callback before any catch; passive fetches never pass it.
h.staticTest("Sheet recalc mark fires only after cells are applied and rendered", function () {
  var rr = between(GRID_GEOMETRY, "VirtualGrid.requestRange = function (afterApply) {",
    "VirtualGrid.renderRowHeaders = function (range) {");
  [
    "if (seq !== self.requestSeq) return;",
    "self.cells.set(k, data.cells[k]);",
    "self.render();",
    'if (typeof afterApply === "function") afterApply();',
    ".catch(function (err) {"
  ].forEach(function (line) {
    h.assert(rr.indexOf(line) !== -1, "requestRange must contain <" + line + ">");
  });
  var stale = rr.indexOf("if (seq !== self.requestSeq) return;");
  var apply = rr.indexOf("self.cells.set(k, data.cells[k]);");
  var render = rr.indexOf("self.render();");
  var callback = rr.indexOf('if (typeof afterApply === "function") afterApply();');
  var failure = rr.indexOf(".catch(function (err) {");
  h.assert(stale < apply && apply < render && render < callback && callback < failure,
    "stale guard -> cells applied -> render -> afterApply -> catch order required");
  h.assertEqual(countOf(rr, "afterApply();"), 1, "requestRange must invoke the callback exactly once");
  absent(rr.slice(failure), "afterApply", "the recalc callback must never run on a failed fetch");

  // Passive range fetches must not emit a recalc mark.
  function passive(path, start, end, call, label) {
    var span = between(path, start, end);
    h.assert(span.indexOf(call) !== -1, label + " must still refresh a range :: <" + span.slice(0, 200) + ">");
    absent(span, "markSheetRecalc", label + " must not signal recalc");
    absent(span, "afterApply", label + " must not pass a callback");
  }
  passive(GRID_GEOMETRY, "VirtualGrid.onScroll = function () {",
    "VirtualGrid.getOrCreateHeaderColNode = function (idx) {", "this.requestRange();", "scroll");
  passive(GRID_BOOT, 'window.addEventListener("resize", function () {',
    "function initCollab(scopeBody) {", "g.requestRange();", "resize");
  passive(GRID_BOOT, "function formatSelectedRange(payload, gridOrCtx) {",
    "function initToolbar(scopeBody) {", "grid.requestRange();", "format");
  passive(SHEET_SORT, "function reloadFromServer() {",
    "function openMenu(col, anchor) {", "g.requestRange();", "sort");

  // The only two paths that opt in are an accepted cell write and an accepted
  // formula-bar edit, both after the server confirmed the change.
  var edit = between(GRID_EDITING, "VirtualGrid.onCellBlur = function (e) {",
    "VirtualGrid.onCellKeyDown = function (e) {");
  var okEdit = edit.indexOf("if (res && res.success) {");
  var editHook = edit.indexOf("self.requestRange(D.markSheetRecalc);");
  h.assert(okEdit !== -1 && editHook !== -1 && okEdit < editHook,
    "only a successful cell write requests the recalc callback");
  var formula = between(GRID_BOOT, "var commitFormula = function () {",
    'formulaInput.addEventListener("keydown", function (e) {');
  var okFormula = formula.indexOf("if (res && res.success) {");
  var formulaHook = formula.indexOf("if (g.requestRange) g.requestRange(D.markSheetRecalc);");
  h.assert(okFormula !== -1 && formulaHook !== -1 && okFormula < formulaHook,
    "only a successful formula-bar edit requests the recalc callback");
});
