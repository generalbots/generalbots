/**
 * #2505 labelled [STATIC] source assertions and KNOWN BUG characterisations.
 *
 * STATIC checks read shipped source/catalogs only: they document criteria the
 * issue asks for that are NOT shipped and never produce a runtime PASS. The
 * KNOWN BUGs execute shipped code (i18n fallback) or pair source facts with an
 * executed observation; the fixes belong to production, not to this harness.
 */
"use strict";

var h = require("./00_harness");
var fx = require("./02_sandbox");
var staticTest = h.staticTest;
var knownBug = h.knownBug;
var assert = h.assert;
var equal = h.assertEqual;
var read = h.readFile;
var repo = h.readRepo;

var LOCALES = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];
var PALETTE = "js/command-palette.js";
var SEARCH = "js/gb-search.js";
var ENDPOINT = "botserver/src/main_module/routes/unified_search.rs";
var SEARCH_RS = "botserver/crates/botsearch/src/search/search.rs";
var SEARCH_MOD = "botserver/crates/botsearch/src/search/mod.rs";
var SEARCH_MIGRATION = "botserver/crates/botsearch/src/search/create_search_index_migration.rs";

function contains(text, token, msg) {
  assert(text.indexOf(token) >= 0, msg || "missing token: " + token);
}
function lacks(text, token, msg) {
  assert(text.indexOf(token) < 0, msg || "unexpected token: " + token);
}

staticTest("#2505: both shipped browser surfaces are wired into the desktop shell", function () {
  var desktop = read("desktop.html");
  contains(desktop, "/suite/js/command-palette.js", "palette script tag");
  contains(desktop, "/suite/js/gb-search.js", "search script tag");
  contains(read("desktop-inline/06_taskbar_search.js"), "GBUniversalSearch.mount",
    "taskbar mounts the universal search bar");
  var palette = read(PALETTE);
  contains(palette, "e.stopImmediatePropagation()", "Ctrl+K owns the shortcut");
  contains(palette, "results.sort(", "results are ranked before display");
  contains(palette, "function escapeHtml(s)", "rendered titles are escaped");
});

staticTest("#2505: the unified endpoint is org-permissioned and product-gated", function () {
  var src = repo(ENDPOINT);
  contains(src, "/api/ui/search", "unified endpoint path");
  contains(src, "org_column: Option<&'static str>", "per-source tenant column");
  contains(src, "websearch_to_tsquery('english', $1)", "bound full-text query");
  contains(src, "ORDER BY ts_rank(", "ranked results");
  contains(src, "Role::Admin", "admin bypass is explicit");
  contains(src, "u.organization_id.unwrap_or_else(uuid::Uuid::nil)", "anonymous nil scope");
  contains(src, "PRODUCT_CONFIG", "edition gating");
  contains(src, "enabled_apps.contains(source.app)", "sources outside the edition are skipped");
  contains(src, "bot_id IN (SELECT id FROM bots WHERE org_id = '{id}')",
    "drive and KB rows are scoped through the owning bot");
  contains(src, "diesel::sql_query(&sql)", "SQL is a hardcoded expression");
  contains(src, "MAX_QUERY_LEN: usize = 128", "query length is bounded");
  contains(src, "MAX_RESULTS_PER_SOURCE: i64 = 8", "per-source result cap");
});

staticTest("#2505: the FTS index crate ships snippets, ranking and reindex tooling", function () {
  var sql = repo(SEARCH_RS);
  contains(sql, "StartSel=<mark>, StopSel=</mark>", "highlighted snippet markup");
  contains(sql, "ts_headline($1, content, plainto_tsquery($1, $2)", "snippet generation");
  contains(sql, "ts_rank_cd(search_vector, plainto_tsquery($1, $2))", "ranked score");
  contains(sql, "WHERE organization_id = $3", "index rows are tenant-filtered");
  contains(sql, "ORDER BY score DESC", "score ordering");
  contains(sql, "{date_filter}", "date filters are supported");
  contains(sql, "pub async fn delete_document(", "delete path exists");
  contains(sql, "DELETE FROM search_index WHERE organization_id = $1 AND id = $2 AND source = $3",
    "delete is tenant and source scoped");
  var migration = repo(SEARCH_MIGRATION);
  contains(migration, "CREATE TABLE IF NOT EXISTS search_index", "index table");
  contains(migration, "organization_id UUID NOT NULL", "tenant column");
  contains(migration, "USING GIN(search_vector)", "GIN index");
  contains(migration, "BEFORE INSERT OR UPDATE", "freshness trigger");
  assert(h.repoFileExists("botserver/crates/botsearch/src/search/reindex_source.rs"),
    "reindex tool exists");
  contains(repo("botserver/crates/botsearch/src/routes.rs"), "/api/search/reindex/:source",
    "reindex route exists");
});

