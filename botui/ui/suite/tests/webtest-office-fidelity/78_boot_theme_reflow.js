/**
 * #2933 final lane — deterministic static coverage for the last two >100 ms
 * Chrome violations that survived chunking: the 141 ms boot requestIdleCallback
 * in messages/10_scroll.js (geometry reads + overlay rebuild) and the 127 ms
 * forced reflow in the theme switch (a computed-style read after the owner's
 * token writes).
 *
 * The fix keeps guard 68/74/75 literals intact: the full marker rebuild is
 * still idle-delegated, but a boot-pending branch makes that first callback a
 * flag-only arm; real geometry work moves to the scroll rAF pass and the
 * mutation coalesced pass. The scheme helper caches the value derived from its
 * single pre-write read so the owner's pinned post-write sync call stays
 * write-only, with the cold read deferred.
 *
 * Every case is a literal source assertion: reverting any fix — or disturbing
 * a pinned guard literal — flips a case to FAIL. No server, browser, cargo or
 * network. The driver owns issue closure and the live Chrome UAT.
 */
"use strict";

var h = require("./00_harness");

var SCROLL = "botui/ui/suite/chat/messages/10_scroll.js";
var SCHEME = "botui/ui/suite/js/theme-scheme.js";
var OWNED = [SCROLL, SCHEME];

var LAYOUT = ["getComputedStyle", "getBoundingClientRect", "offsetWidth",
  "offsetHeight", "offsetTop", "scrollTop", "scrollHeight", "clientHeight",
  "clientWidth", "innerText"];

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
function noLayout(text, label) {
  LAYOUT.forEach(function (read) {
    absent(text, read, label + " must not force layout: <" + read + ">");
  });
}
function countIn(src, needle) { return src.split(needle).length - 1; }

// (a) The boot idle callback is a cheap arm, not the marker rebuild. The full
// rebuild stays pinned on the non-boot branch for guards 68/75.
h.staticTest("#2933 boot marker idle callback only flips an arming flag", function () {
  var src = h.readRepo(SCROLL);
  var sched = span(src, "function scheduleScrollbarMarkers() {", "function initScrollbarMarkers() {");
  contains(sched, "if (gbMarkersBootPending) {", "boot branch is gated by a pending flag");
  var boot = span(sched, "if (gbMarkersBootPending) {",
    "// #2933 — marker rebuild was a 129-158 ms rAF handler; run it on idle.");
  contains(boot, "window.gbIdle(function () {", "boot work stays on the shared idle scheduler");
  contains(boot, "gbMarkersRefreshQueued = false;", "the coalescing flag is released on idle");
  contains(boot, 'box.dataset.gbMarkersArmed = "1";', "boot only flips a data flag");
  contains(boot, "gbMarkersBootPending = false;", "boot consumes its one-shot pending flag");
  contains(boot, "return;", "the cheap branch never falls through to the rebuild");
  noLayout(boot, "the boot idle callback");
  contains(sched, "window.gbIdle(function () { gbMarkersRefreshQueued = false; gbRefreshScrollbarMarkers(); });",
    "the full rebuild stays pinned on the non-boot branch");
});

// (b) Marker geometry reads live only in the scroll rAF pass and the mutation
// coalesced pass; the boot pending flag keeps them out of the boot task.
h.staticTest("#2933 marker geometry reads move to the single rAF scroll pass", function () {
  var src = h.readRepo(SCROLL);
  var sync = span(src, "function gbSyncMarkerOverlay() {", "function scheduleMarkerSync() {");
  contains(sync, "if (!overlay) {", "a missing overlay arms the geometry pass");
  contains(sync, "if (!gbMarkersBootPending) gbRefreshScrollbarMarkers();",
    "the rebuild runs on the scroll rAF pass, off the boot path");
  contains(sync, "var top = messages.scrollTop;", "scroll geometry is read in the rAF pass");
  contains(sync, "var height = messages.clientHeight;", "client height is read in the rAF pass");
  var frame = span(src, "function scheduleMarkerSync() {", "/* #2933 — idle scheduler");
  contains(frame, "requestAnimationFrame(function () { gbMarkerSyncQueued = false; gbSyncMarkerOverlay(); });",
    "one coalesced rAF pass per scroll burst");
  var init = span(src, "function initScrollbarMarkers() {", "if (document.readyState === \"loading\") {");
  contains(init, 'messages.addEventListener("scroll", scheduleMarkerSync, { passive: true });',
    "scroll uses the coalesced rAF sync");
  contains(init, "if (changed) scheduleScrollbarMarkers();",
    "mutation triggers the coalesced rebuild");
});

