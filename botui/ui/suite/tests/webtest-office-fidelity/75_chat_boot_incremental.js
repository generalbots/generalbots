/**
 * #2933 follow-up — deterministic static coverage for the chat boot handlers
 * that still exceeded the 100 ms forced-reflow budget: the 182 ms
 * requestIdleCallback handler in messages/10_scroll.js and the 116 ms setTimeout
 * handler in chat-init_a.js.
 *
 * Cases (all literal source assertions, no execution, no browser):
 *   (a) the marker rebuild captures every geometry/label read before its
 *       write-only phase; no layout read survives below the phase boundary;
 *   (b) the idle/timer handlers themselves read no layout, and every focus()
 *       call is confined to one coalesced idle pass;
 *   (c) scroll, resize, mutation and init work is coalesced into one rAF/idle
 *       pass per burst instead of running per event;
 *   (d) non-vacuity: small shipped modules, preserved APIs and guard-pinned
 *       literals from guards 68/74.
 *
 * Reverting the batching, reintroducing a layout read in a boot callback, or
 * un-coalescing a listener flips the matching case to FAIL. No server, browser,
 * cargo or network. The driver owns issue closure and the live Chrome UAT.
 */
"use strict";

var h = require("./00_harness");

var SCROLL = "botui/ui/suite/chat/messages/10_scroll.js";
var CHAT_A = "botui/ui/suite/chat/chat-init_a.js";
var CHAT_B = "botui/ui/suite/chat/chat-init_b.js";
var OWNED = [SCROLL, CHAT_A, CHAT_B];

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

// (a) The 182 ms marker rebuild: one read phase, then a write-only phase.
h.staticTest("chat marker rebuild batches every DOM read before its write phase", function () {
  var src = h.readRepo(SCROLL);
  var rebuild = span(src, "function gbRefreshScrollbarMarkers() {",
    "// Scroll path: one coalesced geometry read/write per frame, never per event.");
  contains(rebuild, "// READ PHASE — no writes until every geometry/label read is captured.",
    "the read phase is explicitly bounded");
  contains(rebuild, "// WRITE PHASE — no layout reads below this line.",
    "the write phase is explicitly bounded");
  ordered(rebuild, ["// READ PHASE", "// WRITE PHASE"], "read phase precedes write phase");
  var readPart = rebuild.slice(0, rebuild.indexOf("// WRITE PHASE"));
  var writePart = rebuild.slice(rebuild.indexOf("// WRITE PHASE"));
  ["messages.scrollHeight", "messages.clientHeight", "messages.scrollTop",
    "getComputedStyle(", "it.el.offsetHeight", "m.offsetTop", "gbMarkerLabel("].forEach(function (read) {
    contains(readPart, read, "layout/label read is captured up front: <" + read + ">");
  });
  noLayout(writePart, "the marker write phase");
  contains(writePart, "overlay.innerHTML = \"\";", "the overlay is cleared in the write phase");
  contains(writePart, "overlay.appendChild(marker);", "markers are appended in the write phase");

  // innerText forces layout; labels come from textContent captured in the read phase.
  var labelFn = span(src, "function gbMarkerLabel(message) {", "// Marker jumps are user-initiated");
  absent(labelFn, "innerText", "marker labels never read innerText");
  contains(labelFn, "body.textContent", "marker labels use textContent");

  // Live geometry is read only in a user-initiated jump, never during boot.
  var jump = span(src, "function gbMarkerJump(messages, target, top, live) {",
    "/* #2933 — marker rebuild");
  contains(jump, "messages.getBoundingClientRect();", "jump reads container geometry on click");
  absent(rebuild, "getBoundingClientRect", "the boot rebuild never reads bounding rects");
  contains(rebuild, "marker.addEventListener(\"click\", function () { gbMarkerJump(messages, target, at, live); });",
    "marker clicks delegate to the out-of-band jump helper");
});

// (b) The idle/timer handlers read no layout; focus lives in one idle pass.
h.staticTest("chat idle and timer handlers never read layout; focus is coalesced", function () {
  var src = h.readRepo(SCROLL);
  var idle = span(src, "window.gbIdle = function (fn) {", "function scheduleScrollbarMarkers() {");
  contains(idle, "requestIdleCallback(fn, { timeout: 800 })", "idle path preserved");
  contains(idle, "setTimeout(fn, 1)", "timer fallback preserved");
  noLayout(idle, "the shared idle scheduler");
  var markers = span(src, "function scheduleScrollbarMarkers() {", "function initScrollbarMarkers() {");
  contains(markers, "if (gbMarkersRefreshQueued) return;", "marker rebuild coalesces duplicate events");
  contains(markers, "window.gbIdle(function () { gbMarkersRefreshQueued = false; gbRefreshScrollbarMarkers(); });",
    "marker rebuild is a pure idle delegation");
  noLayout(markers, "the marker idle callback");

  var a = h.readRepo(CHAT_A);
  var focus = span(a, "var gbFocusQueued = false;", "// #1271 — a deep link can carry");
  contains(focus, "if (gbFocusQueued) return;", "focus requests coalesce");
  contains(focus, "if (window.gbIdle) window.gbIdle(gbApplyComposerFocus);", "focus runs on the shared idle pass");
  contains(focus, "else setTimeout(gbApplyComposerFocus, 1);", "focus keeps a timer fallback");
  noLayout(span(a, "function gbApplyComposerFocus() {", "// #1271 — a deep link can carry"),
    "the one focus handler");
  h.assertEqual(countIn(a, ".focus("), 1, "exactly one focus() call site in chat-init_a");

  var auto = span(a, "function autoFocusInput() {", "// #2171 — when chat opens");
  contains(auto, "gbScheduleComposerFocus();", "autofocus no longer focuses inside its timer");
  noLayout(auto, "the autofocus timer");
  var interval = span(a, "var iv = setInterval(function () {", "  }, 150);");
  noLayout(interval, "the deep-link retry tick");
  absent(interval, ".focus(", "the retry tick never focuses inline");
  var initA = span(a, "function proceedWithChatInit() {", "// #2933 — focus() forces layout");
  noLayout(initA, "the deferred chat-init path");
  var tail = a.slice(a.indexOf("function refocusComposerAfterLayoutChange() {"));
  contains(tail, "setTimeout(gbScheduleComposerFocus, 120);", "refocus only queues the idle pass");
  absent(tail, ".focus(", "no inline focus in the refocus timer");
});

