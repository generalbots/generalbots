"use strict";
/* #2435 — bounded windowed range reads with ETag/304 caching.
 * The grid's viewport-change path is requestRange -> SheetAPI.getRange. This
 * module wraps that path so it reads GET /api/sheet/:id/window (with
 * If-None-Match) first and keeps a small, bounded cache of cell windows. The
 * original POST /api/sheet/range implementation stays as the fallback, so any
 * error, 304 without a cached body, out-of-budget range or missing prerequisite
 * keeps the previous behaviour. No user-facing strings are introduced here. */

(function () {
  var D = window.SheetDefaults;
  if (!D) return;

  // Mirrors the server caps in botsheet::handlers::window (MAX_WINDOW_ROWS/COLS)
  // so an over-sized window is never sent and simply falls back to /range.
  var MAX_ENTRIES = 40;
  var MAX_ROWS = 1000;
  var MAX_COLS = 256;

  var cache = new Map(); // key "sheetId|ws|A1:RANGE" -> { cells, version, etag, range, worksheet }
  var installed = false;

  function ctxOf(ctx) {
    if (ctx && typeof ctx === "object" &&
        (ctx.sheetId || ctx.windowId || ctx.host || ctx.body || ctx.cells)) return ctx;
    return D.getActiveSheetInstance ? D.getActiveSheetInstance() : null;
  }

  function sheetIdFor(ctx) {
    if (ctx && ctx.sheetId) return ctx.sheetId;
    var inst = ctxOf(ctx);
    if (inst) {
      if (inst.loadedSheet && inst.loadedSheet.id) return inst.loadedSheet.id;
      if (inst.sheetId) return inst.sheetId;
    }
    return D.currentSheetId ? D.currentSheetId(ctx || inst) : "current";
  }

  function wsIndexFor(ctx) {
    if (ctx && typeof ctx.worksheetIndex === "number") return ctx.worksheetIndex;
    var inst = ctxOf(ctx);
    if (inst && typeof inst.worksheetIndex === "number") return inst.worksheetIndex;
    return typeof D.WORKSHEET_INDEX === "number" ? D.WORKSHEET_INDEX : 0;
  }

  // Map insertion order is used as an LRU order: re-inserting a touched key
  // moves it to the end; eviction always removes the oldest entry.
  function touch(key, entry) {
    if (cache.has(key)) cache.delete(key);
    cache.set(key, entry);
    while (cache.size > MAX_ENTRIES) {
      var oldest = cache.keys().next();
      if (oldest.done) break;
      cache.delete(oldest.value);
    }
  }

  function parseA1(ref) {
    var m = /^([A-Za-z]{1,3})([0-9]+)$/.exec(String(ref || "").trim());
    if (!m) return null;
    var letters = m[1].toUpperCase();
    var col = 0;
    for (var i = 0; i < letters.length; i++) col = col * 26 + (letters.charCodeAt(i) - 64);
    var row = parseInt(m[2], 10) - 1;
    if (!Number.isFinite(col) || !Number.isFinite(row) || col < 1 || row < 0) return null;
    return { row: row, col: col - 1 };
  }

  function withinBudget(range) {
    var parts = String(range || "").split(":");
    if (parts.length !== 2) return false;
    var a = parseA1(parts[0]);
    var b = parseA1(parts[1]);
    if (!a || !b || a.row > b.row || a.col > b.col) return false;
    return ((b.row - a.row + 1) <= MAX_ROWS && (b.col - a.col + 1) <= MAX_COLS);
  }

  // Returns a normalized { cells, version, range, worksheet, etag, fromCache }
  // on a usable window, or null on any miss/error. Callers fall back to /range.
  function getWindow(range, ctx) {
    try {
      var normalized = String(range || "").trim().toUpperCase();
      if (!withinBudget(normalized)) return Promise.resolve(null);
      var sheetId = sheetIdFor(ctx);
      if (!sheetId) return Promise.resolve(null);
      var ws = wsIndexFor(ctx);
      var key = String(sheetId) + "|" + ws + "|" + normalized;
      var prior = cache.get(key);
      var headers = {};
      if (prior && prior.etag) headers["If-None-Match"] = prior.etag;
      var url = "/api/sheet/" + encodeURIComponent(String(sheetId)) +
        "/window?range=" + encodeURIComponent(normalized) +
        "&sheet=" + encodeURIComponent(String(ws));

      return fetch(url, { method: "GET", headers: headers })
        .then(function (response) {
          if (response && response.status === 304) {
            if (prior && prior.cells) {
              touch(key, prior);
              return {
                cells: prior.cells,
                version: prior.version,
                range: prior.range,
                worksheet: prior.worksheet,
                etag: prior.etag,
                fromCache: true
              };
            }
            return null;
          }
          if (!response || !response.ok) return null;
          var etag = (response.headers && typeof response.headers.get === "function")
            ? response.headers.get("etag") : null;
          return response.json().then(function (body) {
            if (!body || typeof body !== "object") return null;
            var cells = (body.cells && typeof body.cells === "object" && !Array.isArray(body.cells))
              ? body.cells : null;
            if (!cells) return null;
            var entry = {
              cells: cells,
              version: (typeof body.version === "number") ? body.version : null,
              range: (typeof body.range === "string") ? body.range : normalized,
              worksheet: ws,
              etag: (typeof etag === "string" && etag) ? etag : (prior ? prior.etag : null)
            };
            touch(key, entry);
            return {
              cells: cells,
              version: entry.version,
              range: entry.range,
              worksheet: ws,
              etag: entry.etag,
              fromCache: false
            };
          });
        })
        .catch(function (err) { return null; });
    } catch (err) {
      return Promise.resolve(null);
    }
  }

  function invalidate(ctx) {
    try {
      var sheetId = sheetIdFor(ctx);
      var ws = wsIndexFor(ctx);
      var prefix = String(sheetId) + "|" + ws + "|";
      var keys = Array.from(cache.keys());
      for (var i = 0; i < keys.length; i++) {
        if (keys[i].indexOf(prefix) === 0) cache.delete(keys[i]);
      }
    } catch (err) { /* best effort only */ }
  }

  // Wrap the single SheetAPI.getRange method once (idempotent). requestRange
  // in the geometry module then transparently prefers /window on every
  // viewport change; the untouched original handles all fallback cases.
  function installPrefetch() {
    if (installed) return true;
    var api = D.SheetAPI;
    if (!api || typeof api.getRange !== "function") return false;
    if (api.getRange.__gbWindowPrefetch) { installed = true; return true; }
    var original = api.getRange;
    var wrapped = function (startRow, startCol, endRow, endCol, ctx) {
      var colName = D.colName;
      if (typeof colName !== "function" ||
          !Number.isFinite(startRow) || !Number.isFinite(startCol) ||
          !Number.isFinite(endRow) || !Number.isFinite(endCol) ||
          startRow > endRow || startCol > endCol) {
        return original.call(api, startRow, startCol, endRow, endCol, ctx);
      }
      var range = colName(startCol) + (startRow + 1) + ":" + colName(endCol) + (endRow + 1);
      return getWindow(range, ctx).then(function (res) {
        if (res && res.cells && typeof res.cells === "object") {
          return { cells: res.cells };
        }
        return original.call(api, startRow, startCol, endRow, endCol, ctx);
      });
    };
    wrapped.__gbWindowPrefetch = true;
    api.getRange = wrapped;
    installed = true;
    return true;
  }

  window.SheetWindowCache = {
    getWindow: getWindow,
    invalidate: invalidate,
    installPrefetch: installPrefetch,
    ensureInstalled: installPrefetch,
    cacheSize: function () { return cache.size; }
  };

  // This module is the last script in sheet.html, so SheetAPI already exists.
  installPrefetch();
})();
