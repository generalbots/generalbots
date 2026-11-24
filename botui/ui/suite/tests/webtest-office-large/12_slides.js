/**
 * Slides (#2529 large .pptx path) capability probes.
 * The shipped path opens a whole deck and re-renders presentation-view plus the
 * full slide-thumbnails fragment. No windowed read, virtualization, byte budget,
 * progress/cancel or OOM path is present — each is recorded as a GAP with an
 * explicit file:line anchor.
 */
"use strict";

var H = require("./00_harness");
var M = require("./01_matrix");

var INIT = "botui/ui/suite/slides/modules/99_init.js";

function anchor(rel, pattern) { return H.findLine(rel, pattern); }

// Windowed / paged reads — ABSENT; whole deck fetched and rendered.
H.matrixCell("Slides", "windowed_reads", "GAP", function () {
  H.assert(anchor(INIT, "fetch\\('/api/slides/").line > 0);
  H.assert(anchor(INIT, "presentation-view").line > 0);
  H.equal(M.scanAppJs(H, "Slides", M.PATTERNS.serverWindow), 0,
    "no windowed/range deck read in slides JS");
}, anchor(INIT, "fetch\\('/api/slides/"),
  "GET /api/slides/<id> plus presentation-view loads the whole deck; no windowed read");

// Progress + cancel on import/export — ABSENT (only a transient status string).
H.matrixCell("Slides", "progress_cancel", "GAP", function () {
  H.equal(M.scanAppJs(H, "Slides", M.PATTERNS.progressCancel), 0,
    "no AbortController/progress-bar/cancel affordance in slides JS");
  H.assert(anchor(INIT, "Opening deck").line > 0);
}, anchor(INIT, "Opening deck"),
  "setStatus('Opening deck...') is a one-shot label; no progress meter or cancel action for import/export");

// Virtualized rendering — ABSENT; all thumbnails and slides are rendered.
H.matrixCell("Slides", "virtualized_render", "GAP", function () {
  H.equal(M.scanAppJs(H, "Slides", M.PATTERNS.virtualization), 0,
    "no virtual grid / render window / overscan in slides JS");
  H.assert(anchor(INIT, "slide-thumbnails").line > 0);
}, anchor(INIT, "slide-thumbnails"),
  "presentation-view + slide-thumbnails fragments render the full deck and every thumbnail; no viewport windowing");

// Memory caps / byte budgets — PRESENT server-side (#2530); the client still
// fetches a whole deck, which is recorded separately in windowed_reads.
H.matrixCell("Slides", "memory_caps", "PASS", function () {
  H.assert(anchor("botserver/crates/botslides/src/handlers/admission.rs",
    "per-request memory admission and import budget").line > 0);
  H.assert(anchor("botserver/crates/botslides/src/pptx/import.rs",
    "import_budget").line > 0);
}, anchor("botserver/crates/botslides/src/handlers/admission.rs",
  "per-request memory admission and import budget"),
  "server uses a shared cgroup-derived gate + import budget for Slides (#2530); the client path still loads a whole deck");

// Documented OOM path — PRESENT through the shared typed refusal contract.
H.matrixCell("Slides", "documented_oom", "PASS", function () {
  H.assert(anchor("botserver/crates/botcore/src/shared/memory_admission/error.rs",
    "HTTP 507").line > 0);
  H.assert(anchor("botserver/crates/botslides/src/handlers/admission.rs",
    "fn reject").line > 0);
}, anchor("botserver/crates/botcore/src/shared/memory_admission/error.rs", "HTTP 507"),
  "slides reject() maps shared AdmissionError to 413/507/429; 507 documents peak-heap-over-budget");