staticTest("#2505 GAP: no unified index over Docs footnotes, slide notes, sheet cells or PDF OCR", function () {
  var endpoint = repo(ENDPOINT);
  var mod = repo(SEARCH_MOD);
  ["footnote", "ocr", "cell_value", "slide_note", "presentation_slides", "pdf_text"].forEach(function (token) {
    lacks(endpoint, token, "endpoint has no " + token + " source");
    lacks(mod, token, "index crate has no " + token + " source");
  });
  contains(mod, "pub enum SearchSource", "index source enum is the coverage contract");
  ["Email", "Drive", "Calendar", "Tasks", "Transcription", "Chat", "Contacts", "Notes"]
    .forEach(function (variant) { contains(mod, variant, "source variant " + variant); });
  ["Docs", "Slides", "Sheet", "PDF"].forEach(function (variant) {
    var re = new RegExp("^\\s*" + variant + "\\s*[,}]", "m");
    assert(!re.test(mod), "unexpected office variant " + variant + " in SearchSource");
  });
  contains(endpoint, "searchable_columns: &[\"name\", \"file_path\"]",
    "drive search is filename metadata only");
  contains(endpoint, "searchable_columns: &[\"file_path\", \"collection_name\"]",
    "KB search is collection metadata only");
  h.note("GAP (not shipped): acceptance criterion 'one query finds a Docs footnote, a slide note and a sheet cell' has no index writer or source. [STATIC] only.");
});

staticTest("#2505 GAP: permission filtering is org-level, not per-object ACL", function () {
  var endpoint = repo(ENDPOINT);
  var sql = repo(SEARCH_RS);
  contains(endpoint, "Some(uuid::Uuid::nil())", "anonymous callers get the global scope");
  contains(repo(SEARCH_MOD), "pub user_id: Option<Uuid>", "query model carries a user id");
  var usesUser = /user_id\s*(?:=|IN|LIKE|=)/.test(sql);
  assert(!usesUser, "no SQL predicate consumes user_id — ACL is not per user");
  lacks(sql, "acl", "no ACL column is joined in the index query");
  lacks(endpoint, "can_open|can_read|permission", "no per-object permission predicate");
  h.note("GAP (not shipped): 'no snippet from an object the user cannot open, verified by an ACL test' is only org-scoped today. A real ACL/bucket test needs a running server. [STATIC] only.");
});

staticTest("#2505 GAP: shipped deep links stop at app landing pages", function () {
  var endpoint = repo(ENDPOINT);
  var search = read(SEARCH);
  contains(endpoint, "\"url\": source.url", "search rows carry a static app URL");
  ["page_number", "slide_index", "cell_ref", "deep_link"].forEach(function (token) {
    lacks(endpoint, token, "no " + token + " in search result mapping");
  });
  contains(search, "r.type === \"person\" || r.type === \"contact\"",
    "only person/contact rows deep-link");
  contains(search, "person_id: r.id", "person deep link carries the record id");
  var palette = read(PALETTE);
  contains(palette, "r.deepLink.match(", "palette commands support link templates");
  contains(palette, "{key?}", "optional placeholders are documented in source");
  h.note("GAP (not shipped): 'deep links open the exact page/slide/cell' has no producer — /api/ui/search emits an app landing URL and the bar deep-links people only. CDP verification is deferred. [STATIC] only.");
});

staticTest("#2505 GAP: filters, saved/recent searches, analytics and freshness SLA are absent", function () {
  var all = repo(ENDPOINT) + repo(SEARCH_RS) + repo(SEARCH_MOD) + read(SEARCH);
  ["saved_search", "savedSearch", "recent_search", "recentSearch",
    "zero-result", "clickThrough", "click_through", "shared-with-me", "sharedWithMe",
    "workspace_filter", "index_freshness", "60 s"].forEach(function (token) {
    lacks(all, token, "no " + token + " in shipped search surfaces");
  });
  contains(repo(SEARCH_RS), "{date_filter}", "only a date filter exists today");
  h.note("DEFERRED (browser/CI only): p95 < 300 ms at 100k objects, 60 s edit freshness, immediate ACL/delete propagation and keyboard-only end-to-end flows. No runtime PASS is claimed here.");
});

staticTest("#2505: shipped palette keys exist in all eight locale catalogs", function () {
  ["palette-placeholder", "palette-open-app", "palette-kind-app", "palette-hint",
    "palette-empty", "palette-value-prompt", "palette-execute", "palette-use-command"]
    .forEach(function (key) {
      LOCALES.forEach(function (locale) {
        var catalog = h.readRepo("botlib/locales/" + locale + "/ui-shell.ftl");
        assert(new RegExp("^" + key + "\\s*=", "m").test(catalog),
          locale + " missing " + key);
      });
    });
});

knownBug("#2505 KNOWN BUG: three palette keys are absent from every locale catalog", function () {
  var missing = ["palette-show-tour", "palette-show-tour-hint", "palette-kind-tour"];
  var palette = read(PALETTE);
  missing.forEach(function (key) {
    contains(palette, '"' + key + '"', "shipped code asks gbT for " + key);
    LOCALES.forEach(function (locale) {
      var catalog = h.readRepo("botlib/locales/" + locale + "/ui-shell.ftl");
      assert(!new RegExp("^" + key + "\\s*=", "m").test(catalog),
        locale + " unexpectedly defines " + key);
    });
  });
  /* Execute the shipped render with a tour so the authored fallback is used. */
  var f = fx.create({ tourIds: ["drive"] });
  f.ctrlK();
  equal(f.gbTCalls.filter(function (call) { return call.key === "palette-show-tour"; }).length,
    1, "gbT was called with the missing key");
  assert(f.overlay().innerHTML.indexOf("Show tour: Drive") >= 0,
    "OBSERVED: authored English fallback rendered instead of a catalog translation");
  h.note("KNOWN BUG (reported, not patched): js/command-palette.js uses palette-show-tour, palette-show-tour-hint and palette-kind-tour, but none of the eight ui-shell.ftl catalogs define them. gbT returns the authored English fallback, so those three labels are untranslated in every locale. Add the keys to all catalogs in production.");
});
