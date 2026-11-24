/**
 * [STATIC] deterministic guards for the #2457 fixes that landed after the
 * original office-fidelity harness. Every case reads real shipped source as
 * text and FAILS if the fix regresses: no runtime claim, no server, no browser.
 *
 * Covered fixes:
 *   - PPTX export writes native a:tbl / c:chart parts (never a flattened image)
 *   - Docs translate surfaces backend failure instead of pretending success
 *   - Docs nav renders real search snippets + click-to-navigation handlers
 *   - i18n: a representative docs-nav key exists in all 8 locale catalogs
 *   - No CDN links in the changed suite JS modules
 */
"use strict";

var h = require("./00_harness");

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(relPath, needle, msg) {
  var src = h.readRepo(relPath);
  h.assert(src.indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath);
}

var PPTX_EXPORT = "botserver/crates/botslides/src/pptx/export.rs";
var PPTX_TABLE = "botserver/crates/botslides/src/pptx/table.rs";
var PPTX_CHART = "botserver/crates/botslides/src/pptx/chart.rs";
var DOCS_TRANSLATE = "botui/ui/suite/docs/modules/17_translate.js";
var DOCS_NAV = "botui/ui/suite/docs/modules/20_nav_pane.js";
var LOCALES = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];
var I18N_KEY = "docs-nav-no-results";
var I18N_FILE = "docs-nav.ftl";

// 1. Native table/chart writers. The exporter dispatches table/chart elements to
//    dedicated writers; those emit OOXML shapes/parts, not a flattened image.
//    (The agent-side rate-limit error is surfaced by the backend as
//    result:"unavailable"/error and handled by the translate case below — there
//    is no literal PROMOTION_MODEL_LIMIT_REACHED token in the shipped JS.)
h.staticTest("PPTX export writes native table/chart XML, not a flattened image [pptx:tables] [pptx:charts]", function () {
  contains(PPTX_EXPORT, "table::table_shape_xml(element, id)", "native table writer dispatch");
  contains(PPTX_EXPORT, "chart::chart_frame_xml(element, id, charts)", "native chart writer dispatch");
  contains(PPTX_TABLE, "<a:tbl>", "native a:tbl table body");
  contains(PPTX_TABLE, "<p:graphicFrame>", "table graphicFrame wrapper");
  contains(PPTX_CHART, "<c:chartSpace", "native chart part");
  contains(PPTX_CHART, "<c:ser>", "cached chart series");
  contains(PPTX_EXPORT, "ppt/charts/", "chart part written into the package");
});

// 2. Translate honesty: the backend returns HTTP 200 with result:"unavailable"
//    + error on LLM failure/rate-limit. The UI must reject that and show an
//    explicit failure status — never the "Translated to X" success banner over
//    an unchanged document.
h.staticTest("Docs translate surfaces LLM failure instead of faking success", function () {
  contains(DOCS_TRANSLATE, 'j.result === "unavailable" || j.error', "backend failure detected");
  contains(DOCS_TRANSLATE, 'j.result === "success"', "only genuine success accepted");
  contains(DOCS_TRANSLATE, "setSaveStatus(\"Translation unavailable\", true)", "failure surfaced in status");
  var src = h.readRepo(DOCS_TRANSLATE);
  var failure = src.indexOf("setSaveStatus(\"Translation unavailable\", true)");
  var success = src.indexOf("setSaveStatus(\"Translated to \" + lang, false)");
  h.assert(failure !== -1 && success !== -1 && failure < success,
    "failure status must be set before any success status");
});

// 3. Search snippets + click nav: result rows show a context snippet and carry
//    a match index that resolveJump() turns into a real scroll target.
h.staticTest("Docs nav renders real search snippets and click-to-navigation", function () {
  contains(DOCS_NAV, "function renderResults(section)", "search results renderer");
  contains(DOCS_NAV, "blockText.slice(0, 120)", "context snippet from matched block text");
  contains(DOCS_NAV, "data-nav-match", "match-index row attribute");
  contains(DOCS_NAV, "searchMatches = []", "stale matches cleared on render");
  contains(DOCS_NAV, 'event.target.closest("[data-nav-jump]")', "jump-row click target");
  contains(DOCS_NAV, "NS.jumpTo(resolveJump(jump))", "jump handler invoked");
  contains(DOCS_NAV, "searchMatches[matchIndex]", "match index resolved to a target node");
});

// 4. i18n: the representative key used by the search-snippet fix above must be
//    defined in every shipped locale catalog, not just en.
h.staticTest("i18n: representative docs-nav key exists in all 8 locale catalogs", function () {
  contains(DOCS_NAV, '"' + I18N_KEY + '"', "key used by shipped JS");
  LOCALES.forEach(function (loc) {
    contains("botlib/locales/" + loc + "/" + I18N_FILE, I18N_KEY + " =", loc + " catalog");
  });
});

// 5. Local assets only: the changed suite modules must not reference a CDN.
h.staticTest("No CDN links in the changed suite JS modules", function () {
  var forbidden = [/\bhttps?:\/\//i, /\b(?:cdn|unpkg|jsdelivr|cdnjs)\b/i];
  [DOCS_TRANSLATE, DOCS_NAV].forEach(function (relPath) {
    var src = h.readRepo(relPath);
    forbidden.forEach(function (re) {
      h.assert(!re.test(src), relPath + " must not reference an external CDN :: " + re);
    });
  });
});
