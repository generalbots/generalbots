/**
 * Deterministic static coverage for the shipped blank-document collaboration
 * reconnect-storm fix (#doc-collab-blank-reconnect). A brand-new Docs document
 * has no id in Drive yet: 00_boot.startBlank() calls docsEnsureArticleDocId(),
 * which mints a UUID and writes it into data-doc-id. The collab watcher saw that
 * id, opened /ws/docs/{id}, botdocs refused it (the document does not exist),
 * and GBCollab's close handler reopened the refused socket in a loop. The fix:
 *
 *   (a) the mint point is the ONLY place that flags an article unsaved —
 *       docsEnsureArticleDocId() sets data-docs-unsaved="1" only inside its
 *       `if (!id || id === "current")` branch, never for a real Drive id;
 *   (b) renderedDocId() returns "" while that flag is set, so sync()'s
 *       `if (!id || id === currentDocId) return;` short-circuits before the
 *       single gb.connect(opts) — no refused socket can be opened;
 *   (c) a successful save (autosave via docsRefreshSidebar, or title blur via
 *       saveTitle) calls markActiveDocPersisted(), which clears the flag and
 *       explicitly invokes window.docsSyncCollab() = sync; the connect-once /
 *       re-target lifecycle from 51_docs_collab_lifecycle.js stays intact.
 *
 * All assertions are literal substrings of production source. Nothing is
 * executed: no server, no browser, no cargo, no network. NON-VACUITY: each
 * case first proves the real artifacts ship and the extracted spans are
 * non-empty, then asserts exact substrings, strict ordering and negative
 * guards. Removing the mint-time marking or the renderedDocId guard FAILs.
 */
"use strict";

var h = require("./00_harness");

var STATE = "botui/ui/suite/docs/modules/01_state.js";
var COLLAB = "botui/ui/suite/docs/modules/05_collab.js";
var BOOT = "botui/ui/suite/docs/modules/00_boot.js";

var MINT_START = "function docsMintDocId() {";
var MARK_START = "function docsMarkBlankDocUnsaved(article) {";
var IS_UNSAVED_START = "function docsIsBlankDocUnsaved(article) {";
var CLEAR_START = "function docsClearBlankDocUnsaved(article) {";
var ENSURE_START = "function docsEnsureArticleDocId(article) {";
var ENSURE_END = "function $(s, r) {";
var RENDER_START = "function renderedDocId() {";
var RENDER_END = "function collabUrl(id) {";
var SYNC_START = "function sync() {";
var SYNC_END = "function scheduleSync() {";
var PERSIST_START = "function markActiveDocPersisted() {";
var PERSIST_END = "function processHtmx(host) {";
var SAVE_TITLE_START = "function saveTitle() {";
var SAVE_TITLE_END = "function startBlank() {";
var START_BLANK_START = "function startBlank() {";
var START_BLANK_END = "function docsRefreshSidebar() {";
var REFRESH_START = "function docsRefreshSidebar() {";
var REFRESH_END = "window.docsRefreshSidebar = docsRefreshSidebar;";

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

