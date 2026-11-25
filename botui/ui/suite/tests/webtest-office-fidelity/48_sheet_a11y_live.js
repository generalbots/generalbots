/**
 * #2435 — deterministic static coverage for the Sheet accessibility live-region
 * hosting fix:
 *   (a) the polite status region is hosted on document.body (never on the grid
 *       root), keyed by a window-scoped id and reused via getElementById, while
 *       role=status / aria-live=polite / aria-atomic=true are retained;
 *   (b) announce() defensively re-hosts a detached node onto document.body
 *       before writing, and writes only via textContent (never innerHTML);
 *   (c) repeated init() cannot create a duplicate live node (id guard), and the
 *       grid.announce() API is unchanged (prototype method + call sites intact).
 *
 * All assertions are literal substrings of production source. Nothing is
 * executed: no server, no browser, no cargo, no network. NON-VACUITY: each case
 * first proves the real artifacts ship and that the extracted span is non-empty
 * (via `between`), then asserts exact substrings and strict ordering, so
 * reverting hosting to `this.root` or deleting the idempotency guard FAILs.
 */
"use strict";

var h = require("./00_harness");

var A11Y = "botui/ui/suite/sheet/js/constants/03a_grid_a11y.js";
var RENDER = "botui/ui/suite/sheet/js/constants/03_grid_render.js";

/** Literal substring assertion — no regex metacharacter escaping needed. */
function contains(relPath, needle, msg) {
  h.assert(h.readRepo(relPath).indexOf(needle) !== -1,
    (msg || "source must contain") + " <" + needle + "> in " + relPath);
}

/** Count exact, literal occurrences of a needle in a file. */
function countIn(relPath, needle) {
  return h.readRepo(relPath).split(needle).length - 1;
}

/** Exact, non-empty source span between two literal markers (end exclusive). */
function between(src, startMarker, endMarker) {
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + ">");
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1, "missing end marker <" + endMarker + ">");
  var span = src.slice(start, end);
  h.assert(span.length > 0, "empty source span between <" + startMarker + "> and <" + endMarker + ">");
  return span;
}

/** Literal substring assertion on an already-read source span. */
function spanHas(span, needle, msg) {
  h.assert(span.indexOf(needle) !== -1,
    (msg || "span must contain") + " <" + needle + "> :: " + span.slice(0, 200));
}

/** Assert `before` occurs strictly earlier than `after` inside a source span. */
function ordered(span, before, after, msg) {
  var b = span.indexOf(before);
  var a = span.indexOf(after);
  h.assert(b !== -1 && a !== -1 && b < a,
    (msg || "ordering regressed") + " :: <" + before + "> before <" + after +
    "> in <" + span.slice(0, 160) + ">");
}

// (a) Body-hosted live region, window-scoped id, getElementById idempotency,
//     retained polite status semantics.
h.staticTest("#2435 sheet a11y live region is document.body-hosted with a window-scoped id, getElementById idempotency and role=status/aria-live=polite/aria-atomic=true retained", function () {
  var a11y = h.readRepo(A11Y);

  // Non-vacuity: the initA11y host and the live-region id expression really ship.
  contains(A11Y, "VirtualGrid.initA11y = function () {", "initA11y is shipped");
  contains(A11Y, 'var liveId = "gb-sheet-a11y-live-" + (this.windowId || "main");',
    "window-scoped live-region id expression is shipped");

  // Scope tightly to the live-region setup so an unrelated root appendChild
  // (the a11yRowGroup) cannot satisfy or invalidate the hosting assertions.
  var live = between(a11y,
    'var liveId = "gb-sheet-a11y-live-" + (this.windowId || "main");',
    'this.a11yRowGroup = document.createElement("div");');

  // Window-scoped id: keyed on windowId with a stable "main" fallback.
  spanHas(live, '"gb-sheet-a11y-live-" + (this.windowId || "main")',
    "live-region id is scoped per window");
  spanHas(live, "this.a11yLive.id = liveId;", "computed id is assigned to the node");

  // Idempotency: look up an existing node by id and reuse it instead of
  // unconditionally creating a second one.
  spanHas(live, "var existingLive = document.getElementById(liveId);",
    "existing live region is looked up by window-scoped id");
  spanHas(live, "if (existingLive) {", "reuse branch guards on an existing node");
  spanHas(live, "this.a11yLive = existingLive;", "existing node is repointed/reused");
  ordered(live, "document.getElementById(liveId)", "document.body.appendChild(this.a11yLive);",
    "idempotency lookup must precede any body append");

  // Retained polite status semantics on the live node.
  spanHas(live, 'this.a11yLive.setAttribute("role", "status");', "role=status retained");
  spanHas(live, 'this.a11yLive.setAttribute("aria-live", "polite");', "aria-live=polite retained");
  spanHas(live, 'this.a11yLive.setAttribute("aria-atomic", "true");', "aria-atomic=true retained");

  // Hosting: appended to document.body, NEVER to the rebuilt grid root.
  spanHas(live, "document.body.appendChild(this.a11yLive);",
    "live region is hosted on document.body");
  h.assert(live.indexOf("this.root.appendChild(this.a11yLive)") === -1,
    "live region must NOT be hosted on this.root (detaches on root rebuild)");
});

