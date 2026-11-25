/**
 * Deterministic static coverage for the shipped Docs collaboration lifecycle
 * fix (#doc-collab-doc-id). `99_init.js` calls initCollab() before any article
 * exists, so the old "connect once at startup with the title/"current" id"
 * strategy either sent an invalid hello or stayed attached to the first
 * document forever. The fix turns initCollab() into an install-once watcher:
 *
 *   (a) renderedDocId() accepts ONLY the editable article's data-doc-id — it
 *       rejects empty, "current" and non UUID/slug ids, never reads #docTitle,
 *       never polls, and the single WS hello carries that derived id;
 *   (b) a MutationObserver on document.documentElement filtered to
 *       data-doc-id drives a debounced sync that connects ONCE and re-targets
 *       an already-open socket via gb.disconnect() + gb._scheduleReconnect() —
 *       never a second connect();
 *   (c) three idempotence guards (docsCollabReady, id equality, syncScheduled)
 *       and a single observer installation survive repeated docsInit() runs.
 *
 * All assertions are literal substrings of production source. Nothing is
 * executed: no server, no browser, no cargo, no network. NON-VACUITY: each
 * case first proves the real artifacts ship and the extracted spans are
 * non-empty, then asserts exact substrings, strict ordering and negative
 * guards, so restoring the poll/title fallback or a second connect() FAILs.
 */
"use strict";

var h = require("./00_harness");

var COLLAB = "botui/ui/suite/docs/modules/05_collab.js";
var INIT = "botui/ui/suite/docs/modules/99_init.js";

var INIT_START = "function initCollab() {";
var INIT_END = "var gb = window.GBCollab;";
var EXTRACT_START = "function renderedDocId() {";
var EXTRACT_END = "function collabUrl(id) {";
var SYNC_START = "function sync() {";
var SYNC_END = "function scheduleSync() {";
var SCHEDULE_START = "function scheduleSync() {";
var SCHEDULE_END = "function onMutations(records) {";
var MUTATIONS_START = "function onMutations(records) {";
var MUTATIONS_END = "  sync();";
var OBSERVER_START = "new MutationObserver(onMutations).observe(document.documentElement, {";
var OBSERVER_END = "});";

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

