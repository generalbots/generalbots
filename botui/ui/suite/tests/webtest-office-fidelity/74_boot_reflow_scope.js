/**
 * #2933 follow-up — deterministic static coverage for the boot reflow scope bug.
 *
 * Live UAT confirmed the 50 ms theme timer fix, but boot still exceeded the
 * 100 ms forced-reflow budget and reported an Uncaught ReferenceError:
 * `deferIdle is not defined` at the boot-chrome initializer. The helper was
 * declared inside the DOMContentLoaded handler and called at module scope, so
 * widget/pinned-launcher/shortcuts/virtual-desktop init never ran.
 *
 * Cases (all literal source assertions, no execution, no browser):
 *   (a) deferIdle is declared ONCE at module scope, exported, and still sits
 *       inside guard 68's larger literal span (no guard literal is disturbed);
 *   (b) read-batch-before-write in the status update and clock tick;
 *   (c) chrome init, clock paint, status strip, conversation header and the
 *       duplicate Settings htmx refresh share an idle/rAF coalescing pass;
 *   (d) the three boot interval handlers never do DOM work inline;
 *   (e) non-vacuity: small shipped modules, preserved APIs and pinned literals.
 *
 * Reverting the scope fix, reintroducing read-after-write, or moving DOM work
 * back into an interval handler flips the matching case to FAIL. No server,
 * browser, cargo or network. The driver owns issue closure and Chrome UAT.
 */
"use strict";

var h = require("./00_harness");

var AUTH = "botui/ui/suite/desktop-inline/04_auth_fullscreen_a.js";
var CLOCK = "botui/ui/suite/desktop-inline/04_auth_fullscreen_b.js";
var STATUS = "botui/ui/suite/chat/chat-engineer-status.js";
var HEADER = "botui/ui/suite/chat/chat-conversation-header.js";
var SETTINGS = "botui/ui/suite/js/auto-update-settings.js";
var OWNED = [AUTH, CLOCK, STATUS, HEADER, SETTINGS];

var HEAVY = ["getElementById(", "querySelector(", "textContent", "innerHTML",
  "getComputedStyle", "getBoundingClientRect", "offsetWidth", "offsetHeight"];

function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1, (msg || "source must contain") + " <" + needle + ">");
}
function absent(src, needle, msg) {
  h.assert(src.indexOf(needle) === -1,
    (msg || "source must not contain") + " <" + needle + "> :: " + src.slice(0, 200));
}
function span(src, startMarker, endMarker) {
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + ">");
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + ">");
  h.assert(end > start, "empty span between <" + startMarker + "> and <" + endMarker + ">");
  return src.slice(start, end);
}
function ordered(spanText, markers, msg) {
  var last = -1;
  markers.forEach(function (marker) {
    var at = spanText.indexOf(marker);
    h.assert(at !== -1, (msg || "span") + " must contain <" + marker + ">");
    h.assert(at > last, (msg || "span") + " order broken at <" + marker + ">");
    last = at;
  });
}
function countIn(src, needle) { return src.split(needle).length - 1; }

// (a) The ReferenceError root cause: one module-scope declaration, exported,
// with the local DOMContentLoaded-scoped copy gone. Guard 68's larger span
// (which ends at "Listen for auth state changes") is left intact.
h.staticTest("#2933 deferIdle is declared once at module scope and exported, not trapped in DOMContentLoaded", function () {
  var src = h.readRepo(AUTH);
  var helper = "var deferIdle = function (fn) {";
  h.assertEqual(countIn(src, helper), 1, "exactly one deferIdle helper ships");
  var handler = span(src, 'document.addEventListener("DOMContentLoaded", async () => {',
    "            // #2933 — module-scope idle scheduler");
  absent(handler, helper, "the helper is no longer scoped to the boot handler");
  var boot = span(src, "            // #2933 — module-scope idle scheduler",
    "// Listen to HTMX afterRequest event");
  contains(boot, helper, "the helper ships at module scope");
  contains(boot, "window.requestIdleCallback(fn, { timeout: 800 })", "idle path preserved");
  contains(boot, "window.setTimeout(fn, 1)", "safe timer fallback preserved");
  contains(boot, "window.gbDeferIdle = deferIdle;", "helper is exported to split modules");
  var chrome = span(src, "// Widget pane + pinned launcher (#1160) — non-critical chrome (#2933).",
    "// Listen for auth state changes");
  contains(chrome, "deferIdle(function () {", "boot chrome init uses the module-scope helper");
  contains(chrome, "window.GBWidgets.init(", "widgets still initialize on boot");
  contains(chrome, "window.LockScreen.init();", "lock screen still initializes on boot");
  h.assertEqual(countIn(src, "deferIdle(function () {"), 1, "one idle chrome initializer");
});

