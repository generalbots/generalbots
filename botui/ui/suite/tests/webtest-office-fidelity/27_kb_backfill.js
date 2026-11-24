/**
 * [STATIC] deterministic guards for the boot-time Docs KB backfill (#2499,
 * tracked under #2457). Every case reads real shipped Rust source as text and
 * FAILS if the contract regresses: no runtime claim, no server, no browser.
 *
 * Covered contracts:
 *   - the backfill lists persisted keys under `users/{owner}/docs/*.html`,
 *     parses them strictly, and calls `project_to_kb` for each document
 *   - listing/read failures log a warning and never propagate into startup
 *     (plain `usize` return, no `?`/panic on the boot path)
 *   - the boot invocation binds and logs the reindexed count
 *   - it runs once from the router-merge boot task, not a per-request handler
 */
"use strict";

var h = require("./00_harness");

var KB_BACKFILL = "botserver/crates/botdocs/src/handlers_api/kb_backfill.rs";
var DOC_STATE = "botserver/crates/botdocs/src/state.rs";
var BOOT = "botserver/src/main_module/routes/feature_merge_office/sub_routes.rs";

var FN_MARKER = "pub async fn reindex_persisted_documents(state: &DocState) -> usize {";

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(relPath, needle, msg) {
  var src = h.readRepo(relPath);
  h.assert(src.indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath);
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

// 1. Only real document HTML is replayed: the namespace prefix is listed, the
//    key shape is parsed strictly (sidecars rejected), and each parsed document
//    is projected into the in-memory KB exactly once.
h.staticTest("KB backfill lists users/{owner}/docs keys and projects each .html document", function () {
  contains(KB_BACKFILL, 'const USERS_PREFIX: &str = "users/";',
    "Drive namespace prefix stays users/");
  contains(KB_BACKFILL, ".list_objects(&state.bucket_name, Some(USERS_PREFIX))",
    "backfill lists persisted objects under the users/ prefix");

  var parser = between(KB_BACKFILL, "fn parse_document_key(key: &str)", "#[cfg(test)]");
  h.assert(parser.indexOf('if segments.next()? != "docs" {') !== -1,
    "key parser requires the docs path segment :: <" + parser + ">");
  h.assert(parser.indexOf('file.strip_suffix(".html")?') !== -1,
    "key parser accepts only .html documents :: <" + parser + ">");
  h.assert(parser.indexOf("segments.next().is_some()") !== -1,
    "key parser rejects extra path segments :: <" + parser + ">");
  contains(KB_BACKFILL,
    'parse_document_key("users/owner-1/docs/doc-1.meta.json").is_none()',
    "unit test pins sidecar rejection");

  var body = between(KB_BACKFILL, FN_MARKER, "/// Extract");
  h.assert(body.indexOf("for key in objects {") !== -1,
    "listed keys are iterated :: <" + body + ">");
  h.assert(body.indexOf("let Some((owner, doc_id)) = parse_document_key(&key)") !== -1,
    "each key goes through the strict parser :: <" + body + ">");
  h.assert(body.indexOf("state.project_to_kb(") !== -1,
    "each parsed document is projected into the KB :: <" + body + ">");
  h.assert(body.indexOf("indexed += 1;") !== -1,
    "projected documents are counted :: <" + body + ">");
});

// 2. Best effort is structural, not just documented: the function returns a
//    bare count (never Result), both failure branches warn and continue/return
//    0, and no error-propagation or panic token exists on the boot path.
h.staticTest("KB backfill listing/read failures warn and never fail startup", function () {
  var body = between(KB_BACKFILL, FN_MARKER, "/// Extract");
  h.assert(body.indexOf(FN_MARKER) === 0,
    "backfill returns a plain usize count, not Result :: <" + body + ">");

  var listErr = body.indexOf("Err(error) => {");
  var listWarn = body.indexOf("log::warn!(", listErr);
  var listReturn = body.indexOf("return 0;", listErr);
  h.assert(listErr !== -1 && listWarn !== -1 && listReturn !== -1,
    "listing failure warns and returns 0 :: <" + body + ">");
  h.assert(listErr < listWarn && listWarn < listReturn,
    "listing error branch warns before bailing out gracefully :: <" + body + ">");
  contains(KB_BACKFILL, "docs: KB backfill could not list persisted documents",
    "listing failure is logged as a warning");

  h.assert(body.indexOf("Err(_) => failed += 1,") !== -1,
    "unreadable documents are counted and skipped :: <" + body + ">");
  h.assert(body.indexOf("if failed > 0 {") !== -1,
    "skipped reads are reported once :: <" + body + ">");
  contains(KB_BACKFILL, "docs: KB backfill skipped {failed} persisted document(s)",
    "read failures are logged as a warning");

  ["?", ".unwrap(", ".expect(", "panic!(", "todo!(", "unimplemented!("].forEach(function (forbidden) {
    h.assert(body.indexOf(forbidden) === -1,
      "boot path must not contain <" + forbidden + "> :: <" + body + ">");
  });

  contains(DOC_STATE, "pub async fn reindex_persisted_documents(&self) -> usize {",
    "state delegate also returns a plain count");
  contains(DOC_STATE, "crate::handlers_api::kb_backfill::reindex_persisted_documents(self).await",
    "state delegate awaits the backfill without propagating an error");
});

// 3. The boot caller must observe the result: bind the count and log it,
//    guarded so a clean boot with nothing persisted stays quiet.
h.staticTest("Boot invocation binds and logs the reindexed document count", function () {
  var src = h.readRepo(BOOT);
  var spawn = src.indexOf("tokio::spawn(async move {");
  h.assert(spawn !== -1, "boot task present in router-merge module");
  var block = src.slice(spawn, src.indexOf("\n        });", spawn));

  var call = block.indexOf("let reindexed = legacy_state.reindex_persisted_documents().await;");
  h.assert(call !== -1, "reindex result is bound, not discarded :: <" + block + ">");
  var guard = block.indexOf("if reindexed > 0 {");
  var log = block.indexOf('log::info!("botdocs: reindexed {reindexed} persisted documents into KB");');
  h.assert(guard !== -1 && log !== -1,
    "reindex count is logged under a positive-count guard :: <" + block + ">");
  h.assert(call < guard && guard < log,
    "call must precede the guard and the log :: <" + block + ">");
});

// 4. Idempotence of placement: the call lives in the one-shot router-merge
//    boot task alongside legacy namespace maintenance, BEFORE the docs router
//    is registered — never inside an axum request handler.
h.staticTest("KB backfill runs once at router-merge boot, not per request", function () {
  var src = h.readRepo(BOOT);
  var mergeFn = src.indexOf("pub(super) async fn merge_office_sub_routes(");
  var cfgDocs = src.indexOf('#[cfg(feature = "docs")]');
  var spawn = src.indexOf("tokio::spawn(async move {");
  var call = src.indexOf("reindex_persisted_documents().await");
  var docsMerge = src.indexOf("crate::docs::configure_docs_routes()");
  h.assert(mergeFn !== -1, "router-merge boot function present");
  h.assert(cfgDocs !== -1 && docsMerge !== -1,
    "docs feature block and docs router registration present");
  h.assert(mergeFn < cfgDocs && cfgDocs < spawn && spawn < call && call < docsMerge,
    "backfill must sit in the docs boot block before route registration");
  h.assert(src.indexOf("botdocs::legacy::adopt_legacy_namespace", spawn) < call &&
    src.indexOf("botdocs::legacy::audit_legacy_namespace", spawn) < call,
    "backfill runs alongside the one-shot legacy namespace maintenance");
  var prelude = src.slice(spawn, call);
  h.assert(!/\.route\(|axum::extract|Request<Body>/.test(prelude),
    "no request handler wraps the boot-time backfill :: <" + prelude + ">");
});
