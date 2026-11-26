/**
 * #2945 — deterministic static guard for the authenticated app-sweep fix.
 *
 * A signed-in admin sweep of the Suite desktop saw guaranteed-403 requests
 * from two shell pollers on every app open:
 *
 *   GET /api/automations/schedules     -> 403 (default-deny, no catalogue row)
 *   GET /api/conversation-autotasks    -> 403 (only POST was catalogued)
 *
 * The flood itself came from automations-render.js arming raw setIntervals:
 * closing the Automations window removed its DOM but left the timers running,
 * so the two fetches kept firing from inside every subsequently opened app.
 * The fix has two halves:
 *
 *   (1) frontend — the automations timers are registered with GBAppLifecycle
 *       (which clears them on `gb-window-close`) and each poll is additionally
 *       gated on the owning app root still being connected;
 *   (2) backend — the authenticated RBAC catalogue gains the missing rows, so
 *       an authenticated admin gets a real response instead of a 403.
 *
 * A third request, GET /api/computer/actions/pending, answered 401 because the
 * chat pollers pre-attached a chat-only token that the central interceptor
 * deliberately will not overwrite. They now use the canonical admin accessor.
 *
 * No server, browser, cargo or i18n change is involved; the driver owns live
 * UAT, issue closure and CI deployment.
 */
"use strict";

var h = require("./00_harness");
var fs = require("fs");
var path = require("path");

var AUTO_RENDER = "botui/ui/suite/automations/automations-render.js";
var CHAT_BROWSER = "botui/ui/suite/chat/modules/60_computer_use.js";
var CHAT_TERMINAL = "botui/ui/suite/chat/modules/61_terminal_use.js";
var RBAC_ROUTES = "botserver/crates/botsecurity-auth/src/rbac_middleware/routes_user/app_routes.rs";
var RUNNER = "botui/ui/suite/tests/webtest-office-fidelity.js";
var GUARD = "botui/ui/suite/tests/webtest-office-fidelity/92_app_scoped_pollers.js";
var PROBE_ROOT = process.env.GB_APP_SCOPE_PROBE_ROOT || "";

function read(rel) {
  if (PROBE_ROOT && path.basename(rel) === "automations-render.js") {
    var candidate = path.join(PROBE_ROOT, "automations-render.js");
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
function countIn(src, needle) { return src.split(needle).length - 1; }
function span(src, startMarker, endMarker) {
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + ">");
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + ">");
  h.assert(end > start, "empty span between <" + startMarker + "> and <" + endMarker + ">");
  return src.slice(start, end);
}

h.staticTest("#2945 the automations timers are owned by GBAppLifecycle", function () {
  var src = read(AUTO_RENDER);
  h.assert(src.length > 8000, "non-vacuity: automations-render.js is the shipped module, not a stub");
  contains(src, 'if (window.GBAppLifecycle) window.GBAppLifecycle.begin("automations");',
    "the app must announce its lifecycle bucket on init");
  contains(src, 'AutoState.pollTimer = window.GBAppLifecycle.interval("automations", poll, 15000);',
    "the 15s schedules poll must be a lifecycle-owned timer");
  contains(src, 'window.GBAppLifecycle.interval("automations", tickElapsedTimers, 1000);',
    "the 1s elapsed ticker must also be lifecycle-owned");
  contains(src, 'if (window.GBAppLifecycle) window.GBAppLifecycle.end("automations");',
    "re-arming must tear the previous bucket down");
  absent(src, "setInterval(loadSchedulesAndRender, 15000)",
    "the leaked global timer is gone");
  contains(src, "} else {\n        AutoState.pollTimer = setInterval(poll, 15000);",
    "any raw fallback timer must still route through the owning-app gate");
  h.assert(countIn(src, "GBAppLifecycle.interval(\"automations\"") === 2,
    "exactly two timers may be registered for the automations app");
});

