/**
 * Docs (#2533 large .docx path) capability probes.
 * The shipped 84-91_large_* modules are a bounded FRONTEND PREVIEW; where a
 * capability really exists in that preview it is a PASS, with the local-only
 * limitation stated in the note. Where it is absent, a GAP is recorded with a
 * concrete file:line anchor that documents the deferral or the missing guard.
 */
"use strict";

var H = require("./00_harness");
var M = require("./01_matrix");

var STATE = "botui/ui/suite/docs/modules/84_large_state.js";
var WINDOW = "botui/ui/suite/docs/modules/86_large_window.js";

function anchor(rel, pattern) { return H.findLine(rel, pattern); }

// Windowed / paged reads — PRESENT as a local preview (visibleRange/mountPlan).
H.matrixCell("Docs", "windowed_reads", "PASS", function () {
  var e = anchor(WINDOW, "function visibleRange");
  H.assert(e.line > 0);
  H.assert(M.scanAppJs(H, "Docs", M.PATTERNS.windowedRead) > 0,
    "Docs windowed-read markers must exist");
}, anchor(WINDOW, "function visibleRange"),
  "visibleRange()+mountPlan() bound mounted blocks to viewport+overscan; LOCAL preview only (#2533), not the server load path");

// Progress + cancel on import/export — ABSENT.
H.matrixCell("Docs", "progress_cancel", "GAP", function () {
  H.equal(M.scanAppJs(H, "Docs", M.PATTERNS.progressCancel), 0,
    "no AbortController/progress-bar/cancel affordance expected in Docs large modules");
  H.assert(anchor(STATE, "Deferred: streaming import").line > 0);
}, anchor(STATE, "Deferred: streaming import"),
  "no progress/cancel affordance; large modules are a local simulation and list streaming/async export as deferred");

// Virtualized rendering — PRESENT for the preview (bounded window mount).
H.matrixCell("Docs", "virtualized_render", "PASS", function () {
  H.assert(anchor(WINDOW, "OVERSCAN =").line > 0);
  H.assert(anchor(WINDOW, "function mountPlan").line > 0);
}, anchor(WINDOW, "OVERSCAN ="),
  "only the visible block range is mounted contenteditable; container is never editable — preview scope only");

// Memory caps / byte budgets — PRESENT server-side (#2530 landed after the
// 2026-11-14 issue table). The UI preview's own 2 MB ceiling remains simulated.
H.matrixCell("Docs", "memory_caps", "PASS", function () {
  H.assert(anchor(STATE, "CEILING_BYTES = 2").line > 0, "UI preview keeps its simulated ceiling");
  H.assert(anchor("botserver/crates/botdocs/src/handlers_api/admission.rs",
    "per-request memory admission").line > 0);
  H.assert(anchor("botserver/crates/botdocs/src/handlers_api/import.rs",
    "import_budget").line > 0);
}, anchor("botserver/crates/botdocs/src/handlers_api/admission.rs",
  "per-request memory admission"),
  "server has cgroup-derived per-request admission + import budget (#2530); UI large preview cap is still a local simulation");

// Documented OOM path — PRESENT server-side (typed 413/507/429 contract).
H.matrixCell("Docs", "documented_oom", "PASS", function () {
  H.assert(anchor("botserver/crates/botdocs/src/handlers_api/admission.rs",
    "413/507/429").line > 0);
  H.assert(anchor("botserver/crates/botdocs/src/handlers_api/admission.rs",
    "small_ceiling_maps_peak_estimate_to_507").line > 0);
}, anchor("botserver/crates/botdocs/src/handlers_api/admission.rs", "413/507/429"),
  "admission.rs documents and tests the stable 413/507/429 refusal contract for Docs import/export");
