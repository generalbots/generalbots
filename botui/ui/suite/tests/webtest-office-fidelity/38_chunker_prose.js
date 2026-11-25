/**
 * [STATIC] deterministic coverage for the #2499 chunker clause-prose fix.
 * Every case reads real shipped Rust source as text and FAILS if any part of
 * the guard disappears: no runtime claim, no server, no browser, no cargo.
 * Each case first proves the surrounding function exists (non-vacuity), then
 * asserts literal substrings and their strict ordering, so a truncated, empty
 * or reverted file can never satisfy a case by accident.
 *
 * Covered:
 *   (a) numbered_heading rejects long / sentence-ending lines, so prose that
 *       starts with "Clause 7.3:" falls through to the generic body push;
 *   (b) a short clause heading still sets the numeric clause locator AND the
 *       full heading line is pushed into embedded chunk content;
 *   (c) the three chunker regression tests exist and the botdocs empty-chunk
 *       whole-text fallback remains for clause-only paragraphs.
 */
"use strict";

var h = require("./00_harness");

var CHUNKER = "botserver/crates/botcore/src/kb/structured/chunker.rs";
var DOC_STATE = "botserver/crates/botdocs/src/state.rs";

var NUMBERED_FN = "fn numbered_heading(line: &str) -> Option<(usize, String, String)> {";
var SET_HEADING_FN = "fn set_heading(";
var CHUNK_DOC_FN = "pub fn chunk_document(";
var FLUSH_FN = "fn flush(";

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(relPath, needle, msg) {
  h.assert(h.readRepo(relPath).indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath);
}

