/**
 * #2933 — deterministic static coverage for the shell reflow/idle fixes.
 *
 * Chrome reported ~35 forced reflows, 160-1343 ms DOMContentLoaded/load
 * handlers and a 1072 ms visibilitychange handler during boot, theme switch
 * and unlock. Three concrete fixes are pinned here:
 *   (a) the token bridge batches every DOM read before its first write and
 *       derives the colour scheme from values it already read (no read-after-
 *       write syncColorScheme tail);
 *   (b) non-critical paint in the named long handlers runs on an idle callback
 *       with a setTimeout fallback — rail, keyboard flag, widget chrome,
 *       post-reveal clamp, empty-state card, marker overlay;
 *   (c) the lock-screen dialog is built exactly once and kept in the DOM;
 *       lock/unlock and the deferred visibilitychange path only flip its
 *       `hidden` flag, with no markup parse, style injection or layout read;
 *   (d) i18n skips its boot `localeChanged` announce when the resolved locale
 *       matches the language the shell was already rendered in;
 *   (e) the four named setInterval ticks skip hidden/overlapping work and
 *       never rewrite unchanged text or read layout.
 *
 * Every case is a literal source assertion. Non-vacuity: restoring any of the
 * removed inline calls, or removing a guard, flips a case to FAIL.
 * No server, browser, cargo or network. The driver owns issue closure/UAT.
 */
"use strict";

var h = require("./00_harness");

var SCHEME = "botui/ui/suite/js/theme-scheme.js";
var SIDEBAR = "botui/ui/suite/js/sidebar_b_events.js";
var VIEWPORT = "botui/ui/suite/js/mobile-viewport.js";
var WCHROME = "botui/ui/suite/js/window-manager/03_window_chrome.js";
var AUTH = "botui/ui/suite/desktop-inline/04_auth_fullscreen_a.js";
var LOCK = "botui/ui/suite/js/lock-screen.js";
var CHAT_INIT = "botui/ui/suite/chat/chat-init_b.js";
var SCROLL = "botui/ui/suite/chat/messages/10_scroll.js";
var I18N = "botui/ui/suite/js/i18n.js";
var CLOCK = "botui/ui/suite/desktop-inline/04_auth_fullscreen_b.js";
var COMPUTER = "botui/ui/suite/chat/modules/60_computer_use.js";
var TERMINAL = "botui/ui/suite/chat/modules/61_terminal_use.js";
var AUTOTASKS = "botui/ui/suite/chat/modules/23_autotasks.js";
var EDITED = [SCHEME, SIDEBAR, VIEWPORT, WCHROME, AUTH, LOCK, CHAT_INIT, SCROLL,
  I18N, CLOCK, COMPUTER, TERMINAL, AUTOTASKS];

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
function ordered(src, before, after, msg) {
  var b = src.indexOf(before), a = src.indexOf(after);
  h.assert(b !== -1 && a !== -1 && b < a,
    (msg || "ordering regressed") + " :: <" + before + "> before <" + after + ">");
}
function countIn(src, needle) { return src.split(needle).length - 1; }

// (a) Token bridge: one computed-style snapshot, every DOM read captured
// before the first inline write, and a pure scheme derivation replacing the
// post-write syncColorScheme() re-read.
h.staticTest("#2933 token bridge batches DOM reads before writes; no read-after-write reflow", function () {
  var src = h.readRepo(SCHEME);
  var bridge = span(src, "function bridge(themeId, themeName, onDone) {", "window.GBThemeScheme = {");

  contains(src, "function isDarkFromValues(background, solid) {", "pure luminance helper ships");
  var pure = span(src, "function isDarkFromValues(background, solid) {", "function backgroundLuminanceIsDark() {");
  absent(pure, "getComputedStyle", "the pure helper never reads computed style");
  absent(pure, "getPropertyValue", "the pure helper never reads a CSS property");
  contains(src, "return isDarkFromValues(rootStyle.getPropertyValue(\"--background\").trim(),",
    "the public detector still delegates to the pure helper");
  h.assertEqual(countIn(bridge, "getComputedStyle(document.documentElement)"), 1,
    "the bridge snapshots computed style exactly once");

  var firstWrite = bridge.indexOf("document.documentElement.style.setProperty");
  h.assert(firstWrite !== -1, "the bridge must have an inline write phase");
  ["getPropertyValue(", "getComputedStyle(", "getElementById(", "getAttribute("].forEach(function (read) {
    h.assert(bridge.indexOf(read, firstWrite) === -1,
      "no DOM read after the first write: <" + read + ">");
  });
  ordered(bridge, "const themeDropdown = document.getElementById(\"themeDropdown\");",
    "Object.keys(vars).forEach((key) => document.documentElement.style.setProperty(key, vars[key]))",
    "the dropdown node is captured before any write");
  ordered(bridge, "document.documentElement.setAttribute(\"data-color-scheme\"",
    "Object.keys(vars).forEach((key) => document.documentElement.style.setProperty(key, vars[key]))",
    "the colour scheme is derived and written in the same write phase");
  contains(bridge, "isDarkFromValues(background, bgColor) ? \"dark\" : \"light\"",
    "the scheme comes from already-read values, not a fresh layout read");
  absent(bridge, "syncColorScheme();", "the old read-after-write tail is gone");
  absent(bridge, "backgroundLuminanceIsDark()", "the bridge no longer re-detects after writing");

  // Both token vocabularies and the owner-facing API survive (#2930 guard).
  contains(bridge, "vars[\"--background\"] = background;", "base vocabulary preserved");
  contains(bridge, "vars[\"--foreground\"] = foreground;", "foreground vocabulary preserved");
  contains(src, "bridge: bridge,", "bridge API preserved");
  contains(src, "clearInline: clearInline,", "cleanup API preserved");
});

