/**
 * [STATIC] deterministic guards for #2499 "in-chat citation-only injection".
 * Every case reads the real shipped Rust source as text and FAILS if a contract
 * is removed: no runtime claim, no server, no browser, no cargo.
 *
 * Covered contracts:
 *   (a) the Docs feature block builds `Arc<DocState>` and publishes it into
 *       `AppState.extensions` with `insert_blocking` (the producer seam)
 *   (b) `inject_saved_doc_citations` is called in `build_and_stream` after
 *       `inject_kb`, and fails closed on a nil principal or unresolved Docs
 *       state (no system-message mutation in either case)
 *   (c) each injected line is built ONLY from document_id/file/version/section/
 *       page (no content/text/body fields), capped at MAX_CITATIONS = 3, and
 *       both unit tests are compiled into the crate
 *
 * All assertions are literal source substrings. Deleting the injection call,
 * either fail-closed guard, the extensions publication or the field whitelist
 * makes the matching case FAIL. No user-facing string is introduced here.
 */
"use strict";

var h = require("./00_harness");

var SUB_ROUTES = "botserver/src/main_module/routes/feature_merge_office/sub_routes.rs";
var PIPELINE = "botserver/src/core/bot/pipeline/enriched_llm/mod.rs";
var CITATION = "botserver/src/core/bot/pipeline/enriched_llm/citation_inject.rs";

/** Literal substring assertion — no regex escaping needed for the needles. */
function contains(relPath, needle, msg) {
  h.assert(
    h.readRepo(relPath).indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath
  );
}

/** Exact source span between two markers (exclusive of the end marker). */
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

/** `before` must occur strictly earlier than `after` inside a source span. */
function ordered(span, before, after, msg) {
  var first = span.indexOf(before);
  var second = span.indexOf(after);
  h.assert(first !== -1 && second !== -1 && first < second,
    (msg || "ordering regressed") + " :: <" + before + "> before <" + after +
    "> in <" + span.slice(0, 140) + ">");
}

// (a) Producer: the docs feature block is the only place that inserts the
//     shared DocState into AppState.extensions for the chat pipeline to find.
h.staticTest("Docs feature block publishes Arc<DocState> into AppState.extensions", function () {
  var docsBlock = between(SUB_ROUTES, '#[cfg(feature = "docs")]', '#[cfg(feature = "paper")]');

  h.assert(docsBlock.indexOf("let docs_state = Arc::new(botdocs::state::DocState::with_in_memory_kb_index(") !== -1,
    "docs block builds a shared DocState with the in-memory KB index :: <" + docsBlock.slice(0, 220) + ">");
  h.assert(docsBlock.indexOf("app_state.extensions.insert_blocking(docs_state.clone());") !== -1,
    "the shared DocState is published into AppState.extensions :: <" + docsBlock.slice(0, 400) + ">");

  var built = docsBlock.indexOf("let docs_state = Arc::new(");
  var published = docsBlock.indexOf("app_state.extensions.insert_blocking(docs_state.clone());");
  h.assert(built < published,
    "state must exist before it is published :: <" + docsBlock.slice(0, 400) + ">");
  h.assert(docsBlock.indexOf(".with_state(docs_state)") !== -1,
    "the same state also drives the docs routes :: <" + docsBlock.slice(-300) + ">");

  // The publication carries an Arc<DocState>, not document bytes: no content
  // payload is ever inserted into the shared extensions map.
  h.assert(docsBlock.indexOf("insert_blocking(HTML") === -1 &&
    docsBlock.indexOf("insert_blocking(content") === -1,
    "extensions must never receive document content :: <" + docsBlock.slice(0, 400) + ">");
});

