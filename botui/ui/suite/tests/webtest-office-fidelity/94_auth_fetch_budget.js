/**
 * #2968 — regression guard for the /api/auth/me 429 storm in CRM and Mail.
 *
 * #2936 memoized the Settings "This workspace" card, but a second, global
 * caller survived: desktop.html loads sidebar_a_core.js (updateLoginButton =
 * refreshUser -> GET /api/auth/me) and desktop-inline/04_auth_fullscreen_a.js,
 * whose htmx:afterRequest listener re-ran updateLoginButton after EVERY swap.
 * A 30 s sweep of the two swap-heavy apps produced 187 auth/me calls and a
 * 429 storm (IMP-07 caps auth endpoints at 10 req/s).
 *
 * The fix has two halves:
 *   (1) the shell listener no longer re-evaluates auth on HTMX swaps — auth
 *       state changes are already handled by the explicit gb:auth:* events;
 *   (2) refreshUser() is single-flight per session token, so even a caller
 *       that still fires per swap issues at most one /api/auth/me fetch.
 *
 * Cases: two RUNTIME proofs (shipped sidebar IIFE in a stubbed vm) plus
 * STATIC source assertions. No browser, server, cargo or i18n change here.
 */
"use strict";

var h = require("./00_harness");
var vm = require("vm");

var CORE = "botui/ui/suite/js/sidebar_a_core.js";
var SHELL = "botui/ui/suite/desktop-inline/04_auth_fullscreen_a.js";
var CARD = "botui/ui/suite/settings/settings-workspace-card.js";
var RUNNER = "botui/ui/suite/tests/webtest-office-fidelity.js";
var GUARD = "botui/ui/suite/tests/webtest-office-fidelity/94_auth_fetch_budget.js";

function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1, (msg || "source must contain") + " <" + needle + ">");
}
function absent(src, needle, msg) {
  h.assert(src.indexOf(needle) === -1, (msg || "source must not contain") + " <" + needle + ">");
}
function countIn(src, needle) { return src.split(needle).length - 1; }
function span(src, startMarker, endMarker) {
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + ">");
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + ">");
  h.assert(end > start, "empty span between <" + startMarker + "> and <" + endMarker + ">");
  return src.slice(start, end);
}
function ordered(src, before, after, msg) {
  var b = src.indexOf(before);
  var a = src.indexOf(after);
  h.assert(b !== -1 && a !== -1 && b < a,
    (msg || "ordering regressed") + " :: <" + before + "> before <" + after + ">");
}

/** A minimal browser stub: only refreshUser()'s dependencies are needed. */
function makeEnv(token, calls) {
  var store = { "gb-access-token": token || "" };
  var sandbox = {
    window: {
      APPS_REGISTRY: [],
      location: { pathname: "/crm", href: "http://localhost:3000/crm" },
      getGBAccessToken: function () { return store["gb-access-token"]; },
      addEventListener: function () {},
      removeEventListener: function () {},
    },
    localStorage: {
      getItem: function (key) { return store[key] || null; },
      setItem: function (key, value) { store[key] = value; },
      removeItem: function (key) { delete store[key]; },
    },
    sessionStorage: {
      getItem: function () { return null; },
      setItem: function () {},
      removeItem: function () {},
    },
    document: {
      readyState: "complete",
      getElementById: function () { return null; },
      querySelector: function () { return null; },
      querySelectorAll: function () { return []; },
      addEventListener: function () {},
      removeEventListener: function () {},
      createElement: function () { return { style: {}, setAttribute: function () {} }; },
      body: { appendChild: function () {}, addEventListener: function () {} },
    },
    fetch: function (url, options) {
      calls.push({ url: url, options: options });
      return Promise.resolve({
        ok: true,
        status: 200,
        json: function () {
          return Promise.resolve({ display_name: "Ada", email: "ada@example.test" });
        },
      });
    },
    console: { warn: function () {}, error: function () {}, log: function () {}, debug: function () {} },
    setTimeout: function () {},
    clearTimeout: function () {},
    encodeURIComponent: encodeURIComponent,
    decodeURIComponent: decodeURIComponent,
  };
  return { sandbox: sandbox, store: store };
}

// (1) The exact storm shape: 187 simulated swaps must yield one auth/me read.
h.test("#2968 exactly one /api/auth/me fetch across 187 simulated HTMX swaps", function () {
  var calls = [];
  var env = makeEnv("session-token-abc", calls);
  var ctx = h.loadModules(["js/sidebar_a_core.js"], env.sandbox);
  var refresh = ctx.window.__GBSidebarFns.refreshUser;
  h.assert(typeof refresh === "function", "sidebar must export the refreshUser hook");
  h.assertEqual(ctx.window.updateLoginButton, refresh,
    "updateLoginButton must be the same single-flight refreshUser (no second fetch path)");
  for (var i = 0; i < 187; i++) { refresh(); }
  h.assertEqual(calls.length, 1,
    "187 swaps must issue exactly one /api/auth/me fetch, got " + calls.length);
  h.assertEqual(calls[0].url, "/api/auth/me", "the one fetch must target /api/auth/me");
  h.assertEqual(calls[0].options.headers.Authorization, "Bearer session-token-abc",
    "the fetch must carry the session bearer token");
});