// (b) Idle deferral in each of the named long handlers, with a fallback and
// the critical work still synchronous.
h.staticTest("#2933 non-critical paint defers to requestIdleCallback with a setTimeout fallback", function () {
  var sidebar = h.readRepo(SIDEBAR);
  var sidebarInit = span(sidebar, "function init() {", "if (document.readyState === \"loading\") {");
  contains(sidebar, "function deferIdle(fn) {", "sidebar defer helper ships");
  contains(sidebar, "window.requestIdleCallback(fn, { timeout: 800 })", "sidebar uses idle time");
  contains(sidebar, "window.setTimeout(fn, 1)", "sidebar has a timer fallback");
  contains(sidebarInit, "deferIdle(fns.renderApps || window.renderApps || function () {});",
    "section rail renders on idle");
  ordered(sidebarInit, "deferIdle(fns.renderApps", "loadHistory();", "idle scheduling precedes critical data work");
  absent(sidebarInit, "\n    (fns.renderApps", "the old synchronous rail render is gone");

  var viewport = h.readRepo(VIEWPORT);
  contains(viewport, "function queueKeyboard(root, open) {", "keyboard flag is coalesced");
  contains(viewport, "window.requestIdleCallback(fn, { timeout: 800 })", "viewport uses idle time");
  contains(viewport, "window.setTimeout(fn, 1)", "viewport has a timer fallback");
  contains(viewport, "vv.addEventListener(\"resize\", schedule)", "resize is coalesced");
  contains(viewport, "vv.addEventListener(\"scroll\", schedule)", "scroll is coalesced");
  absent(viewport, "vv.addEventListener(\"resize\", apply)", "no direct apply() per resize event");
  var vschedule = span(viewport, "function schedule() {", "function start() {");
  contains(vschedule, "window.requestAnimationFrame", "one apply per frame");

  var auth = h.readRepo(AUTH);
  var authHandler = span(auth, "document.addEventListener(\"DOMContentLoaded\", async () => {", "// Listen for auth state changes");
  contains(authHandler, "var deferIdle = function (fn) {", "auth handler defer helper ships");
  contains(authHandler, "window.requestIdleCallback(fn, { timeout: 800 })", "auth uses idle time");
  contains(authHandler, "window.setTimeout(fn, 1)", "auth has a timer fallback");
  var chrome = authHandler.slice(authHandler.indexOf("deferIdle(function () {"));
  contains(chrome, "window.GBWidgets.init(", "widget pane moved into the idle callback");
  contains(chrome, "window.LockScreen.init();", "lock screen setup moved into the idle callback");
  contains(authHandler, "window.wm = window.WindowManager;", "window manager binding stays synchronous");

  var wchrome = h.readRepo(WCHROME);
  var reveal = span(wchrome, "const reclampAfterReveal = () => {", "if (!links.length) {");
  contains(reveal, "window.requestIdleCallback(() => this._clampWindowIntoView(wid), { timeout: 800 })",
    "post-reveal clamp is idle-deferred");
  contains(reveal, "window.setTimeout(() => this._clampWindowIntoView(wid), 1)", "clamp has a timer fallback");
  var finish = span(wchrome, "const finish = () => {", "const onOne = () => {");
  contains(finish, "body.classList.add(\"gb-window-ready\");", "reveal itself stays synchronous");
  ordered(finish, "body.classList.add(\"gb-window-ready\");", "reclampAfterReveal();", "reveal precedes the deferred clamp");

  var scroll = h.readRepo(SCROLL);
  contains(scroll, "window.gbIdle = function (fn) {", "chat shares one idle scheduler");
  var idleFn = span(scroll, "window.gbIdle = function (fn) {", "function scheduleScrollbarMarkers() {");
  contains(idleFn, "requestIdleCallback(fn, { timeout: 800 })", "chat uses idle time");
  contains(idleFn, "setTimeout(fn, 1)", "chat has a timer fallback");
  var markers = span(scroll, "function scheduleScrollbarMarkers() {", "function initScrollbarMarkers() {");
  contains(markers, "gbIdle(function () { gbMarkersRefreshQueued = false; gbRefreshScrollbarMarkers(); });",
    "marker overlay no longer runs in an rAF handler");
  absent(markers, "requestAnimationFrame", "the old rAF marker rebuild is gone");

  var chat = h.readRepo(CHAT_INIT);
  contains(chat, "window.gbIdle(function(){ if (typeof renderChatEmptyState === \"function\") renderChatEmptyState(); });",
    "empty-state card paints on idle");
  absent(chat, "if (typeof renderChatEmptyState === \"function\") renderChatEmptyState();\n})();",
    "the old inline empty-state render is gone");
});

