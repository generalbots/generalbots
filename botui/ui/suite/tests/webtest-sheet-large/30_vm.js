/**
 * [VM] deterministic local execution of SHIPPED Sheet IIFEs.
 *
 * The production modules below are read verbatim and run unchanged in one vm
 * context. Only window/document/storage/fetch are stubbed, and fetch returns a
 * synchronous thenable so the shipped callbacks complete inside the assertion.
 * This proves request construction and window math only — it never claims a
 * network round trip, server recalculation, RSS figure or browser paint.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var equal = h.equal;

var MODULES = [
  "botui/ui/suite/sheet/js/constants/01_constants.js",
  "botui/ui/suite/sheet/js/constants/05_sheet_api.js",
  "botui/ui/suite/sheet/js/constants/02_grid_geometry.js",
  "botui/ui/suite/sheet/js/modules-sheet-advanced/22_panels.js",
];

var CANNED_SHEET = {
  id: "p-1",
  name: "large.xlsx",
  worksheets: [{ name: "Sheet1", data: { "0,0": { value: "A" }, "99999,15": { value: "B" } } }],
  source_bytes: "cGFja2FnZQ==",
};

/**
 * Minimal synchronous thenable: invokes shipped .then callbacks immediately and
 * applies Promise-style assimilation so a callback returning another thenable
 * yields its value (the sheet modules rely on this in their fetch chains).
 */
function SyncThenable(value) {
  this.value = value;
}
SyncThenable.prototype.then = function (callback) {
  try {
    var result = callback(this.value);
    while (result instanceof SyncThenable) result = result.value;
    return new SyncThenable(result);
  } catch (e) {
    return new SyncThenable(undefined);
  }
};
SyncThenable.prototype.catch = function () {
  return this;
};

function makeFixture() {
  var calls = [];
  var store = { "gb-access-token": "tok" };
  var storage = {
    getItem: function (key) {
      return Object.prototype.hasOwnProperty.call(store, key) ? store[key] : null;
    },
    setItem: function (key, value) { store[key] = String(value); },
    removeItem: function (key) { delete store[key]; },
    clear: function () { Object.keys(store).forEach(function (key) { delete store[key]; }); },
  };
  var sandbox = {
    console: console,
    setTimeout: function () { return 0; },
    clearTimeout: function () {},
    sessionStorage: storage,
    localStorage: storage,
    document: {
      readyState: "complete",
      addEventListener: function () {},
      getElementById: function () { return null; },
      querySelector: function () { return null; },
      createElement: function () { return { style: {}, dataset: {}, appendChild: function () {} }; },
    },
    fetch: function (url, options) {
      calls.push({ url: url, options: options || {} });
      return new SyncThenable({
        ok: true,
        status: 200,
        json: function () { return new SyncThenable(CANNED_SHEET); },
        text: function () { return new SyncThenable("<div>fragment</div>"); },
      });
    },
  };
  sandbox.window = {
    addEventListener: function () {},
    getGBAccessToken: function () { return "tok"; },
    gbLogError: function () {},
    __LOADED_SHEET: { id: "p-1" },
  };
  var context = h.loadModules(MODULES, sandbox);
  return { D: context.window.SheetDefaults, window: context.window, calls: calls };
}

function bodyOf(call) {
  return JSON.parse(call.options.body || "{}");
}

function fakeGrid(D, overrides) {
  var grid = {
    scrollTop: 240,
    scrollLeft: 0,
    viewportHeight: 48,
    viewportWidth: 200,
    totalRows: 100000,
    totalCols: 1000,
    requestSeq: 0,
    lastRenderedRange: null,
    instance: { sheetId: "vm-sheet" },
    colXOf: D.virtualGrid.colXOf,
    colWidthOf: D.virtualGrid.colWidthOf,
    visibleRowRange: D.virtualGrid.visibleRowRange,
    visibleColRange: D.virtualGrid.visibleColRange,
    renderRowHeaders: function () {},
    render: function () {},
  };
  Object.keys(overrides || {}).forEach(function (key) { grid[key] = overrides[key]; });
  return grid;
}

