/**
 * #2933 — deterministic static coverage for the final two >100 ms Chrome
 * violations: the 177/282 ms lock-screen requestIdleCallback handler and the
 * 155 ms window-chrome load handler.
 *
 * The fix splits one-time overlay construction into tiny coalesced idle steps
 * (shell, styles, markup, wiring/labels) and moves non-critical window-chrome
 * load work onto one coalesced idle/rAF pass with read-batch-before-write
 * geometry. Every case below is a literal source assertion: reverting either
 * chunking scheme, restoring a layout read before a write, or disturbing a
 * guard 68/71 literal flips a case to FAIL.
 *
 * No server, browser, cargo or network. The driver owns issue closure and the
 * live Chrome [Violation] <= 100 ms UAT.
 */
"use strict";

var h = require("./00_harness");

var LOCK = "botui/ui/suite/js/lock-screen.js";
var CHROME = "botui/ui/suite/js/window-manager/03_window_chrome.js";

var LAYOUT = ["getComputedStyle", "getBoundingClientRect", "offsetWidth",
  "offsetHeight", "offsetTop", "scrollTop", "scrollHeight", "clientWidth",
  "clientHeight", "innerText"];

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

// (a) Boot and visibility paths never build the dialog. The boot idle callback
// schedules the shell only; every heavy step lives in a later tiny chunk.
h.staticTest("#2933 lock boot idle callback schedules the shell, never styles/markup/labels", function () {
  var src = h.readRepo(LOCK);
  var init = span(src, "function init() {", "window.LockScreen = {");
  contains(init, "deferIdle(function () {", "boot still uses the idle scheduler");
  contains(init, "ensureOverlay();", "the shell is scheduled on boot idle");

  var boot = span(init, "deferIdle(function () {", "// A locked state survives");
  h.assertEqual(countIn(boot, "ensureOverlay();"), 1,
    "the boot idle callback schedules the shell exactly once");
  absent(boot, "ensureStyles(", "no style injection in the boot callback");
  absent(boot, "buildOverlay(", "no markup parse in the boot callback");
  absent(boot, "innerHTML", "no innerHTML in the boot callback");
  absent(boot, "refreshLabels(", "no label re-localization in the boot callback");
  absent(boot, "appendChild(", "no tree mutation beyond the shell in the boot callback");
  noLayout(boot, "the boot idle callback");

  var vis = span(src, "function onVisibilityChange() {", "function listenTauri() {");
  contains(vis, "var hidden = document.hidden;", "the handler still reads only a flag inline");
  contains(vis, "deferIdle(function () {", "its work still runs on idle");
  ["buildOverlay(", "ensureOverlay(", "ensureStyles(", "innerHTML",
    "appendChild(", "setInterval(", "focus("].forEach(function (call) {
    absent(vis, call, "visibility path never constructs: <" + call + ">");
  });
  noLayout(vis, "the visibility path");
  contains(vis, 'if (getState() === "locked") { if (!isLocked()) lock("restore"); }',
    "restore still happens off the event path");
});