/** Literal substring assertion on an already-read source span. */
function spanHas(span, needle, msg) {
  h.assert(span.indexOf(needle) !== -1,
    (msg || "span must contain") + " <" + needle + "> :: " + span.slice(0, 220));
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

/** Every marker must be present, in this strict order. */
function ordered(span, markers, what) {
  var last = -1;
  markers.forEach(function (marker) {
    var at = span.indexOf(marker);
    h.assert(at !== -1, (what || "span") + " must contain <" + marker + "> :: " + span.slice(0, 220));
    h.assert(at > last,
      (what || "span") + " order broken at <" + marker + "> :: " + at + " <= " + last);
    last = at;
  });
}

/** Count literal (non-overlapping) occurrences in a span. */
function count(span, needle) {
  return span.split(needle).length - 1;
}

// (a) The sentence-ending / long-line guard makes numbered_heading return None
//     for "Clause 7.3: ..." prose. The rejected line then reaches the generic
//     state.lines.push at the bottom of the loop, so it survives as body text.
h.staticTest("numbered_heading rejects clause prose so it stays in the body", function () {
  var numbered = between(CHUNKER, NUMBERED_FN, SET_HEADING_FN);
  var chunkDoc = between(CHUNKER, CHUNK_DOC_FN, FLUSH_FN);

  // Non-vacuity: the title is derived before it is screened, and the guard is
  // documented as a prose rejection, not an incidental length cap.
  spanHas(numbered, "let title = candidate[number_end..]",
    "title is extracted before the prose screen");
  ordered(numbered, [
    "// A clause number at the start of prose is not a heading: require a",
    "// short, title-like line so the sentence survives as body content.",
    "if line.chars().count() > 120 || matches!(title.chars().last(), Some('.' | '!' | '?')) {"
  ], "numbered_heading prose guard");

  // The guarded branch must actually bail out before computing the level.
  var guard = "if line.chars().count() > 120 || matches!(title.chars().last(), Some('.' | '!' | '?')) {";
  var rejection = between(CHUNKER, guard, "let level = number.chars()");
  spanHas(rejection, "return None;",
    "long / sentence-ending lines are rejected, not turned into headings");

  // Both pushes survive: the embed push inside the heading branch and the
  // generic body push after it. A rejected prose line reaches the latter.
  spanHas(chunkDoc, "if let Some((level, number, title)) = Self::numbered_heading(line) {",
    "chunk_document still routes candidate headings through numbered_heading");
  h.assertEqual(count(chunkDoc, "state.lines.push(line.to_string());"), 2,
    "chunk_document must keep both the heading embed push and the generic body push");
});

// (b) A short clause heading is still addressed by its number (clause locator
//     -> citation) and its full source line is embedded in the chunk content so
//     heading-only queries can match.
h.staticTest("Short clause headings keep the locator and embed their heading text", function () {
  // The locator and embed line are applied by the numbered-heading *branch* of
  // chunk_document (numbered_heading itself only parses the line).
  contains(CHUNKER, NUMBERED_FN, "numbered_heading is still the heading parser");
  var branchStart = "if let Some((level, number, title)) = Self::numbered_heading(line) {";
  var embedPush = "state.lines.push(line.to_string());";
  var src = h.readRepo(CHUNKER);
  var start = src.indexOf(branchStart);
  h.assert(start !== -1, "missing numbered-heading branch <" + branchStart + "> in " + CHUNKER);
  var end = src.indexOf(embedPush, start);
  h.assert(end !== -1, "missing heading embed push after the numbered-heading branch");
  var branch = src.slice(start, end + embedPush.length);

  ordered(branch, [
    "let label = if title.is_empty() {",
    "Self::set_heading(&mut state.headings, level, label);",
    "if number.contains('.') {",
    "state.clause = Some(number);",
    "// Embed the heading text so heading-only queries can match.",
    "state.lines.push(line.to_string());"
  ], "short-heading locator and embed sequence");

  // The locator chosen here must be the one SourceRef turns into a Clause
  // source, so the citation carries "7.3" rather than a generic page.
  var sourceRef = between(CHUNKER, "fn source_ref(state: &SectionState) -> SourceRef {", "fn window_chunks(");
  ordered(sourceRef, [
    "} else if let Some(clause) = &state.clause {",
    "SourceRef::clause(clause.clone())"
  ], "clause locator becomes the chunk source");
});

// (c) The three regressions pin the whole fix, and the botdocs whole-text
//     fallback keeps a clause-only document searchable when the chunker emits
//     no points at all.
h.staticTest("Clause-prose regressions and the botdocs empty-chunk fallback remain", function () {
  var src = h.readRepo(CHUNKER);
  contains(CHUNKER, "#[cfg(test)]", "chunker test module is compiled in");

  var cases = [
    {
      entry: "fn prose_clause_number_stays_in_body() {",
      marks: [
        "chunk(\"Clause 7.3:",
        "assert_eq!(c[0].source.kind, SourceKind::Page);",
        "ZORBLAXKEY"
      ]
    },
    {
      entry: "fn short_clause_heading_keeps_locator() {",
      marks: [
        "assert!(c.iter().any(|x| x.source.kind == SourceKind::Clause));",
        "assert!(c.iter().any(|x| x.source.locator == \"7.3\"));"
      ]
    },
    {
      entry: "fn heading_text_is_embedded_in_content() {",
      marks: [
        "assert!(c.iter().any(|x| x.content.contains(\"Clause 7.3 Notice\")));"
      ]
    }
  ];

  var last = -1;
  cases.forEach(function (testCase) {
    var at = src.indexOf(testCase.entry);
    h.assert(at !== -1, "missing regression test <" + testCase.entry + "> in " + CHUNKER);
    h.assert(at > last, "regression test order broken at <" + testCase.entry + ">");
    last = at;
    var body = src.slice(at);
    testCase.marks.forEach(function (marker) {
      h.assert(body.indexOf(marker) !== -1,
        "<" + testCase.entry + "> must assert <" + marker + "> :: " + body.slice(0, 240));
    });
  });

  // The botdocs fallback is the second half of the fix: an empty point set
  // must still project one whole-text point for a non-empty document.
  var fallback = between(DOC_STATE, "if points.is_empty() {",
    "if let Err(e) = (self.kb_indexer)(points) {");
  spanHas(fallback, "keep such non-empty documents searchable (#2499)",
    "the empty-chunk fallback is documented as the clause-paragraph guard");
  ordered(fallback, [
    "build_index_document(&input)",
    "if !d.text.trim().is_empty() {",
    "content: d.text,",
    "points.push(chunk.to_upsert_point(DEFAULT_VECTOR_DIMENSION));"
  ], "botdocs whole-text fallback");
  h.assertEqual(count(fallback, "points.push("), 1,
    "the fallback emits exactly one whole-text point");
});