test("[VM] shipped constants and API modules load unchanged in a stubbed sandbox", function () {
  var f = makeFixture();
  assert(f.D, "window.SheetDefaults missing after module load");
  assert(f.D.SheetAPI, "SheetAPI missing after module load");
  assert(f.D.virtualGrid, "virtualGrid missing after module load");
  assert(typeof f.window.GBSheetRefreshCharts === "function", "panel refresh entry missing");
  equal(f.D.OVERSCAN, 5, "shipped overscan");
  equal(f.D.ROW_HEIGHT, 24, "shipped row height");
});

test("[VM] SheetAPI.getRange posts one bounded range request with exact viewport bounds", function () {
  var f = makeFixture();
  f.calls.length = 0;
  f.D.SheetAPI.getRange(5, 2, 10, 7, { sheetId: "s-1", worksheetIndex: 0 });
  equal(f.calls.length, 1, "fetch count");
  var call = f.calls[0];
  equal(call.url, "/api/sheet/range", "range URL");
  equal(call.options.method, "POST", "range method");
  var body = bodyOf(call);
  equal(body.sheet_id, "s-1", "sheet id");
  equal(body.start_row, 5, "start row");
  equal(body.start_col, 2, "start col");
  equal(body.end_row, 10, "end row");
  equal(body.end_col, 7, "end col");
});

test("[VM] SheetAPI.load still requests the whole workbook with no range parameter", function () {
  var f = makeFixture();
  f.calls.length = 0;
  f.D.SheetAPI.load("w-1");
  equal(f.calls.length, 1, "fetch count");
  equal(f.calls[0].url, "/api/sheet/load?id=w-1", "load URL");
  equal(f.calls[0].options.method, "GET", "load method");
  assert(f.calls[0].url.indexOf("range") < 0, "load URL unexpectedly windowed");
});

test("[VM] shipped visibleRowRange/visibleColRange return bounded overscanned windows", function () {
  var f = makeFixture();
  var grid = fakeGrid(f.D, {});
  var rows = f.D.virtualGrid.visibleRowRange.call(grid);
  equal(rows.start, 5, "row window start");
  equal(rows.end, 13, "row window end");
  assert(rows.end - rows.start <= f.D.MAX_VISIBLE_ROWS, "row window exceeds max visible rows");
  var cols = f.D.virtualGrid.visibleColRange.call(grid);
  equal(cols.start, 0, "column window start");
  equal(cols.end, 3, "column window end");
});

test("[VM] shipped requestRange fetches only the visible window, keyed by range", function () {
  var f = makeFixture();
  var grid = fakeGrid(f.D, {});
  f.calls.length = 0;
  f.D.virtualGrid.requestRange.call(grid);
  equal(f.calls.length, 1, "fetch count");
  var body = bodyOf(f.calls[0]);
  equal(f.calls[0].url, "/api/sheet/range", "range URL");
  equal(body.start_row, 5, "start row");
  equal(body.end_row, 12, "end row");
  equal(body.start_col, 0, "start col");
  equal(body.end_col, 2, "end col");
  equal(grid.lastRenderedRange, "5:13|0:3", "range cache key");
  f.D.virtualGrid.requestRange.call(grid);
  equal(f.calls.length, 1, "second identical requestRange must be deduplicated");
});

test("[VM] shipped panel refresh downloads the whole workbook then POSTs it to the fragment", function () {
  var f = makeFixture();
  f.calls.length = 0;
  f.window.GBSheetRefreshCharts();
  equal(f.calls.length, 2, "fetch count");
  equal(f.calls[0].url, "/api/sheet/load?id=p-1", "panel whole-workbook GET");
  equal(f.calls[1].url, "/suite/sheet/fragments/charts", "fragment POST URL");
  equal(f.calls[1].options.method, "POST", "fragment method");
  var posted = bodyOf(f.calls[1]);
  equal(posted.id, "p-1", "posted workbook id");
  equal(posted.name, "large.xlsx", "posted workbook name");
  equal(posted.source_bytes, "cGFja2FnZQ==", "posted retained source copy");
  equal(JSON.stringify(posted), JSON.stringify(CANNED_SHEET), "whole workbook posted verbatim");
  assert(f.calls[1].options.body.length > 100, "expected a full-workbook body, not a narrow payload");
});
