/**
 * #2926 — deterministic static coverage for the bounded collaboration
 * WebSocket reconnect policy in botui/ui/suite/js/collab.js. The old client
 * retried every failed upgrade every 2 s for the whole tab lifetime, with no
 * cap, no backoff and no visible offline state. The fix adds a finite attempt
 * cap, bounded exponential delay, conservative terminal-close classification,
 * exactly one localized aria-live status region and a manual retry control.
 *
 * All cases are literal source assertions: no server, browser, cargo or
 * network is involved. Each case first proves the shipped artifact and the
 * extracted spans are non-empty, then pins exact strings, strict ordering and
 * negative guards — reverting any part of the fix flips a case to FAIL.
 * The driver owns issue closure and browser UAT.
 */
"use strict";

var h = require("./00_harness");

var COLLAB = "botui/ui/suite/js/collab.js";
var SCHEDULE_START = "GBCollab.prototype._scheduleReconnect = function (ev) {";
var SCHEDULE_END = "GBCollab.prototype._startHeartbeat = function () {";

function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1, (msg || "source must contain") + " <" + needle + ">");
}
function absent(src, needle, msg) {
  h.assert(src.indexOf(needle) === -1, (msg || "source must not contain") + " <" + needle + ">");
}
function between(src, start, end) {
  var a = src.indexOf(start);
  h.assert(a !== -1, "missing start marker <" + start + ">");
  var b = src.indexOf(end, a + start.length);
  h.assert(b !== -1, "missing end marker <" + end + ">");
  h.assert(b > a, "empty source span between <" + start + "> and <" + end + ">");
  return src.slice(a, b);
}
function spanHas(span, needle, msg) {
  h.assert(span.indexOf(needle) !== -1,
    (msg || "span must contain") + " <" + needle + "> :: " + span.slice(0, 220));
}
function spanLacks(span, needle, msg) {
  h.assert(span.indexOf(needle) === -1,
    (msg || "span must not contain") + " <" + needle + "> :: " + span.slice(0, 220));
}
function countIn(src, needle) { return src.split(needle).length - 1; }
function ordered(span, markers, msg) {
  var last = -1;
  markers.forEach(function (m) {
    var at = span.indexOf(m);
    h.assert(at !== -1, (msg || "span") + " must contain <" + m + ">");
    h.assert(at > last, (msg || "span") + " order broken at <" + m + ">");
    last = at;
  });
}

// (a) A finite attempt cap, a bounded exponential delay, and a stop branch.
// The old unconditional `setTimeout(..., RECONNECT_DELAY_MS)` must be gone.
h.staticTest("#2926 bounded exponential backoff with a finite attempt cap and a terminal stop branch", function () {
  var src = h.readRepo(COLLAB);
  h.assert(src.length > 12000, "collab.js must be a non-trivial shipped module");
  contains(src, "const RECONNECT_DELAY_MS = 2000;", "initial delay is pinned");
  contains(src, "const BACKOFF_MAX_MS = 30000;", "maximum delay is bounded");
  contains(src, "const MAX_RECONNECT_ATTEMPTS = 5;", "attempt count is finite");

  var schedule = between(src, SCHEDULE_START, SCHEDULE_END);
  spanHas(schedule, "if (this.reconnectTimer || this.terminal) return;", "one timer/terminal guard");
  spanHas(schedule, "this.reconnectAttempts >= MAX_RECONNECT_ATTEMPTS", "exhaustion is checked against the cap");
  spanHas(schedule, "this.terminal = true;", "exhaustion marks the client terminal");
  spanHas(schedule, "this._showUnavailable();", "exhaustion surfaces one status message");
  spanHas(schedule, "const wait = Math.min(RECONNECT_DELAY_MS * Math.pow(2, this.reconnectAttempts), BACKOFF_MAX_MS);",
    "delay grows exponentially and is capped");
  spanHas(schedule, "this.reconnectAttempts += 1;", "each scheduled retry increments the counter");
  spanHas(schedule, "this.reconnectTimer = setTimeout(function () {", "the actual retry lives in a delayed timer");
  h.assertEqual(countIn(schedule, "this._open();"), 1, "the only reconnect open is inside that timer");
  spanLacks(schedule, "while (", "no unbounded retry loop");
  spanLacks(schedule, "setInterval", "no retry interval");
  ordered(schedule, [
    "if (this.reconnectTimer || this.terminal) return;",
    "this.reconnectAttempts >= MAX_RECONNECT_ATTEMPTS",
    "Math.pow(2, this.reconnectAttempts)",
    "this.reconnectAttempts += 1;",
    "setTimeout(function () {",
    "this._open();"
  ], "guard, cap, backoff, increment, delayed open");

  absent(src, "}.bind(this), RECONNECT_DELAY_MS);", "the old fixed 2 s infinite retry is gone");
  h.assertEqual(countIn(src, "MAX_RECONNECT_ATTEMPTS"), 2,
    "cap constant is declared once and enforced once");
});

