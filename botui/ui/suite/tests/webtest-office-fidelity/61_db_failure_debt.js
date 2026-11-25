/**
 * #2543 — deterministic static coverage for the shipped "DB failure vs empty"
 * distinction and the 65-discarded-Results debt sweep (commit ab6c7d69c).
 *
 * Cases (all literal source assertions, no execution):
 *   (a) botcore::error_budget::DbRead is a three-variant outcome (Rows/Empty/
 *       Failed) whose from_result maps Err->Failed, Ok([])->Empty and
 *       Ok(rows)->Rows, with a unit test asserting failed != empty;
 *   (b) the KB count query in botcore::kb wires through DbRead::from_result and
 *       emits log::error! on failure instead of silently calling .ok();
 *   (c) representative pipeline files no longer discard Results with
 *       `let _ =`: each uses `if let Err(e) = ... log::warn!` at the exact
 *       sites the sweep changed.
 *
 * NON-VACUITY: every case first proves the real production files ship with
 * their real function names, enum variants, SQL and log messages and that the
 * extracted spans are non-empty; absence checks are then SCOPED to those
 * exact spans/files. Reverting the helper, the KB wiring or any listed pipeline
 * site flips the matching case to FAIL. No server, browser, cargo or network.
 */
"use strict";

var h = require("./00_harness");

var DB_READ = "botserver/crates/botcore/src/error_budget/db_read.rs";
var BUDGET_MOD = "botserver/crates/botcore/src/error_budget/mod.rs";
var KB_MOD = "botserver/crates/botcore/src/kb/mod.rs";

var PIPELINE = [
  {
    path: "botserver/src/core/bot/pipeline/front_page.rs",
    warns: [
      'log::warn!("front page: browser session close failed: {e}");',
      'log::warn!("front page: response frame not delivered: {e}");'
    ]
  },
  {
    path: "botserver/src/core/bot/pipeline/history.rs",
    warns: ['log::warn!("history: stream-start marker not delivered: {e}");']
  },
  {
    path: "botserver/src/core/bot/pipeline/start_bas.rs",
    warns: [
      'log::warn!("start.bas: drained response not delivered: {e}");',
      'log::warn!("start.bas: suggestions frame not delivered: {e}");'
    ]
  },
  {
    path: "botserver/src/core/bot/pipeline/web_context.rs",
    warns: ['log::warn!("web context: reasoning frame not delivered: {e}");']
  }
];

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1, (msg || "source must contain") + " <" + needle + ">");
}

/** Absence assertion — a forbidden token must not appear in the given span. */
function absent(src, needle, msg) {
  h.assert(src.indexOf(needle) === -1,
    (msg || "source must not contain") + " <" + needle + "> :: " + src.slice(0, 200));
}

/** Exact, non-empty source span between two literal markers (end exclusive). */
function span(src, startMarker, endMarker) {
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + ">");
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + ">");
  h.assert(end > start, "empty source span between <" + startMarker + "> and <" + endMarker + ">");
  return src.slice(start, end);
}

/** Assert `before` occurs strictly earlier than `after` inside a source span. */
function ordered(src, before, after, msg) {
  var b = src.indexOf(before);
  var a = src.indexOf(after);
  h.assert(b !== -1 && a !== -1 && b < a,
    (msg || "ordering regressed") + " :: <" + before + "> before <" + after + ">");
}

/** Count exact, literal occurrences of a needle in a source string. */
function countIn(src, needle) {
  return src.split(needle).length - 1;
}

// (a) The pure helper keeps failure distinct from an empty table, with three
// unit tests proving every mapping arm and the failed != empty inequality.
h.staticTest("#2543 DbRead has Rows/Empty/Failed; from_result maps Err->Failed, Ok([])->Empty, Ok(rows)->Rows and a test asserts failed != empty", function () {
  var src = h.readRepo(DB_READ);
  var mod = h.readRepo(BUDGET_MOD);

  // Non-vacuity: this is the real, dependency-free helper module and it is
  // re-exported from error_budget so callers can use it.
  h.assert(src.length > 1200, "DbRead helper source must be non-trivial");
  contains(src, "//! Typed outcome for DB reads under the issue #2543 error-budget ratchet.",
    "helper names #2543");
  contains(mod, "mod db_read;", "helper module is declared");
  contains(mod, "pub use db_read::DbRead;", "helper is re-exported from error_budget");
  contains(src, "pub enum DbRead<T> {", "typed outcome enum ships");

  var enumBody = span(src, "pub enum DbRead<T> {", "impl<T> DbRead<T> {");
  contains(enumBody, "Rows(Vec<T>)", "Rows variant ships");
  contains(enumBody, "Empty,", "Empty variant ships");
  contains(enumBody, "Failed,", "Failed variant ships");
  contains(enumBody, "Never collapse this into [`DbRead::Empty`].",
    "enum documents that Failed is never collapsed into Empty");

  var mapping = span(src, "pub fn from_result<E>(result: Result<Vec<T>, E>) -> Self {",
    "/// Whether the read failed");
  contains(mapping, "Ok(rows) if rows.is_empty() => DbRead::Empty,", "Ok([]) maps to Empty");
  contains(mapping, "Ok(rows) => DbRead::Rows(rows),", "Ok(rows) maps to Rows");
  contains(mapping, "Err(_) => DbRead::Failed,", "Err maps to Failed");
  ordered(mapping, "Ok(rows) if rows.is_empty() => DbRead::Empty,",
    "Err(_) => DbRead::Failed,", "the failure arm must be present after the empty arm");

  // All three unit tests ship, including the explicit inequality assertion.
  contains(src, "fn error_is_failed_and_distinct_from_empty()", "failure-vs-empty test ships");
  contains(src, "fn ok_empty_is_empty()", "Ok([]) test ships");
  contains(src, "fn ok_rows_are_returned()", "Ok(rows) test ships");

  var failureTest = span(src, "fn error_is_failed_and_distinct_from_empty()",
    "#[test]\n    fn ok_empty_is_empty()");
  contains(failureTest, 'DbRead::<u8>::from_result(Err("connection refused"))',
    "test feeds a real Err into from_result");
  contains(failureTest, "assert_eq!(failed, DbRead::Failed);", "test asserts Err->Failed");
  contains(failureTest, "DbRead::<u8>::from_result::<&str>(Ok(Vec::new()))",
    "test feeds an empty Ok into from_result");
  contains(failureTest, "assert_eq!(empty, DbRead::Empty);", "test asserts Ok([])->Empty");
  contains(failureTest, "failed != empty,",
    "test explicitly asserts a failure is distinguishable from an empty table");
  ordered(failureTest, "assert_eq!(failed, DbRead::Failed);", "failed != empty,",
    "the inequality assertion must follow the two mapping assertions");
});

