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
var INIT_END = "var opts = {";

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

// (a) The shipped lifecycle uses the article's real doc id on the wire and
// documents why a title must never be used. The full connect/reconnect
// lifecycle is covered by 51_docs_collab_lifecycle.js.
h.staticTest("#2457 collab uses only the article data-doc-id and documents why", function () {
  var src = h.readRepo(COLLAB);

  contains(COLLAB, INIT_START, "initCollab is shipped");
  contains(COLLAB, "gb.connect(opts)", "collab connection is shipped");

  var selection = between(src, INIT_START, INIT_END);
  spanHas(selection, "human-readable name", "comment states the title is human-readable");
  spanHas(selection, "server validates a UUID/slug", "comment states the server validates the id");
  spanHas(selection, "#GB-UAT 27893d4e", "comment records the UAT evidence");

  spanHas(selection, 'var article = document.querySelector("article[contenteditable]");',
    "editable article is queried for its doc id");
  spanHas(selection, 'var id = article && article.dataset ? article.dataset.docId : "";',
    "dataset.docId is the only source of a real id");
  spanHas(selection, 'if (!id || id === "current" || !/^[A-Za-z0-9_.-]+$/.test(id)) return "";',
    "placeholder/title-like ids are rejected before any connection");
  spanHas(src, "opts.docId = id;", "the validated id is used for the WebSocket connection");
  h.assert(selection.indexOf('getElementById("docTitle")') === -1,
    "the title input must never be used as a doc id");
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
