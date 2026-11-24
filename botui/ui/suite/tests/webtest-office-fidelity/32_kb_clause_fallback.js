/**
 * [STATIC] deterministic guards for the #2499 "clause-only paragraph KB
 * fallback". Every case reads real shipped Rust source as text and FAILS if a
 * contract is removed: no runtime claim, no server, no browser, no cargo.
 *
 * Covered contracts:
 *   (a) when `build_upsert_points` yields no points but the document text is
 *       non-empty, `project_to_kb` emits ONE whole-text point that carries the
 *       same document/tenant/version/ACL identity as a structured chunk
 *   (b) the fallback does not bypass the prior eviction or the shared
 *       fail-closed indexer call (a failed index is logged and swallowed), and
 *       adds no new error surface to the best-effort save path
 *   (c) the `clause_only_paragraph_stays_searchable` regression test exists, is
 *       compiled into the crate, and asserts a distinctive token becomes
 *       searchable through the fallback
 *
 * All assertions are literal source substrings. Removing the fallback branch,
 * its identity fields, the eviction/logging seam or the regression test makes
 * the matching case FAIL. No user-facing string is introduced here.
 */
"use strict";

var h = require("./00_harness");

var DOC_STATE = "botserver/crates/botdocs/src/state.rs";
var KB_TESTS = "botserver/crates/botdocs/src/state_kb_tests.rs";

var PROJECT_FN = "pub fn project_to_kb(&self, user_id: &str, doc_id: &str, title: &str, content: &str) {";
var REMOVE_FN = "pub fn remove_from_kb(&self, doc_id: &str) {";

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
  var span = src.slice(start, end);
  h.assert(span.length > 0, "empty source span between <" + startMarker + "> and <" + endMarker + ">");
  return span;
}

// (a) The chunker consumes a lone "Clause 7.3: ..." paragraph as a heading, so
//     build_upsert_points returns empty. The fallback rebuilds the whole text
//     as exactly one point, reusing the SAME identity fields as a structured
//     chunk so tenant/ACL/citation filtering is unchanged.
h.staticTest("Empty chunk set with non-empty text emits one whole-text point", function () {
  var projectBody = between(DOC_STATE, PROJECT_FN, REMOVE_FN);
  var fallback = between(DOC_STATE, "if points.is_empty() {", "if let Err(e) = (self.kb_indexer)(points) {");

  h.assert(fallback.indexOf("let d = crate::handlers_api::kb_index::build_index_document(&input);") !== -1,
    "fallback rebuilds the whole document rather than guessing :: <" + fallback + ">");
  h.assert(fallback.indexOf("if !d.text.trim().is_empty() {") !== -1,
    "fallback is gated on genuinely non-empty text :: <" + fallback + ">");

  var identity = [
    "document_id: d.document_id,",
    "file: d.file,",
    "version: d.version,",
    "tenant_id: d.tenant_id,",
    "acl: d.acl,"
  ];
  identity.forEach(function (field) {
    h.assert(fallback.indexOf(field) !== -1,
      "whole-text point must carry <" + field + "> :: <" + fallback + ">");
  });

  h.assert(fallback.indexOf("content: d.text,") !== -1,
    "whole-text point carries the full document text :: <" + fallback + ">");
  h.assert(fallback.indexOf("source: SourceRef::page(1),") !== -1,
    "whole-text point is addressed to page 1 :: <" + fallback + ">");
  h.assert(fallback.indexOf('let id = stable_id(&[&d.tenant_id, &d.document_id, &d.version, "all", &d.text]);') !== -1,
    "point id derives from tenant/document/version/text :: <" + fallback + ">");

  var construct = fallback.indexOf("let chunk = StructuredChunk {");
  var push = fallback.indexOf("points.push(chunk.to_upsert_point(DEFAULT_VECTOR_DIMENSION));");
  h.assert(construct !== -1 && push !== -1 && construct < push,
    "the pushed point is the constructed chunk via the SAME conversion as structured points :: <" + fallback + ">");

  // Non-vacuity: exactly one push — the fallback is a single whole-text point,
  // never a second batch path or a duplicate of the structured chunks.
  var pushes = fallback.split("points.push(").length - 1;
  h.assertEqual(pushes, 1, "fallback must emit exactly one whole-text point, found " + pushes);
});