// (b) Consumer wiring: the injection seam runs inside the same turn, after
//     inject_kb, and both fail-closed guards return before append_lines.
h.staticTest("Citation injection runs after inject_kb and fails closed", function () {
  var turn = between(PIPELINE, "pub(super) async fn build_and_stream(", '#[cfg(feature = "docs")]\nmod citation_inject;');
  var call = "citation_inject::inject_saved_doc_citations(";
  var injectKb = "super::kb::inject_kb(";

  h.assert(turn.indexOf(injectKb) !== -1, "KB injection must be part of the turn :: <" + turn.slice(0, 200) + ">");
  h.assert(turn.indexOf(call) !== -1,
    "build_and_stream must invoke the citation injection seam :: <" + turn.slice(0, 400) + ">");
  ordered(turn, injectKb, call, "citation injection must run after inject_kb");

  // The call forwards the same extensions, bucket, principal, query and prompt
  // array so the seam can resolve the published state and append in place.
  var from = turn.indexOf(call);
  var args = turn.slice(from, from + 320);
  ["&state.extensions,", "&state.bucket_name,", "user_id,", "&user_query,", "&mut messages_val,"]
    .forEach(function (arg) {
      h.assert(args.indexOf(arg) !== -1,
        "citation injection must pass <" + arg + "> :: <" + args + ">");
    });
  h.assert(args.indexOf(".await;") !== -1, "citation injection must be awaited :: <" + args + ">");

  contains(CITATION, "if user_id.is_nil() {\n        return None;\n    }",
    "a nil principal resolves to no UserContext");
  contains(CITATION, "let Some(principal) = principal_for(user_id, bucket_name) else {\n        return;\n    };",
    "an unresolved principal returns before any search or prompt mutation");
  contains(CITATION, "let Some(docs) = docs_from(extensions).await else {\n        return;\n    };",
    "unpublished Docs state returns before any search or prompt mutation");
  contains(CITATION, "extensions.get::<Arc<DocState>>().await",
    "docs_from resolves the Arc<DocState> published by the producer");
  contains(CITATION, "extensions.get::<DocState>().await",
    "docs_from also tolerates a directly inserted DocState");

  var guarded = between(CITATION, "pub(super) async fn inject_saved_doc_citations(", "#[cfg(test)]");
  var append = guarded.indexOf("append_lines(messages, &lines);");
  var principalGuard = guarded.indexOf("let Some(principal) = principal_for(user_id, bucket_name) else");
  var docsGuard = guarded.indexOf("let Some(docs) = docs_from(extensions).await else");
  h.assert(principalGuard !== -1 && docsGuard !== -1 && append !== -1 &&
    principalGuard < docsGuard && docsGuard < append,
    "both fail-closed guards must precede the prompt append :: <" + guarded + ">");
});

// (c)+(d) Shape and tests: the line carries identity metadata only, at most
//         three lines are added, and both regression tests are compiled in.
h.staticTest("Citation lines are identity-only, capped at three, and unit-tested", function () {
  var src = h.readRepo(CITATION);
  var lineBody = between(CITATION, "fn citation_line(", "fn append_lines(");

  var fields = [
    '"document_id": meta.document_id,',
    '"file": meta.file,',
    '"version": meta.version,',
    '"section": hit.section,',
    '"page": hit.source.page,'
  ];
  fields.forEach(function (field) {
    h.assert(lineBody.indexOf(field) !== -1,
      "citation line must include <" + field + "> :: <" + lineBody + ">");
  });
  var emitted = lineBody.match(/"[a-z_]+":/g) || [];
  h.assert(emitted.length === fields.length,
    "citation line must contain exactly the five allowed fields, found " + emitted.length +
    " :: <" + lineBody + ">");

  ["content", "text", "body", "score", "query", "chunk", "html", "payload"].forEach(function (forbidden) {
    h.assert(lineBody.indexOf('"' + forbidden + '"') === -1,
      "citation line must not emit <" + forbidden + "> :: <" + lineBody + ">");
  });

  contains(CITATION, "const MAX_CITATIONS: usize = 3;", "hard ceiling is three citation lines");
  contains(CITATION, "docs.search_text(user_query, &principal, MAX_CITATIONS)",
    "search itself is capped at MAX_CITATIONS");
  contains(CITATION, ".take(MAX_CITATIONS)", "collected hits are capped at MAX_CITATIONS");
  contains(CITATION, 'lines.join("\\n")', "each citation is one newline-separated line");

  contains(CITATION, "#[cfg(test)]", "unit test module is compiled into the crate");
  contains(CITATION, "#[tokio::test]\n    async fn authenticated_turn_gets_citation_only_lines() {",
    "authenticated citation-only test exists");
  contains(CITATION, "#[tokio::test]\n    async fn nil_principal_injects_no_citation_lines() {",
    "nil-principal fail-closed test exists");
  contains(CITATION, "for leaked in [\"ninety days\", \"ZANZIBARCLAUSE\", \"content\", \"payload\"]",
    "authenticated test asserts document bytes and content fields never leak");
  contains(CITATION, "assert_eq!(system_content(&messages), \"base prompt\");",
    "nil-principal test asserts the system message is untouched");

  // Non-vacuity: the capped count assertion must really be inside the test.
  var testSpan = src.slice(src.indexOf("async fn authenticated_turn_gets_citation_only_lines() {"));
  h.assert(testSpan.indexOf("citations >= 1 && citations <= MAX_CITATIONS") !== -1,
    "test asserts the injected line count stays within the cap :: <" + testSpan.slice(0, 160) + ">");
});