// (b) Every DOM read is captured before the first write. The clock tick does
// no inline text paint at all — its write lives in the deferred paintClock().
h.staticTest("#2933 status update and clock tick batch all reads before any write", function () {
  var status = h.readRepo(STATUS);
  var update = span(status, "  function update() {", "  var scheduled = false;");
  var firstWrite = update.indexOf("if (connItem) connItem.dataset.state = conn.key;");
  h.assert(firstWrite !== -1, "status write-phase marker ships");
  ordered(update, ["var conn = connectionInfo();", "var connItem = strip.querySelector(",
    "var bot = currentBot();", "var sid = currentSession();", "var shortSid =",
    "var model = currentModel()", "var turn = turnInfo();",
    "if (connItem) connItem.dataset.state = conn.key;"], "status value reads precede its first write");
  var afterWrite = update.slice(firstWrite);
  ["getComputedStyle", "getBoundingClientRect", "offsetWidth", "offsetHeight", "scrollTop"].forEach(function (read) {
    absent(afterWrite, read, "no layout read after the first write: " + read);
  });

  var clock = h.readRepo(CLOCK);
  var tick = span(clock, "function renderClock() {", "renderClock();");
  absent(tick, "textContent", "clock tick performs no synchronous text write");
  ordered(tick, ["document.getElementById(\"clock-time\")", "new Date()",
    "var key = formatted.time", "if (key === last) return;",
    "deferBoot(function () { paintClock(formatted); });"], "read, derive, dedupe, then defer the paint");
  var paint = span(clock, "function paintClock(formatted) {", "function renderClock() {");
  contains(paint, "if (timeEl) timeEl.textContent = formatted.time;", "paint writes in the idle callback");
});

// (c) One coalescing scheduler is shared by boot chrome, the clock, the chat
// status strip, the conversation header and the Settings htmx refresh.
h.staticTest("#2933 boot chrome, clock, chat status/header and Settings refresh coalesce onto one idle/rAF pass", function () {
  var auth = h.readRepo(AUTH);
  contains(auth, "window.gbDeferIdle = deferIdle;", "shell exports the shared idle scheduler");
  contains(auth, "deferIdle(function () {", "chrome initializer is idle-deferred");

  var clock = h.readRepo(CLOCK);
  contains(clock, "var deferBoot = window.gbDeferIdle || function (fn) {", "clock reuses the shared helper");
  contains(clock, "if (typeof window.requestIdleCallback === \"function\") return window.requestIdleCallback(fn, { timeout: 800 });",
    "clock has an idle path");
  contains(clock, "return window.setTimeout(fn, 1);", "clock has a timer fallback");
  contains(clock, "deferBoot(function () { paintClock(formatted); });", "clock paint is coalesced");

  var status = h.readRepo(STATUS);
  contains(status, "var idle = window.gbIdle || window.setTimeout;", "status reuses the chat idle helper");
  contains(status, "idle(function () { ensureStrip(); update(); });", "boot strip paint is idle-deferred");
  contains(status, "idle(function () { scheduled = false; update(); });", "status ticks coalesce");

  var header = h.readRepo(HEADER);
  contains(header, "var idle = window.gbIdle || window.setTimeout;", "header reuses the chat idle helper");
  var schedule = span(header, "function scheduleTitleRefresh() {", "async function renameSession(");
  contains(schedule, "if (titleRefreshQueued) return;", "title refresh coalesces duplicate events");
  contains(schedule, "idle(function () { titleRefreshQueued = false; refreshTitle(); });",
    "title fetch and write run on idle");
  var mount = span(header, "function mount() {", "if (document.readyState === \"loading\") {");
  contains(mount, "idle(function () { bind(); });", "header mount is idle-deferred");
  absent(header, "setTimeout(refreshTitle, 50);", "old timer-based title refresh is gone");

  var settings = h.readRepo(SETTINGS);
  var refresh = span(settings, "function scheduleRenderSettings() {", "function showReleaseNotes(");
  contains(refresh, "if (refreshQueued) return;", "duplicate htmx refreshes coalesce");
  contains(refresh, "window.requestAnimationFrame(run)", "Settings refresh runs in one frame");
  h.assertEqual(countIn(settings, 'document.addEventListener("htmx:afterSwap", scheduleRenderSettings);'), 1,
    "one coalesced afterSwap listener");
  h.assertEqual(countIn(settings, 'document.addEventListener("htmx:load", scheduleRenderSettings);'), 1,
    "one coalesced load listener");
  absent(settings, 'document.addEventListener("htmx:afterSwap", renderSettings);',
    "no direct afterSwap re-render remains");
});

