/**
 * Sheets (#2528 large .xlsx path) capability probes.
 * The virtualised grid is genuinely present and is a PASS with file:line.
 * Everything around it (windowed wire reads, progress/cancel, request byte
 * budgets, OOM path) is recorded as GAP with concrete anchors.
 */
"use strict";

var H = require("./00_harness");
var M = require("./01_matrix");

var GRID = "botui/ui/suite/sheet/js/constants/03_grid_render.js";
var GEOM = "botui/ui/suite/sheet/js/constants/02_grid_geometry.js";
var LOAD = "botui/ui/suite/sheet/js/modules-sheet-advanced/22_panels.js";
var INGEST = "botui/ui/suite/sheet/js/modules-sheet-advanced/63_ingest_batch.js";
var XLSX = "botserver/crates/botsheet/src/storage/import/xlsx.rs";
var ROUTES = "botserver/crates/botsheet/src/routes.rs";

function anchor(rel, pattern) { return H.findLine(rel, pattern); }

// Windowed / paged reads — ABSENT over the wire; whole workbook JSON is loaded.
H.matrixCell("Sheets", "windowed_reads", "GAP", function () {
  H.assert(anchor(LOAD, "/api/sheet/load\\?id=").line > 0);
  H.equal(M.scanAppJs(H, "Sheets", M.PATTERNS.serverWindow), 0,
    "no /load?range / start_row window parameter in shipped sheet code");
  H.assert(anchor(GEOM, "visibleRowRange").line > 0);
}, anchor(LOAD, "/api/sheet/load\\?id="),
  "grid requests a local visible range, but /api/sheet/load?id= returns the whole workbook JSON — no windowed server read");

// Progress + cancel on import/export — ABSENT (ingest states are simulated).
H.matrixCell("Sheets", "progress_cancel", "GAP", function () {
  H.equal(M.scanAppJs(H, "Sheets", M.PATTERNS.progressCancel), 0,
    "no AbortController/progress-bar/cancel affordance in sheet code");
  H.assert(anchor(INGEST, "All transitions are simulated").line > 0);
}, anchor(INGEST, "All transitions are simulated"),
  "ingest exposes queued/parsing/review statuses only; no cancellable import with progress");

// Virtualized rendering — PRESENT (cell pool + visible row range).
H.matrixCell("Sheets", "virtualized_render", "PASS", function () {
  H.assert(anchor(GRID, '"virtual-grid"').line > 0);
  H.assert(anchor(GEOM, "visibleRowRange").line > 0);
  H.assert(anchor(GRID, "getOrCreateNode").line > 0);
}, anchor(GRID, '"virtual-grid"'),
  "VirtualGrid paints only visibleRowRange()/visibleColRange() via a reused cell pool; the one rendering capability shipped across the three apps");

// Memory caps / byte budgets — PRESENT server-side (#2530 landed after the
// issue table was written): cgroup-derived import budget + streaming row count
// before the workbook DOM is built, plus a per-sheet cell cap.
H.matrixCell("Sheets", "memory_caps", "PASS", function () {
  H.assert(anchor(XLSX, "import_budget").line > 0, "budget checked before DOM build");
  H.assert(anchor(XLSX, "count_rows_streaming").line > 0, "bounded streaming pre-scan");
  H.assert(anchor(XLSX, "cell import limit").line > 0, "per-sheet cell cap");
}, anchor(XLSX, "import_budget"),
  "xlsx import checks a cgroup import budget via an 8 KiB streaming row count before umya builds the workbook DOM (#2530)");

// Documented OOM path — PRESENT server-side (typed 413/507/429 contract).
H.matrixCell("Sheets", "documented_oom", "PASS", function () {
  H.assert(anchor(ROUTES, "upload_policy::body_limit").line > 0, "route body limit still wired");
  H.assert(anchor("botserver/crates/botsheet/src/handlers/admission.rs",
    "413/507/429").line > 0);
  H.assert(anchor("botserver/crates/botsheet/src/handlers/admission.rs",
    "small_ceiling_maps_peak_estimate_to_507").line > 0);
}, anchor("botserver/crates/botsheet/src/handlers/admission.rs", "413/507/429"),
  "admission.rs pins the stable 413/507/429 typed refusal contract, including a peak-RSS -> 507 test");
