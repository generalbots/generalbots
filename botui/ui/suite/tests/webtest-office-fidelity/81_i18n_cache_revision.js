/**
 * #2923 / #2935-smoke — static guard for the i18n catalogue cache revision.
 *
 * The browser smoke still showed status-sending / chat-slow-hint missing after
 * all eight .ftl catalogues were updated: botserver loads catalogues at boot,
 * and the browser kept `gb-i18n-cache-<locale>-<productVersion>` in localStorage.
 * A catalogue-only edit does not change the product version, so that
 * version-keyed entry stayed stale. The fix adds an explicit catalogue revision
 * to getCacheKey so those entries are discarded exactly once after the edit.
 *
 * Probes (all source-only; no browser/server/cargo/network/locale change):
 *  (a) getCacheKey pins locale + version + catalogue revision segments;
 *  (b) the revision constant is declared once with the reason comment;
 *  (c) the version-keyed boot path and the #2921 dev-invalidation survive;
 *  (d) no new user-facing literal (nor gbT inline fallback) was added;
 *  (e) the fidelity runner requires this guard.
 *
 * GB_I18N_PROBE_ROOT may point at a /tmp tree holding a mutated copy of
 * i18n.js. Removing the revision suffix from getCacheKey must fail (a).
 * The driver owns issue closure and live browser UAT.
 */
"use strict";

var h = require("./00_harness");
var fs = require("fs");
var path = require("path");

var I18N = "botui/ui/suite/js/i18n.js";
var RUNNER = "botui/ui/suite/tests/webtest-office-fidelity.js";
var CACHE_KEY_EXPECTED = "gb-i18n-cache-${locale}-${cacheBuild}-${CATALOG_REVISION}";

function readSrc(rel) {
  var probe = process.env.GB_I18N_PROBE_ROOT;
  if (probe) {
    var candidate = path.join(probe, rel);
    if (fs.existsSync(candidate)) return fs.readFileSync(candidate, "utf8");
  }
  return h.readRepo(rel);
}
function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1, (msg || "source must contain") + " <" + needle + ">");
}
function absent(src, needle, msg) {
  h.assert(src.indexOf(needle) === -1, (msg || "source must not contain") + " <" + needle + ">");
}
function span(src, startMarker, endMarker) {
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + ">");
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + ">");
  h.assert(end > start, "empty span between <" + startMarker + "> and <" + endMarker + ">");
  return src.slice(start, end);
}
function countIn(src, needle) { return src.split(needle).length - 1; }

h.staticTest("#2923/#2935-smoke getCacheKey pins locale, version and catalogue revision", function () {
  var src = readSrc(I18N);
  var fn = span(src, "function getCacheKey(locale) {", "function getCachedTranslations(locale) {");
  h.assert(fn.length > 60, "non-vacuity: getCacheKey body must be substantial, got " + fn.length);
  contains(fn, CACHE_KEY_EXPECTED, "cache key must carry locale + version + revision");
  h.assert(fn.indexOf("${cacheBuild}") !== -1, "the product-version segment must survive");
  h.assert(fn.indexOf("${CATALOG_REVISION}") !== -1, "the catalogue-revision segment must be present");
  h.assert(fn.indexOf("${CATALOG_REVISION}") > fn.indexOf("${cacheBuild}"),
    "revision suffix must come after the version segment");
});

h.staticTest("#2923/#2935-smoke the revision is declared once with its reason comment", function () {
  var src = readSrc(I18N);
  h.assertEqual(countIn(src, "const CATALOG_REVISION"), 1,
    "exactly one revision declaration (no scattered/duplicated bump)");
  var decl = span(src, "const CATALOG_REVISION", ";");
  contains(decl, '= "r2"', "the pinned catalogue revision value is r2");
  var comment = span(src, "// #2087", "const CATALOG_REVISION");
  h.assert(comment.length > 100,
    "non-vacuity: the reason comment must be substantial, got " + comment.length);
  contains(comment, "#2923/#2935-smoke", "both issues are named in the reason comment");
  contains(comment, "catalogue-only", "reason: catalogues change without a version bump");
  contains(comment, "version-keyed", "reason mentions version-keyed staleness");
  contains(comment, "exactly once", "reason states the one-shot invalidation");
});

