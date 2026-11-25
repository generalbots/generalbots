/**
 * #2435 — deterministic static coverage for the two SHIPPED Sheet scale fixes:
 *   (1) the bounded windowed-read cache (46_window_cache.js), installed from
 *       VirtualGrid.init and loaded by an absolute-path script tag;
 *   (2) guardrail error surfacing on every client edit path: updateCell turns a
 *       non-2xx reply into {ok:false,error}, each edit path rolls back and
 *       announces through the role=status/aria-live region, undo refuses to
 *       record a persisted-nothing edit, and the message is i18n-backed.
 *
 * All assertions are literal substrings of production source / locale catalogs.
 * Nothing is executed, no server, no browser, no cargo, no network, no new
 * user-facing string. NON-VACUITY: each case first proves the real shipped
 * module/function/markers exist and that the extracted source span is non-empty;
 * reverting any one of the two fixes makes the matching case FAIL.
 */
"use strict";

var h = require("./00_harness");

var WIN_CACHE = "botui/ui/suite/sheet/js/modules-sheet-advanced/46_window_cache.js";
var GRID_RENDER = "botui/ui/suite/sheet/js/constants/03_grid_render.js";
var SHEET_HTML = "botui/ui/suite/sheet/sheet.html";
var A11Y = "botui/ui/suite/sheet/js/constants/03a_grid_a11y.js";
var SHEET_API = "botui/ui/suite/sheet/js/constants/05_sheet_api.js";
var GRID_EDITING = "botui/ui/suite/sheet/js/constants/04_grid_editing.js";
var BOOT = "botui/ui/suite/sheet/js/constants/06_boot.js";
var VALIDATION_EDITOR = "botui/ui/suite/sheet/js/modules-sheet-advanced/09_validation_editor.js";
var UNDO = "botui/ui/suite/sheet/js/modules-sheet-advanced/04_undo.js";
var LOCALES = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(relPath, needle, msg) {
  h.assert(
    h.readRepo(relPath).indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath
  );
}

/** Exact, non-empty source span between two literal markers (end exclusive). */
function betweenIn(src, startMarker, endMarker) {
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + ">");
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + ">");
  var span = src.slice(start, end);
  h.assert(span.length > 0, "empty source span between <" + startMarker + "> and <" + endMarker + ">");
  return span;
}

function between(relPath, startMarker, endMarker) {
  return betweenIn(h.readRepo(relPath), startMarker, endMarker);
}

/** Assert `before` occurs strictly earlier than `after` inside a source span. */
function ordered(span, before, after, msg) {
  var first = span.indexOf(before);
  var second = span.indexOf(after);
  h.assert(first !== -1 && second !== -1 && first < second,
    (msg || "ordering regressed") + " :: <" + before + "> before <" + after +
    "> in <" + span.slice(0, 120) + ">");
}

