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
 *       no raw timer left in the poller modules.
 *
 * Non-vacuity: GB_COLLAB_POLL_GATE_PROBE_ROOT may point at a /tmp tree holding
 * a mutated copy named 01_helpers.js (gate removed). The mutation makes case
 * (1) FAIL. No server, browser, cargo or i18n change is involved; the driver
 * owns live UAT, issue closure and CI deployment.
 */
"use strict";

var h = require("./00_harness");
var fs = require("fs");
var path = require("path");

var HELPERS = "botui/ui/suite/js/collab-comments/01_helpers.js";
var PANEL = "botui/ui/suite/js/collab-comments/99_panel.js";
var DATA = "botui/ui/suite/js/collab-comments/02_data.js";
var CONFLICT = "botui/ui/suite/js/collab-conflict.js";
var RUNNER = "botui/ui/suite/tests/webtest-office-fidelity.js";
var GUARD = "botui/ui/suite/tests/webtest-office-fidelity/90_session_gated_pollers.js";
var HINT_KEY = "cloud-settings-signin-first";
var LOCALES = ["en", "pt-BR", "es", "fr", "de", "ja", "ko", "zh-CN"];
var PROBE_ROOT = process.env.GB_COLLAB_POLL_GATE_PROBE_ROOT || "";

function readSrc(rel) {
  if (PROBE_ROOT && path.basename(rel) === "01_helpers.js") {
    var candidate = path.join(PROBE_ROOT, "01_helpers.js");
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

h.staticTest("#2943 guard 90 is registered and its shipped sources stay focused", function () {
  [HELPERS, PANEL, DATA, CONFLICT].forEach(function (rel) {
    var src = h.readRepo(rel);
    h.assert(src.split("\n").length < 450, rel + " must stay under 450 lines");
    h.assert(src.length > 500, rel + " must remain a shipped module");
  });
  var guardSrc = h.readRepo(GUARD);
  h.assert(guardSrc.split("\n").length < 450, "guard must stay under 450 lines");
  contains(h.readRepo(RUNNER), 'require("./webtest-office-fidelity/90_session_gated_pollers");',
    "guard 90 must be registered in webtest-office-fidelity.js to run in CI");
  var hooks = guardSrc.match(/=\s*process\.env\.GB_COLLAB_POLL_GATE_PROBE_ROOT/g) || [];
  h.assertEqual(hooks.length, 1,
    "the guard exposes exactly one probe-root hook for mutation testing");
});

h.note("#2943: offline static proof only — no browser/server/cargo, no locale edit. The driver owns live UAT and the /tmp mutation probe; removing the gate from 01_helpers.js makes case (1) FAIL.");
