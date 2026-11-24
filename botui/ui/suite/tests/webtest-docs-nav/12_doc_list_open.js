/**
 * [UAT:DOC-05] deterministic static carrier for the "open my last document"
 * code path. UAT.md DOC-05 ("Open my last document about the Q3 report")
 * requires: (1) the document is found/named, (2) it opens in a Docs card with
 * its saved content, (3) no duplicate card is created while it loads. The
 * live, LLM-free re-run is recorded in issue #2031: a seeded doc created via
 * POST /api/docs/new, listed in the Docs sidebar, opened by id, with exactly
 * one Docs window before and after the open.
 *
 * These are [STATIC] source-contract tests: they prove the shipped
 * list -> open-by-id -> saved content -> single-window path exists and is
 * wired. They do NOT claim a browser run or a chat answer. DOC-05 step 1
 * (the bot naming the document from a chat phrase) stays a browser-only gate
 * because it needs the LLM.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var readFile = h.readFile;

var FRAGMENTS = "../../../botserver/crates/botdocs/src/ui_fragments/mod.rs";
var SIDEBARS = "../../../botserver/crates/botdocs/src/ui_fragments/sidebars.rs";
var BOOT = "docs/modules/00_boot.js";
var INIT = "docs/modules/99_init.js";
var DOCS_HTML = "docs/docs.html";
var WM_DEEPLINK = "js/window-manager/06_deeplinks.js";
var WM_CORE = "js/window-manager/02_class_core.js";
var LOCALES = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];

test("[UAT:DOC-05][STATIC] Docs list renders open-by-id cards and registers both fragment routes", function () {
  var src = readFile(FRAGMENTS);
  assert(src.indexOf("pub fn render_metadata_card") >= 0, "metadata card renderer shipped");
  assert(src.indexOf('class="dc-metadata-card"') >= 0, "list card class shipped");
  var cardAt = src.indexOf('hx-get="/suite/docs/fragments/view-by-id?id={id}"');
  assert(cardAt >= 0, "each saved-doc card opens its document by id");
  assert(src.indexOf('hx-target="#docs-content"', cardAt) >= 0, "card targets the docs content host");
  assert(src.indexOf('hx-swap="innerHTML"', cardAt) >= 0, "card replaces the open document");
  assert(src.indexOf('"/suite/docs/fragments/doc-list"') >= 0, "list endpoint registered");
  assert(src.indexOf('"/suite/docs/fragments/view-by-id"') >= 0, "open-by-id endpoint registered");
  assert(src.indexOf("get(handle_document_view_by_id)") >= 0, "route bound to the by-id handler");

  var boot = readFile(BOOT);
  assert(boot.indexOf('"/suite/docs/fragments/doc-list"') >= 0, "client renders the server list fragment");
  assert(boot.indexOf('class="dc-file-row" data-doc-id="') >= 0, "offline fallback list carries doc ids");
  assert(boot.indexOf('"/api/docs/load?id="') >= 0, "fallback list opens the doc by id");
});

test("[UAT:DOC-05][STATIC] by-id open reads saved Drive content into the editor hosts", function () {
  var sidebars = readFile(SIDEBARS);
  assert(sidebars.indexOf("pub async fn handle_document_view_by_id") >= 0, "by-id handler shipped");
  assert(sidebars.indexOf("load_document_from_drive") >= 0,
    "saved document is loaded from Drive, not fabricated client-side");
  assert(sidebars.indexOf('"id": doc.id') >= 0, "saved id forwarded to the view");
  assert(sidebars.indexOf('"title": doc.title') >= 0, "saved title forwarded to the view");
  assert(sidebars.indexOf('"content": doc.content') >= 0, "saved content forwarded to the view");
  assert(sidebars.indexOf('id="doc-view" data-id="{id}" data-title="{title}"') >= 0,
    "view carries the id/title of the opened doc");
  assert(sidebars.indexOf('data-doc-id="{id}" contenteditable="true"') >= 0,
    "opened article is the editable, id-bearing host");
  assert(sidebars.indexOf("{content}</article>") >= 0, "saved content is written into that article");

  var html = readFile(DOCS_HTML);
  assert(html.indexOf('id="docTitle"') >= 0, "title input host shipped");
  assert(html.indexOf('id="docs-content"') >= 0, "content host shipped");
  var boot = readFile(BOOT);
  assert(boot.indexOf('document.getElementById("docTitle")') >= 0, "editor reads the shipped title host");
  assert(boot.indexOf('document.querySelector("#docs-content article[contenteditable]")') >= 0,
    "editor reads the same article the saved content is loaded into");

  var init = readFile(INIT);
  assert(init.indexOf("function loadDriveFileIntoEditor(linkParams)") >= 0, "deep-link open handler shipped");
  assert(init.indexOf("var eventParams = linkParams || null;") >= 0,
    "re-target event params win over stale globals (#2276)");
  assert(init.indexOf("docsSanitizeHtml(doc.content") >= 0, "saved content is rendered into the editor");
  assert(init.indexOf("article.dataset.docId = doc.id || file") >= 0, "opened article keeps the doc id");
  assert(init.indexOf("docsT('docs-opened-from-drive'") >= 0, "open status is localized");
});

test("[UAT:DOC-05][STATIC] list and open status use catalog keys available in all eight locales", function () {
  var boot = readFile(BOOT);
  assert(boot.indexOf('docsT("docs-no-documents-yet"') >= 0,
    "empty Docs list uses a catalog key, not a baked literal");
  var init = readFile(INIT);
  assert(init.indexOf("docsT('docs-opened-from-drive'") >= 0, "opened status uses a catalog key");
  assert(init.indexOf("docsT(") >= 0 && init.indexOf("'docs-open-from-drive-failed'") >= 0,
    "failed open status is localized too");
  LOCALES.forEach(function (loc) {
    var catalog = readFile("../../../botlib/locales/" + loc + "/ui-apps-runtime.ftl");
    assert(catalog.indexOf("docs-no-documents-yet =") >= 0,
      loc + " catalog defines the Docs list empty-state key");
  });
});

test("[UAT:DOC-05][STATIC] deep-link re-targets one Docs window instead of creating a duplicate card", function () {
  var wm = readFile(WM_DEEPLINK);
  assert(wm.indexOf("openDeepLink(appId, params, opts)") >= 0, "deep-link entry point shipped");
  var targetAt = wm.indexOf("const targetId = forceNew ?");
  assert(targetAt >= 0, "window id derives from the app id (dedup key)");
  var dedupAt = wm.indexOf("const existed = !forceNew && this.getWindow(targetId) !== null;");
  assert(dedupAt > targetAt, "existing window is detected before opening");
  var openAt = wm.indexOf('this.open(targetId, title, "", {', dedupAt);
  assert(openAt > dedupAt, "the same id is used to focus the existing window");
  var branchAt = wm.indexOf("if (existed) {", openAt);
  assert(branchAt > openAt, "re-target branch present");
  var eventAt = wm.indexOf('new CustomEvent("gb:deep-link"', branchAt);
  assert(eventAt > branchAt, "re-target dispatches gb:deep-link to the open Docs window");
  var qsAt = wm.indexOf("const qs = Object.keys(params || {})", branchAt);
  assert(qsAt > eventAt, "re-target returns before a second body is fetched");
  assert(wm.indexOf("window-body-${targetId}") >= 0, "one body host per app id");

  var core = readFile(WM_CORE);
  var existingAt = core.indexOf("const existingWindow = this.openWindows.find((w) => w.id === id);");
  assert(existingAt >= 0, "open() deduplicates by window id");
  var block = core.slice(existingAt, existingAt + 320);
  assert(block.indexOf("if (existingWindow) {") >= 0, "second open takes the existing-window branch");
  assert(block.indexOf("this.focus(id);") >= 0 && block.indexOf("return;") >= 0,
    "second open focuses and returns without creating another card");

  var init = readFile(INIT);
  assert(init.indexOf('document.addEventListener("gb:deep-link", function (event)') >= 0,
    "Docs listens for the re-target event");
  assert(init.indexOf('appId.indexOf("docs-") === 0') >= 0, "only Docs windows handle that re-target");
});