// (c) The dialog is built once and cached. Neither the deferred visibility
// path nor lock/unlock constructs layout: they flip one `hidden` attribute.
h.staticTest("#2933 lock-screen builds the overlay once; visibility/lock paths only flip hidden", function () {
  var src = h.readRepo(LOCK);
  contains(src, "document.addEventListener(\"visibilitychange\", onVisibilityChange);",
    "the visibility handler is registered");
  contains(src, "function deferIdle(fn) {", "lock-screen defer helper ships");
  contains(src, "window.requestIdleCallback(fn, { timeout: 1000 })", "restore uses idle time");
  contains(src, "window.setTimeout(fn, 1)", "restore has a timer fallback");

  var vis = span(src, "function onVisibilityChange() {", "function listenTauri() {");
  contains(vis, "var hidden = document.hidden;", "only a cheap flag is read inline");
  var deferAt = vis.indexOf("deferIdle(");
  h.assert(deferAt !== -1, "visibilitychange must defer its work");
  // No construction on this path AT ALL — including inside the idle callback.
  ["buildOverlay(", "wireOverlay(", "ensureStyles(", "appendChild(", "innerHTML",
    "setContentHidden(", "focusPassword(", "tick(", "setInterval("].forEach(function (call) {
    absent(vis, call, "no overlay construction on the visibility path: <" + call + ">");
  });
  var deferred = vis.slice(deferAt);
  contains(deferred, "if (getState() === \"locked\") { if (!isLocked()) lock(\"restore\"); }",
    "restore still happens, off the event path");
  contains(deferred, "else if (!isLocked()) resetIdle();", "resume re-arms idle without re-locking");
  absent(deferred, "lock(\"resume\")", "resume never re-locks");

  // One idempotent builder; the node is appended hidden and wired once.
  h.assertEqual(countIn(src, "buildOverlay("), 2,
    "dialog is constructed in exactly one place (definition + one call)");
  var ensure = span(src, "function ensureOverlay() {", "function setReason(reason) {");
  contains(ensure, "if (overlayEl && overlayEl.isConnected) return overlayEl;", "building is once-only");
  contains(ensure, "overlay.hidden = true;", "the cached node starts hidden");
  contains(ensure, "wireOverlay(overlay);", "listeners are attached once");
  var initSpan = span(src, "function init() {", "window.LockScreen = {");
  contains(initSpan, "deferIdle(function () {", "construction is scheduled on boot idle");
  contains(initSpan, "ensureOverlay();", "the dialog exists before any lock/visibility event");

  // lock/unlock are attribute flips, never a rebuild or a layout read.
  var setLocked = span(src, "function setLocked(on, reason) {", "function unlock() {");
  contains(setLocked, "overlay.hidden = false;", "lock reveals the cached node");
  contains(setLocked, "overlay.hidden = true;", "unlock hides the cached node");
  ["buildOverlay(", "ensureStyles(", "innerHTML", "focusPassword();"].forEach(function (call) {
    absent(setLocked, call, "no synchronous construction/focus on lock: <" + call + ">");
  });
  contains(setLocked, "deferIdle(focusPassword);", "focus() moved off the lock task");
  var unlockSpan = span(src, "function unlock() {", "function lock(reason) {");
  absent(unlockSpan, ".remove(", "unlock keeps the dialog in the DOM");

  // The clock tick stays bounded: text writes only, no layout reads.
  var tickSpan = span(src, "function tick() {", "function setStatus(");
  ["getBoundingClientRect", "getComputedStyle", "offsetWidth", "offsetHeight", "scrollTop"].forEach(function (read) {
    absent(tickSpan, read, "tick never reads layout: <" + read + ">");
  });
  absent(src, "visibility:hidden", "no body-wide visibility invalidation per lock");
  contains(src, "window.addEventListener(\"localeChanged\", refreshLabels);",
    "cached labels follow a runtime locale switch");

  // The overlay's i18n contract and public API are untouched.
  var overlay = span(src, "function buildOverlay(reason) {", "function wireOverlay(overlay) {");
  contains(overlay, "t(\"taskbar-lock\", \"Lock screen\")", "lock dialog label preserved");
  contains(overlay, "t(\"auth-login-password\", \"Password\")", "password label preserved");
  contains(overlay, "role=\"status\" aria-live=\"polite\"", "status region contract preserved");
  contains(src, "function lock(reason) {", "the explicit lock path still locks");
  contains(src, "window.LockScreen = { init: init, lock: lock,", "public API preserved");
});

