/**
 * [STATIC] characterizations of the #2528 criteria against shipped source.
 *
 * A [GAP] test PASSES by confirming the gap still exists — it is an honest
 * observation, never a claim the criterion is met. A [PARTIAL] test records
 * what is shipped and what is not. A [DEFERRED] test names a criterion this
 * offline audit cannot measure. Nothing here fabricates runtime evidence.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var finding = h.finding;
var src = h.src;
var exists = h.exists;
var sheetJsFiles = h.sheetJsFiles;
var rustFiles = h.rustFiles;
var filesMatching = h.filesMatching;
var countMatches = h.countMatches;
var containsAll = h.containsAll;

var IMPORT = "botserver/crates/botsheet/src/storage/import/mod.rs";
var XLSX_PARSE = "botserver/crates/botsheet/src/storage/import/xlsx.rs";
var SOURCE = "botserver/crates/botsheet/src/storage/xlsx_source.rs";
var RECALC = "botserver/crates/botsheet-core/src/recalc.rs";
var CRUD = "botserver/crates/botsheet/src/handlers/crud.rs";
var ROUTES = "botserver/crates/botsheet/src/routes.rs";
var CELL_OPS = "botserver/crates/botsheet/src/handlers/cell_ops.rs";
var PANELS = "botui/ui/suite/sheet/js/modules-sheet-advanced/22_panels.js";
var BOOT = "botui/ui/suite/sheet/sheet-inline/02_boot.js";
var LOAD_QUERY = "botserver/crates/botsheet-core/src/types/requests/document.rs";

test("[STATIC][DEFERRED] R1/A1 measured baseline table is an issue artifact, not a repo artifact", function () {
  assert(
    !exists("botui/ui/suite/tests/webtest-sheet-large/baseline.json"),
    "this harness must not own a ratchet/baseline file"
  );
  assert(!exists("botserver/crates/botsheet/bench/sheet-large"), "no shipped 5/10/25 MB fixture set");
  finding(
    "baseline table not verifiable offline",
    "acceptance requires a measured table linked from the issue; issue content and RSS/timing numbers are outside a static source audit"
  );
});

test("[STATIC][GAP] R2/A2 no streaming import, progress indicator or cancel control", function () {
  [IMPORT, XLSX_PARSE].forEach(function (rel) {
    equal(countMatches(src(rel), /stream|chunk|rows\(\)/gi), 0, "streaming tokens in " + rel);
  });
  equal(
    sheetJsFiles().reduce(function (n, rel) { return n + countMatches(src(rel), /progress/gi); }, 0),
    0,
    "progress tokens in shipped sheet JS"
  );
  equal(
    sheetJsFiles().filter(function (rel) { return /AbortController|\.abort\(/.test(src(rel)); }).length,
    0,
    "modules with an abort controller"
  );
});

test("[STATIC][GAP] R3 source package copy is still retained in RAM for normal editing", function () {
  containsAll(IMPORT, ["Some(bytes.to_vec())"]);
  containsAll(SOURCE, ["pub fn plan_for(sheet: &Spreadsheet) -> XlsxPlan"]);
  assert(src(SOURCE).indexOf("lazy") < 0, "lazy source loading unexpectedly present");
});

test("[STATIC][GAP] R4 full recalculation still runs before the import response", function () {
  containsAll(IMPORT, ["botsheet_core::recalc::recalculate_if_needed(&mut worksheets)"]);
  containsAll(XLSX_PARSE, ["botsheet_core::recalc::recalculate_if_needed(&mut worksheets)"]);
  assert(src(IMPORT).indexOf("spawn") < 0, "recalc unexpectedly detached from the request path");
});

test("[STATIC][PARTIAL] R5/A3 viewport windowing ships, but boot still transfers the whole workbook", function () {
  containsAll(LOAD_QUERY, ["pub struct LoadQuery {", "pub id: Option<String>,"]);
  containsAll(BOOT, ["fetch('/api/sheet/load?id=' + encodeURIComponent(sheetRef)"]);
  containsAll(ROUTES, ['.route("/api/sheet/range", post(handle_get_range))']);
  finding(
    "first paint still waits for whole workbook",
    "the grid fetches a bounded viewport window via /api/sheet/range, but boot() first fetches /api/sheet/load (or load-from-drive) and stores the full sheet"
  );
});

test("[STATIC][GAP] R5 no ETag or conditional request on the whole-workbook load", function () {
  assert(src(LOAD_QUERY).indexOf("etag") < 0, "LoadQuery has an ETag field");
  equal(filesMatching(rustFiles(), /If-None-Match|\bETag\b/i).length, 0, "Rust files with conditional headers");
  equal(
    filesMatching(sheetJsFiles(), /If-None-Match|\bETag\b/i).length,
    0,
    "sheet JS files with conditional headers"
  );
});

test("[STATIC][GAP] R6/A4 side-panel refresh transfers a whole workbook twice", function () {
  containsAll(PANELS, [
    'fetch("/api/sheet/load?id=" + encodeURIComponent(id)',
    "body: JSON.stringify(sheet)",
    'fetch("/suite/sheet/fragments/" + name',
  ]);
  assert(
    src(PANELS).indexOf("body: JSON.stringify(sheet)") >= 0 &&
      src(PANELS).indexOf("body: JSON.stringify({ sheet_id:") < 0,
    "panel fragment payload unexpectedly narrowed"
  );
});

test("[STATIC][PARTIAL] R7/A5 explicit save limit ships; compressed transport does not", function () {
  var routes = src(ROUTES);
  var at = routes.indexOf('"/api/sheet/save"');
  assert(at >= 0, "save route registration missing");
  var saveBlock = routes.slice(at, at + 260);
  assert(saveBlock.indexOf("body_limit(") >= 0, "save route lost its explicit limit");
  assert(src("botserver/Cargo.toml").indexOf("compression") < 0, "compression feature enabled");
  equal(filesMatching(rustFiles(), /CompressionLayer/).length, 0, "crates with a compression layer");
  finding(
    "413 beyond the save limit is a runtime criterion",
    "the 25 MB layer is visible in routes.rs; the clear-413 response for JSON >2 MB still needs a running server"
  );
});

test("[STATIC][GAP] R8 full workbook JSON remains in client global and per-window state", function () {
  containsAll(BOOT, [
    "window.__LOADED_SHEET = obj.loadedSheet;",
    "if ('loadedSheet' in obj) inst.loadedSheet = obj.loadedSheet;",
  ]);
  containsAll(PANELS, ["window.__LOADED_SHEET = sheet;"]);
});

test("[STATIC][GAP] R9/A2 no size threshold, preview mode or read-only degradation gate", function () {
  var js = sheetJsFiles();
  equal(
    js.filter(function (rel) { return /25\s*\*\s*1024|26214400|sizeThreshold|previewOnly/.test(src(rel)); }).length,
    0,
    "modules with a large-file threshold"
  );
  equal(
    js.filter(function (rel) { return /slow[- ]device|degraded[- ]mode|read[- ]only[- ]preview/i.test(src(rel)); }).length,
    0,
    "modules with a slow-device gate"
  );
});

test("[STATIC][POSITIVE] A6 one-cell edits already use an incremental dependency graph", function () {
  containsAll(CELL_OPS, [
    "use botsheet_core::dependency_graph::DepGraph;",
    "pub const MAX_RECALC_CELLS: usize = 1000;",
    "graphs[index].on_edit(&sheet.worksheets[index], &[(req.row, req.col)]);",
    "graphs[index].recalc_cascade_typed(",
  ]);
  finding(
    "issue claim now stale: incremental edit recalc",
    "A6 is already shipped in cell_ops.rs via DepGraph::on_edit + recalc_cascade_typed (MAX_RECALC_CELLS=1000); only import-time recalculation is still workbook-wide"
  );
});

test("[STATIC][GAP] A7 workbook JSON is not compressed on the wire", function () {
  equal(countMatches(src(ROUTES), /compression|gzip|brotli/gi), 0, "compression tokens in sheet routes");
  assert(src(CRUD).indexOf("Json<Spreadsheet>") >= 0, "load response type changed");
});