// (b) Construction is a coalesced queue that runs exactly one step per idle
// task. No draining callback reads layout, and three separate chunks own the
// style, markup and wiring/label jobs that used to share one 282 ms callback.
h.staticTest("#2933 lock overlay construction is chunked and coalesced with no layout reads", function () {
  var src = h.readRepo(LOCK);
  var drain = span(src, "function drainOverlayChunks() {", "function queueOverlayChunk(fn) {");
  contains(drain, "var step = overlayChunks.shift();", "one step is dequeued per idle task");
  contains(drain, "if (step) step();", "exactly one step runs per task");
  contains(drain, "if (overlayChunks.length) queueOverlayChunk();",
    "remaining steps coalesce onto the next task");
  noLayout(drain, "the overlay chunk drain");

  var queue = span(src, "function queueOverlayChunk(fn) {", "function logErr(error, context) {");
  contains(queue, "if (overlayChunkQueued) return;", "bursts schedule only one task");
  contains(queue, "deferIdle(drainOverlayChunks);", "the queue runs on the shared idle helper");
  noLayout(queue, "the overlay chunk scheduler");

  var ensure = span(src, "function ensureOverlay() {", "function setReason(reason) {");
  h.assertEqual(countIn(ensure, "queueOverlayChunk(function () {"),
    3, "styles, markup and wiring/labels are three distinct chunks");
  contains(ensure, "var overlay = document.createElement(\"div\");",
    "only the empty shell is built synchronously");
  absent(ensure, "ensureStyles();\n", "style injection is not inline in the shell builder");
  var buildChunk = span(ensure, "pendingShell = overlay;", "queueOverlayChunk(function () {\n      wireOverlay(overlay);");
  contains(buildChunk, "buildOverlay(lastReason);", "markup parse lives in its own chunk");
  contains(buildChunk, "pendingShell = null;", "the shell handoff is one-shot");
  absent(buildChunk, "ensureStyles(", "the markup chunk never injects styles");
  var wireChunk = span(ensure, "wireOverlay(overlay);", "return overlay;");
  contains(wireChunk, "if (window.i18n && typeof window.i18n.getLocale === \"function\") refreshLabels();",
    "label localization lives in the final chunk");
  absent(wireChunk, "innerHTML", "the wiring chunk never reparses markup");
  h.assertEqual(countIn(src, "buildOverlay("), 2,
    "the dialog builder still has exactly one definition and one call site");
});

// (c) The window-chrome load handler only flips a counter; reveal, clamp and
// dock-icon work join one coalesced idle/rAF pass instead of running inline.
h.staticTest("#2933 window chrome load handler defers all non-critical work to one coalesced pass", function () {
  var src = h.readRepo(CHROME);
  var scheduler = span(src, "var chromeWork = [], chromeWorkQueued = false;", "WM.defineMethods(");
  contains(scheduler, "if (chromeWorkQueued) return;", "chrome jobs coalesce into one task");
  contains(scheduler, "window.requestIdleCallback(drainChromeWork, { timeout: 800 });",
    "the pass runs on idle");
  contains(scheduler, "window.requestAnimationFrame(drainChromeWork);",
    "the pass has an rAF fallback");
  var drain = span(scheduler, "function drainChromeWork() {", "function queueChromeWork(job) {");
  contains(drain, "var jobs = chromeWork;", "queued jobs are drained as one batch");
  noLayout(drain, "the chrome work drain");

  var onOne = span(src, "const onOne = () => {", "const timer = setTimeout(");
  contains(onOne, "remaining -= 1;", "a link load only decrements a counter");
  contains(onOne, "if (remaining <= 0) queueChromeWork(finish);",
    "reveal work joins the shared pass, never runs inline");
  absent(onOne, "finish();", "no direct reveal call in the load handler");
  noLayout(onOne, "the link load handler");
  contains(src, "const timer = setTimeout(function () { queueChromeWork(finish); }, 3000);",
    "the guard timer also joins the shared pass");

  var dock = span(src, "_addTaskbarDockItem(id) {", "_trackRecent(id, title) {");
  contains(dock, "queueChromeWork(() => {", "dock icon rendering is deferred");
  var job = span(dock, "queueChromeWork(() => {", "center.appendChild(dockItem);");
  ordered(job, ["const center = this.getTaskbarCenter();",
    "const app = (window.APPS_REGISTRY || APPS_REGISTRY).find((a) => a.id === id);",
    "const icon = this.getIconSvg(id);", "// READ PHASE done"],
    "geometry, title and icon are read before the node is built");
  var dockWrites = dock.slice(dock.indexOf("// READ PHASE done"),
    dock.indexOf("center.appendChild(dockItem);") + "center.appendChild(dockItem);".length);
  noLayout(dockWrites, "the dock icon write phase");
});

