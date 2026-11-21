/**
 * [BASELINE-OBSERVE] characterizations of the shipped botserver XLSX path.
 *
 * Facts here are read from source only. No timing, RSS, transfer size or HTTP
 * status is measured: those need a running server and belong to a browser/CI
 * gate. A few observations contradict the issue's 2025 code snapshot; those
 * are recorded with h.finding() and never patched here.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var finding = h.finding;
var src = h.src;
var exists = h.exists;
var rustFiles = h.rustFiles;
var filesMatching = h.filesMatching;
var countMatches = h.countMatches;
var containsAll = h.containsAll;
var containsNone = h.containsNone;

var IMPORT = "botserver/crates/botsheet/src/storage/import/mod.rs";
var XLSX_PARSE = "botserver/crates/botsheet/src/storage/import/xlsx.rs";
var SOURCE = "botserver/crates/botsheet/src/storage/xlsx_source.rs";
var RECALC = "botserver/crates/botsheet-core/src/recalc.rs";
var CRUD = "botserver/crates/botsheet/src/handlers/crud.rs";
var RANGE_HANDLER = "botserver/crates/botsheet/src/handlers/cell_structure.rs";
var ROUTES = "botserver/crates/botsheet/src/routes.rs";
var LOAD_QUERY = "botserver/crates/botsheet-core/src/types/requests/document.rs";
var RANGE_TYPE = "botserver/crates/botsheet-core/src/types/requests_feature.rs";
var POLICY = "botserver/crates/botsecurity-core/src/upload_policy.rs";
var TRANSPORT = "botserver/crates/botsecurity-core/src/upload_transport.rs";
var LAYERS = "botserver/src/main_module/server/server_layers.rs";
var CARGO = "botserver/Cargo.toml";

test("[BASELINE-OBSERVE] 165 botsheet + botsheet-core Rust files scanned", function () {
  equal(rustFiles().length, 165, "Rust source file count");
});

test("[BASELINE-OBSERVE] import lives in storage/import/{mod,xlsx}.rs, not the issue's import.rs", function () {
  assert(exists(IMPORT), "storage/import/mod.rs missing");
  assert(exists(XLSX_PARSE), "storage/import/xlsx.rs missing");
  assert(!exists("botserver/crates/botsheet/src/storage/import.rs"), "stale flat import.rs now exists");
  finding(
    "issue path drift",
    "#2528 cites botsheet/src/storage/import.rs; the shipped tree splits it into storage/import/mod.rs and storage/import/xlsx.rs"
  );
});

test("[BASELINE-OBSERVE] whole file buffer is parsed by one umya read_reader call", function () {
  containsAll(IMPORT, [
    "pub fn import_spreadsheet_bytes(",
    "bytes: &[u8]",
    'parse_xlsx_to_worksheets(bytes, "xlsx")?',
  ]);
  containsAll(XLSX_PARSE, [
    "pub fn parse_xlsx_to_worksheets(",
    "bytes: &[u8]",
    "umya_spreadsheet::reader::xlsx::read_reader(std::io::Cursor::new(package), true)",
  ]);
  assert(countMatches(src(XLSX_PARSE), /read_reader\(/g) === 1, "unexpected extra reader call");
});

test("[BASELINE-OBSERVE] no chunked/streaming xlsx reader on the import path", function () {
  [IMPORT, XLSX_PARSE].forEach(function (rel) {
    containsNone(rel, ["stream", "Chunk", "chunk", "rows()", "Iterator"], "import path " + rel);
  });
});

test("[BASELINE-OBSERVE] full recalculation runs on import with MAX_PASSES = 12", function () {
  containsAll(IMPORT, [
    "botsheet_core::recalc::recalculate_if_needed(&mut worksheets)",
    "botsheet_core::chart_data::refresh_chart_data(&mut worksheets)",
  ]);
  containsAll(RECALC, [
    "pub const MAX_PASSES: usize = 12;",
    "if needs_recalculation(worksheets)",
    "pub fn recalculate_workbook(worksheets: &mut [Worksheet]) -> usize",
    "let snapshot: Vec<Worksheet> = worksheets.to_vec();",
  ]);
  equal(countMatches(src(IMPORT) + src(XLSX_PARSE), /recalculate_if_needed\(/g), 2,
    "recalculate call sites on the xlsx import path");
  finding(
    "redundant full-workbook scan",
    "recalculate_if_needed is called both inside parse_xlsx_to_worksheets and again in import_spreadsheet_bytes; the second call is a no-op after the first but still walks every cell"
  );
});

test("[BASELINE-OBSERVE] source package copy is retained in the in-memory model", function () {
  containsAll(IMPORT, [
    "source_bytes: if super::xlsx_source::retains_snapshot(detected) {",
    "Some(bytes.to_vec())",
  ]);
  containsAll(SOURCE, [
    "pub fn retains_snapshot(detected_format: &str) -> bool",
    'matches!(detected_format, "xlsx" | "xlsm" | "xls" | "xlsb" | "ods")',
    "pub fn plan_for(sheet: &Spreadsheet) -> XlsxPlan",
    "source_bytes.as_deref()",
  ]);
});

test("[BASELINE-OBSERVE] /api/sheet/load clones the whole Spreadsheet into one JSON response", function () {
  containsAll(CRUD, [
    "pub async fn handle_load_sheet(",
    "let mut sheet = session.sheet.read().await.clone();",
    "Ok(Json(sheet))",
  ]);
  containsAll(LOAD_QUERY, ["pub struct LoadQuery {", "pub id: Option<String>,"]);
  containsAll(ROUTES, ['.route("/api/sheet/load", get(handle_load_sheet))']);
  assert(src(LOAD_QUERY).indexOf("range") < 0, "LoadQuery unexpectedly has a range field");
  equal(rustFiles().filter(function (rel) {
    return /etag|if-none-match/i.test(src(rel));
  }).length, 0, "Rust files with ETag/If-None-Match");
});

test("[BASELINE-OBSERVE] windowed /api/sheet/range response is narrow (cells + dimensions)", function () {
  containsAll(RANGE_HANDLER, [
    "pub async fn handle_get_range(",
    "for row in req.start_row..=req.end_row {",
    "for col in req.start_col..=req.end_col {",
    "Ok(Json(RangeResponse {",
  ]);
  containsAll(RANGE_TYPE, [
    "pub struct RangeResponse {",
    "pub cells: std::collections::HashMap<String, super::core::CellData>,",
    "pub total_rows: u32,",
    "pub total_cols: u32,",
    "pub range_start: String,",
    "pub range_end: String,",
  ]);
  containsAll(ROUTES, ['.route("/api/sheet/range", post(handle_get_range))']);
  finding(
    "wire windowing already shipped",
    "a bounded /api/sheet/range handler and the client viewport caller exist; the gap is the boot/panel whole-workbook loads, not the range endpoint"
  );
});

test("[BASELINE-OBSERVE] import cap is 25 MB for spreadsheet formats", function () {
  containsAll(POLICY, [
    "static SHEET_IMPORT_LIMITS: UploadLimits = UploadLimits {",
    "max_bytes: 25 * 1024 * 1024,",
    'extensions: &["xlsx", "xls", "ods", "csv", "tsv", "txt"]',
    "UploadClass::SpreadsheetImport => &SHEET_IMPORT_LIMITS,",
  ]);
  containsAll(TRANSPORT, [
    "pub fn body_limit(class: UploadClass) -> DefaultBodyLimit {",
    "DefaultBodyLimit::max(max_bytes(class))",
  ]);
});

test("[BASELINE-OBSERVE] save route carries the same 25 MB limit as import", function () {
  var routes = src(ROUTES);
  var at = routes.indexOf('"/api/sheet/save"');
  assert(at >= 0, "save route registration missing");
  var block = routes.slice(at, at + 260);
  equal(routes.split('\"/api/sheet/save\"').length - 1, 1, "save route registrations");
  assert(block.indexOf("body_limit(") >= 0, "save route lost its explicit body limit");
  assert(
    block.indexOf("UploadClass::SpreadsheetImport") >= 0,
    "save route lost its SpreadsheetImport upload class"
  );
  finding(
    "issue claim now stale: save limit",
    "#2528 says /api/sheet/save has no body_limit (2 MB axum default); routes.rs:68 now applies SpreadsheetImport body_limit (25 MB). The 413 response still needs runtime verification"
  );
});

test("[BASELINE-OBSERVE] no HTTP compression layer is registered", function () {
  assert(src(LAYERS).indexOf("CompressionLayer") < 0, "compression layer unexpectedly registered");
  assert(src(CARGO).indexOf("compression") < 0, "tower-http compression feature unexpectedly enabled");
  containsAll(CARGO, ['tower-http = { workspace = true, features = ["cors", "fs", "trace"] }']);
  assert(filesMatching(rustFiles(), /CompressionLayer/).length === 0,
    "a crate registers a compression layer");
});