// (d) The boot announce re-rendered ~20 imperative surfaces with text that was
// already correct. Skip it when the locale matches the rendered <html lang>.
h.staticTest("#2933 i18n skips the redundant boot announce when the locale is unchanged", function () {
  var src = h.readRepo(I18N);
  contains(src, "function sameRenderedLocale(locale, renderedLang) {", "locale comparison helper ships");
  contains(src, "var renderedLang = document.documentElement.getAttribute(\"lang\");",
    "rendered language is captured before it is overwritten");
  var announce = span(src, "function announceLocale() {", "if (document.readyState === \"loading\") {");
  contains(announce, "if (sameRenderedLocale(currentLocale, renderedLang)) return;",
    "unchanged locale skips the redundant re-render pass");
  contains(announce, "window.dispatchEvent(new CustomEvent(\"localeChanged\"", "real switches still announce");
  contains(src, "window.__gbI18nShared.translatePage();", "static DOM translation preserved");
  contains(src, "setupBodyListeners();", "late-node observer setup preserved");
  var setLocale = span(src, "async function setLocale(locale) {", "function setupBodyListeners() {");
  contains(setLocale, "new CustomEvent(\"i18n:localeChanged\"", "runtime switches keep both events");
  contains(setLocale, "await loadTranslations(locale);", "runtime switches still load the catalogue");
});

// #2921 — an early gbT() miss is queued, not warned. Once the catalogue is
// live, every queued key is re-resolved and only genuinely absent keys warn.
h.staticTest("#2921 early gbT calls do not warn when the key resolves later", function () {
  var src = h.readRepo(I18N);
  contains(src, "const earlyKeys = new Set();", "early misses are queued");
  var flush = span(src, "function reportMissing(key) {", "function t(key, params) {");
  contains(flush, "function resolveEarlyKeys() {", "a re-resolution pass ships");
  contains(flush, "earlyKeys.forEach((key) => {", "queued keys are revisited");
  contains(flush, "translations[key] === undefined && englishFallback[key] === undefined &&",
    "all catalogues are consulted before warning");
  contains(flush, "MINIMAL_FALLBACK[key] === undefined) reportMissing(key);",
    "only keys still absent everywhere are warned");
  contains(flush, "earlyKeys.clear();", "the queue is drained once");
  contains(flush, "console.warn(", "genuinely missing keys still warn");
  contains(flush, "missingKeysReported.add(key);", "one warning per key is preserved");

  var tSpan = span(src, "function t(key, params) {", "window.__gbI18nShared = { t: t };");
  var missAt = tSpan.indexOf("text = key;");
  h.assert(missAt !== -1, "t() still returns the key on a miss");
  var miss = tSpan.slice(missAt, tSpan.indexOf("if (params &&"));
  contains(miss, "if (!isInitialized) earlyKeys.add(key);", "early call queues instead of warning");
  absent(miss, "console.warn", "the early-call branch emits no warning");
  contains(tSpan.slice(missAt), "else reportMissing(key);", "post-boot misses still warn directly");

  var initSpan = span(src, "async function init() {", "async function getAvailableLocales() {");
  ordered(initSpan, "await loadTranslations(locale);", "resolveEarlyKeys();",
    "re-resolution runs after the catalogue arrives");
  ordered(initSpan, "isInitialized = true;", "resolveEarlyKeys();",
    "the queue is drained with live catalogues");
});

