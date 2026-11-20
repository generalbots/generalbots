/**
 * #2511 labelled [STATIC] source assertions across the Office trio.
 *
 * These tests read shipped source only. They document exactly which required
 * translation features are NOT shipped, so no runtime PASS is fabricated for
 * missing behaviour. Real defects found while writing them are marked KNOWN
 * BUG; the one defect actually fixed (the untranslated modal) is guarded by a
 * REGRESSION test below instead.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var assertEqual = h.assertEqual;
var readFile = h.readFile;
var fileExists = h.fileExists;

var DOC_TRANSLATE = "docs/modules/17_translate.js";
var GLOSSARY_PANEL = "docs/modules/31_lang_panels_b.js";

function walkJs(relDir) {
  var root = h.path.join(h.SUITE_DIR, relDir);
  var out = [];
  (function visit(dir) {
    h.fs.readdirSync(dir, { withFileTypes: true }).forEach(function (entry) {
      var full = h.path.join(dir, entry.name);
      if (entry.isDirectory()) visit(full);
      else if (entry.name.slice(-3) === ".js") out.push(h.path.relative(h.SUITE_DIR, full));
    });
  })(root);
  return out.sort();
}

function joined(relPaths) {
  return relPaths.map(function (relPath) { return readFile(relPath); }).join("\n");
}

var SLIDES_JS = walkJs("slides");
var SHEET_JS = walkJs("sheet");
var DOCS_REVIEW_JS = walkJs("docs").filter(function (relPath) {
  return /docs\/modules\/(49|5[0-7])_review_/.test(relPath);
});

test("[STATIC] #2511: docs translation ships text apply + server-history undo only", function () {
  assert(fileExists(DOC_TRANSLATE), "the shipped docs translation module must exist");
  var src = readFile(DOC_TRANSLATE);
  assert(src.indexOf("/api/docs/ai/translate") >= 0, "docs translation endpoint must be present");
  assert(src.indexOf("/api/docs/autosave") >= 0, "autosave-based undo seeding must be present");
  assert(/GBCollabVersions/.test(src), "collaboration snapshot must be present");
  ["accept", "reject", "segment", "translationMemory", "glossary", "variant",
    "overflow", "formula", "do-not-translate", "outOfDate"].forEach(function (token) {
    assert(src.toLowerCase().indexOf(token.toLowerCase()) < 0,
      "docs translate module must NOT claim shipped support for " + token);
  });
});

test("[STATIC] #2511: layout-preserving deck translation is NOT shipped in Slides", function () {
  var names = SLIDES_JS.join("\n");
  var src = joined(SLIDES_JS);
  assert(SLIDES_JS.length > 50, "slides must ship its real module set");
  assert(!/17_translate|openTranslateModal|GBSlidesTranslate|translateSlide/.test(names + src),
    "no slides translation module or entry point may be claimed");
  assert(src.indexOf("/api/slides/ai/translate") < 0, "no slides translation endpoint is shipped");
  assert(!/layoutPreserv|textFit|translateOverflow|fitTranslated/.test(src),
    "no translation-specific layout/fit/overflow handling is shipped");
});

test("[STATIC] #2511: formula-safe sheet translation is NOT shipped in Sheets", function () {
  var src = joined(SHEET_JS);
  assert(SHEET_JS.length > 20, "sheet must ship its real module set");
  assert(/formula/i.test(src), "sheet must ship its existing formula engine (context for the gap)");
  assert(!/17_translate|openTranslateModal|GBSheetTranslate|translateSheet|translateCell/.test(src),
    "no sheet translation module or entry point may be claimed");
  assert(src.indexOf("/api/sheet/ai/translate") < 0, "no sheet translation endpoint is shipped");
  assert(!/formulaSafe|preserveFormulas|namedRangeTranslat|numberFormatTranslat/.test(src),
    "no formula/named-range/number-format protection for translation is shipped");
});

test("[STATIC] #2511: org glossary/termbase enforcement is NOT shipped (local list only)", function () {
  var docs = joined(walkJs("docs"));
  assert(docs.indexOf("docs-lang-glossary-local") >= 0, "shipped glossary is advertised as browser-local");
  assert(docs.indexOf("docs-lang-glossary-note") >= 0, "shipped UI must state the server-dictionary limitation");
  assert(!/\/api\/(glossary|termbase|dictionary)/.test(docs), "no org glossary/termbase endpoint is shipped");
  var src = readFile(DOC_TRANSLATE);
  assert(src.indexOf("glossary") < 0 && src.indexOf("termbase") < 0,
    "the translate apply path never consults a glossary or termbase");
});

test("[STATIC] #2511: per-segment review/accept workflow for translations is NOT shipped", function () {
  assert(DOCS_REVIEW_JS.length >= 5, "docs ships human-edit review modules (context for the gap)");
  var review = joined(DOCS_REVIEW_JS);
  assert(!/translateSegment|translationMemory|sideBySide|bilingual|acceptTranslation|rejectTranslation/i.test(review),
    "existing review modules are not wired to translation segments");
  var src = readFile(DOC_TRANSLATE);
  assert(!/accept|reject|approve|segment|review/i.test(src),
    "translate apply contains no accept/reject/approve review step");
});

test("[STATIC] #2511: linked language variants and out-of-date detection are NOT shipped", function () {
  var lang = joined([
    DOC_TRANSLATE,
    "docs/modules/25_lang_core.js",
    "docs/modules/25b_lang_data.js",
    "docs/modules/29_lang_readability.js"
  ]);
  assert(lang.indexOf("setLanguage") >= 0, "shipped code has a single selected-language state");
  assert(!/setLanguage\(/.test(readFile(DOC_TRANSLATE)),
    "translate apply does not record the target as a variant");
  assert(!/outOfDate|out.of.date|languageVersion|linkedVariant|variantOf|sourceVersion/i.test(lang),
    "no variant linkage or out-of-date detection is shipped");
  assert(!/pt-BR/.test(readFile("docs/modules/25b_lang_data.js")),
    "the shipped language catalog uses pt, not a pt-BR variant code");
});

test("[STATIC] #2511: per-language export/search indexing is NOT shipped in translate apply", function () {
  var src = readFile(DOC_TRANSLATE);
  assert(!/\/api\/.*export|indexLanguage|searchIndex|indexForSearch/i.test(src),
    "translate apply has no export or per-language search-index step");
  var exported = readFile("docs/modules/02b_export.js");
  assert(!/translat|language/i.test(exported),
    "the shipped export module has no language-variant parameter");
});

test("REGRESSION #2511: translate modal resolves every user-facing string via gbT catalog keys", function () {
  var src = readFile(DOC_TRANSLATE);
  var pairs = [
    ["docs-translate-title", "Translate document"],
    ["docs-translate-target-language", "Target language"],
    ["docs-translate-action", "Translate"],
    ["action-cancel", "Cancel"],
    ["docs-lang-lang-en", "English"],
    ["docs-lang-lang-pt", "Portuguese"],
    ["docs-lang-lang-es", "Spanish"],
    ["docs-lang-lang-fr", "French"],
    ["docs-lang-lang-de", "German"],
    ["docs-lang-lang-it", "Italian"]
  ];
  pairs.forEach(function (pair) {
    assert(src.indexOf('gbT("' + pair[0] + '", "' + pair[1] + '")') >= 0,
      "modal must resolve " + pair[1] + " via gbT key " + pair[0]);
  });
  assert(src.indexOf('gbT("docs-translating", "Translating...")') >= 0,
    "busy label must keep its existing catalogue key");
  assert(src.indexOf(">Translate document<") < 0 &&
    src.indexOf(">Target language") < 0 &&
    src.indexOf(">Cancel</button>") < 0 &&
    src.indexOf(">Translate</button>") < 0 &&
    src.indexOf(">English</option>") < 0,
    "modal HTML must not emit raw user-facing literals");
  // The values sent to the server stay full names; only display text is keyed.
  assert(src.indexOf('<option value="Portuguese">') >= 0,
    "option values must remain the server-facing language names (variant link still deferred)");
  var locales = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];
  var newKeys = ["docs-translate-title", "docs-translate-target-language", "docs-translate-action"];
  locales.forEach(function (loc) {
    var backlog = readFile("../../../botlib/locales/" + loc + "/ui-hardcoded-backlog.ftl");
    newKeys.forEach(function (key) {
      assert(new RegExp("^" + key + " = .+$", "m").test(backlog),
        loc + " catalogue must define " + key);
    });
    var lang = readFile("../../../botlib/locales/" + loc + "/docs-language.ftl");
    assert(/^docs-lang-lang-it = .+$/m.test(lang), loc + " catalogue must define docs-lang-lang-it");
  });
  assertEqual(/^docs-translate-title = Translate document$/m.test(
    readFile("../../../botlib/locales/en/ui-hardcoded-backlog.ftl")), true,
    "en fallback must match the shipped English literal");
});