// (b) Defensive re-host on a detached node before writing; textContent only.
h.staticTest("#2435 announce() re-hosts a detached live region on document.body before writing and uses textContent only", function () {
  var a11y = h.readRepo(A11Y);

  // Non-vacuity: the announce prototype method ships.
  contains(A11Y, "VirtualGrid.announce = function (text, immediate) {",
    "announce prototype method is shipped");

  var announce = between(a11y,
    "VirtualGrid.announce = function (text, immediate) {",
    "VirtualGrid.cellLabel = function (");

  spanHas(announce, "if (!text || !this.a11yLive) return;", "empty text / missing node is guarded");
  spanHas(announce, "var live = this.a11yLive;", "live node is captured locally");

  // A stale root change can detach the region; re-host it on the connected
  // body before the write so the message actually lands in role=status.
  var rehost = "if (!live.isConnected && document.body) document.body.appendChild(live);";
  spanHas(announce, rehost, "detached node is re-hosted on document.body");
  ordered(announce, rehost, "live.textContent = text + suffix;",
    "defensive re-host must run before writing text");

  // textContent only — no innerHTML (which would drop the live semantics).
  spanHas(announce, "live.textContent === text ?",
    "repeated-message alternation reads textContent");
  spanHas(announce, "live.textContent = text + suffix;", "message is written via textContent");
  h.assert(announce.indexOf("innerHTML") === -1,
    "announce must write textContent only (no innerHTML)");

  // Both the immediate and debounced write paths remain.
  spanHas(announce, "if (immediate) write();", "immediate write path retained");
  spanHas(announce, "this.a11yAnnounceTimer = setTimeout(write, A11Y_DEBOUNCE_MS);",
    "debounced write path retained");
});

// (c) Repeated init cannot duplicate the live node; grid.announce() API intact.
h.staticTest("#2435 repeated grid init keeps a single body live region and the grid.announce() API is unchanged", function () {
  var a11y = h.readRepo(A11Y);
  var render = h.readRepo(RENDER);

  // Non-vacuity: init() rebuilds the root and calls initA11y; initA11y guards.
  contains(RENDER, "VirtualGrid.init = function (host) {", "grid init is shipped");
  contains(A11Y, "VirtualGrid.initA11y = function () {", "initA11y is shipped");
  contains(A11Y, "if (!this.root || this.a11yGridId) return;",
    "initA11y early-return guard is shipped");

  var init = between(render,
    "VirtualGrid.init = function (host) {",
    "VirtualGrid.renderHeaders = function () {");
  spanHas(init, 'this.root = document.createElement("div");',
    "init rebuilds the grid root on every (re-)init");
  spanHas(init, 'if (typeof this.initA11y === "function") this.initA11y();',
    "init invokes initA11y on the rebuilt grid");
  ordered(init, 'this.root = document.createElement("div");',
    'if (typeof this.initA11y === "function") this.initA11y();',
    "root must be rebuilt before a11y is initialized");

  // A fresh grid object for the same window must reuse the existing body node
  // rather than create a duplicate — the getElementById guard is that reuse.
  var live = between(a11y,
    'var liveId = "gb-sheet-a11y-live-" + (this.windowId || "main");',
    'this.a11yRowGroup = document.createElement("div");');
  spanHas(live, "var existingLive = document.getElementById(liveId);",
    "repeated init reuses the existing node by window-scoped id");
  spanHas(live, "if (existingLive) {", "reuse branch prevents a duplicate node");

  // Idempotency: exactly one create + one body append, both inside the else
  // branch of the getElementById guard — a reverted/duplicated guard FAILs.
  h.assertEqual(countIn(A11Y, 'this.a11yLive = document.createElement("div");'), 1,
    "live node is created exactly once (reuse branch otherwise)");
  h.assertEqual(countIn(A11Y, "document.body.appendChild(this.a11yLive);"), 1,
    "live node is appended to body exactly once");

  // The grid.announce() surface is unchanged: prototype method still declared,
  // still copied onto the per-window GridProto, and still called by the grid.
  contains(A11Y, "VirtualGrid.announce = function (text, immediate) {",
    "grid.announce(text, immediate) signature is unchanged");
  contains(A11Y, "D.GridProto[k] = VirtualGrid[k];",
    "announce is synced onto GridProto (per-window grids) in a11y module");
  contains(RENDER, "D.GridProto[_k3] = VirtualGrid[_k3];",
    "announce is synced onto GridProto (per-window grids) in render module");
  h.assert(countIn(A11Y, "this.announce(") >= 2,
    "grid.announce() call sites are retained");
});

h.note("FOLLOW-UP #2435: live region is body-hosted, idempotent by window-scoped id and " +
  "defensively re-hosted in announce(); these are source assertions only — no browser/server/cargo run.");
