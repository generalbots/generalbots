/**
 * [STATIC] deterministic coverage for the #2435 recalc budget guard and the
 * recalc duration metric hook. Every case reads real shipped Rust source as
 * text and FAILS if a contract regresses: no runtime claim, no server, no
 * browser, no cargo.
 *
 * Covered contracts:
 *   (a) both the single-cell edit and the paste paths preflight the cascade
 *       BEFORE any cell is mutated, and an over-budget refusal is an HTTP 400
 *       with a fixed, actionable message
 *   (b) the estimate is priced from WorkbookTier::classify +
 *       MemoryBudget::for_tier and refuses work that exceeds the tier budget
 *   (c) successful recalcs record a real measured duration through
 *       record_office_recalc_optional with app="sheet" and no PII labels
 *   (d) pure unit tests pin refusal, within-budget and saturation behaviour
 *
 * Assertions are literal source substrings: removing any hook or test makes
 * the matching case FAIL. No user-facing UI string is introduced by this file.
 */
"use strict";

var h = require("./00_harness");

var CELL_OPS = "botserver/crates/botsheet/src/handlers/cell_ops.rs";
var GRID_OPS = "botserver/crates/botsheet/src/handlers/grid_ops.rs";
var BUDGET = "botserver/crates/botsheet/src/scale/budget.rs";
var SCALE_TESTS = "botserver/crates/botsheet/src/scale/tests.rs";
var COLLECT = "botserver/crates/botcore/src/shared/analytics/collect.rs";

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(relPath, needle, msg) {
  h.assert(
    h.readRepo(relPath).indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath
  );
}

/** Return the exact source span between two markers (exclusive of the end). */
function between(relPath, startMarker, endMarker) {
  var src = h.readRepo(relPath);
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + "> in " + relPath);
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + "> in " + relPath);
  return src.slice(start, end);
}

/** Assert `before` occurs strictly earlier than `after` inside a source span. */
function ordered(span, before, after, msg) {
  var first = span.indexOf(before);
  var second = span.indexOf(after);
  h.assert(first !== -1 && second !== -1 && first < second,
    (msg || "ordering regressed") + " :: <" + before + "> before <" + after +
    "> in <" + span.slice(0, 120) + ">");
}

// (a) Both mutating handlers price the cascade before writing any cell. The
//     guard maps an estimate failure to a fixed HTTP 400 JSON body.
h.staticTest("Sheet edit and paste preflight the cascade before mutating; refusal is HTTP 400", function () {
  var edit = between(CELL_OPS, "pub async fn handle_update_cell(", "sheet.updated_at = Utc::now();");
  ordered(edit,
    "preflight_recalc(&sheet.worksheets[index], &[(req.row, req.col)])?;",
    "cell.value = value;",
    "single-cell edit must be priced before the value is written");
  h.assert(edit.indexOf("let recalc_started = std::time::Instant::now();") !== -1,
    "edit starts the measured clock before the cascade :: <" + edit.slice(0, 120) + ">");

  var paste = between(GRID_OPS, "pub async fn handle_paste(", "sheet.updated_at = Utc::now();");
  ordered(paste,
    "preflight_recalc(&sheet.worksheets[req.worksheet_index], &paste_roots)?;",
    "entry.value = payload.display;",
    "paste must be priced before any pasted cell is written");
  h.assert(paste.indexOf("let recalc_started = std::time::Instant::now();") !== -1,
    "paste starts the measured clock before the cascade :: <" + paste.slice(0, 120) + ">");

  var guard = between(CELL_OPS, "pub(crate) fn guard_recalc_budget(", "/// Count formulas reachable");
  h.assert(guard.indexOf("StatusCode::BAD_REQUEST") !== -1,
    "budget refusal is a 400, not a 500 or a silent success :: <" + guard + ">");
  h.assert(guard.indexOf('let message = format!("Recalculation refused: {reason}; undo or split the edit");') !== -1,
    "refusal carries the fixed actionable message :: <" + guard + ">");
  h.assert(guard.indexOf('Json(serde_json::json!({ "error": message }))') !== -1,
    "refusal returns the message as a JSON error body :: <" + guard + ">");
  h.assert(guard.indexOf('"recalculation memory budget exceeded"') !== -1 &&
    guard.indexOf('"too many dependent formulas"') !== -1,
    "both budget error classes get a specific reason :: <" + guard + ">");
  contains(CELL_OPS, "pub(crate) fn preflight_recalc(", "edit-side preflight helper present");
  contains(GRID_OPS, "use crate::handlers::cell_ops::{MAX_RECALC_CELLS, preflight_recalc};",
    "paste reuses the same preflight helper, not a private copy");
});