// (b) A close carrying 1008, or 1006 with a rejection reason, is terminal:
// browsers hide the HTTP status of a denied upgrade, so 401/403/404/forbidden
// named in the reason must be treated conservatively and never retried.
h.staticTest("#2926 1008 and 1006 policy denials are terminal; only transient closes reach the backoff", function () {
  var src = h.readRepo(COLLAB);
  var close = between(src, "GBCollab.prototype._isTerminalClose = function (ev) {",
    "GBCollab.prototype._showUnavailable = function () {");
  spanHas(close, "if (!ev) return false;", "no close info is not assumed to be a denial");
  spanHas(close, "const code = ev.code;", "close code is inspected");
  spanHas(close, 'const reason = ev.reason || "";', "close reason is inspected");
  spanHas(close, "code === 1008", "1008 policy violation is terminal");
  spanHas(close, "code === 1006 && TERMINAL_REASON.test(reason)",
    "1006 with a denial reason is terminal");
  ordered(close, ["const code = ev.code;", 'const reason = ev.reason || "";',
    "code === 1008", "code === 1006 && TERMINAL_REASON.test(reason)"],
    "close classification order: code, reason, 1008, 1006");

  var regex = between(src, "const TERMINAL_REASON =", "const HEARTBEAT_MS");
  ["policy", "forbidden", "unauthori", "not found", "denied", "reject", "401|403|404"].forEach(function (token) {
    spanHas(regex, token, "terminal reason pattern must include " + token);
  });
  contains(src, "TERMINAL:", "the conservative 1008/1006 rule is documented in the module header");
  contains(src, "never retried", "the policy-denial rule is documented as terminal");

  var schedule = between(src, SCHEDULE_START, SCHEDULE_END);
  spanHas(schedule, "this._isTerminalClose(ev) || this.reconnectAttempts >= MAX_RECONNECT_ATTEMPTS",
    "terminal close and exhaustion share one stop branch");
  spanHas(schedule, "return;", "the stop branch returns before scheduling");
  h.assertEqual(countIn(schedule, "setTimeout("), 1,
    "only the transient branch schedules another attempt");
  h.assertEqual(countIn(schedule, "this._isTerminalClose(ev)"), 1,
    "classification is applied exactly once");
});

// (c) Exactly ONE status region, with an accessible live region, localized
// text via gbT (English fallbacks), and a manual retry control.
h.staticTest("#2926 one localized aria-live status region with a manual retry control", function () {
  var src = h.readRepo(COLLAB);
  var status = between(src, "GBCollab.prototype._showUnavailable = function () {",
    "GBCollab.prototype._clearUnavailable = function () {");
  spanHas(status, "if (!document.body) return;", "safe before the body exists");
  spanHas(status, "let node = document.getElementById(STATUS_ID);", "the existing region is reused");
  spanHas(status, "if (!node) {", "creation is guarded by absence");
  spanHas(status, 'id: STATUS_ID, class: "gb-collab-status"', "the single region has a stable id");
  spanHas(status, 'role: "status", "aria-live": "polite"', "the region is announced politely");
  spanHas(status, "document.body.appendChild(node);", "the region is appended only inside the guard");
  spanHas(status, 'node.innerHTML = "";', "the status is rebuilt, never stacked");
  spanHas(status, 'T("collab-unavailable", "Collaboration unavailable")',
    "status text uses the localized key with an English fallback");
  spanHas(status, 'T("collab-retry", "Retry")',
    "retry label uses the localized key with an English fallback");
  spanHas(status, "this.retry();", "the retry control is wired to the manual hook");
  ordered(status, [
    "let node = document.getElementById(STATUS_ID);",
    "if (!node) {",
    'role: "status", "aria-live": "polite"',
    'node.innerHTML = "";',
    'T("collab-unavailable"',
    'T("collab-retry"'
  ], "look up, create once, announce, rebuild, localize");

  var clear = between(src, "GBCollab.prototype._clearUnavailable = function () {",
    "GBCollab.prototype.retry = function () {");
  spanHas(clear, "document.getElementById(STATUS_ID)", "clear targets the same single region");
  h.assertEqual(countIn(src, "document.getElementById(STATUS_ID)"), 2,
    "exactly two lookups: show and clear");
  h.assertEqual(countIn(src, "document.body.appendChild(node)"), 1,
    "the status is appended exactly once");
  contains(src, "function T(key, fallback) { return window.gbT ? window.gbT(key, fallback) : fallback; }",
    "user-facing text goes through window.gbT");
  absent(src, "auto-", "no machine-generated i18n key was invented");
  absent(src, 'setAttribute("data-i18n"', "no catalog binding was invented in JS");
});

