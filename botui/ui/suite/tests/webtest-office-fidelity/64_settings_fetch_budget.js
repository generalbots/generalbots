/**
 * #2936 — deterministic static coverage for the settings "This workspace"
 * card fetch budget. The card used to call renderIdentity/renderVersions/
 * renderPlan on every htmx:afterSwap, each issuing a fresh fetch — including
 * /api/auth/me — which produced a 429 storm under rapid fragment swaps.
 *
 * Cases (all literal source assertions, no execution, no browser):
 *   (a) one memoized fetch path, keyed by panel load (token + edition); all
 *       three chips route through it and never call fetch() directly;
 *   (b) exactly ONE guarded htmx:afterSwap listener that never clears the
 *       cache, with cache invalidation confined to gb-product-config;
 *   (c) Retry-After parsing (seconds + HTTP-date), bounded exponential
 *       backoff and a hard retry cap of 2 with a stop-on-exhaustion path;
 *   (d) non-vacuity: the original chip strings and endpoints survive, no new
 *       i18n keys are introduced, and the module stays under 450 lines.
 *
 * Reverting the memoization, the afterSwap guard, Retry-After handling or the
 * retry cap flips the matching case to FAIL. The driver owns issue closure.
 */
"use strict";

var h = require("./00_harness");

var CARD = "botui/ui/suite/settings/settings-workspace-card.js";

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1, (msg || "source must contain") + " <" + needle + ">");
}

/** Absence assertion — a forbidden token must not appear in the given span. */
function absent(src, needle, msg) {
  h.assert(src.indexOf(needle) === -1,
    (msg || "source must not contain") + " <" + needle + "> :: " + src.slice(0, 200));
}

/** Exact, non-empty source span between two literal markers (end exclusive). */
function span(src, startMarker, endMarker) {
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + ">");
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + ">");
  h.assert(end > start, "empty source span between <" + startMarker + "> and <" + endMarker + ">");
  return src.slice(start, end);
}

/** Assert `before` occurs strictly earlier than `after` inside a source span. */
function ordered(src, before, after, msg) {
  var b = src.indexOf(before);
  var a = src.indexOf(after);
  h.assert(b !== -1 && a !== -1 && b < a,
    (msg || "ordering regressed") + " :: <" + before + "> before <" + after + ">");
}

/** Count exact, literal occurrences of a needle in a source string. */
function countIn(src, needle) {
  return src.split(needle).length - 1;
}

// (a) A single memoized fetch path keyed by the panel load. The cache hit is
// returned BEFORE any fetch() call, and all three chips use memoFetch rather
// than calling fetch() themselves.
h.staticTest("#2936 one memoized panel-load fetch path serves identity, versions and plan; no direct fetch() in the chips", function () {
  var src = h.readRepo(CARD);
  h.assert(src.length > 4000, "workspace-card source must be non-trivial");

  contains(src, "var fetchCache = Object.create(null);", "cache container ships");
  contains(src, "function cacheId(key) {", "panel-load key resolver ships");
  var keyFn = span(src, "function cacheId(key) {", "function retryDelayMs(response, attempt) {");
  contains(keyFn, 'return key + "|" + token() + "|" + edition();',
    "cache is keyed by auth token plus active edition");

  contains(src, "function memoFetch(key, url, options) {", "memoized fetch ships");
  var memo = span(src, "function memoFetch(key, url, options) {", "function clearFetchCache() {");
  contains(memo, "var id = cacheId(key);", "cache id is derived from the panel load");
  contains(memo, "var hit = fetchCache[id];", "cache is read first");
  contains(memo, "if (hit) { return hit; }", "a cache hit returns before any network call");
  contains(memo, "return fetch(url, options).then(function (response) {",
    "the single real fetch call lives inside memoFetch");
  contains(memo, "var pending = Promise.resolve().then(run);", "the pending promise is created once");
  contains(memo, "fetchCache[id] = pending;", "the promise is memoized, not just the result");
  ordered(memo, "if (hit) { return hit; }", "return fetch(url, options).then(function (response) {",
    "the cache guard must precede the network call");
  ordered(memo, "return fetch(url, options).then(function (response) {",
    "fetchCache[id] = pending;", "the fetch must be stored into the cache");

  // Non-vacuity: exactly one raw fetch() site, exactly three memoFetch call
  // sites, and each chip passes the documented key/URL pair.
  h.assert(countIn(src, "return fetch(url, options)") === 1,
    "there must be exactly one raw fetch() call, inside memoFetch");
  h.assert(countIn(src, 'memoFetch("') === 3,
    "there must be exactly three memoFetch chip call sites, got " + countIn(src, 'memoFetch("'));
  contains(src, 'memoFetch("identity", "/api/auth/me", { headers: headers(), cache: "no-store" })',
    "identity chip uses the memoized /api/auth/me read");
  contains(src, 'memoFetch("versions", "/api/system/versions", { headers: headers(), cache: "no-store" })',
    "versions chip uses the memoized read");
  contains(src, 'memoFetch("plan", "/api/user/billing/plan", { headers: headers(), cache: "no-store" })',
    "plan chip uses the memoized read");
  absent(src, 'fetch("/api/auth/me"', "the old per-render /api/auth/me fetch must be gone");
});