// (c) One rAF/idle pass per burst for markers, live edge, scroll and focus.
h.staticTest("chat scroll, resize, mutation and init work coalesces into one pass", function () {
  var src = h.readRepo(SCROLL);
  var sync = span(src, "function scheduleMarkerSync() {", "/* #2933 — idle scheduler");
  contains(sync, "if (gbMarkerSyncQueued) return;", "scroll marker sync coalesces per frame");
  contains(sync, "requestAnimationFrame(function () { gbMarkerSyncQueued = false; gbSyncMarkerOverlay(); });",
    "one geometry pass per frame");
  var init = span(src, "function initScrollbarMarkers() {", "if (document.readyState === \"loading\") {");
  contains(init, "messages.addEventListener(\"scroll\", scheduleMarkerSync, { passive: true });",
    "scroll uses the coalesced sync");
  contains(init, "window.addEventListener(\"resize\", scheduleScrollbarMarkers);",
    "resize uses the coalesced rebuild");
  h.assertEqual(countIn(init, "scheduleScrollbarMarkers()"), 2,
    "mutation and boot both use the same coalesced rebuild");
  absent(init, "gbSyncMarkerOverlay,", "scroll no longer syncs the overlay on every event");

  var b = h.readRepo(CHAT_B);
  var live = span(b, "var gbLiveEdgeQueued = false;", "var gbScrollFrameQueued = false;");
  contains(live, "if (gbLiveEdgeQueued) return;", "live-edge pin coalesces resize + mutation");
  contains(live, "requestAnimationFrame(gbPinLiveEdge);", "one live-edge pass per frame");
  var pin = span(b, "function gbPinLiveEdge() {", "function gbScheduleLiveEdge() {");
  ordered(pin, ["var bottom = box.scrollHeight;", "box.scrollTop = bottom;"],
    "the live-edge pass batches its read before its write");
  var frame = span(b, "function gbOnScrollFrame() {", "function setupEventHandlers() {");
  h.assertEqual(countIn(frame, "box.scrollHeight"), 1, "scroll reads geometry once per frame");
  ordered(frame, ["var dist = box.scrollHeight - box.scrollTop - box.clientHeight;",
    "ChatState.isUserAtBottom = dist < threshold;", "updateScrollButton(dist);"],
    "scroll reads once, then writes state and button from the captured distance");
  var handlers = span(b, "if (messagesEl) {", "document.addEventListener(\"click\", function (e) {");
  contains(handlers, "new ResizeObserver(gbScheduleLiveEdge)", "resize schedules the shared pass");
  contains(handlers, "new MutationObserver(gbScheduleLiveEdge)", "mutation schedules the shared pass");
  h.assertEqual(countIn(handlers, "box.scrollTop = box.scrollHeight;"), 0,
    "no observer pins layout inline");
  contains(handlers, "if (gbScrollFrameQueued) return;", "scroll listener coalesces bursts");
  contains(handlers, "requestAnimationFrame(gbOnScrollFrame);", "scroll schedules one frame");
  absent(b, "input.focus(", "chat-init_b has no direct focus call");
  contains(b, "gbScheduleComposerFocus();", "idle refocus reuses the shared focus pass");
});

// (d) Non-vacuity: small modules, preserved APIs and guard 68/74 literals.
h.staticTest("incremental chat boot keeps shipped modules, APIs and pinned literals", function () {
  OWNED.forEach(function (path) {
    var src = h.readRepo(path);
    h.assert(src.split("\n").length < 450, path + " must stay under 450 lines");
    h.assert(src.length > 500, path + " must remain a non-trivial shipped module");
  });
  var src = h.readRepo(SCROLL);
  contains(src, "function scrollToBottom(animate) {", "scroll API preserved");
  contains(src, "window.gbIdle = function (fn) {", "guard 68 idle helper literal survives");
  contains(src, "gbIdle(function () { gbMarkersRefreshQueued = false; gbRefreshScrollbarMarkers(); });",
    "guard 68 marker callback literal survives as a substring");
  contains(src, "function updateScrollButton(dist) {", "button API preserved");
  h.assertEqual(countIn(src, "gbT("), 2, "no new i18n key was added");
  var a = h.readRepo(CHAT_A);
  contains(a, "function proceedWithChatInit() {", "chat boot API preserved");
  var b = h.readRepo(CHAT_B);
  contains(b, "function setupEventHandlers() {", "chat init API preserved");
  contains(b, 'window.gbIdle(function(){ if (typeof renderChatEmptyState === "function") renderChatEmptyState(); });',
    "guard 68 empty-state idle literal survives");
  absent(b, "if (typeof renderChatEmptyState === \"function\") renderChatEmptyState();\n})();",
    "empty-state card is still idle-deferred");
});

h.note("FOLLOW-UP #2933: static guard only — batched marker rebuild, layout-free idle/timer handlers, coalesced scroll/resize/init passes. Live Chrome [Violation] boot budget (<= 100 ms) still needs a driver run; no browser/server/cargo was used here.");