// (a) The unsaved flag is written at exactly ONE place: the mint branch of
// docsEnsureArticleDocId(), after docsMintDocId() produced a brand new UUID.
// A document that already has a real Drive id takes the `return id;` path and
// is never flagged. No other docs module may write the flag.
h.staticTest("#doc-collab-blank-reconnect unsaved flag is set only at the single UUID-mint point, never for a real Drive id", function () {
  var state = h.readRepo(STATE);
  var collab = h.readRepo(COLLAB);
  var boot = h.readRepo(BOOT);

  // Non-vacuity: the UUID minter, the flag setter/reader/clearer and the
  // id helper all ship, and blank start really invokes the id helper.
  contains(STATE, MINT_START, "UUID minter is shipped");
  contains(STATE, MARK_START, "blank-doc flag setter is shipped");
  contains(STATE, IS_UNSAVED_START, "blank-doc flag reader is shipped");
  contains(STATE, CLEAR_START, "blank-doc flag clearer is shipped");
  contains(STATE, ENSURE_START, "article id helper is shipped");
  contains(BOOT, START_BLANK_START, "blank-start path is shipped");

  var mint = between(state, MINT_START, MARK_START);
  spanHas(mint, "crypto.randomUUID()", "the mint point produces a real UUID");
  h.assertEqual(countIn(state, "crypto.randomUUID()"), 1, "exactly one UUID mint expression");

  var mark = between(state, MARK_START, IS_UNSAVED_START);
  spanHas(mark, "if (!article || !article.dataset) return;", "setter rejects a missing article");
  spanHas(mark, 'article.dataset.docsUnsaved = "1";', "setter writes the transient flag");
  sequence(mark, ["if (!article || !article.dataset) return;", 'article.dataset.docsUnsaved = "1";'],
    "flag setter order: guard, then write");
  h.assertEqual(countIn(state, 'article.dataset.docsUnsaved = "1";'), 1,
    "the flag is written exactly once in the state module");

  var reader = between(state, IS_UNSAVED_START, CLEAR_START);
  spanHas(reader, 'article.dataset.docsUnsaved === "1"', "reader checks the exact flag value");

  // The marking call lives ONLY inside the mint branch, after the new id is
  // assigned to the article. The real-id path returns before any marking.
  var ensure = between(state, ENSURE_START, ENSURE_END);
  spanHas(ensure, 'var id = article.dataset.docId || "";', "existing article id is read first");
  spanHas(ensure, 'if (!id || id === "current") {', "mint is gated on an absent/placeholder id");
  var mintBranch = between(ensure, 'if (!id || id === "current") {', "return id;");
  spanHas(mintBranch, "id = docsMintDocId();", "mint branch calls the UUID minter");
  spanHas(mintBranch, "article.dataset.docId = id;", "mint branch stores the new id");
  spanHas(mintBranch, "docsMarkBlankDocUnsaved(article);", "mint branch flags the new id unsaved");
  sequence(mintBranch, ["id = docsMintDocId();", "article.dataset.docId = id;", "docsMarkBlankDocUnsaved(article);"],
    "mint order: generate id, store it, flag it unsaved");
  sequence(ensure, [
    'var id = article.dataset.docId || "";',
    'if (!id || id === "current") {',
    "id = docsMintDocId();",
    "docsMarkBlankDocUnsaved(article);",
    "return id;"
  ], "existing id skips the flag; only a minted id is flagged before return");
  h.assertEqual(countIn(state, "docsMarkBlankDocUnsaved(article);"), 1,
    "exactly one flag-set call, inside the mint branch");
  h.assertEqual(countIn(state, "docsMarkBlankDocUnsaved("), 2,
    "one definition plus one call — no second marking site");
  h.assertEqual(countIn(collab, "docsMarkBlankDocUnsaved("), 0, "collab never writes the flag");
  h.assertEqual(countIn(boot, "docsMarkBlankDocUnsaved("), 0, "boot never writes the flag");

  // Non-vacuity: a blank start really reaches docsEnsureArticleDocId(), so the
  // mint-time flag is actually armed in production.
  var startBlank = between(boot, START_BLANK_START, START_BLANK_END);
  spanHas(startBlank, "docsEnsureArticleDocId(article);", "blank start mints and flags the new id");
  spanHas(startBlank, "flags a freshly minted id as unsaved", "blank start documents the flag lifecycle");
});