// (b) One afterSwap listener, guarded, that never invalidates the cache. Cache
// invalidation belongs only to the product-config event (a real panel reload).
h.staticTest("#2936 exactly one guarded htmx:afterSwap listener reuses the cache; invalidation is confined to gb-product-config", function () {
  var src = h.readRepo(CARD);

  h.assert(countIn(src, '"htmx:afterSwap"') === 1,
    "there must be exactly ONE htmx:afterSwap listener, got " + countIn(src, '"htmx:afterSwap"'));
  contains(src, 'document.addEventListener("htmx:afterSwap", function () {',
    "the guarded afterSwap listener ships");
  var swap = span(src, 'document.addEventListener("htmx:afterSwap", function () {',
    'if (document.readyState === "loading") {');
  contains(swap, 'if (!byId("workspace-card")) { return; }',
    "afterSwap is guarded on the card actually being present");
  contains(swap, "render();", "the guard re-renders chips from cached data");
  absent(swap, "clearFetchCache()", "afterSwap must never clear the fetch cache");
  absent(swap, "fetch(", "afterSwap must never start a new network read");
  ordered(swap, 'if (!byId("workspace-card")) { return; }', "render();",
    "the guard must precede the render call");

  contains(src, 'document.addEventListener("gb-product-config", function () {',
    "product-config listener ships");
  var cfg = span(src, 'document.addEventListener("gb-product-config", function () {',
    'document.addEventListener("htmx:load", render);');
  contains(cfg, "clearFetchCache();", "a real config change invalidates the panel cache");
  contains(cfg, "render();", "a real config change re-renders");
  ordered(cfg, "clearFetchCache();", "render();",
    "invalidation must precede the re-render");
  h.assert(countIn(src, "clearFetchCache()") === 2,
    "cache clearing must exist only as a definition plus the config call, got " +
      countIn(src, "clearFetchCache()"));
});

