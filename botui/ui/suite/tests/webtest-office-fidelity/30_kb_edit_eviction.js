/**
 * [STATIC] deterministic guards for the #2499 "edit leaves a stale KB point"
 * regression. Every case reads real shipped Rust source as text and FAILS if a
 * contract is removed: no runtime claim, no server, no browser, no cargo.
 *
 * Covered contracts:
 *   (a) `project_to_kb` evicts `{doc_id}.html` points via `remove_from_kb`
 *       BEFORE it rebuilds and inserts the new points (call ordering in source)
 *   (b) `remove_from_kb` removes with the same `{doc_id}.html` prefix used when
 *       projecting; a busy lock is logged and both helpers stay infallible
 *       (plain `pub fn`, no Result) so an edit never fails a successful save
 *   (c) the `editing_document_replaces_indexed_points` regression test exists,
 *       is wired into the crate, and asserts old-term absence + new-term
 *       presence after a second projection of the same document id
 *
 * All assertions are literal source substrings. Deleting the eviction call,
 * changing the removal prefix, or dropping either half of the regression test
 * makes the matching case FAIL. No user-facing string is introduced here.
 */
"use strict";

var h = require("./00_harness");

var DOC_STATE = "botserver/crates/botdocs/src/state.rs";
var KB_TESTS = "botserver/crates/botdocs/src/state_kb_tests.rs";

var PROJECT_FN = "pub fn project_to_kb(&self, user_id: &str, doc_id: &str, title: &str, content: &str) {";
var REMOVE_FN = "pub fn remove_from_kb(&self, doc_id: &str) {";
var REINDEX_FN = "pub async fn reindex_persisted_documents(&self) -> usize {";
var PREFIX = 'format!("{doc_id}.html")';

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
    "> in <" + span.slice(0, 140) + ">");
}

// (a) An edit is a second projection of the same doc id. The stale chunks must
//     be dropped first, then the replacement points built and inserted.
h.staticTest("project_to_kb evicts doc_id.html points before reindexing an edit", function () {
  var body = between(DOC_STATE, PROJECT_FN, REMOVE_FN);

  h.assert(body.indexOf("self.remove_from_kb(doc_id);") !== -1,
    "projection must evict previous points first :: <" + body.slice(0, 200) + ">");
  h.assert(body.indexOf("source_ref = " + PREFIX) !== -1,
    "projection still builds points under the {doc_id}.html source :: <" + body + ">");

  var evict = body.indexOf("self.remove_from_kb(doc_id);");
  var build = body.indexOf("crate::handlers_api::kb_index::build_upsert_points(&input);");
  var insert = body.indexOf("(self.kb_indexer)(points)");
  h.assert(build !== -1 && insert !== -1,
    "replacement points are built and handed to the indexer :: <" + body + ">");
  h.assert(evict < build && build < insert,
    "eviction must precede point construction and index insertion :: <" + body + ">");
  ordered(body, "self.remove_from_kb(doc_id);", "let source_ref = " + PREFIX,
    "eviction must happen before the replacement source reference is created");
});

// (b) The eviction and the projection address the same source prefix, and the
//     busy-lock path is a logged skip rather than an error that fails a save.
h.staticTest("Removal uses the same source prefix and never fails a save", function () {
  var removeBody = between(DOC_STATE, REMOVE_FN, REINDEX_FN);
  var projectBody = between(DOC_STATE, PROJECT_FN, REMOVE_FN);

  h.assert(removeBody.indexOf("source_prefix = " + PREFIX) !== -1,
    "removal addresses the {doc_id}.html prefix :: <" + removeBody + ">");
  h.assert(projectBody.indexOf("source_ref = " + PREFIX) !== -1,
    "projection addresses the same {doc_id}.html prefix :: <" + projectBody + ">");
  h.assert(removeBody.indexOf("let removed = index.remove(&source_prefix);") !== -1,
    "the prefix is actually used to remove points, not just computed :: <" + removeBody + ">");

  h.assert(removeBody.indexOf("match self.kb_index.try_write() {") !== -1,
    "removal observes lock contention :: <" + removeBody + ">");
  h.assert(removeBody.indexOf('Err(_) => log::warn!("botdocs: KB index busy; removal for {doc_id} skipped"),') !== -1,
    "a busy lock is a logged skip, not a propagated error :: <" + removeBody + ">");

  // Both helpers return unit: literal signatures carry no `-> Result<...>`, so
  // a projection/eviction failure can never surface as an HTTP error.
  contains(DOC_STATE, REMOVE_FN, "remove_from_kb stays infallible (plain pub fn)");
  contains(DOC_STATE, PROJECT_FN, "project_to_kb stays infallible (plain pub fn)");
  h.assert(projectBody.indexOf('log::warn!("botdocs: KB index projection for document {doc_id} failed: {e}")') !== -1,
    "indexer failure is logged and swallowed :: <" + projectBody + ">");

  [removeBody, projectBody].forEach(function (span, index) {
    ["-> Result", "unwrap(", "expect(", "panic!"].forEach(function (forbidden) {
      h.assert(span.indexOf(forbidden) === -1,
        "save path " + index + " must not contain <" + forbidden + "> :: <" + span + ">");
    });
  });
});

// (c) The fix is pinned by a pure regression test that replays the same doc id
//     with revised content and checks both the removal and the replacement.
h.staticTest("Regression test asserts old-term absence and new-term presence after an edit", function () {
  var entry = "fn editing_document_replaces_indexed_points() {";
  var src = h.readRepo(KB_TESTS);
  var start = src.indexOf(entry);
  h.assert(start !== -1, "editing regression test must exist in " + KB_TESTS);
  var body = src.slice(start);

  contains(DOC_STATE, '#[path = "state_kb_tests.rs"]',
    "regression test module is compiled into the crate");

  h.assert(body.indexOf('state.project_to_kb(OWNER, "doc-edit", "Employment", HTML);') !== -1,
    "test projects the original document under a fixed id :: <" + body.slice(0, 200) + ">");
  h.assert(body.indexOf('state.project_to_kb(OWNER, "doc-edit", "Employment", revised);') !== -1,
    "test re-projects the SAME id with revised content :: <" + body.slice(0, 200) + ">");

  var oldEmpty = body.indexOf('state.search_text("ZANZIBARCLAUSE", &owner, 5).is_empty()');
  var staleMessage = body.indexOf('"stale chunks must be evicted when a document is edited"');
  var newPresent = body.indexOf('state.search_text("QUASARCLAUSE", &owner, 5).len(), 1');
  h.assert(oldEmpty !== -1 && staleMessage !== -1 && newPresent !== -1,
    "test checks both old-term absence and new-term presence :: <" + body + ">");
  h.assert(oldEmpty < newPresent,
    "old-term eviction is asserted before the new-term hit :: <" + body + ">");
});