h.staticTest("#2923/#2935-smoke version-keyed boot and #2921 dev invalidation survive", function () {
  var src = readSrc(I18N);
  var start = src.indexOf("let productVersion = null;");
  var end = src.indexOf('const response = await fetch("/api/i18n/locales")', start);
  h.assert(start !== -1 && end > start, "locate the /api/product boot block in i18n.js");
  var block = src.slice(start, end);
  h.assert(block.length > 200, "non-vacuity: boot block must be substantial, got " + block.length);
  contains(block, "if (productVersion)", "a resolved version takes the normal cached path");
  contains(block, "cacheBuild = productVersion;", "the cache key still carries the real build version");
  contains(block, 'cacheBuild = "dev";', "offline boots keep the dev key");
  contains(block, "clearCache();", "a missing build version must discard the stale dev cache (#2921)");
  h.assert(src.indexOf("const CACHE_TTL_MS = 3600000;") !== -1,
    "normal versioned loads keep their one-hour cache TTL");
  h.assert(src.indexOf("if (age < CACHE_TTL_MS)") !== -1,
    "normal versioned loads still read the cache");
  var clear = span(src, "function clearCache() {", "window.i18n = {");
  contains(clear, 'key.startsWith("gb-i18n-cache-")',
    "clearCache still sweeps every revisioned cache key shape");
});

// Pre-fix set of sentence-like string literals in i18n.js. Every entry is a
// technical log/debug (i18n:, [gb-i18n], "use strict") or the existing HTTP
// status message — none is user-facing UI copy. If a future change legitimately
// edits log text, update this baseline deliberately; adding a new English or
// localized UI literal must FAIL this guard.
var PINNED_SENTENCE_LITERALS = [
  " (English fallback has no entry either)",
  "HTTP ${response.status}",
  "[gb-i18n] Locale persistence failed:",
  "i18n: Already initialized, skipping",
  "i18n: Cache check for ${locale}: age=${Math.round(age/1000)}s, valid=${age < CACHE_TTL_MS}, keys=${Object.keys(data || {}).length}",
  "i18n: Cache miss, fetching from API for ${locale}",
  "i18n: Cleared all translation caches",
  "i18n: Detected locale: ${locale}",
  "i18n: Failed to fetch available locales",
  "i18n: Failed to fetch translations for ${locale}",
  "i18n: Failed to read cache",
  "i18n: Failed to write cache",
  "i18n: Falling back to ${DEFAULT_LOCALE}",
  "i18n: Fetching translations for locale: ${locale}",
  "i18n: Initialization complete, current locale: ${currentLocale}",
  "i18n: Initialization failed",
  "i18n: Initialization started",
  "i18n: Loaded ${Object.keys(result.translations || {}).length} translations for ${locale}",
  "i18n: Missing translation key: ${key} in ${currentLocale}",
  "i18n: No translations found, using minimal fallback",
  "i18n: Using cached translations for ${locale}",
  "i18n: loadTranslations called for locale: ${locale}",
  "use strict",
];

h.staticTest("#2923/#2935-smoke no new user-facing literal or gbT fallback was added", function () {
  var src = readSrc(I18N);
  h.assert(PINNED_SENTENCE_LITERALS.length > 15,
    "non-vacuity: pinned literal baseline must be populated, got " + PINNED_SENTENCE_LITERALS.length);

  var re = /(["'`])((?:\\.|(?!\1)[^\\])*?)\1/g;
  var found = new Set();
  var match;
  while ((match = re.exec(src)) !== null) {
    if (/\s/.test(match[2]) && /[A-Za-z]/.test(match[2])) found.add(match[2]);
  }
  var actual = Array.from(found).sort();
  var expected = PINNED_SENTENCE_LITERALS.slice().sort();
  h.assertEqual(actual.length, expected.length,
    "sentence-like literal count changed (a user-facing string may have been added)");
  h.assert(JSON.stringify(actual) === JSON.stringify(expected),
    "sentence-like literals differ from the pinned baseline:\n  actual:   " +
    JSON.stringify(actual) + "\n  expected: " + JSON.stringify(expected));

  // The only gbT( text in the module is the #2921 comment, never a call with an
  // inline English fallback. Strip line comments, then require zero calls.
  var code = src.replace(/\/\/.*/g, "");
  h.assertEqual(countIn(code, "gbT("), 0, "no inline gbT() fallback call may be added");

  var minimal = span(src, "const MINIMAL_FALLBACK", "};");
  h.assert(minimal.indexOf('"label-loading": "Loading..."') !== -1 &&
           minimal.indexOf('"status-error": "Error"') !== -1 &&
           minimal.indexOf('"action-retry": "Retry"') !== -1,
    "the three original minimal-fallback literals are preserved verbatim");
});

h.staticTest("#2923/#2935-smoke the fidelity runner requires this guard", function () {
  contains(readSrc(RUNNER), 'require("./webtest-office-fidelity/81_i18n_cache_revision")',
    "guard 81 must be wired into the fidelity runner");
});

h.note("#2923/#2935-smoke: offline static proof only — no browser/server/cargo, no locale edit. Driver owns the live Chrome re-smoke and issue closure.");