/** Literal substring must be absent from an already-read source span. */
function spanLacks(span, needle, msg) {
  h.assert(span.indexOf(needle) === -1,
    (msg || "span must not contain") + " <" + needle + "> :: " + span.slice(0, 200));
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

// (a) Only the editable article's real data-doc-id may go on the wire. The
// title/"current" fallback and any polling loop must be gone, and the single
// WS hello must carry the id renderedDocId() derived from the article.
h.staticTest("#doc-collab-doc-id renderedDocId accepts only the article data-doc-id (no title, no 'current', no poll) and the one WS hello uses it", function () {
  var src = h.readRepo(COLLAB);

  // Non-vacuity: the watcher, the id extractor and the connection all ship.
  contains(COLLAB, INIT_START, "initCollab is shipped");
  contains(COLLAB, EXTRACT_START, "renderedDocId is shipped");
  contains(COLLAB, "gb.connect(opts);", "a collab connection is shipped");

  var header = between(src, '"use strict";', "var docsCollabReady = false;");
  spanHas(header, "sent the placeholder title as the doc id (rejected by the server)",
    "header pins the old title-as-doc-id failure");
  spanHas(header, "stayed attached to the first document forever",
    "header pins the old single-connect failure");

  var extraction = between(src, EXTRACT_START, EXTRACT_END);
  spanHas(extraction, 'document.querySelector("article[contenteditable]")',
    "id is read from the editable article");
  spanHas(extraction, "article.dataset.docId", "data-doc-id is the id source");
  spanHas(extraction, 'if (!id || id === "current" || !/^[A-Za-z0-9_.-]+$/.test(id)) return "";',
    "empty, 'current' and non UUID/slug ids are rejected");
  spanHas(extraction, "return id;", "only the validated id is returned");
  sequence(extraction, [
    'document.querySelector("article[contenteditable]")',
    "article.dataset.docId",
    'id === "current"',
    "return id;"
  ], "valid-id extraction: query article, read data-doc-id, reject placeholders, return id");

  // Regression guards: the title fallback, the literal 'current' as a value
  // and any polling fallback must not return anywhere in the module.
  spanLacks(extraction, 'getElementById("docTitle")', "title input is not an id fallback");
  spanLacks(extraction, "return \"current\"", "'current' is never returned as an id");
  h.assertEqual(countIn(src, 'getElementById("docTitle")'), 0,
    "no title-input fallback exists anywhere in the collab module");
  h.assertEqual(countIn(src, "setInterval"), 0, "no polling fallback exists in the collab module");
  h.assertEqual(countIn(src, "docId: docId,"), 0, "the old title/placeholder hello is gone");

  // The hello is built from the derived id: id = renderedDocId() -> opts.docId
  // -> gb.connect(opts), in that order, with exactly one connect call.
  var syncBody = between(src, SYNC_START, SYNC_END);
  spanHas(syncBody, "var id = renderedDocId();", "sync derives the id from renderedDocId");
  spanHas(syncBody, "opts.docId = id;", "hello options carry the derived article id");
  spanHas(syncBody, "gb.connect(opts);", "the connection uses those options");
  sequence(syncBody, ["var id = renderedDocId();", "opts.docId = id;", "gb.connect(opts);"],
    "WS hello order: derive id, assign to opts, connect once");
  h.assertEqual(countIn(src, "gb.connect("), 1, "exactly one connect on the wire path");
  h.assertEqual(countIn(src, "window.GBCollab.connect("), 0, "no startup-time connect remains");
});

// (b) The observer on document.documentElement (attributeFilter data-doc-id)
// drives a debounced sync that connects once, then re-targets by mutating the
// singleton's id/url and calling disconnect + _scheduleReconnect — never a
// second connect().
h.staticTest("#doc-collab-doc-id MutationObserver re-targets one socket via disconnect + _scheduleReconnect, never a second connect", function () {
  var src = h.readRepo(COLLAB);

  // Non-vacuity: the mutation handler and the observer installation ship.
  contains(COLLAB, MUTATIONS_START, "onMutations handler is shipped");
  contains(COLLAB, OBSERVER_START, "documentElement MutationObserver is shipped");

  var mutations = between(src, MUTATIONS_START, MUTATIONS_END);
  spanHas(mutations, 'record.type === "attributes"', "attribute mutations drive a sync");
  spanHas(mutations, "scheduleSync();", "attribute/article mutations schedule a sync");
  spanHas(mutations, "node.nodeType === 1", "only element nodes are considered");
  spanHas(mutations, 'node.tagName === "ARTICLE"', "an inserted article drives a sync");
  spanHas(mutations, 'node.querySelector("article[contenteditable]")',
    "an inserted article container drives a sync");

  var observer = between(src, OBSERVER_START, OBSERVER_END);
  spanHas(observer, "childList: true", "observer watches childList (body swaps)");
  spanHas(observer, "subtree: true", "observer watches the subtree");
  spanHas(observer, "attributes: true", "observer watches attributes");
  spanHas(observer, 'attributeFilter: ["data-doc-id"]', "observer filters on data-doc-id");
  sequence(src, [MUTATIONS_START, OBSERVER_START, "window.docsSyncCollab = sync;"],
    "handler is defined, observer installed, then sync exposed for deep links");

  var syncBody = between(src, SYNC_START, SYNC_END);
  spanHas(syncBody, "var hadConnection = !!currentDocId;", "connect-once branch is keyed on prior id");
  spanHas(syncBody, "if (!hadConnection) {", "first valid id opens the only socket");
  spanHas(syncBody, "if (gb.opts) gb.opts.docId = id;", "re-target updates singleton opts doc id");
  spanHas(syncBody, "gb.docId = id;", "re-target updates singleton doc id");
  spanHas(syncBody, "gb.url = collabUrl(id);", "re-target updates the socket url");
  spanHas(syncBody, "gb.disconnect();", "re-target closes the old socket");
  spanHas(syncBody, "gb._scheduleReconnect();", "re-target uses the guarded singleton reconnect");
  sequence(syncBody, ["gb.docId = id;", "gb.url = collabUrl(id);", "gb.disconnect();", "gb._scheduleReconnect();"],
    "re-target order: id, url, disconnect, schedule reconnect");

  // The re-target code performs no connect() of its own — only the one call
  // inside the connect-once branch above.
  var wireOps = between(src, "if (gb.opts) gb.opts.docId = id;", SYNC_END);
  h.assertEqual(countIn(wireOps, "gb.connect("), 0,
    "re-target never calls connect() a second time");
  h.assertEqual(countIn(src, "gb.connect("), 1, "only one connect() exists on the wire path");
  h.assertEqual(countIn(src, "_scheduleReconnect()"), 2,
    "singleton reconnect is referenced (comment + re-target call)");
});

// (c) Repeated docsInit() runs cannot install a second watcher or reopen a
// socket: docsCollabReady (install once), id equality (same-doc no-op) and
// syncScheduled (debounce) all guard. 99_init.js calls initCollab() before the
// article exists, which is the lifecycle this fix handles.
h.staticTest("#doc-collab-doc-id install-once + id-equality + syncScheduled idempotence guard repeated docsInit runs", function () {
  var src = h.readRepo(COLLAB);

  // Non-vacuity: all three guard variables and the observer ship.
  contains(COLLAB, "var docsCollabReady = false;", "install-once flag ships");
  contains(COLLAB, "var currentDocId = \"\";", "id-equality state ships");
  contains(COLLAB, "var syncScheduled = false;", "debounce flag ships");
  contains(COLLAB, OBSERVER_START, "observer installation ships");

  var initHead = between(src, INIT_START, INIT_END);
  spanHas(initHead, "if (!window.GBCollab || docsCollabReady) return;",
    "level 1: missing singleton or already installed is a no-op");
  spanHas(initHead, "docsCollabReady = true;", "level 1: the flag is set before any work");
  sequence(initHead, ["if (!window.GBCollab || docsCollabReady) return;", "docsCollabReady = true;"],
    "install-once guard order");

  var syncBody = between(src, SYNC_START, SYNC_END);
  spanHas(syncBody, "if (!id || id === currentDocId) return;",
    "level 2: absent id or unchanged id is a no-op");
  spanHas(syncBody, "currentDocId = id;", "level 2: the active id is recorded");
  sequence(syncBody, ["if (!id || id === currentDocId) return;", "currentDocId = id;"],
    "id-equality guard order");

  var scheduleBody = between(src, SCHEDULE_START, SCHEDULE_END);
  spanHas(scheduleBody, "if (syncScheduled) return;", "level 3: a pending sync is a no-op");
  spanHas(scheduleBody, "syncScheduled = true;", "level 3: the debounce is armed");
  spanHas(scheduleBody, "setTimeout(function () { syncScheduled = false; sync(); }, 0);",
    "level 3: a 0ms debounce coalesces mutation bursts");
  sequence(scheduleBody, ["if (syncScheduled) return;", "syncScheduled = true;", "setTimeout(function () { syncScheduled = false; sync(); }, 0);"],
    "sync debounce order");

  h.assertEqual(countIn(src, "new MutationObserver("), 1, "the observer is installed exactly once");
  h.assertEqual(countIn(src, "var docsCollabReady ="), 1, "one install-once flag");
  h.assertEqual(countIn(src, "var currentDocId ="), 1, "one active-id state variable");
  h.assertEqual(countIn(src, "var syncScheduled ="), 1, "one debounce flag");

  // Root cause non-vacuity: docsInit really runs and calls initCollab() before
  // ensureDocArticle()/loadDriveFileIntoEditor() create the article.
  contains(INIT, "function docsInit() {", "docsInit is shipped");
  contains(INIT, "docsInit();", "docsInit is actually invoked on a loaded page");
  contains(INIT, 'window.addEventListener("DOMContentLoaded", docsInit);',
    "docsInit is wired to DOMContentLoaded");
  var initFn = between(h.readRepo(INIT), "function docsInit() {",
    "// The desktop shell injects this fragment");
  sequence(initFn, ["initCollab();", "ensureDocArticle();", "loadDriveFileIntoEditor();"],
    "initCollab runs before any article exists — the lifecycle the watcher handles");
});

h.note("FOLLOW-UP #doc-collab-doc-id: collab now validates the article data-doc-id (no title/'current'/poll), " +
  "observes document.documentElement for data-doc-id, connects once and re-targets via disconnect + " +
  "_scheduleReconnect, and guards install/id/debounce; source assertions only — no browser/server/cargo.");
