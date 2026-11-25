/**
 * #2499 — deterministic static coverage for the citation-surfacing directive.
 *
 * The model ignored bare JSON identity lines and answered "no document found
 * in Drive". A self-delimited CITATION_DIRECTIVE now precedes those lines in
 * the system message: it mandates citing file/version/section/page and forbids
 * claiming the document is absent. This module reads the shipped Rust source
 * as text; no server, browser, cargo or network is involved.
 *
 * NON-VACUITY: every case proves the artifact ships (the directive constant,
 * the five-field DTO whitelist, the four #[tokio::test] bodies). Removing the
 * directive, moving it after lines.join, emitting a content field or deleting
 * an ordering assertion makes the matching case FAIL.
 */
"use strict";

var h = require("./00_harness");

var CITATION = "botserver/src/core/bot/pipeline/enriched_llm/citation_inject.rs";
var PIPELINE = "botserver/src/core/bot/pipeline/enriched_llm/mod.rs";

/** Literal substring assertion — no regex escaping needed for the needles. */
function contains(relPath, needle, msg) {
  h.assert(
    h.readRepo(relPath).indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath
  );
}

/** Exact source span between two literal markers (exclusive of end marker). */
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
    (msg || "ordering regressed") + " :: <" + before + "> before <" + after + ">");
}

/** The literal directive string body, without the surrounding Rust quotes. */
function directiveBody() {
  return between(CITATION, 'const CITATION_DIRECTIVE: &str = "', '";');
}

// (a) The directive is its own self-delimited block: a distinct header, four
//     bullet rules and a closing sentence. It mandates identity citations,
//     forbids an "absent" claim, and is joined BEFORE the JSON lines both in
//     append_lines and at the build_and_stream call site.
h.staticTest("#2499 CITATION_DIRECTIVE is self-delimited, mandates citations and precedes the JSON lines", function () {
  var d = directiveBody();
  h.assert(d.length > 400, "non-vacuity: directive must be substantial, found " + d.length);

  var headerAt = d.indexOf("SAVED DOCUMENT CITATIONS (authoritative identity metadata)");
  h.assert(headerAt !== -1 && headerAt < 60,
    "directive starts with its own header :: <" + d.slice(0, 100) + ">");
  h.assert(d.slice(-120).indexOf(
    "never fabricate or reproduce the document's text, clauses or body.") !== -1,
    "directive ends with its own closing rule :: <" + d.slice(-120) + ">");
  [
    "These records prove the listed documents EXIST in the user's Drive",
    "Treat them as valid evidence for this answer.",
    "You MUST cite the relevant document by its exact file name, version, section and page",
    "Never omit the citation when a record matches the question.",
    "Do NOT say the document was not found, is missing, or is absent from Drive when a record matches."
  ].forEach(function (rule) {
    h.assert(d.indexOf(rule) !== -1, "directive must contain rule <" + rule + ">");
  });

  var appendBody = between(CITATION, "fn append_lines(", "/// Resolve the shared Docs state");
  // Exact format string: directive placeholder, then newline, then the joined
  // identity records. A swap to "{}\n{CITATION_DIRECTIVE}" must FAIL here.
  h.assert(appendBody.indexOf('format!("{CITATION_DIRECTIVE}\\n{}", lines.join("\\n"));') !== -1,
    "block must join directive first, then newline-separated JSON lines :: <" +
    appendBody.slice(0, 900) + ">");
  ordered(appendBody, "{CITATION_DIRECTIVE}", "lines.join(",
    "directive must precede the identity records in append_lines");
  ordered(appendBody, "{content}", "{block}",
    "injected block must follow the original system prompt");

  var turn = between(PIPELINE, "pub(super) async fn build_and_stream(",
    '#[cfg(feature = "docs")]\nmod citation_inject;');
  ordered(turn, "super::kb::inject_kb(", "citation_inject::inject_saved_doc_citations(",
    "citation injection must run after inject_kb");
  contains(PIPELINE, '#[cfg(feature = "docs")]\n    citation_inject::inject_saved_doc_citations(',
    "citation injection is gated by the docs feature");
});