// (b) The fallback must not create a side door: eviction still runs first, and
//     the fallback's point still flows through the shared indexer call whose
//     failure is logged and swallowed. No new fatal error path is introduced.
h.staticTest("Fallback keeps eviction and fail-closed index logging", function () {
  var projectBody = between(DOC_STATE, PROJECT_FN, REMOVE_FN);

  // A re-projected clause-only document must not leave its previous whole-text
  // point behind — eviction still precedes the fallback guard.
  var evict = projectBody.indexOf("self.remove_from_kb(doc_id);");
  var fallbackGuard = projectBody.indexOf("if points.is_empty() {");
  h.assert(evict !== -1 && fallbackGuard !== -1 && evict < fallbackGuard,
    "eviction must run before the fallback :: <" + projectBody.slice(0, 260) + ">");

  // Every point — including the fallback's — goes through the same indexer call,
  // so a failed index is still a logged skip, never an HTTP error.
  var emit = projectBody.indexOf("if let Err(e) = (self.kb_indexer)(points) {");
  h.assert(emit !== -1 && fallbackGuard < emit,
    "fallback must precede the shared indexer call :: <" + projectBody.slice(-340) + ">");
  h.assert(projectBody.indexOf('log::warn!("botdocs: KB index projection for document {doc_id} failed: {e}");') !== -1,
    "indexer failure stays logged and swallowed for fallback points too :: <" + projectBody + ">");

  // Best-effort: the projection/fallback path adds no new error surface.
  ["-> Result", "unwrap(", "expect(", "panic!", "return Err("].forEach(function (forbidden) {
    h.assert(projectBody.indexOf(forbidden) === -1,
      "projection path must not contain <" + forbidden + "> :: <" + projectBody + ">");
  });
  contains(DOC_STATE, PROJECT_FN, "project_to_kb stays infallible (plain pub fn)");
});

// (c) The regression test replays a clause-only paragraph carrying a unique
//     token and checks it is searchable with the right document identity.
h.staticTest("Regression test proves a clause-only paragraph stays searchable", function () {
  var src = h.readRepo(KB_TESTS);
  var entry = "fn clause_only_paragraph_stays_searchable() {";
  var start = src.indexOf(entry);
  h.assert(start !== -1, "clause-only regression test must exist in " + KB_TESTS);
  var body = src.slice(start);

  contains(DOC_STATE, '#[path = "state_kb_tests.rs"]',
    "the regression test module is compiled into the crate");

  h.assert(body.indexOf("ZORBLAXKEY") !== -1,
    "test uses a distinctive token only the whole-text fallback can surface :: <" + body.slice(0, 260) + ">");
  h.assert(body.indexOf('state.project_to_kb(OWNER, "doc-clause-only", "Chat KB UAT", html);') !== -1,
    "test projects a clause-only paragraph under a fixed id :: <" + body.slice(0, 260) + ">");
  h.assert(body.indexOf('state.search_text("ZORBLAXKEY", &owner, 5);') !== -1,
    "test searches for the distinctive token :: <" + body.slice(0, 320) + ">");

  var hitAssert = body.indexOf('assert_eq!(hits.len(), 1, "clause-like paragraph must remain searchable");');
  var idAssert = body.indexOf('assert_eq!(state.citation_meta(&hits[0].id).document_id, "doc-clause-only");');
  h.assert(hitAssert !== -1 && idAssert !== -1,
    "test asserts both a search hit and document identity :: <" + body + ">");
  h.assert(hitAssert < idAssert,
    "search-hit assertion precedes the citation-identity assertion :: <" + body + ">");

  // Non-vacuity: project -> search -> assert, with the distinctive token tying
  // them together so the test genuinely exercises the fallback path.
  var projected = body.indexOf('state.project_to_kb(OWNER, "doc-clause-only"');
  var searched = body.indexOf('state.search_text("ZORBLAXKEY"');
  h.assert(projected !== -1 && searched !== -1 && projected < searched && searched < hitAssert,
    "clause-only flow is project -> search -> assert :: <" + body.slice(0, 400) + ">");
});
