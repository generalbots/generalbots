/**
 * #2457 — deterministic static coverage for the shipped Docs collaboration
 * doc-id fix:
 *   (a) initCollab derives the WebSocket doc id from the editable article's
 *       data-doc-id BEFORE the human-readable #docTitle input, still falls back
 *       to the literal "current", and carries the explanatory comment for why
 *       the title input must not be used;
 *   (b) a shipped Docs template really puts data-doc-id on the editable
 *       article — both the server-rendered view fragment and the docs/ local
 *       fallback — so the preference can actually fire (non-vacuity).
 *
 * All assertions are literal substrings of production source. Nothing is
 * executed: no server, no browser, no cargo, no network. NON-VACUITY: each case
 * first proves the real initCollab / template artifacts ship and that the
 * extracted span is non-empty, then asserts exact substrings and strict
 * ordering, so reverting the preference or deleting the attribute FAILs.
 */
"use strict";

var h = require("./00_harness");

var COLLAB = "botui/ui/suite/docs/modules/05_collab.js";
var BOOT = "botui/ui/suite/docs/modules/00_boot.js";
var FRAGMENT = "botserver/crates/botdocs/src/ui_fragments/sidebars.rs";

var INIT_START = "function initCollab() {";
var INIT_END = 'var typingEl = document.getElementById("typing-indicator");';

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(relPath, needle, msg) {
  h.assert(h.readRepo(relPath).indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath);
}

/** Count exact, literal occurrences of a needle in a source string. */
function countIn(src, needle) {
  return src.split(needle).length - 1;
}

/** Exact, non-empty source span between two literal markers (end exclusive). */
function between(src, startMarker, endMarker) {
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + ">");
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + ">");
  var span = src.slice(start, end);
  h.assert(span.length > 0, "empty source span between <" + startMarker + "> and <" + endMarker + ">");
  return span;
}

/** Literal substring assertion on an already-read source span. */
function spanHas(span, needle, msg) {
  h.assert(span.indexOf(needle) !== -1,
    (msg || "span must contain") + " <" + needle + "> :: " + span.slice(0, 200));
}

/** Every marker must be present, in strict order. */
function sequence(span, markers, msg) {
  var last = -1;
  markers.forEach(function (marker) {
    var at = span.indexOf(marker);
    h.assert(at !== -1, (msg || "span") + " must contain <" + marker + ">");
    h.assert(at > last, (msg || "span") + " order broken at <" + marker + ">");
    last = at;
  });
}

// (a) The article's data-doc-id is preferred over the title input, with a
// literal "current" fallback and the comment explaining the failure mode.
h.staticTest("#2457 initCollab prefers article.dataset.docId over the docTitle input, falls back to 'current', and documents why", function () {
  var src = h.readRepo(COLLAB);

  // Non-vacuity: initCollab really ships and actually connects with docId.
  contains(COLLAB, INIT_START, "initCollab is shipped");
  contains(COLLAB, "window.GBCollab.connect({", "collab connection is shipped");
  contains(COLLAB, "docId: docId,", "derived docId is used for the WebSocket connection");

  // Scope tightly to initCollab's doc-id selection so onEdit's separate
  // `article` query cannot satisfy or invalidate any assertion here.
  var selection = between(src, INIT_START, INIT_END);

  // The explanatory comment must survive the fix (it pins the root cause).
  spanHas(selection, "The server validates the doc id (UUID/slug); the title input holds the",
    "comment states the server validates the doc id, not the title");
  spanHas(selection, "human-readable document name, so using it made every docs WebSocket",
    "comment explains the title input is human-readable");
  spanHas(selection, "malformed document id", "comment names the malformed-id failure");
  spanHas(selection, "collaboration (#GB-UAT 27893d4e).", "comment records the UAT evidence");

  // The article is queried, then its dataset.docId is selected first.
  spanHas(selection, 'var article = document.querySelector("article[contenteditable]");',
    "editable article is queried for its doc id");
  spanHas(selection, "var docId = (article && article.dataset && article.dataset.docId) ||",
    "dataset.docId is the first, guarded preference");
  spanHas(selection,
    '(document.getElementById("docTitle") && document.getElementById("docTitle").value) || "current";',
    "title input is only the secondary source and 'current' is the final fallback");

  // Strict preference order: dataset doc id, then title input, then "current".
  // Reverting to title-first, or dropping the fallback, breaks these checks.
  sequence(selection, [
    "article.dataset.docId",
    'getElementById("docTitle")',
    '"current"'
  ], "doc-id preference order is article dataset > docTitle > 'current'");
  h.assertEqual(countIn(selection, "var docId ="), 1,
    "the doc id is assigned from a single selection expression");
  h.assertEqual(countIn(src, "var docId ="), 1,
    "no second, overriding doc-id assignment exists in the module");
});

// (b) The shipped Docs template actually sets data-doc-id on the editable
// article, so the preference above can fire. Both the server-rendered fragment
// (primary) and the docs/ local fallback are asserted (non-vacuity).
h.staticTest("#2457 shipped Docs templates set data-doc-id on the editable article so the collab fix can fire", function () {
  // docs/ fallback template: localViewHtml builds the editable article.
  contains(BOOT, "function localViewHtml(doc) {", "local Docs view template is shipped");
  contains(BOOT, "host.innerHTML = localViewHtml(doc);", "local template is actually used by renderView");

  var boot = h.readRepo(BOOT);
  var template = between(boot, "function localViewHtml(doc) {", "function bindLocalView(host) {");
  var article = between(template, '<article class="docs-doc-view" contenteditable="true"', "</article>");

  spanHas(article, 'contenteditable="true"', "local article is editable");
  spanHas(article, 'data-doc-id="', "local article carries data-doc-id");
  spanHas(article, "escapeHtml(doc.id)", "local data-doc-id is populated with the real doc id");
  sequence(article, [
    'contenteditable="true"',
    'data-doc-id="',
    "escapeHtml(doc.id)"
  ], "local template sets data-doc-id on the contenteditable article");

  // Primary server-rendered view fragment used by /suite/docs/fragments/view.
  contains(FRAGMENT, '<article class="dc-content docs-doc-view"', "server view-article fragment is shipped");
  contains(FRAGMENT, 'data-doc-id="{id}" contenteditable="true"',
    "server fragment sets data-doc-id on the editable article");
});

h.note("FOLLOW-UP #2457: initCollab selects the editable article's data-doc-id before the docTitle input " +
  "(fallback 'current'), and both shipped Docs templates supply that attribute; source assertions only — " +
  "no browser/server/cargo run.");