// (c) Retry-After (numeric seconds and HTTP-date), bounded exponential
// backoff, a hard cap of two retries and a terminal stop path.
h.staticTest("#2936 Retry-After seconds/date parsing with bounded backoff and a hard two-retry cap", function () {
  var src = h.readRepo(CARD);

  contains(src, "var MAX_RETRIES = 2;", "retry cap is two");
  contains(src, "var BACKOFF_CAP_MS = 8000;", "backoff ceiling is bounded");
  contains(src, "var RETRY_AFTER_CAP_S = 30;", "Retry-After ceiling is bounded");

  var delay = span(src, "function retryDelayMs(response, attempt) {", "function isRetryable(response) {");
  contains(delay, 'response.headers.get("Retry-After")', "Retry-After header is read");
  contains(delay, "var seconds = Number(String(header).trim());", "numeric seconds are parsed");
  contains(delay, "if (isFinite(seconds) && seconds >= 0) {", "numeric seconds are validated");
  contains(delay, "return Math.min(seconds, RETRY_AFTER_CAP_S) * 1000;",
    "numeric Retry-After is honored and capped");
  contains(delay, "var at = Date.parse(String(header));", "HTTP-date Retry-After is parsed");
  contains(delay, "if (isFinite(at)) {", "HTTP-date is validated");
  contains(delay, "return Math.max(0, Math.min(at - Date.now(), RETRY_AFTER_CAP_S * 1000));",
    "HTTP-date Retry-After is honored and capped");
  contains(delay, "return Math.min(1000 * Math.pow(2, attempt), BACKOFF_CAP_MS);",
    "fallback is bounded exponential backoff");
  ordered(delay, 'response.headers.get("Retry-After")',
    "return Math.min(1000 * Math.pow(2, attempt), BACKOFF_CAP_MS);",
    "Retry-After must be preferred over the computed backoff");

  var retryable = span(src, "function isRetryable(response) {", "function memoFetch(key, url, options) {");
  contains(retryable, "var status = response ? response.status : 0;", "status is read defensively");
  contains(retryable, "return status === 429 || (status >= 500 && status < 600);",
    "only 429 and 5xx are retried");

  var memo = span(src, "function memoFetch(key, url, options) {", "function clearFetchCache() {");
  contains(memo, "if (isRetryable(response) && attempt < MAX_RETRIES) {",
    "retry is gated by the hard cap");
  contains(memo, "var wait = retryDelayMs(response, attempt);", "the delay comes from the parser");
  contains(memo, "attempt += 1;", "each retry increments the attempt counter");
  contains(memo, "return new Promise(function (resolve) { setTimeout(resolve, wait); }).then(run);",
    "a retry waits for the computed delay before re-running");
  contains(memo, "return response;", "exhausted or non-retryable responses stop the loop");
  ordered(memo, "if (isRetryable(response) && attempt < MAX_RETRIES) {", "attempt += 1;",
    "the cap check must precede the counter increment");
  ordered(memo, "attempt += 1;",
    "return new Promise(function (resolve) { setTimeout(resolve, wait); }).then(run);",
    "the increment must precede the delayed retry");
  absent(memo, "while (", "no unbounded retry loop");
});

// (d) Non-vacuity: chip labels/endpoints are unchanged and no new user-facing
// string or i18n key was smuggled in with the caching fix.
h.staticTest("#2936 non-vacuity: original chip strings and endpoints survive, no new i18n keys, module under 450 lines", function () {
  var src = h.readRepo(CARD);
  var lines = src.split("\n").length;
  h.assert(lines < 450, "workspace-card must stay under 450 lines, got " + lines);

  var preserved = [
    '"Identity unavailable"',
    '"Versions: unavailable"',
    '"Plan: unavailable"',
    '"Plan: n/a on this edition"',
    '"Cloud workspace"',
    '"This device"',
    '"Version: "',
    '"Preview: "',
    '"/api/auth/me"',
    '"/api/system/versions"',
    '"/api/user/billing/plan"'
  ];
  preserved.forEach(function (needle) {
    contains(src, needle, "preserved chip string/endpoint must remain: " + needle);
  });

  // The shipped card has no catalogue keys: the fix must not add any.
  absent(src, "gbT(", "no new gbT key/user-facing literal was introduced");
  absent(src, "data-i18n", "no new data-i18n key was introduced");
  contains(src, 'setText("workspace-identity", name + " · " + where);',
    "the visible identity chip is still rendered from the fetched user");
  contains(src, "renderIdentity();", "identity chip still renders");
  contains(src, "renderVersions();", "versions chip still renders");
  contains(src, "renderPlan();", "plan chip still renders");
});

h.note("FOLLOW-UP #2936: static coverage only — memoized panel-load fetch, one guarded afterSwap listener, Retry-After/backoff with a two-retry cap and preserved chip strings. No browser, server or cargo was run; the driver owns issue closure and UAT.");
