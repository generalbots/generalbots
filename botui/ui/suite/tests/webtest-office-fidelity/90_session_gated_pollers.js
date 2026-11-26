/**
 * #2943 — deterministic static guard for the unauthenticated-app-sweep fix.
 *
 * A real 67-app open/close sweep on a tab with no session produced 1,117
 * console 401s because every app that loads the shared collaboration modules
 * armed its own periodic authenticated poller (unread badge, @mentions feed,
 * presence heartbeat, conflict watch) with no session at all. The fix puts ONE
 * session gate at the scheduler used by all of them, in the shared module that
 * loads before every consumer:
 *
 *   botui/ui/suite/js/collab-comments/01_helpers.js
 *     hasSession()  — window.getGBAccessToken() is the single authority
 *     schedule(fn, ms) — no token -> no timer, no request, one localized hint
 *
 * This guard pins both halves so neither can silently regress:
 *   (1) the gate short-circuits BEFORE the immediate poll and setInterval;
 *   (2) every collab/mentions/actions poller registers through that gate, with
 *       no raw timer left in the poller modules — including the two chat
 *       actions/pending pollers (60_computer_use.js, 61_terminal_use.js).
 *
 * Non-vacuity: GB_COLLAB_POLL_GATE_PROBE_ROOT may point at a /tmp tree holding
 * a mutated copy named 01_helpers.js (gate removed); that makes case (1) FAIL.
 * GB_CHAT_POLL_GATE_PROBE_ROOT may point at a /tmp tree holding a mutated
 * 60_computer_use.js (raw setInterval); that makes the chat-poller case FAIL.
 * Follow-up (same issue): non-collab apps still emitted guaranteed-401 fetches
 * from an unauthenticated tab. The fix puts ONE more gate in the central
 * security-bootstrap fetch interceptor that every suite fetch() passes through:
 * no getGBAccessToken() -> no network request for a same-origin private API,
 * while /api/product, /api/i18n/*, /api/auth and /api/client-errors stay public.
 * The sign-in toast is still the one owned by GBCollabPollScheduler.
 *
 * No server, browser, cargo or i18n change is involved; the driver owns live
 * UAT, issue closure and CI deployment.
 */
"use strict";

var h = require("./00_harness");
var fs = require("fs");
var path = require("path");

var HELPERS = "botui/ui/suite/js/collab-comments/01_helpers.js";
var PANEL = "botui/ui/suite/js/collab-comments/99_panel.js";
var DATA = "botui/ui/suite/js/collab-comments/02_data.js";
var CONFLICT = "botui/ui/suite/js/collab-conflict.js";
var CHAT_BROWSER = "botui/ui/suite/chat/modules/60_computer_use.js";
var CHAT_TERMINAL = "botui/ui/suite/chat/modules/61_terminal_use.js";
var INTERCEPTOR = "botui/ui/suite/js/security-bootstrap/02_interceptors.js";
var CHAT_AUTOTASKS = "botui/ui/suite/chat/modules/23_autotasks.js";
var TASKS_STATS_A = "botui/ui/suite/tasks/tasks-scoped-filters.js";
var TASKS_STATS_B = "botui/ui/suite/tasks/tasks-toasts_b.js";
var CHAT_HISTORY = "botui/ui/suite/chat/modules/21_tabs_events.js";
var CHAT_MODELS = "botui/ui/suite/chat/chat-model-picker.js";
var RUNNER = "botui/ui/suite/tests/webtest-office-fidelity.js";
var GUARD = "botui/ui/suite/tests/webtest-office-fidelity/90_session_gated_pollers.js";
var HINT_KEY = "cloud-settings-signin-first";
var LOCALES = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];
var PROBE_ROOT = process.env.GB_COLLAB_POLL_GATE_PROBE_ROOT || "";
var CHAT_PROBE_ROOT = process.env.GB_CHAT_POLL_GATE_PROBE_ROOT || "";
var GATE_PROBE_ROOT = process.env.GB_SESSION_GATE_PROBE_ROOT || "";
var CALL_SITE_PROBE_ROOT = process.env.GB_SESSION_CALLSITE_PROBE_ROOT || "";

