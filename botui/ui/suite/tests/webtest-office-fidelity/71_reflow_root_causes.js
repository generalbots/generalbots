/**
 * #2933 — deterministic static coverage for the last reflow root cause and a
 * non-vacuous re-pin of the already-landed shell fixes.
 *
 * Guard 68 pinned the token bridge in theme-scheme.js, idle deferral in the
 * named long handlers and the once-built lock overlay. This guard complements
 * it with the piece guard 68 never covered: the theme *owner* still deferred
 * its bridge call through a 50 ms setTimeout in the stylesheet onload handler.
 * It also re-pins the four invariants with independent, complementary literal
 * assertions so reverting any one of them flips a case to FAIL.
 *
 * Every case is a source assertion. No server, browser, cargo or network. The
 * driver owns issue closure and the live Chrome [Violation] budget UAT.
 */
"use strict";

var h = require("./00_harness");

var OWNER = "botui/ui/suite/js/theme-manager.js";
var SCHEME = "botui/ui/suite/js/theme-scheme.js";
var SIDEBAR = "botui/ui/suite/js/sidebar_b_events.js";
var VIEWPORT = "botui/ui/suite/js/mobile-viewport.js";
var LOCK = "botui/ui/suite/js/lock-screen.js";
var OWNED = [OWNER, SCHEME, SIDEBAR, VIEWPORT, LOCK];

var LAYOUT_READS = ["getComputedStyle", "getBoundingClientRect",
  "offsetWidth", "offsetHeight", "scrollTop", "clientWidth", "clientHeight"];

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

// (a) The theme owner no longer defers the bridge onto a 50 ms task. The
// stylesheet load event is the style-resolution barrier, so the batched
// read/compute/write bridge runs inline and the dropdown follows it.
h.staticTest("#2933 theme owner bridges on load without a 50 ms timer or its own layout reads", function () {
  var owner = h.readRepo(OWNER);
  var onload = span(owner, "link.onload = () => {", "link.onerror =");

  contains(onload, "scheme.bridge(id, theme.name, announce);",
    "the bridge runs directly in the stylesheet load handler");
  absent(onload, "setTimeout", "the load handler never defers its bridge again");
  absent(owner, "}, 50);", "the 50 ms theme timer is removed from the owner");
  absent(owner, "setTimeout(() => {", "the deferred bridge wrapper is gone");
  ordered(onload, ["getElementById(\"theme-css\")", "scheme.bridge(", "updateDropdown();"],
    "single read, then one batched bridge/write, then dropdown sync");

  // The owner delegates every style read to the one batched bridge.
  absent(owner, "getPropertyValue(", "the owner never reads a raw CSS property itself");
  LAYOUT_READS.forEach(function (read) {
    absent(owner, read, "the owner never forces layout: <" + read + ">");
  });
  contains(owner, "scheme.bridge(", "the owner delegates to the single bridge");
  contains(owner, 'document.documentElement.setAttribute("data-theme", id);',
    "the single data-theme write site is preserved");
});

// (b) The token bridge is read-batch-before-write. This is an independent
// phrasing of guard 68's invariant: every DOM/style read is captured before the
// first inline write, and the colour scheme is derived from values already read.
h.staticTest("#2933 token bridge captures all reads before its single write phase", function () {
  var src = h.readRepo(SCHEME);
  var bridge = span(src, "function bridge(themeId, themeName, onDone) {", "window.GBThemeScheme = {");

  h.assertEqual(countIn(bridge, "getComputedStyle(document.documentElement)"), 1,
    "computed style is snapshotted exactly once");
  ordered(bridge, [
    "const rootStyle = getComputedStyle(document.documentElement);",
    "const themeDropdown = document.getElementById(\"themeDropdown\");",
    "document.documentElement.setAttribute(\"data-color-scheme\"",
    "Object.keys(vars).forEach((key) => document.documentElement.style.setProperty(key, vars[key]))",
  ], "snapshot, late reads, then one write phase");

  var firstWrite = bridge.indexOf("document.documentElement.setAttribute(\"data-color-scheme\"");
  h.assert(firstWrite !== -1, "the bridge must have an inline write phase");
  ["getPropertyValue(", "getComputedStyle(", "getAttribute(", "getElementById(",
    "getBoundingClientRect", "offsetWidth", "offsetHeight", "scrollTop"].forEach(function (read) {
    h.assert(bridge.indexOf(read, firstWrite) === -1,
      "no DOM/layout read after the first write: <" + read + ">");
  });

  var pure = span(src, "function isDarkFromValues(background, solid) {", "function backgroundLuminanceIsDark() {");
  absent(pure, "getComputedStyle", "the pure luminance helper never reads computed style");
  absent(pure, "getPropertyValue", "the pure luminance helper never reads a CSS property");
  contains(bridge, "isDarkFromValues(background, bgColor) ? \"dark\" : \"light\"",
    "the scheme is derived from already-read values");
  absent(bridge, "syncColorScheme();", "no read-after-write scheme re-detection tail");
});