// (2) Single-flight means concurrent in-flight callers share the promise too.
h.test("#2968 concurrent swaps share one in-flight promise, and a new session token re-fetches once", function () {
  var calls = [];
  var env = makeEnv("token-A", calls);
  env.sandbox.fetch = function (url, options) {
    calls.push({ url: url, options: options });
    return new Promise(function () {}); // deliberately never settles
  };
  var ctx = h.loadModules(["js/sidebar_a_core.js"], env.sandbox);
  var refresh = ctx.window.__GBSidebarFns.refreshUser;
  for (var i = 0; i < 50; i++) { refresh(); }
  h.assertEqual(calls.length, 1,
    "50 in-flight swaps must share one request, got " + calls.length);

  env.store["gb-access-token"] = "token-B";
  for (var j = 0; j < 10; j++) { refresh(); }
  h.assertEqual(calls.length, 2,
    "a genuinely different session token may re-fetch exactly once, got " + calls.length);

  env.store["gb-access-token"] = "";
  for (var k = 0; k < 10; k++) { refresh(); }
  h.assertEqual(calls.length, 2, "signed-out renders must never fetch auth/me");
});

// (3) The global shell listener must not re-evaluate auth on swaps.
h.staticTest("#2968 the desktop htmx:afterRequest listener no longer calls updateLoginButton", function () {
  var src = h.readRepo(SHELL);
  h.assertEqual(countIn(src, 'document.body.addEventListener("htmx:afterRequest"'), 1,
    "exactly one afterRequest listener must ship");
  var listener = span(src, 'document.body.addEventListener("htmx:afterRequest"',
    "// Listen for auth state changes");
  absent(listener, "updateLoginButton", "per-swap auth re-evaluation must be gone");
  absent(listener, 'fetch("', "the swap listener must never fetch directly");
  contains(listener, "ThemeManager.init()", "the theme re-injection must be preserved");

  var auth = span(src, "function initAuthUI() {",
    "if (window.GBSecurity && window.GBSecurity.initialized) {");
  contains(auth, 'gb:auth:login', "login still re-evaluates auth");
  contains(auth, 'gb:auth:logout', "logout still re-evaluates auth");
  contains(auth, 'gb:auth:expired', "expiry still re-evaluates auth");
  h.assertEqual(countIn(src, "updateLoginButton()"), 4,
    "the only auth/me re-evaluations are init plus the three auth events");
});

// (4) The sidebar fetch itself is guarded and bounded per session.
h.staticTest("#2968 refreshUser is single-flight per token with an anonymous short circuit", function () {
  var src = h.readRepo(CORE);
  var lines = src.split("\n").length;
  h.assert(lines < 450, "sidebar_a_core.js must stay under 450 lines, got " + lines);
  contains(src, "var identityCache = Object.create(null);", "session cache container ships");
  contains(src, "function clearIdentityCache() {", "cache invalidation ships");
  var fn = span(src, "function refreshUser() {", "window.updateLoginButton = refreshUser;");
  contains(fn, "if (!token) { clearIdentityCache(); renderUserSignedOut(); return; }",
    "anonymous path clears the cache and short circuits before any fetch");
  contains(fn, "var pending = identityCache[token];", "cache is read first");
  contains(fn, "if (!pending) {", "single-flight guard ships");
  contains(fn, 'fetch("/api/auth/me", { headers: authHeaders() })',
    "the one real fetch lives inside the guard");
  contains(fn, "identityCache[token] = pending;", "the in-flight promise is memoized");
  contains(fn, "return pending;", "callers share the same promise");
  ordered(fn, "if (!pending) {", 'fetch("/api/auth/me"', "guard must precede the network call");
  ordered(fn, 'fetch("/api/auth/me"', "identityCache[token] = pending;",
    "the fetch must be stored into the cache");
  h.assertEqual(countIn(src, 'fetch("/api/auth/me"'), 1,
    "there must be exactly one raw auth/me fetch site in the sidebar");
});

// (5) Route contract preserved: all three GET endpoints and their memoized
// Settings consumers still ship; no new i18n surface or user-facing literal.
h.staticTest("#2968 route contract unchanged; no new i18n key or user-facing literal", function () {
  var core = h.readRepo(CORE);
  var shell = h.readRepo(SHELL);
  var card = h.readRepo(CARD);
  contains(core, 'fetch("/api/auth/me"', "sidebar keeps the GET /api/auth/me contract");
  absent(shell, "/api/auth/me", "the shell must not re-issue the identity endpoint");
  contains(card, 'memoFetch("identity", "/api/auth/me"', "identity route unchanged");
  contains(card, 'memoFetch("versions", "/api/system/versions"', "versions route unchanged");
  contains(card, 'memoFetch("plan", "/api/user/billing/plan"', "billing plan route unchanged");
  var added = span(core, "// #2968 — identity is session-scoped", "var identityCache = Object.create(null);");
  contains(added, "Single-flight", "the session-scoped single-flight rationale is documented at the cache");
  // No locale catalog was touched by this fix; the changed files carry no
  // new data-i18n attribute and the run adds no gbT key.
  absent(span(core, "var identityCache = Object.create(null);", "function refreshUser() {"),
    "data-i18n", "the cache helper introduces no i18n attribute");
  contains(h.readRepo(RUNNER), 'require("./webtest-office-fidelity/94_auth_fetch_budget");',
    "guard 94 must be registered in webtest-office-fidelity.js to run in CI");
  var guardSrc = h.readRepo(GUARD);
  h.assert(guardSrc.split("\n").length < 450, "guard must stay under 450 lines");
});

h.note("#2968: RUNTIME math on the shipped sidebar IIFE (187 swaps -> 1 fetch; 50 in-flight -> 1 fetch) " +
  "plus STATIC proof that the shell swap listener no longer re-evaluates auth. No browser, " +
  "server, cargo or locale change; the driver owns live UAT and issue closure.");