// (e) setInterval ticks: hidden tabs and overlapping requests skip; unchanged
// clock text is not rewritten; no tick reads layout.
h.staticTest("#2933 setInterval ticks are bounded and skip hidden tabs", function () {
  [["BCOMP", COMPUTER, "pollPendingBrowser", 10000],
    ["BTERM", TERMINAL, "pollPendingTerminal", 10000]].forEach(function (spec) {
    var src = h.readRepo(spec[1]);
    var poll = span(src, "function " + spec[2] + "() {", "setInterval(");
    contains(poll, "document.hidden || polling", "hidden tabs and in-flight polls skip");
    contains(poll, "polling = true;", "in-flight flag is set before fetch");
    contains(poll, "polling = false;", "in-flight flag is cleared on settle");
    contains(src, "var POLL_MS = " + spec[3] + ", polling = false;", "poll cadence raised above 3 s");
    contains(src, "setInterval(" + spec[2] + ", POLL_MS);", "interval uses the bounded cadence");
    ["getBoundingClientRect", "getComputedStyle", "offsetWidth", "offsetHeight"].forEach(function (read) {
      absent(poll, read, spec[0] + " poll never reads layout: <" + read + ">");
    });
  });

  var at = h.readRepo(AUTOTASKS);
  contains(at, "var POLL_MS = 60000;", "autotask cadence raised to 60 s");
  var pollAt = span(at, "function poll() {", "function start() {");
  contains(pollAt, "document.hidden || AT.state.refreshing", "hidden tabs and in-flight refreshes skip");
  contains(pollAt, "AT.state.refreshing = true;", "refresh flag is set");
  contains(pollAt, "AT.state.refreshing = false;", "refresh flag is cleared");

  var clock = h.readRepo(CLOCK);
  var render = span(clock, "function renderClock() {", "renderClock();");
  contains(render, "if (document.hidden) return;", "hidden tab skips the clock tick");
  contains(render, "if (key === last) return;", "unchanged clock text is not rewritten");
  ["getBoundingClientRect", "getComputedStyle", "offsetWidth", "offsetHeight"].forEach(function (read) {
    absent(render, read, "clock tick never reads layout: <" + read + ">");
  });
  contains(clock, "if (timeEl) timeEl.textContent = formatted.time;", "clock still paints");
  contains(clock, "setInterval(renderClock, 1000);", "minute-precision cadence preserved");
});

// (f) Non-vacuity: no hardcoded user-facing strings were added, all edited
// modules stay under 450 lines, and the shipped APIs/behaviour remain.
h.staticTest("#2933 non-vacuity: no new user strings, small modules, preserved behaviour", function () {
  EDITED.forEach(function (path) {
    var src = h.readRepo(path);
    var lines = src.split("\n").length;
    h.assert(lines < 450, path + " must stay under 450 lines, got " + lines);
    h.assert(src.length > 500, path + " must remain a non-trivial shipped module");
    // These three modules shipped without any user-facing literal; the fix
    // must not smuggle one in. WCHROME/AUTH already use gbT with fallbacks.
    if (path === SCHEME || path === SIDEBAR || path === VIEWPORT) {
      absent(src, "gbT(", path + " must not introduce a user-facing literal");
      absent(src, "data-i18n", path + " must not introduce a raw i18n key");
    }
  });
  contains(h.readRepo(SCHEME), "window.GBThemeScheme = {", "scheme global preserved");
  contains(h.readRepo(SIDEBAR), "window.GBSidebar = {", "sidebar global preserved");
  contains(h.readRepo(VIEWPORT), "window.MobileViewport = MobileViewport;", "viewport global preserved");
  contains(h.readRepo(SCROLL), "function scrollToBottom(animate) {", "scroll API preserved");
  contains(h.readRepo(CHAT_INIT), "function setupEventHandlers() {", "chat init API preserved");
  contains(h.readRepo(AUTH), "initAuthUI();", "auth UI still initializes");
  contains(h.readRepo(WCHROME), "_clampWindowIntoView(id) {", "clamp API preserved");
});

h.note("FOLLOW-UP #2933: static guard only — batched token reads, once-built lock overlay with flag-only visibility/lock paths, skipped redundant boot announce and bounded setInterval ticks. Chrome CPU profiling (no [Violation] ... took > 100 ms during boot, unlock and one deep-link open) still needs a driver browser run; no browser/server/cargo was used here.");