function readSrc(rel) {
  if (PROBE_ROOT && path.basename(rel) === "01_helpers.js") {
    var candidate = path.join(PROBE_ROOT, "01_helpers.js");
    if (fs.existsSync(candidate)) return fs.readFileSync(candidate, "utf8");
  }
  return h.readRepo(rel);
}
function readChat(rel) {
  if (CHAT_PROBE_ROOT) {
    var candidate = path.join(CHAT_PROBE_ROOT, path.basename(rel));
    if (fs.existsSync(candidate)) return fs.readFileSync(candidate, "utf8");
  }
  return h.readRepo(rel);
}
function readGate(rel) {
  if (GATE_PROBE_ROOT) {
    var candidate = path.join(GATE_PROBE_ROOT, path.basename(rel));
    if (fs.existsSync(candidate)) return fs.readFileSync(candidate, "utf8");
  }
  return h.readRepo(rel);
}
function readCallSite(rel) {
  if (CALL_SITE_PROBE_ROOT) {
    var candidate = path.join(CALL_SITE_PROBE_ROOT, path.basename(rel));
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

h.staticTest("#2943 the shared scheduler gates every poll on a live session before arming a timer", function () {
  var src = readSrc(HELPERS);
  h.assert(src.length > 8000, "non-vacuity: 01_helpers.js is the shipped shared module, not a stub");
  contains(src, "function hasSession()", "the session predicate must live in the shared module");
  contains(src, 'typeof window.getGBAccessToken === "function" && window.getGBAccessToken()',
    "getGBAccessToken() is the single session authority");
  contains(src, "window.GBCollabPollScheduler = { hasSession: hasSession, schedule: schedule };",
    "the scheduler must be exported for consumers that load after the helpers");

  // Verbatim body of the scheduler. The gate is the mutation-probe target:
  // removing it must make this case FAIL.
  var schedule = span(src, "function schedule(fn, ms) {", "\n  }");
  h.assert(schedule.length > 120, "non-vacuity: scheduler body must be substantial, got " + schedule.length);
  var flat = schedule.replace(/\s/g, "");
  contains(flat, "if(!hasSession()){showPollHintOnce();returnnull;}",
    "the session gate must short-circuit before the immediate poll and setInterval");

  var atGate = schedule.indexOf("if (!hasSession())");
  var atHint = schedule.indexOf("showPollHintOnce();");
  var atReturn = schedule.indexOf("return null;");
  var atPoll = schedule.indexOf("fn();");
  var atTimer = schedule.indexOf("return setInterval(fn, ms);");
  h.assert(atGate !== -1 && atHint > atGate && atReturn > atHint && atPoll > atReturn && atTimer > atPoll,
    "order must be session gate -> hint -> return -> poll -> timer " +
    "(gate=" + atGate + " hint=" + atHint + " return=" + atReturn +
    " poll=" + atPoll + " timer=" + atTimer + ")");
  h.assertEqual(countIn(flat, "setInterval(fn,ms)"), 1,
    "exactly one timer may be armed, and only after the gate");
});

h.staticTest("#2943 one localized sign-in hint covers the whole tab", function () {
  var src = readSrc(HELPERS);
  h.assertEqual(countIn(src, "if (pollHintShown) return;"), 1,
    "the hint must be emitted at most once for the tab");
  h.assertEqual(countIn(src, "pollHintShown = true;"), 1,
    "the latch must be set in exactly one place");
  h.assertEqual(countIn(src, "showPollHintOnce();"), 1,
    "the single hint call lives only in the scheduler gate");
  contains(src, 'var message = T("' + HINT_KEY + '", "Please sign in first.");',
    "the hint must reuse an existing localized key, never a new string");
  contains(src, 'window.GBToasts.show(message, "", "info")',
    "the hint must be visibly shown as a single toast");
  absent(src, "T(\"sign-in-required\"", "no invented i18n key may be introduced");

  var english = h.readRepo("botlib/locales/en/cloud-pages.ftl");
  contains(english, HINT_KEY + " = ", "the pinned hint key must exist in the English catalogue");
  LOCALES.forEach(function (locale) {
    var rel = "botlib/locales/" + locale + "/cloud-pages.ftl";
    h.assert(h.fileExistsRepo(rel), locale + " cloud-pages catalogue must ship");
    contains(h.readRepo(rel), HINT_KEY + " = ", locale + " must author the reused hint key");
  });
});

h.staticTest("#2943 the mentions feed and unread badge register only through the scheduler", function () {
  var src = h.readRepo(PANEL);
  contains(src, "var schedule = N.schedule;", "the panel must consume the shared scheduler");
  contains(src, "schedule(pollMentions, 60000);",
    "the @mentions poller must be scheduled through the gate");
  contains(src, "container.__gbcBadgeTimer = schedule(refresh, 30000);",
    "the unread-badge poller must be scheduled through the gate");
  h.assertEqual(countIn(src, "setInterval("), 0,
    "no raw timer may remain in the panel module");
  contains(src, 'req("/mentions/me?limit=5")', "the mentions endpoint must stay unchanged");
  contains(src, '"/comments/unread?resource_type="', "the unread endpoint must stay unchanged");
});

h.staticTest("#2943 the presence heartbeat and conflict watch register only through the scheduler", function () {
  var data = h.readRepo(DATA);
  contains(data, "var schedule = N.schedule;", "the data module must consume the shared scheduler");
  contains(data, 'schedule(function () { sendPresence(S.typingSent); }, HEARTBEAT_MS);',
    "the presence heartbeat must be scheduled through the gate");
  h.assertEqual(countIn(data, "setInterval("), 0,
    "no raw heartbeat timer may remain in the data module");
  contains(data, 'req("/presence", {', "the presence endpoint must stay unchanged");

  var conflict = h.readRepo(CONFLICT);
  contains(conflict, "window.GBCollabPollScheduler.schedule(refresh, state.pollMs);",
    "the conflict watch must be scheduled through the gate");
  h.assertEqual(countIn(conflict, "setInterval("), 0,
    "no raw conflict timer may remain");
  contains(conflict, 'api("/api/collab/conflicts?resource_type="',
    "the collab conflicts endpoint must stay unchanged");
});

h.staticTest("#2943 no unguarded periodic authenticated registration remains in the collab pollers", function () {
  var dir = path.join(h.REPO_DIR, "botui", "ui", "suite", "js", "collab-comments");
  var files = fs.readdirSync(dir).filter(function (name) {
    // 01_helpers.js owns the single gated timer; every consumer is gated.
    return /\.js$/.test(name) && name !== "01_helpers.js";
  });
  h.assert(files.length >= 2, "non-vacuity: expected the shipped collab-comments consumers");
  files.forEach(function (name) {
    var src = fs.readFileSync(path.join(dir, name), "utf8");
    h.assertEqual(countIn(src, "setInterval("), 0,
      name + " must not arm an unguarded periodic authenticated request");
  });
  var all = [PANEL, DATA].map(function (rel) {
    return h.readRepo(rel);
  }).join("\n");
  h.assert(countIn(all, "schedule(") >= 3,
    "non-vacuity: all three pollers must register via the gate");
});

h.staticTest("#2943 the chat browser/terminal action pollers arm only through the shared gate", function () {
  var browser = readChat(CHAT_BROWSER);
  var terminal = readChat(CHAT_TERMINAL);
  contains(browser, "window.GBCollabPollScheduler.schedule(pollPendingBrowser, POLL_MS);",
    "the browser pending poller must arm via the shared session gate");
  contains(terminal, "window.GBCollabPollScheduler.schedule(pollPendingTerminal, POLL_MS);",
    "the terminal pending poller must arm via the shared session gate");
  h.assertEqual(countIn(browser, "setInterval("), 0,
    "no raw timer may arm the browser actions/pending poll");
  h.assertEqual(countIn(terminal, "setInterval("), 0,
    "no raw timer may arm the terminal actions/pending poll");
  // The gate may be loaded lazily on a chat-only tab, but only the shared
  // helper may supply it — never a locally re-implemented session check.
  contains(browser, '"/suite/js/collab-comments/01_helpers.js";',
    "the browser poller must bootstrap the shared gate, not a local one");
  contains(terminal, '"/suite/js/collab-comments/01_helpers.js";',
    "the terminal poller must bootstrap the shared gate, not a local one");
  contains(browser, 'fetch("/api/computer/actions/pending?kind=browser&session_id=" + encodeURIComponent(sid), {',
    "the browser pending endpoint must stay unchanged");
  contains(terminal, 'fetch("/api/computer/actions/pending?kind=terminal&session_id=" + encodeURIComponent(sid), {',
    "the terminal pending endpoint must stay unchanged");
});

h.staticTest("#2943 follow-up one central fetch gate drops guaranteed-401 requests with no session", function () {
  var src = readGate(INTERCEPTOR);
  h.assert(src.length > 8000,
    "non-vacuity: 02_interceptors.js is the shipped central interceptor, not a stub");
  h.assertEqual(countIn(src, "window.fetch = function (input, init)"), 1,
    "there must be exactly one central fetch interceptor");
  contains(src, "function hasSession()", "the gate must use one shared session predicate");
  contains(src, 'typeof window.getGBAccessToken === "function" && window.getGBAccessToken()',
    "getGBAccessToken() is the single session authority for this gate too");
  contains(src, "function noSessionResponse(url)",
    "the gate must return a Response-shaped sentinel that callers can drain");

  var fetchFn = span(src, "window.fetch = function (input, init) {", "\n      };");
  h.assert(fetchFn.length > 1500, "non-vacuity: interceptor body must be substantial, got " + fetchFn.length);
  var gate = span(fetchFn, "        // #2943: drop guaranteed-401 private fetches", "\n        }");
  var flat = gate.replace(/\s/g, "");
  contains(flat, "isApiPath(url)&&self.isSameOrigin(url)&&!self.isPublicEndpoint(url)&&!isPublicNoSessionPath(url)&&!hasSession()",
    "the gate must require an API path, same origin, a private endpoint and no session");
  contains(src.replace(/\s/g, ""), 'pathname.indexOf("/api/")===0',
    "static /suite assets must never be dropped by the API gate");
  contains(flat, "returnPromise.resolve(noSessionResponse(url));",
    "no network request may be issued without a session");

  var atGate = fetchFn.indexOf("!hasSession()");
  var atReturn = fetchFn.indexOf("return Promise.resolve(noSessionResponse(url));");
  var atFetch = fetchFn.indexOf("return originalFetch");
  h.assert(atGate !== -1 && atReturn > atGate && atFetch > atReturn,
    "order must be session gate -> sentinel return -> network fetch " +
    "(gate=" + atGate + " return=" + atReturn + " fetch=" + atFetch + ")");
  absent(gate, "GBToasts", "the central gate must not emit a second toast");
  absent(gate, "showPollHintOnce", "the single hint stays owned by GBCollabPollScheduler");

  // HTMX issues XHR, which the fetch interceptor never sees: private API
  // requests must be cancelled at htmx:configRequest when no session exists.
  var htmxFn = span(src, "document.addEventListener(\"htmx:configRequest\", function (event) {",
    "\n      });");
  contains(htmxFn.replace(/\s/g, ""),
    "if(!hasSession()&&isApiPath(event.detail.path)&&!self.isPublicEndpoint(event.detail.path)&&!isPublicNoSessionPath(event.detail.path)){event.preventDefault();return;}",
    "htmx must cancel guaranteed-401 private API requests without a session");
});

h.staticTest("#2943 follow-up public endpoints stay reachable without a session", function () {
  var src = readGate(INTERCEPTOR);
  var list = span(src, "var NO_SESSION_PUBLIC = [", "];");
  [
    "/api/product",
    "/api/i18n/",
    "/api/auth",
    "/api/client-errors",
  ].forEach(function (p) {
    contains(list, '"' + p + '"', "public allowlist must keep " + p + " unauthenticated");
  });
  contains(src, 'var suffix = prefix.charAt(prefix.length - 1) === "/" ? "" : "/";',
    "a trailing-slash prefix (i18n) must not gain a second separator");
  contains(src, "path.indexOf(prefix + suffix) === 0",
    "prefix matching must respect a path boundary (so /api/autotask is not /api/auth)");
  contains(src, "!self.isPublicEndpoint(url) &&",
    "the pre-existing anonymous routes must still bypass the gate");
  contains(src, "init.headers = self.buildAuthHeaders(init.headers);",
    "authenticated header injection must be unchanged");
  var fetchFn = span(src, "window.fetch = function (input, init) {", "\n      };");
  var atExempt = fetchFn.indexOf("!isPublicNoSessionPath(url)");
  var atSession = fetchFn.indexOf("!hasSession()");
  h.assert(atExempt !== -1 && atSession > atExempt,
    "the public exemption must be evaluated before the session gate " +
    "(exempt=" + atExempt + " session=" + atSession + ")");
});

h.staticTest("#2943 follow-up the four named 401 sources still go through the central gate", function () {
  var cases = [
    { rel: CHAT_AUTOTASKS,
      needles: ['fetch("/api/conversations/" + encodeURIComponent(sessionId) + "/autotasks"'] },
    { rel: TASKS_STATS_A,
      needles: ['fetch("/api/ui/tasks/stats")', "response.text()"] },
    { rel: TASKS_STATS_B,
      needles: ['fetch("/api/ui/tasks/stats")', "response.text()"] },
    { rel: CHAT_HISTORY,
      needles: ["return fetch(", '"/api/chat/history/sessions?limit=8&bot_name="'] },
    { rel: CHAT_MODELS,
      needles: ['var url = "/api/chat/models"', 'fetch(url, { cache: "no-store", headers: authHeaders() })'] },
  ];
  cases.forEach(function (c) {
    var src = readCallSite(c.rel);
    h.assert(src.length > 500, "non-vacuity: " + c.rel + " must be a shipped module");
    c.needles.forEach(function (needle) {
      contains(src, needle, c.rel + " must keep calling the intercepted fetch for " + needle);
    });
    absent(src, "originalFetch", c.rel + " must not bypass the central interceptor");
    absent(src, "new XMLHttpRequest", c.rel + " must not bypass the central interceptor");
  });
});

h.staticTest("#2943 guard 90 is registered and its shipped sources stay focused", function () {
  [
    HELPERS, PANEL, DATA, CONFLICT, CHAT_BROWSER, CHAT_TERMINAL,
    INTERCEPTOR, CHAT_AUTOTASKS, TASKS_STATS_A, TASKS_STATS_B, CHAT_HISTORY, CHAT_MODELS,
  ].forEach(function (rel) {
    var src = h.readRepo(rel);
    h.assert(src.split("\n").length < 450, rel + " must stay under 450 lines");
    h.assert(src.length > 500, rel + " must remain a shipped module");
  });
  var guardSrc = h.readRepo(GUARD);
  h.assert(guardSrc.split("\n").length < 450, "guard must stay under 450 lines");
  contains(h.readRepo(RUNNER), 'require("./webtest-office-fidelity/90_session_gated_pollers");',
    "guard 90 must be registered in webtest-office-fidelity.js to run in CI");
  [
    "GB_COLLAB_POLL_GATE_PROBE_ROOT",
    "GB_CHAT_POLL_GATE_PROBE_ROOT",
    "GB_SESSION_GATE_PROBE_ROOT",
    "GB_SESSION_CALLSITE_PROBE_ROOT",
  ].forEach(function (name) {
    var hooks = guardSrc.match(new RegExp("=\\s*process\\.env\\." + name + "\\b", "g")) || [];
    h.assertEqual(hooks.length, 1,
      "the guard exposes exactly one " + name + " hook for mutation testing");
  });
});

h.note("#2943: offline static proof only — no browser/server/cargo, no locale edit. The driver owns live UAT and the /tmp mutation probes: removing the gate from 01_helpers.js fails the collab case, GB_CHAT_POLL_GATE_PROBE_ROOT pointing at a mutated 60_computer_use.js (raw setInterval) fails the chat-poller case, GB_SESSION_GATE_PROBE_ROOT pointing at a mutated 02_interceptors.js (gate or public exemption removed) fails the central-gate/public cases, and GB_SESSION_CALLSITE_PROBE_ROOT pointing at a mutated chat-model-picker.js (fetch bypassed) fails the four-call-site case.");