// (b) The DTO still carries only the five identity fields, and the directive
//     deliberately spells out "text, clauses or body" instead of the word
//     "content", keeping the no-document-content guarantee statically visible.
h.staticTest("#2499 citation DTO is identity-only and the directive avoids the word content", function () {
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
      "citation DTO must emit <" + field + "> :: <" + lineBody + ">");
  });
  var emitted = lineBody.match(/"[a-z_]+":/g) || [];
  h.assert(emitted.length === fields.length,
    "citation DTO must emit exactly five identity fields, found " + emitted.length +
    " :: <" + lineBody + ">");
  ["content", "text", "body", "score", "query", "chunk", "html", "payload"].forEach(function (forbidden) {
    h.assert(lineBody.indexOf('"' + forbidden + '"') === -1,
      "citation DTO must not emit <" + forbidden + "> :: <" + lineBody + ">");
  });

  var d = directiveBody();
  h.assert(d.indexOf("content") === -1 && d.indexOf("Content") === -1,
    "directive deliberately avoids the literal word 'content' :: <" + d + ">");
});

// (c) All four unit tests ship: the two originals plus the directive-first
//     ordering test and the no-system-message branch test. The ordering and
//     index-0 assertions must be present in the right test bodies.
h.staticTest("#2499 four unit tests assert directive ordering and the no-system-message branch", function () {
  var src = h.readRepo(CITATION);
  var names = [
    "authenticated_turn_gets_citation_only_lines",
    "nil_principal_injects_no_citation_lines",
    "rendered_prompt_has_citation_instruction_before_identity_lines",
    "created_system_message_keeps_instruction_first"
  ];
  names.forEach(function (name) {
    contains(CITATION, "#[tokio::test]\n    async fn " + name + "() {",
      "unit test " + name + " exists");
  });
  var declared = src.match(/#\[tokio::test\]/g) || [];
  h.assert(declared.length >= 4,
    "non-vacuity: at least four tokio tests must ship, found " + declared.length);

  var orderAt = src.indexOf("async fn rendered_prompt_has_citation_instruction_before_identity_lines() {");
  var createdAt = src.indexOf("async fn created_system_message_keeps_instruction_first() {");
  h.assert(orderAt !== -1 && createdAt !== -1 && orderAt < createdAt,
    "ordering test must precede the created-system-message test");
  var orderTest = src.slice(orderAt, createdAt);
  ["system.contains(\"SAVED DOCUMENT CITATIONS\")",
    "system.contains(\"MUST cite\")",
    "system.contains(\"file name, version, section and page\")",
    "system.contains(\"Do NOT say the document was not found\")"].forEach(function (needle) {
    h.assert(orderTest.indexOf(needle) !== -1,
      "ordering test must detect <" + needle + "> :: <" + orderTest.slice(0, 1600) + ">");
  });
  h.assert(orderTest.indexOf("base_at < directive_at") !== -1 &&
    orderTest.indexOf("directive_at < first_json_at") !== -1,
    "ordering test must assert base < directive < first JSON :: <" + orderTest.slice(0, 1600) + ">");

  var created = src.slice(createdAt);
  h.assert(created.indexOf('assert_eq!(messages[0]["role"], "system");') !== -1 &&
    created.indexOf('assert_eq!(messages[1]["role"], "user");') !== -1,
    "no-system-message test must assert the block is inserted at index 0 :: <" +
    created.slice(0, 1400) + ">");
  h.assert(created.indexOf("directive_at < first_json_at") !== -1,
    "no-system-message test must re-assert directive-before-JSON ordering");
  contains(CITATION,
    'array.insert(0, serde_json::json!({ "role": "system", "content": block }));',
    "append_lines creates a system message at index 0 when none exists");
});