// (c) Non-critical paint still defers to idle with a timer fallback. Re-pinned
// on the two owned shell files so reverting either idle helper is caught.
h.staticTest("#2933 sidebar and viewport keep idle deferral with a setTimeout fallback", function () {
  var sidebar = h.readRepo(SIDEBAR);
  var init = span(sidebar, "function init() {", "if (document.readyState === \"loading\") {");
  contains(sidebar, "function deferIdle(fn) {", "sidebar keeps its defer helper");
  contains(sidebar, "window.requestIdleCallback(fn, { timeout: 800 })", "sidebar keeps idle time");
  contains(sidebar, "window.setTimeout(fn, 1)", "sidebar keeps the timer fallback");
  contains(init, "deferIdle(fns.renderApps || window.renderApps || function () {});",
    "rail render stays on idle");
  ordered(init, ["deferIdle(fns.renderApps", "loadHistory();"],
    "idle scheduling precedes critical history work");
  absent(init, "\n    (fns.renderApps || window.renderApps)();",
    "no synchronous rail render was restored");

  var viewport = h.readRepo(VIEWPORT);
  contains(viewport, "function queueKeyboard(root, open) {", "keyboard flag stays coalesced");
  contains(viewport, "window.requestIdleCallback(fn, { timeout: 800 })", "viewport keeps idle time");
  contains(viewport, "window.setTimeout(fn, 1)", "viewport keeps the timer fallback");
  contains(viewport, "deferIdle(function () { keyIdle = 0; root.classList.toggle(\"gb-keyboard-open\", keyState); });",
    "keyboard class toggles on idle, not in the event");
  contains(viewport, "vv.addEventListener(\"resize\", schedule)", "resize is coalesced");
  contains(viewport, "vv.addEventListener(\"scroll\", schedule)", "scroll is coalesced");
  absent(viewport, "vv.addEventListener(\"resize\", apply)", "no direct apply() per resize event");
  var schedule = span(viewport, "function schedule() {", "function start() {");
  contains(schedule, "window.requestAnimationFrame", "one apply per frame");
});

// (d) The lock overlay is built once and the visibility path only flips a
// flag. Re-pinned with different literals than guard 68.
h.staticTest("#2933 lock visibility handler never constructs the overlay", function () {
  var src = h.readRepo(LOCK);
  var vis = span(src, "function onVisibilityChange() {", "function listenTauri() {");
  contains(vis, "var hidden = document.hidden;", "the handler reads only a cheap flag inline");
  contains(vis, "deferIdle(function () {", "its work runs on idle");
  absent(vis, "buildOverlay(", "no overlay build on the visibility path");
  absent(vis, "ensureOverlay(", "no lazy build on the visibility path");
  absent(vis, "innerHTML", "no markup parse on the visibility path");
  absent(vis, "appendChild(", "no node insertion on the visibility path");
  absent(vis, "setInterval(", "no timer is armed on the visibility path");
  absent(vis, "focus(", "no layout-forcing focus on the visibility path");
  LAYOUT_READS.forEach(function (read) {
    absent(vis, read, "the visibility path never reads layout: <" + read + ">");
  });
  contains(vis, "if (getState() === \"locked\") { if (!isLocked()) lock(\"restore\"); }",
    "restore still happens, off the event path");

  h.assertEqual(countIn(src, "buildOverlay("), 2,
    "the dialog is constructed in exactly one place (definition + one call)");
  var ensure = span(src, "function ensureOverlay() {", "function setReason(reason) {");
  contains(ensure, "if (overlayEl && overlayEl.isConnected) return overlayEl;", "building is once-only");
  contains(ensure, "overlay.hidden = true;", "the cached node starts hidden");
  var setLocked = span(src, "function setLocked(on, reason) {", "function unlock() {");
  contains(setLocked, "overlay.hidden = false;", "lock flips the cached node visible");
  contains(setLocked, "overlay.hidden = true;", "unlock flips the cached node hidden");
  absent(setLocked, "innerHTML", "lock/unlock never reparses markup");
  var tick = span(src, "function tick() {", "function setStatus(");
  LAYOUT_READS.forEach(function (read) {
    absent(tick, read, "the clock tick never reads layout: <" + read + ">");
  });
});

// (e) Non-vacuity: owned files stay small and shipped; no user-facing literal
// was smuggled into the two modules that had none; public APIs survive.
h.staticTest("#2933 non-vacuity: owned modules stay small, shipped and API-stable", function () {
  OWNED.forEach(function (path) {
    var src = h.readRepo(path);
    var lines = src.split("\n").length;
    h.assert(lines < 450, path + " must stay under 450 lines, got " + lines);
    h.assert(src.length > 500, path + " must remain a non-trivial shipped module");
  });
  [SIDEBAR, VIEWPORT].forEach(function (path) {
    var src = h.readRepo(path);
    absent(src, "gbT(", path + " must not introduce a user-facing literal");
    absent(src, "data-i18n", path + " must not introduce a raw i18n key");
  });
  contains(h.readRepo(OWNER), "window.ThemeManager = api;", "theme owner API preserved");
  contains(h.readRepo(OWNER), "window.GBTheme = api;", "legacy theme API preserved");
  contains(h.readRepo(SIDEBAR), "window.GBSidebar = {", "sidebar API preserved");
  contains(h.readRepo(VIEWPORT), "window.MobileViewport = MobileViewport;", "viewport API preserved");
  contains(h.readRepo(LOCK), "window.LockScreen = { init: init, lock: lock,", "lock API preserved");
  contains(h.readRepo(SCHEME), "window.GBThemeScheme = {", "scheme API preserved");
});

h.note("FOLLOW-UP #2933: static guard only — owner bridges on load without the 50 ms timer, bridge batches reads before writes, idle deferral and once-built lock overlay re-pinned. Live Chrome CPU profiling (no [Violation] ... took > 100 ms during boot, theme switch and unlock) still needs a driver browser run; no browser/server/cargo was used here.");