// (c) The theme switch never reads computed style after the owner's writes:
// the bridge captures the scheme from pre-write tokens and caches it, so the
// pinned post-write syncColorScheme() call is write-only.
h.staticTest("#2933 theme switch has no read-after-write style snapshot", function () {
  var scheme = h.readRepo(SCHEME);
  var cached = span(scheme, "var gbSchemeSnapshot = null;", "function watchColorScheme() {");
  contains(cached, 'if (gbSchemeSnapshot === null) gbSchemeSnapshot = backgroundLuminanceIsDark() ? "dark" : "light";',
    "the cold read is captured once; the owner's post-write call is write-only");
  contains(cached, 'document.documentElement.setAttribute("data-color-scheme", gbSchemeSnapshot);',
    "the write uses the cached snapshot, never a fresh style read");
  absent(cached, "getComputedStyle", "no computed-style read after the token writes");
  absent(cached, "getPropertyValue", "no CSS property read after the token writes");
  var bridge = span(scheme, "function bridge(themeId, themeName, onDone) {", "window.GBThemeScheme = {");
  ordered(bridge, [
    "const rootStyle = getComputedStyle(document.documentElement);",
    'gbSchemeSnapshot = isDarkFromValues(background, bgColor) ? "dark" : "light";',
    'document.documentElement.setAttribute("data-color-scheme", gbSchemeSnapshot);',
    "Object.keys(vars).forEach((key) => document.documentElement.style.setProperty(key, vars[key]))",
  ], "all tokens read once, the scheme cached, then one write phase");
  var firstWrite = bridge.indexOf('document.documentElement.setAttribute("data-color-scheme"');
  ["getPropertyValue(", "getComputedStyle(", "getAttribute(", "getElementById("].forEach(function (read) {
    h.assert(bridge.indexOf(read, firstWrite) === -1,
      "no DOM read after the theme write: <" + read + ">");
  });
});

// (d) Non-vacuity: small shipped modules, preserved APIs, pinned literals, no
// new i18n key. Restoring a removed read or dropping a branch fails a case.
h.staticTest("#2933 non-vacuity: small modules, APIs and pinned literals survive", function () {
  OWNED.forEach(function (path) {
    var src = h.readRepo(path);
    var lines = src.split("\n").length;
    h.assert(lines < 450, path + " must stay under 450 lines, got " + lines);
    h.assert(src.length > 500, path + " must remain a non-trivial shipped module");
  });
  var scroll = h.readRepo(SCROLL);
  h.assertEqual(countIn(scroll, "gbT("), 2, "no new i18n key in the scroll module");
  contains(scroll, "function scrollToBottom(animate) {", "scroll API preserved");
  contains(scroll, "window.gbIdle = function (fn) {", "guard 68 idle helper preserved");
  contains(scroll, "gbIdle(function () { gbMarkersRefreshQueued = false; gbRefreshScrollbarMarkers(); });",
    "guard 68/75 rebuild callback literal survives");
  var scheme = h.readRepo(SCHEME);
  absent(scheme, "gbT(", "no user-facing literal added to the scheme helper");
  contains(scheme, "window.GBThemeScheme = {", "scheme global preserved");
  contains(scheme, "bridge: bridge,", "bridge API preserved");
  contains(scheme, "sync: syncColorScheme,", "sync API preserved");
});

h.note("FOLLOW-UP #2933: static guard only — flag-only boot idle arm, marker geometry on the scroll rAF, and a cached theme snapshot with no post-write read. Live Chrome <= 100 ms boot/theme UAT still needs a driver run; no browser/server/cargo used here.");