// (b) A flagged article must yield NO id from renderedDocId(), and sync()
// must return before its single gb.connect(opts). This is what prevents the
// refused-socket reconnect storm; deleting the guard or moving connect ahead
// of the `!id` return makes this case FAIL.
h.staticTest("#doc-collab-blank-reconnect renderedDocId refuses a flagged blank doc so sync() opens no socket", function () {
  var collab = h.readRepo(COLLAB);

  // Non-vacuity: the id extractor, the sync that connects and the flag reader
  // all ship.
  contains(COLLAB, RENDER_START, "renderedDocId is shipped");
  contains(COLLAB, SYNC_START, "sync is shipped");
  contains(COLLAB, "gb.connect(opts);", "a collab connection is shipped");
  contains(COLLAB, "docsIsBlankDocUnsaved(article)", "the flag reader is referenced by collab");

  var render = between(collab, RENDER_START, RENDER_END);
  spanHas(render, 'document.querySelector("article[contenteditable]")',
    "id is read from the editable article");
  spanHas(render, 'var id = article && article.dataset ? article.dataset.docId : "";',
    "data-doc-id is the id source");
  spanHas(render, 'if (id && typeof docsIsBlankDocUnsaved === "function" && docsIsBlankDocUnsaved(article)) return "";',
    "a flagged blank doc returns no id at all");
  spanHas(render, 'if (!id || id === "current" || !/^[A-Za-z0-9_.-]+$/.test(id)) return "";',
    "the pre-existing placeholder/format guard is still present");
  spanHas(render, "return id;", "only an unflagged, validated id is returned");
  sequence(render, [
    'document.querySelector("article[contenteditable]")',
    'var id = article && article.dataset ? article.dataset.docId : "";',
    'if (id && typeof docsIsBlankDocUnsaved === "function" && docsIsBlankDocUnsaved(article)) return "";',
    'if (!id || id === "current" || !/^[A-Za-z0-9_.-]+$/.test(id)) return "";',
    "return id;"
  ], "render order: read id, reject flagged blank doc, reject placeholders, return id");
  spanLacks(render, 'return "current"', "'current' is never returned as an id");

  // sync() consumes renderedDocId() first and bails on an empty id before it
  // can assign opts.docId or call connect().
  var sync = between(collab, SYNC_START, SYNC_END);
  spanHas(sync, "var id = renderedDocId();", "sync derives its id from the guarded extractor");
  spanHas(sync, "if (!id || id === currentDocId) return;", "empty id is a no-op");
  spanHas(sync, "var hadConnection = !!currentDocId;", "connect-once branch is keyed on prior id");
  spanHas(sync, "if (!hadConnection) {", "only the first valid id opens a socket");
  sequence(sync, [
    "var id = renderedDocId();",
    "if (!id || id === currentDocId) return;",
    "var hadConnection = !!currentDocId;",
    "if (!hadConnection) {",
    "opts.docId = id;",
    "gb.connect(opts);"
  ], "no-id return strictly precedes the single connect");

  var connectBranch = between(collab, "if (!hadConnection) {", "// Re-target without a second connect()");
  spanHas(connectBranch, "opts.docId = id;", "connect options carry the derived id");
  spanHas(connectBranch, "gb.connect(opts);", "connect lives inside the first-valid-id branch");
  h.assertEqual(countIn(collab, "gb.connect(opts);"), 1, "exactly one connect() on the wire path");

  // The re-target tail cannot open a second socket either.
  var reTarget = between(collab, "if (gb.opts) gb.opts.docId = id;", SYNC_END);
  h.assertEqual(countIn(reTarget, "gb.connect("), 0, "re-target never calls connect() again");
});