// (a) The window cache negotiates with If-None-Match, recycles a 304's prior
// body, evicts LRU at a cap no higher than the server's 40 window entries,
// ships as an absolute-path script and is installed before the first read.
h.staticTest("#2435 window cache negotiates If-None-Match, reuses 304 bodies, caps LRU at <=40 and is installed by VirtualGrid.init", function () {
  var cacheSrc = h.readRepo(WIN_CACHE);

  // Non-vacuity: the real cache module and its three public pieces ship.
  contains(WIN_CACHE, "function getWindow(range, ctx) {", "windowed reader is shipped");
  contains(WIN_CACHE, "function touch(key, entry) {", "LRU touch helper is shipped");
  contains(WIN_CACHE, "function installPrefetch() {", "getRange wrapper installer is shipped");
  contains(WIN_CACHE, "window.SheetWindowCache = {", "public window-cache hook is shipped");

  // Hard cap is present and no larger than the server window budget.
  var cap = cacheSrc.match(/var MAX_ENTRIES = (\d+);/);
  h.assert(cap !== null, "MAX_ENTRIES must be a fixed integer cap");
  h.assert(Number(cap[1]) <= 40,
    "LRU cap must stay <= 40, found <" + cap[1] + ">");

  var getWindow = between(WIN_CACHE, "  function getWindow(range, ctx) {", "  function invalidate(ctx) {");
  h.assert(getWindow.indexOf('headers["If-None-Match"] = prior.etag;') !== -1,
    "a cached ETag must be sent as If-None-Match :: <" + getWindow.slice(0, 160) + ">");
  h.assert(getWindow.indexOf('"/window?range="') !== -1,
    "the bounded /window endpoint must be requested, not /range");
  h.assert(getWindow.indexOf("if (response && response.status === 304) {") !== -1,
    "304 Not Modified must be handled distinctly");

  var reuse = betweenIn(getWindow, "if (response && response.status === 304) {", "if (!response || !response.ok) return null;");
  h.assert(reuse.indexOf("touch(key, prior);") !== -1,
    "a 304 must refresh the entry's LRU position");
  h.assert(reuse.indexOf("cells: prior.cells,") !== -1,
    "a 304 must reuse the cached cell body");
  h.assert(reuse.indexOf("version: prior.version,") !== -1,
    "a 304 must reuse the cached version");
  h.assert(reuse.indexOf("etag: prior.etag,") !== -1,
    "a 304 must reuse the cached ETag");
  h.assert(reuse.indexOf("fromCache: true") !== -1,
    "a 304 hit must be marked as coming from the cache");

  var touch = between(WIN_CACHE, "  function touch(key, entry) {", "  function parseA1(ref) {");
  h.assert(touch.indexOf("if (cache.has(key)) cache.delete(key);") !== -1,
    "touch must re-insert to move a key to the LRU tail");
  h.assert(touch.indexOf("while (cache.size > MAX_ENTRIES) {") !== -1,
    "eviction must run whenever the cap is exceeded");
  h.assert(touch.indexOf("var oldest = cache.keys().next();") !== -1 &&
    touch.indexOf("cache.delete(oldest.value);") !== -1,
    "eviction must drop the oldest insertion-order key");

  var install = between(WIN_CACHE, "  function installPrefetch() {", "  window.SheetWindowCache = {");
  h.assert(install.indexOf("var original = api.getRange;") !== -1 &&
    install.indexOf("wrapped.__gbWindowPrefetch = true;") !== -1 &&
    install.indexOf("api.getRange = wrapped;") !== -1,
    "the getRange wrapper must be installed idempotently");
  h.assert(install.indexOf("return original.call(api, startRow, startCol, endRow, endCol, ctx);") !== -1,
    "the untouched /range original must remain the fallback");
  contains(WIN_CACHE, "ensureInstalled: installPrefetch,", "defensive installer hook is exported");
  contains(WIN_CACHE, "installPrefetch();", "module self-installs when loaded last");

  // Absolute path matters: this script is injected into a desktop shell and a
  // relative src would 404 when the page is reached through a launcher route.
  contains(SHEET_HTML,
    '<script src="/suite/sheet/js/modules-sheet-advanced/46_window_cache.js"></script>',
    "window cache is loaded from an absolute /suite path");
  ordered(h.readRepo(SHEET_HTML), "js/constants/06_boot.js",
    "js/modules-sheet-advanced/46_window_cache.js",
    "window cache must load after SheetAPI/boot");

  var init = between(GRID_RENDER, "  VirtualGrid.init = function (host) {", "  VirtualGrid.renderHeaders = function () {");
  h.assert(init.indexOf('if (window.SheetWindowCache && typeof window.SheetWindowCache.ensureInstalled === "function") {') !== -1,
    "VirtualGrid.init must guard on the installer hook");
  h.assert(init.indexOf("window.SheetWindowCache.ensureInstalled();") !== -1,
    "VirtualGrid.init must install the prefetch");
  ordered(init, "window.SheetWindowCache.ensureInstalled();", "this.requestRange();",
    "installation must precede the first viewport read");
});