// (d) The three boot interval handlers only flip a flag and schedule work.
h.staticTest("#2933 boot interval handlers never do DOM work inline", function () {
  var status = h.readRepo(STATUS);
  var tick = span(status, "function scheduleUpdate() {", "function bind() {");
  HEAVY.forEach(function (heavy) {
    absent(tick, heavy, "status interval handler stays light: " + heavy);
  });
  contains(tick, "idle(function () { scheduled = false; update(); });", "status tick schedules one idle pass");
  contains(status, "setInterval(scheduleUpdate, 2000);", "interval calls the light scheduler");
  absent(status, "setInterval(update, 2000);", "no synchronous status tick survives");

  var header = h.readRepo(HEADER);
  var poll = span(header, "var timer = setInterval(function () {", "}, 250);");
  HEAVY.forEach(function (heavy) {
    absent(poll, heavy, "header poll handler stays light: " + heavy);
  });
  contains(poll, "scheduleTitleRefresh();", "poll only schedules the idle refresh");

  var clock = h.readRepo(CLOCK);
  var render = span(clock, "function renderClock() {", "renderClock();");
  ["textContent", "innerHTML", "getComputedStyle", "getBoundingClientRect"].forEach(function (heavy) {
    absent(render, heavy, "clock tick defers its write: " + heavy);
  });
  contains(clock, "setInterval(renderClock, 1000);", "clock cadence preserved");
});

// (e) Non-vacuity: small shipped modules, APIs and guard-pinned literals survive.
h.staticTest("#2933 non-vacuity: owned modules stay small, shipped and API-stable", function () {
  OWNED.forEach(function (path) {
    var src = h.readRepo(path);
    var lines = src.split("\n").length;
    h.assert(lines < 450, path + " must stay under 450 lines, got " + lines);
    h.assert(src.length > 500, path + " must remain a non-trivial shipped module");
  });
  var auth = h.readRepo(AUTH);
  contains(auth, "initAuthUI();", "auth UI still initializes");
  var guardSpan = span(auth, 'document.addEventListener("DOMContentLoaded", async () => {',
    "// Listen for auth state changes");
  contains(guardSpan, "var deferIdle = function (fn) {", "guard 68 helper literal survives");
  contains(guardSpan, "window.requestIdleCallback(fn, { timeout: 800 })", "guard 68 idle literal survives");
  contains(guardSpan, "window.setTimeout(fn, 1)", "guard 68 fallback literal survives");
  contains(h.readRepo(CLOCK), "setInterval(renderClock, 1000);", "clock cadence survives");
  contains(h.readRepo(STATUS), "window.gbUpdateEngineerStatus = update;", "status API survives");
  contains(h.readRepo(HEADER), 'window.addEventListener("gb-chat-ready", mount);', "chat-ready mount survives");
  contains(h.readRepo(SETTINGS), "window.GBUpdateSettings = {", "Settings API survives");
  contains(h.readRepo(SETTINGS), "/api/product", "release-notes fallback survives");
});

h.note("FOLLOW-UP #2933: static guard only — module-scope deferIdle with an idle/timer fallback, read-batch-before-write, coalesced idle/rAF boot paint and light interval handlers. The live Chrome forced-reflow <= 100 ms boot UAT still needs a driver run; no browser/server/cargo was used here.");
