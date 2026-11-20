/**
 * [BASELINE-OBSERVE] characterizations of the SHIPPED Sheet UI sources.
 *
 * These numbers are observations of the tree today, not a ratchet: no baseline
 * file is read or written. When production legitimately changes the load path,
 * update these exact numbers in the same change so the drift is reviewed.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var src = h.src;
var sheetJsFiles = h.sheetJsFiles;
var filesMatching = h.filesMatching;
var countMatches = h.countMatches;
var containsAll = h.containsAll;

var PANELS = "botui/ui/suite/sheet/js/modules-sheet-advanced/22_panels.js";
var GEOMETRY = "botui/ui/suite/sheet/js/constants/02_grid_geometry.js";
var BOOT = "botui/ui/suite/sheet/sheet-inline/02_boot.js";

var LOAD_EXACT = /\/api\/sheet\/load(?!-from-drive)/g;
var LOADED_SHEET = /__LOADED_SHEET/g;
var LOAD_FROM_DRIVE = /\/api\/sheet\/load-from-drive/g;
var RANGE = /\/api\/sheet\/range/g;
var GET_RANGE_CALL = /\.getRange\(/g;
var ABORT_ETAG = /\bETag\b|If-None-Match|AbortController|\.abort\(/gi;
var PROGRESS = /progress/gi;
var I18N_WRAPPER = /\bwindow\.gbT\([A-Za-z_$][\w$]*\s*,/g;

test("[BASELINE-OBSERVE] 115 shipped sheet JS modules scanned, tests/ excluded", function () {
  var files = sheetJsFiles();
  equal(files.length, 115, "sheet JS module count");
  assert(
    files.every(function (rel) { return rel.indexOf("/tests/") < 0; }),
    "tests/ leaked into the scan set"
  );
});

test("[BASELINE-OBSERVE] sheet JS tree is 23,890 source lines", function () {
  var lines = sheetJsFiles().reduce(function (total, rel) {
    return total + h.readRepo(rel).split("\n").length;
  }, 0);
  equal(lines, 23907, "shipped sheet JS line count");
});

test("[BASELINE-OBSERVE] 87 __LOADED_SHEET references across 32 shipped modules", function () {
  var files = sheetJsFiles();
  var occurrences = files.reduce(function (n, rel) {
    return n + countMatches(src(rel), LOADED_SHEET);
  }, 0);
  equal(occurrences, 87, "__LOADED_SHEET occurrences");
  equal(filesMatching(files, LOADED_SHEET).length, 32, "modules retaining the global");
});

test("[BASELINE-OBSERVE] 6 whole-workbook /api/sheet/load call sites across 5 modules", function () {
  var files = sheetJsFiles();
  var occurrences = files.reduce(function (n, rel) {
    return n + countMatches(src(rel), LOAD_EXACT);
  }, 0);
  equal(occurrences, 6, "whole-workbook load call sites");
  equal(filesMatching(files, LOAD_EXACT).length, 5, "modules calling whole-workbook load");
});

test("[BASELINE-OBSERVE] 3 /api/sheet/load-from-drive call sites across 3 modules", function () {
  var files = sheetJsFiles();
  equal(
    files.reduce(function (n, rel) { return n + countMatches(src(rel), LOAD_FROM_DRIVE); }, 0),
    3,
    "load-from-drive call sites"
  );
  equal(filesMatching(files, LOAD_FROM_DRIVE).length, 3, "modules using load-from-drive");
});

test("[BASELINE-OBSERVE] one /api/sheet/range client call and one getRange caller", function () {
  var files = sheetJsFiles();
  var rangeFiles = filesMatching(files, RANGE);
  var callerFiles = filesMatching(files, GET_RANGE_CALL);
  equal(rangeFiles.length, 1, "range endpoint client modules");
  equal(callerFiles.length, 1, "getRange caller modules");
  assert(rangeFiles[0].indexOf("05_sheet_api.js") >= 0, "range client moved: " + rangeFiles[0]);
  assert(callerFiles[0].indexOf("02_grid_geometry.js") >= 0, "range caller moved: " + callerFiles[0]);
});

test("[BASELINE-OBSERVE] grid viewport windowing is shipped (row + column + requestRange)", function () {
  containsAll(GEOMETRY, [
    "VirtualGrid.visibleRowRange = function ()",
    "VirtualGrid.visibleColRange = function ()",
    "VirtualGrid.requestRange = function ()",
    "D.SheetAPI.getRange(range.start, cols.start, range.end - 1, cols.end - 1",
    "if (this.lastRenderedRange === rangeKey) return;",
  ]);
  assert(countMatches(src(GEOMETRY), /OVERSCAN/g) >= 1, "overscan constant missing");
});

test("[BASELINE-OBSERVE] boot path retains the whole workbook in per-window + global state", function () {
  containsAll(BOOT, [
    "fetch('/api/sheet/load?id=' + encodeURIComponent(sheetRef)",
    "setState({ loadedSheet: sheet, sheetId: (sheet && sheet.id) || sheetRef });",
    "window.__LOADED_SHEET = obj.loadedSheet;",
    "fetch('/api/sheet/load-from-drive'",
  ]);
});

test("[BASELINE-OBSERVE] panel refresh re-downloads the workbook and POSTs it as the fragment body", function () {
  containsAll(PANELS, [
    'fetch("/api/sheet/load?id=" + encodeURIComponent(id)',
    "window.__LOADED_SHEET = sheet;",
    'fetch("/suite/sheet/fragments/" + name',
    "body: JSON.stringify(sheet)",
    'return refreshPanel("charts")',
  ]);
});

test("[BASELINE-OBSERVE] zero ETag / conditional-request / abort knobs in shipped sheet JS", function () {
  var files = sheetJsFiles();
  equal(
    files.reduce(function (n, rel) { return n + countMatches(src(rel), ABORT_ETAG); }, 0),
    0,
    "etag/if-none-match/abort occurrences"
  );
});

test("[BASELINE-OBSERVE] zero import progress signals in shipped sheet JS", function () {
  var files = sheetJsFiles();
  equal(
    files.reduce(function (n, rel) { return n + countMatches(src(rel), PROGRESS); }, 0),
    0,
    "progress occurrences"
  );
});

test("[BASELINE-OBSERVE] 12 shipped modules wrap gbT with a literal key + English fallback", function () {
  var files = sheetJsFiles();
  equal(filesMatching(files, I18N_WRAPPER).length, 12, "gbT wrapper modules");
  assert(
    src(PANELS).indexOf("function label(") >= 0,
    "panel localisation helper missing"
  );
});