// (b) updateCell must keep the success payload verbatim but discriminate a
// guardrail 400 into {ok:false,error}, and all three user edit entry points
// must restore the pre-edit value and announce through the live region.
h.staticTest("#2435 updateCell discriminates non-2xx into {ok:false,error} while keeping success, and every edit path rolls back + announces", function () {
  var up = between(SHEET_API, "    updateCell: function (ref, value, ctx) {", "    evaluateFormula: function (formula, ctx) {");
  h.assert(up.indexOf("if (r.ok) return r.json();") !== -1,
    "the raw success JSON shape must stay unchanged");
  h.assert(up.indexOf("const out = { ok: false };") !== -1,
    "a non-2xx reply must become a discriminated failure");
  h.assert(up.indexOf('if (body && typeof body.error === "string") out.error = body.error;') !== -1,
    "the guardrail's server message must be carried in out.error");
  h.assert(up.indexOf("return out;") !== -1,
    "the failure object must be returned to every caller");
  h.assert(up.indexOf('}).catch(function () { return { ok: false }; });') !== -1,
    "an unparseable error body must still be a typed failure");
  ordered(up, "if (r.ok) return r.json();", "const out = { ok: false };",
    "success must be checked before the failure branch");

  // (1) In-cell editing (dblclick / blur).
  var blur = between(GRID_EDITING, "  VirtualGrid.onCellBlur = function (e) {", "  VirtualGrid.onCellKeyDown = function (e) {");
  h.assert(blur.indexOf("D.SheetAPI.updateCell(ref, val, self.instance || self).then(function (res) {") !== -1,
    "cell blur must go through updateCell");
  h.assert(blur.indexOf("if (res && res.success) {") !== -1,
    "cell blur must keep its success path");
  h.assert(blur.indexOf("cell.textContent = origVal;") !== -1,
    "cell blur must restore the old cell text on refusal");
  h.assert(blur.indexOf('const message = (res && res.error) || tr("status-error", "Error");') !== -1,
    "cell blur must surface the server message via the i18n fallback");

  // (2) Formula bar commit.
  var commit = between(BOOT, "      var commitFormula = function () {", '      formulaInput.addEventListener("keydown", function (e) {');
  h.assert(commit.indexOf("SheetAPI.updateCell(ref, val, inst || g).then(function (res) {") !== -1,
    "formula bar must go through updateCell");
  h.assert(commit.indexOf("if (res && res.ok === false) {") !== -1,
    "formula bar must branch on the refusal discriminant");
  h.assert(commit.indexOf("formulaInput.value = oldBarValue;") !== -1,
    "formula bar must restore the old value on refusal");
  h.assert(commit.indexOf('var message = res.error || tr("status-error", "Error");') !== -1,
    "formula bar must surface the server message via the i18n fallback");

  // (3) List-validation dropdown commit.
  var dropdown = between(VALIDATION_EDITOR, "    function commit() {", '    sel.addEventListener("change", commit);');
  h.assert(dropdown.indexOf("const req = a.updateCell(ref, val);") !== -1,
    "dropdown must go through updateCell");
  h.assert(dropdown.indexOf("if (!res || res.ok !== false) return;") !== -1,
    "dropdown must branch on the refusal discriminant");
  h.assert(dropdown.indexOf("if (before) g.cells.set(key, before);") !== -1 &&
    dropdown.indexOf("else g.cells.delete(key);") !== -1,
    "dropdown must restore the old cached cell on refusal");
  h.assert(dropdown.indexOf("if (node.isConnected) node.textContent = oldText;") !== -1,
    "dropdown must restore the old visible text on refusal");
  h.assert(dropdown.indexOf('const message = res.error || tr("status-error", "Error");') !== -1,
    "dropdown must surface the server message via the i18n fallback");

  // Every refusal is announced into the real role=status/aria-live region.
  [GRID_EDITING, BOOT, VALIDATION_EDITOR].forEach(function (rel) {
    var src = h.readRepo(rel);
    h.assert(src.indexOf("if (window.sheetNotify) window.sheetNotify(message, true);") !== -1,
      rel + " must show the refusal in the error toast");
    h.assert(src.indexOf("grid.announce(message, true);") !== -1 &&
      src.indexOf('document.querySelector(".sheet-a11y-live")') !== -1,
      rel + " announce helper must target the live region");
    h.assert(src.indexOf("window.gbT(key, fallback)") !== -1,
      rel + " tr helper must route labels through gbT");
  });
  var a11y = h.readRepo(A11Y);
  h.assert(a11y.indexOf('this.a11yLive.className = "sheet-a11y-live";') !== -1 &&
    a11y.indexOf('this.a11yLive.setAttribute("role", "status");') !== -1 &&
    a11y.indexOf('this.a11yLive.setAttribute("aria-live", "polite");') !== -1,
    "the grid must ship the role=status / aria-live=polite live region");
  var announces = [
    { rel: GRID_EDITING, needle: "announce(self, message);" },
    { rel: BOOT, needle: "announce(g, message);" },
    { rel: VALIDATION_EDITOR, needle: "announce(g, message);" }
  ];
  announces.forEach(function (entry) { contains(entry.rel, entry.needle, "refusal must be announced"); });
});