// (d) Manual retry and lifecycle transitions must reset the bounded state and
// never let an intentional disconnect trigger an automatic reconnect.
h.staticTest("#2926 manual retry resets state; successful open and disconnect/retarget clear the offline state", function () {
  var src = h.readRepo(COLLAB);
  var retry = between(src, "GBCollab.prototype.retry = function () {", SCHEDULE_START);
  spanHas(retry, "clearTimeout(this.reconnectTimer)", "a pending retry is cancelled on manual retry");
  spanHas(retry, "this.reconnectAttempts = 0;", "the attempt counter is reset");
  spanHas(retry, "this.terminal = false;", "the terminal flag is reset");
  spanHas(retry, "this._clearUnavailable();", "the status is closed");
  spanHas(retry, "this._open();", "manual retry reopens the socket");
  ordered(retry, ["clearTimeout(this.reconnectTimer)", "this.reconnectAttempts = 0;",
    "this.terminal = false;", "this._clearUnavailable();", "this._open();"], "manual retry order");

  var close = between(src, 'ws.addEventListener("close", function (ev) {',
    'ws.addEventListener("error", function () {');
  spanHas(close, "if (!this._manualClose) this._scheduleReconnect(ev);",
    "the close event passes its code/reason and respects manual teardown");
  var opened = between(src, 'ws.addEventListener("open", function () {',
    'ws.addEventListener("message", function (ev) {');
  spanHas(opened, "this.reconnectAttempts = 0;", "a successful open resets attempts");
  spanHas(opened, "this.terminal = false;", "a successful open clears terminal");
  spanHas(opened, "this._clearUnavailable();", "a successful open hides the status");
  var conn = between(src, "GBCollab.prototype.connect = function (opts) {",
    "GBCollab.prototype._open = function () {");
  spanHas(conn, "this.reconnectAttempts = 0;", "a new connect resets attempts");
  spanHas(conn, "this.terminal = false;", "a new connect clears terminal");
  var disco = between(src, "GBCollab.prototype.disconnect = function () {",
    "GBCollab.prototype.getUser = function () {");
  spanHas(disco, "this._manualClose = true; this.terminal = false;",
    "intentional teardown suppresses auto-retry and clears terminal");
  var openStart = between(src, "GBCollab.prototype._open = function () {",
    'ws.addEventListener("open", function () {');
  spanHas(openStart, "this._manualClose = false;", "opening re-enables close-event handling");
});

// (e) Non-vacuity: the public API used by docs/slides/sheet is intact, the
// module stays under 450 lines, and the old fixed-delay loop is absent.
h.staticTest("#2926 non-vacuity: public API preserved, no fixed-interval loop, under 450 lines", function () {
  var src = h.readRepo(COLLAB);
  var lines = src.split("\n").length;
  h.assert(lines <= 449, "collab.js must stay under 450 lines, got " + lines);
  ["GBCollab.prototype.connect =", "GBCollab.prototype.disconnect =",
    "GBCollab.prototype.send =", "GBCollab.prototype.retry =",
    "GBCollab.prototype._scheduleReconnect =", "GBCollab.helpers =",
    "window.GBCollab = new GBCollab();"].forEach(function (needle) {
    contains(src, needle, "public API must be preserved: " + needle);
  });
  absent(src, "}.bind(this), RECONNECT_DELAY_MS);", "the old 2 s infinite retry must be gone");
  absent(src, "while (", "no while loop can resurrect the tab-lifetime retry");
});

h.note("FOLLOW-UP #2926: static guard only — bounded exponential backoff, terminal 1008/1006 policy handling, one localized aria-live status with retry. Driver must add catalog keys collab-unavailable and collab-retry to all 8 locales and run browser UAT; no browser/server/cargo was used here.");