h.staticTest("#2945 no automations request can fire outside the owning app", function () {
  var src = read(AUTO_RENDER);
  contains(src, "function autoAppActive()", "the owning-app predicate must exist");
  var active = span(src, "function autoAppActive() {", "\n}");
  contains(active, 'document.getElementById("automations-app")',
    "the predicate must resolve the automations app root");
  contains(active, "root.isConnected",
    "a closed window's detached root must block the poll");
  contains(active, "!document.hidden",
    "a backgrounded tab must not poll either");
  contains(src, "const poll = function () { if (autoAppActive()) loadSchedulesAndRender(); };",
    "the poll wrapper must gate every fetch on the owning app");
  var load = span(src, "async function loadSchedulesAndRender() {", "\n}");
  contains(load, 'autoApi("/api/automations/schedules")',
    "the schedules endpoint stays in the gated loader");
  contains(load, "await loadConversationJobs();",
    "the autotasks loader stays behind the same gate");
  var jobs = span(src, "async function loadConversationJobs() {", "\n}");
  contains(jobs, 'autoApi("/api/conversation-autotasks")',
    "the autotasks endpoint stays in the gated loader");
});

h.staticTest("#2945 the RBAC catalogue admits authenticated reads of both endpoints", function () {
  var src = read(RBAC_ROUTES);
  [
    'RoutePermission::new("/api/conversation-autotasks/**", "GET", "")',
    'RoutePermission::new("/api/automations/**", "GET", "")',
    'RoutePermission::new("/api/automations/**", "POST", "")',
    'RoutePermission::new("/api/automations/**", "PUT", "")',
    'RoutePermission::new("/api/automations/**", "DELETE", "")',
  ].forEach(function (needle) {
    h.assertEqual(countIn(src, needle), 1,
      "the authenticated catalogue must carry exactly one " + needle);
  });
  var lines = src.split("\n").filter(function (line) {
    return line.indexOf("/api/automations/**") !== -1 ||
      line.indexOf("/api/conversation-autotasks/**") !== -1;
  });
  h.assert(lines.length === 6, "non-vacuity: all six catalogue entries must be present");
  lines.forEach(function (line) {
    h.assert(line.indexOf("with_anonymous(true)") === -1,
      "these per-user routes must never be anonymous: " + line.trim());
  });
});

h.staticTest("#2945 the actions/pending pollers use the canonical admin token", function () {
  [
    { rel: CHAT_BROWSER, fn: "function pollPendingBrowser()",
      endpoint: 'fetch("/api/computer/actions/pending?kind=browser&session_id=" + encodeURIComponent(sid), {' },
    { rel: CHAT_TERMINAL, fn: "function pollPendingTerminal()",
      endpoint: 'fetch("/api/computer/actions/pending?kind=terminal&session_id=" + encodeURIComponent(sid), {' },
  ].forEach(function (c) {
    var src = h.readRepo(c.rel);
    h.assert(src.length > 500, "non-vacuity: " + c.rel + " must be a shipped module");
    var body = span(src, c.fn, "\n  }");
    contains(body, 'window.getGBAccessToken()',
      c.rel + " must attach the canonical admin token");
    absent(body, "getChatAccessToken()",
      c.rel + " must not pre-attach the stale chat-only token");
    contains(body, c.endpoint, "the pending endpoint must stay unchanged");
    absent(body, "originalFetch", c.rel + " must not bypass the central interceptor");
  });
});

h.staticTest("#2945 guard 92 is registered and mutation-probeable", function () {
  [
    AUTO_RENDER, CHAT_BROWSER, CHAT_TERMINAL, RBAC_ROUTES,
  ].forEach(function (rel) {
    var src = h.readRepo(rel);
    h.assert(src.split("\n").length < 450, rel + " must stay under 450 lines");
    h.assert(src.length > 500, rel + " must remain a shipped source");
  });
  var guardSrc = h.readRepo(GUARD);
  h.assert(guardSrc.split("\n").length < 450, "guard must stay under 450 lines");
  contains(h.readRepo(RUNNER), 'require("./webtest-office-fidelity/92_app_scoped_pollers");',
    "guard 92 must be registered in webtest-office-fidelity.js to run in CI");
  var hooks = guardSrc.match(/=\s*process\.env\.GB_APP_SCOPE_PROBE_ROOT\b/g) || [];
  h.assertEqual(hooks.length, 1,
    "the guard exposes exactly one GB_APP_SCOPE_PROBE_ROOT hook for mutation testing");
});

h.note("#2945: offline static proof only — no browser/server/cargo, no locale edit. " +
  "The driver owns live UAT and the /tmp mutation probe: GB_APP_SCOPE_PROBE_ROOT " +
  "pointing at a mutated automations-render.js (raw setInterval restored) fails the " +
  "lifecycle case. The driver also owns the server-side validation that an " +
  "authenticated admin now gets a real response from all three endpoints.");