// (c) Nothing was persisted on a guardrail refusal, so undo must not record it,
// and the client must not duplicate the server's error text as a hardcoded
// literal: every path reuses the translated status-error key.
h.staticTest("#2435 undo never records a refused edit and no hardcoded error literal was introduced (status-error key reused)", function () {
  var wrap = between(UNDO, "    a.updateCell = function (ref, value) {", "  function showToast(msg) {");
  h.assert(wrap.indexOf("const res = orig(ref, value);") !== -1,
    "the undo wrapper must delegate to the real updateCell");
  h.assert(wrap.indexOf("guardrail refusal resolves with {ok:false}.") !== -1,
    "the refusal contract must be documented at the recording site");
  h.assert(wrap.indexOf("res.then(function (result) {") !== -1 &&
    wrap.indexOf("if (result && result.ok === false) return;") !== -1,
    "a refusal must short-circuit before any undo entry is pushed");
  h.assert(wrap.indexOf('pushEntry("Edit", oldSnap, newSnap);') !== -1,
    "successful edits must still be recorded as undoable");
  ordered(wrap, "if (result && result.ok === false) return;", 'pushEntry("Edit", oldSnap, newSnap);',
    "the refusal guard must precede pushEntry");

  // The server owns the wording; the client only falls back to the shared key.
  var serverLiterals = [
    "Recalculation refused",
    "recalculation memory budget exceeded",
    "too many dependent formulas"
  ];
  [SHEET_API, GRID_EDITING, BOOT, VALIDATION_EDITOR, UNDO, WIN_CACHE].forEach(function (rel) {
    var src = h.readRepo(rel);
    serverLiterals.forEach(function (literal) {
      h.assert(src.indexOf(literal) === -1,
        rel + " must receive the guardrail text via res.error, no hardcoded <" + literal + ">");
    });
  });
  contains(GRID_EDITING, 'tr("status-error", "Error")', "cell edit reuses status-error");
  contains(BOOT, 'tr("status-error", "Error")', "formula bar reuses status-error");
  contains(VALIDATION_EDITOR, 'tr("status-error", "Error")', "dropdown reuses status-error");

  var values = [];
  LOCALES.forEach(function (loc) {
    var rel = "botlib/locales/" + loc + "/common.ftl";
    var catalog = h.readRepo(rel);
    var match = catalog.match(/^status-error\s*=\s*(\S.*)$/m);
    h.assert(match !== null, rel + " must define status-error on its own line");
    var value = match[1].trim();
    h.assert(value.length > 0, rel + " status-error translation must not be empty");
    values.push(value);
  });
  h.assertEqual(values.length, 8, "all 8 catalogs must define status-error");
  var distinct = {};
  values.forEach(function (value) { distinct[value] = true; });
  h.assert(Object.keys(distinct).length >= 2,
    "status-error must be genuinely translated, found <" + values.join(" | ") + ">");
});

h.note("FOLLOW-UP #2435: window reads are cached/bounded and guardrail refusals are rolled back, " +
  "announced and kept out of the undo stack; the Rust-side recalc budget guards remain covered by 29_sheet_scale_guard.js.");