// (c) A successful save clears the flag and explicitly re-syncs collab, while
// the connect-once / re-target lifecycle stays intact. Both production save
// success hooks (autosave refresh and title blur) must call the clearer.
h.staticTest("#doc-collab-blank-reconnect successful save clears the flag, triggers docsSyncCollab and preserves connect-once re-targeting", function () {
  var state = h.readRepo(STATE);
  var collab = h.readRepo(COLLAB);
  var boot = h.readRepo(BOOT);

  // Non-vacuity: the clearer, the explicit sync bridge and both save hooks ship.
  contains(STATE, CLEAR_START, "flag clearer is shipped");
  contains(COLLAB, "window.docsSyncCollab = sync;", "sync is exposed as the save bridge");
  contains(BOOT, PERSIST_START, "persisted-doc hook is shipped");
  contains(BOOT, REFRESH_END, "autosave refresh hook is exposed");
  contains(BOOT, SAVE_TITLE_START, "title-save hook is shipped");

  var clear = between(state, CLEAR_START, ENSURE_START);
  spanHas(clear, "if (!docsIsBlankDocUnsaved(article)) return false;", "clear is a no-op for a real Drive doc");
  spanHas(clear, "delete article.dataset.docsUnsaved;", "clear removes the transient flag");
  spanHas(clear, "return true;", "clear reports that it actually removed a flag");
  sequence(clear, [
    "if (!docsIsBlankDocUnsaved(article)) return false;",
    "delete article.dataset.docsUnsaved;",
    "return true;"
  ], "clear order: check flag, delete it, report success");
  spanLacks(clear, "docsMarkBlankDocUnsaved(", "clear never re-arms the flag");

  var persist = between(boot, PERSIST_START, PERSIST_END);
  spanHas(persist, 'if (typeof docsClearBlankDocUnsaved !== "function") return;',
    "hook tolerates a missing clearer");
  spanHas(persist, "if (!article || !docsClearBlankDocUnsaved(article)) return;",
    "hook clears the active article's flag and no-ops for a real doc");
  spanHas(persist, 'if (typeof window.docsSyncCollab === "function") window.docsSyncCollab();',
    "hook re-syncs collab after the save persists the id");
  sequence(persist, [
    'if (typeof docsClearBlankDocUnsaved !== "function") return;',
    "if (!article || !docsClearBlankDocUnsaved(article)) return;",
    'if (typeof window.docsSyncCollab === "function") window.docsSyncCollab();'
  ], "persist order: guard, clear, then force a collab sync");
  h.assertEqual(countIn(boot, "window.docsSyncCollab();"), 1, "one explicit save-to-collab bridge");

  var saveTitle = between(boot, SAVE_TITLE_START, SAVE_TITLE_END);
  spanHas(saveTitle, 'status("All changes saved", false);', "title save reports success");
  spanHas(saveTitle, "markActiveDocPersisted();", "title save clears the flag on success");
  sequence(saveTitle, ['status("All changes saved", false);', "markActiveDocPersisted();"],
    "title save clears only after a successful response");

  var refresh = between(boot, REFRESH_START, REFRESH_END);
  spanHas(refresh, "markActiveDocPersisted();", "autosave success hook clears the flag");
  spanHas(refresh, "autosave success hook (02_save.js)", "refresh is documented as the autosave hook");

  // The lifecycle from 51_docs_collab_lifecycle.js is unchanged: one connect,
  // then re-target by mutating the singleton and using its guarded reconnect.
  var sync = between(collab, SYNC_START, SYNC_END);
  spanHas(sync, "var hadConnection = !!currentDocId;", "connect-once state is intact");
  spanHas(sync, "if (!hadConnection) {", "connect-once branch is intact");
  spanHas(sync, "if (gb.opts) gb.opts.docId = id;", "re-target updates singleton opts");
  spanHas(sync, "gb.docId = id;", "re-target updates singleton id");
  spanHas(sync, "gb.url = collabUrl(id);", "re-target updates the socket url");
  spanHas(sync, "gb.disconnect();", "re-target closes the old socket");
  spanHas(sync, "gb._scheduleReconnect();", "re-target uses the guarded reconnect");
  sequence(sync, ["gb.docId = id;", "gb.url = collabUrl(id);", "gb.disconnect();", "gb._scheduleReconnect();"],
    "re-target order: id, url, disconnect, schedule reconnect");
  h.assertEqual(countIn(collab, "gb.connect(opts);"), 1, "still exactly one connect()");
  h.assertEqual(countIn(collab, "window.docsSyncCollab = sync;"), 1, "one save bridge binding");
});

h.note("FOLLOW-UP #doc-collab-blank-reconnect: only the UUID-mint point flags a blank doc unsaved, renderedDocId withholds that id so sync() never opens a refused socket, and a successful save clears the flag and forces one guarded collab sync; source assertions only — no browser/server/cargo.");