// (b) The KB count query keeps the failure signal instead of calling .ok().
h.staticTest("#2543 KB count query uses DbRead::from_result and logs error! on failure instead of .ok()", function () {
  var src = h.readRepo(KB_MOD);

  // Non-vacuity: this is the real KB manager with the typed read wired in.
  h.assert(src.length > 6000, "KB manager source must be non-trivial");
  contains(src, "use crate::error_budget::DbRead;", "KB imports the typed read helper");
  contains(src, "pub async fn handle_gbkb_change(", "KB change handler ships");
  contains(src, "struct KbDocCount {", "count-query row type ships");

  var query = span(src, "let existing = DbRead::<KbDocCount>::from_result(",
    "if let Some(row) = row {");
  contains(query, "SELECT document_count FROM kb_collections WHERE bot_id = $1 AND name = $2",
    "the real count-query SQL ships");
  contains(query, ".load(&mut conn),", "the fallible load feeds from_result");
  contains(query, "let failed = existing.is_failed();", "failure flag is read from the outcome");
  contains(query, "DbRead::Rows(rows) => rows.into_iter().next(),",
    "Rows arm is matched");
  contains(query, "DbRead::Empty | DbRead::Failed => None,",
    "Empty and Failed both fall through to re-index");
  contains(query, "if failed {", "failure branch ships");
  contains(query, 'error!("KB count query failed for bot',
    "failure is logged with error! on the real call site");
  ordered(query, "let existing = DbRead::<KbDocCount>::from_result(",
    'error!("KB count query failed for bot', "the query must run before the failure is logged");
  ordered(query, "let failed = existing.is_failed();",
    'error!("KB count query failed for bot',
    "the failure flag must be read before the failure is logged");

  // The old silent-discard patterns must be gone from this span: reverting to
  // `.ok()` or `.unwrap_or_default()` would collapse Failed into Empty.
  absent(query, ".ok()", "silent .ok() discard must be gone from the count query");
  absent(query, "unwrap_or_default", "unwrap_or_default discard must be gone from the count query");
});

// (c) The 65-result sweep replaced `let _ =` with `if let Err(e) ... log::warn!`
// at the listed pipeline sites. Checks are scoped to four representative files
// because unrelated `let _ =` sites remain elsewhere in the same directory.
h.staticTest("#2543 representative pipeline files use if let Err(e) + log::warn! and retain no let _ = at the changed sites", function () {
  var totalIfLet = 0;
  var totalWarn = 0;

  PIPELINE.forEach(function (entry) {
    var src = h.readRepo(entry.path);
    h.assert(src.length > 900, "non-vacuity: " + entry.path + " must be non-trivial");
    h.assert(countIn(src, "if let Err(e) =") >= 1,
      "non-vacuity: " + entry.path + " must use if let Err(e) =");
    h.assert(countIn(src, "log::warn!") >= 1,
      "non-vacuity: " + entry.path + " must call log::warn!");
    absent(src, "let _ =", "discarded Result must be gone in " + entry.path);

    entry.warns.forEach(function (warn) {
      contains(src, warn, "log message must ship in " + entry.path);
    });
    totalIfLet += countIn(src, "if let Err(e) =");
    totalWarn += countIn(src, "log::warn!");
  });

  h.assert(totalIfLet >= 6,
    "non-vacuity: the four pipeline files must show at least 6 if let Err(e) sites, got " + totalIfLet);
  h.assert(totalWarn >= 6,
    "non-vacuity: the four pipeline files must show at least 6 log::warn! calls, got " + totalWarn);

  // Ordering inside the two front-page sites: the guard precedes its warning.
  var front = h.readRepo(PIPELINE[0].path);
  ordered(front, "if let Err(e) = session.close().await {",
    'log::warn!("front page: browser session close failed: {e}");',
    "front page close must log its error rather than discard it");
  ordered(front, "if let Err(e) = sink.send_bot_response(&response).await {",
    'log::warn!("front page: response frame not delivered: {e}");',
    "front page response frame must log its error rather than discard it");
});

h.note("FOLLOW-UP #2543: static coverage only — the typed DbRead failure/empty distinction, its KB count-query wiring with error! logging, and the representative pipeline sites that now log warned Results instead of discarding them. No browser, server or cargo was run; the driver owns issue closure.");