// (d) The clamp captures geometry once and derives the resized rect instead
// of re-reading layout between its size and position writes.
h.staticTest("#2933 window clamp batches its single geometry read before one write phase", function () {
  var src = h.readRepo(CHROME);
  var clamp = span(src, "_clampWindowIntoView(id) {", "// #2341 — universal per-app sanity affordance.");
  ordered(clamp, ["// READ PHASE —", "const rect = win.getBoundingClientRect();",
    "// WRITE PHASE"],
    "one bounded read phase precedes the write phase");
  h.assertEqual(countIn(clamp, "getBoundingClientRect("), 1,
    "the post-resize re-read is gone");
  contains(clamp, "const finalH = clampH === null ? rect.height : clampH;",
    "clamped height is derived from the captured rect");
  contains(clamp, "const finalW = clampW === null ? rect.width : clampW;",
    "clamped width is derived from the captured rect");
  noLayout(span(clamp, "// WRITE PHASE", "  },"),
    "the clamp write phase");
});

// (e) Non-vacuity: both modules remain small and shipped, and every literal
// pinned by guards 68/71 that surrounds this work survives unchanged.
h.staticTest("#2933 non-vacuity: shipped modules, prior guard literals and APIs survive", function () {
  [LOCK, CHROME].forEach(function (path) {
    var source = h.readRepo(path);
    h.assert(source.split("\n").length < 450, path + " must stay under 450 lines");
    h.assert(source.length > 500, path + " must remain a non-trivial shipped module");
  });

  var lock = h.readRepo(LOCK);
  contains(lock, "function deferIdle(fn) {", "guard 68/71 defer helper survives");
  contains(lock, "window.requestIdleCallback(fn, { timeout: 1000 })", "lock idle budget survives");
  contains(lock, "window.setTimeout(fn, 1)", "lock timer fallback survives");
  contains(lock, "overlay.hidden = false;", "lock still flips the cached node visible");
  contains(lock, "overlay.hidden = true;", "unlock still flips the cached node hidden");
  contains(lock, "deferIdle(focusPassword);", "focus stays off the lock task");
  contains(lock, "window.addEventListener(\"localeChanged\", refreshLabels);",
    "runtime locale switching survives");
  contains(lock, "window.LockScreen = { init: init, lock: lock,", "public lock API survives");
  var dialog = span(lock, "function buildOverlay(reason) {", "function wireOverlay(overlay) {");
  contains(dialog, 't("taskbar-lock", "Lock screen")', "dialog label literal survives");
  contains(dialog, 't("auth-login-password", "Password")', "password label literal survives");
  contains(dialog, 'role="status" aria-live="polite"', "status region contract survives");

  var chrome = h.readRepo(CHROME);
  var reveal = span(chrome, "const reclampAfterReveal = () => {", "if (!links.length) {");
  contains(reveal, "window.requestIdleCallback(() => this._clampWindowIntoView(wid), { timeout: 800 })",
    "guard 68 idle clamp survives");
  contains(reveal, "window.setTimeout(() => this._clampWindowIntoView(wid), 1)",
    "guard 68 clamp fallback survives");
  var finish = span(chrome, "const finish = () => {", "const onOne = () => {");
  contains(finish, "body.classList.add(\"gb-window-ready\");", "reveal stays synchronous in finish");
  ordered(finish, ["body.classList.add(\"gb-window-ready\");", "reclampAfterReveal();"],
    "guard 68 reveal/clamp order survives");
  contains(chrome, "_clampWindowIntoView(id) {", "clamp API survives");
});

h.note("FOLLOW-UP #2933: static guard only — chunked/coalesced lock overlay construction, layout-free load and visibility handlers, and one coalesced idle/rAF pass for window chrome with batched reads. Live Chrome boot/unlock budget (<= 100 ms) still needs a driver UAT; no browser/server/cargo was used here.");