// (b) The price comes from the tier model, not an arbitrary constant.
h.staticTest("Recalc estimate is priced from the workbook tier budget and refuses over-budget work", function () {
  var guard = between(CELL_OPS, "pub(crate) fn guard_recalc_budget(", "/// Count formulas reachable");
  h.assert(guard.indexOf("let budget = MemoryBudget::for_tier(WorkbookTier::classify(cells, total_formulas));") !== -1,
    "tier is classified from the workbook, then budgeted :: <" + guard + ">");
  h.assert(guard.indexOf("estimate_recalc(&request, &budget).map_err(|error| {") !== -1,
    "estimate failure is the refusal path :: <" + guard + ">");
  h.assert(guard.indexOf("bytes_per_cell: RECALC_BYTES_PER_CELL,") !== -1,
    "estimate uses the declared per-cell cost :: <" + guard + ">");

  contains(CELL_OPS, "const RECALC_BYTES_PER_CELL: u64 = 1_024;", "per-cell cost is a fixed constant");
  contains(BUDGET, "pub fn classify(cells: u64, formulas: u64) -> Self {",
    "tier classifier is a pure function of size");
  contains(BUDGET, "pub const fn for_tier(tier: WorkbookTier) -> Self {",
    "immutable canonical budget per tier");
  contains(BUDGET, "pub enum BudgetError {", "every refusal is a typed BudgetError");
  contains(BUDGET, "RecalcOverBudget {", "over-budget refusal variant exists");
  contains(BUDGET, "TooManyFormulas {", "formula-ceiling refusal variant exists");

  var preflight = between(CELL_OPS, "pub(crate) fn preflight_recalc(", "/// Excel-style protection");
  h.assert(preflight.indexOf("let dependents = reachable.len().saturating_sub(roots.len()) as u64;") !== -1,
    "dependents are actually counted, so the estimate is non-vacuous :: <" + preflight + ">");
  h.assert(preflight.indexOf("guard_recalc_budget(") !== -1,
    "preflight delegates to the budget guard :: <" + preflight + ">");
});

// (c) A successful recalc records a genuinely measured duration under a
//     sanitized, PII-free label; (d) pure tests pin the scale primitives.
h.staticTest("Successful recalcs record measured, PII-safe duration with refusal/budget tests", function () {
  [
    { path: CELL_OPS, start: "pub async fn handle_update_cell(", kind: "edit" },
    { path: GRID_OPS, start: "pub async fn handle_paste(", kind: "paste" }
  ].forEach(function (entry) {
    var span = between(entry.path, entry.start, "sheet.updated_at = Utc::now();");
    h.assert(span.indexOf("let recalc_ms = recalc_started.elapsed().as_secs_f64() * 1000.0;") !== -1,
      entry.kind + " duration is measured with Instant::elapsed, not a hardcoded constant");
    h.assert(span.indexOf('record_office_recalc_optional(Some(state.metrics_collector.as_ref()), recalc_ms, "sheet").await;') !== -1,
      entry.kind + " measured duration is recorded with app=\"sheet\"");
  });

  contains(COLLECT, 'pub const OFFICE_RECALC_DURATION_MS: &str = "office_recalc_duration_ms";',
    "stable canonical metric name");
  contains(COLLECT, "pub async fn record_office_recalc_optional(", "optional recorder entry point");
  contains(COLLECT, "record_duration_distribution(collector, OFFICE_RECALC_DURATION_MS, scope, duration_ms).await;",
    "duration distribution is actually emitted");
  contains(COLLECT, 'scope.insert("app".to_string(), sanitize_label(app));',
    "app label is sanitized before it reaches the exporter");
  contains(COLLECT, "fn sanitize_label(raw: &str) -> String {", "label sanitizer is present");
  contains(COLLECT, "no file name, user id or tenant id is accepted.",
    "documented PII boundary for the metric label");

  contains(SCALE_TESTS, "fn budget_refuses_over_budget_recalculation() {", "refusal test exists");
  contains(SCALE_TESTS, "fn incremental_estimate_prices_only_the_affected_cells() {", "within-budget test exists");
  contains(SCALE_TESTS, "fn large_workbook_plan_stays_under_cap_and_is_deterministic() {",
    "saturated/capped workbook test exists");
  h.assert(h.readRepo(SCALE_TESTS).indexOf("assert!(estimate.estimated_bytes <= budget.max_recalc_bytes);") !== -1,
    "within-budget test asserts the estimate stays under the tier cap");
  h.assert(h.readRepo(SCALE_TESTS).indexOf("Err(BudgetError::RecalcOverBudget {") !== -1,
    "saturation test asserts the concrete over-budget error");
});
