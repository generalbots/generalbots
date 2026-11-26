/**
 * #2933 — deterministic static guard for the remaining boot idle violation.
 *
 * Chrome's load UAT (load ~15) still reported one 160 ms
 * `requestIdleCallback` handler and a 158 ms forced reflow, both attributed to
 * `suite/js/window-manager/03_window_chrome.js`. The old single `drainChromeWork`
 * callback ran EVERY queued chrome job (taskbar dock construction + body
 * reveal class flips) in one task; the reveals invalidated layout and the
 * separate post-reveal clamp callbacks then forced a synchronous reflow.
 *
 * The fix keeps the coalesced queue (guards 68/76 literals are untouched) and
 * splits the drain:
 *   (1) the FIRST idle task only flips a flag/class — no job, no DOM, no
 *       geometry;
 *   (2) every later task runs a bounded slice of at most CHROME_CHUNK jobs,
 *       re-scheduling itself until the queue is empty;
 *   (3) layout reads still live only in the separately scheduled clamp task,
 *       whose read phase precedes its write phase.
 *
 * Every case is a literal source assertion. NON-VACUITY:
 * `GB_WCHROME_PROBE_ROOT` may point at a /tmp directory holding a mutated copy
 * named `03_window_chrome.js`; only this guard reads from it, so a targeted
 * mutation (removed boot flag guard, unbounded chunk, injected layout read)
 * makes exactly the matching case FAIL while guards 68/76 keep reading the
 * shipped file. No server, browser, cargo, locale or issue edit is involved;
 * the driver owns the live Chrome <= 100 ms UAT and issue closure.
 */
"use strict";

var h = require("./00_harness");
var fs = require("fs");
var path = require("path");

var CHROME = "botui/ui/suite/js/window-manager/03_window_chrome.js";
var RUNNER = "botui/ui/suite/tests/webtest-office-fidelity.js";
var GUARD = "botui/ui/suite/tests/webtest-office-fidelity/91_chrome_idle_task_budget.js";
var PROBE_ROOT = process.env.GB_WCHROME_PROBE_ROOT || "";

var LAYOUT = ["getComputedStyle", "getBoundingClientRect", "offsetWidth",
  "offsetHeight", "offsetTop", "scrollTop", "scrollHeight", "clientWidth",
  "clientHeight", "innerText", "visualViewport", "innerWidth"];

function readChrome() {
  if (PROBE_ROOT) {
    var candidate = path.join(PROBE_ROOT, "03_window_chrome.js");
    if (fs.existsSync(candidate)) return fs.readFileSync(candidate, "utf8");
  }
  return h.readRepo(CHROME);
}
function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1, (msg || "source must contain") + " <" + needle + ">");
}
function absent(src, needle, msg) {
  h.assert(src.indexOf(needle) === -1,
    (msg || "source must not contain") + " <" + needle + ">");
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
function ordered(text, markers, msg) {
  var last = -1;
  markers.forEach(function (marker) {
    var at = text.indexOf(marker);
    h.assert(at !== -1, (msg || "span") + " must contain <" + marker + ">");
    h.assert(at > last, (msg || "span") + " order broken at <" + marker + ">");
    last = at;
  });
}
function noLayout(text, label) {
  LAYOUT.forEach(function (read) {
    absent(text, read, label + " must not read geometry: <" + read + ">");
  });
}

h.staticTest("#2933 the boot idle task flips a flag/class and runs no chrome job", function () {
  var src = readChrome();
  h.assert(src.length > 500, "non-vacuity: 03_window_chrome.js is the shipped module");
  var boot = span(src, "function drainChromeWork() {", "var jobs = chromeWork;");
  h.assert(boot.length > 120, "non-vacuity: boot task has a real body, got " + boot.length);

  contains(boot, "if (!chromeBootFlipped) {", "the first task is gated by a one-shot flag");
  contains(boot, "chromeBootFlipped = true;", "the flag is flipped in the boot task");
  contains(boot, "document.documentElement.classList.add(", "the cheap class flip survives");
  contains(boot, "return;", "the boot task returns before touching the queue");
  ordered(boot, ["if (!chromeBootFlipped) {", "chromeBootFlipped = true;",
    "document.documentElement.classList.add(", "return;"],
  "boot task order is gate -> flag -> class flip -> early return");
  h.assertEqual(countIn(boot, "classList.add("), 1,
    "the boot task performs exactly one class flip");

  // No construction, no job execution and no geometry in the boot task.
  ["jobs[", "forEach(", "splice(", "createElement", "appendChild", "innerHTML",
    "queueChromeWork(finish", "body.classList.add(\"gb-window-ready\")",
    "getTaskbarCenter", "getIconSvg"].forEach(function (call) {
    absent(boot, call, "boot task must not do chrome work: <" + call + ">");
  });
  noLayout(boot, "the boot idle task");

  // No user-facing literal is smuggled into the new task.
  absent(boot, "gbT(", "the boot task adds no localized user string");
  absent(boot, "data-i18n", "the boot task adds no raw i18n key");
});

h.staticTest("#2933 queued chrome work drains in bounded, coalesced idle slices", function () {
  var src = readChrome();
  var scheduler = span(src, "var chromeWork = [], chromeWorkQueued = false;", "WM.defineMethods(");
  contains(scheduler, "if (chromeWorkQueued) return;", "bursts coalesce into one scheduled task");
  contains(scheduler, "window.requestIdleCallback(drainChromeWork, { timeout: 800 });",
    "the pass runs on idle");
  contains(scheduler, "window.requestAnimationFrame(drainChromeWork);",
    "the pass keeps its rAF fallback");
  h.assertEqual(countIn(src, "window.requestIdleCallback(drainChromeWork, { timeout: 800 });"), 1,
    "there is exactly one chrome-drain scheduling site");

  var drain = span(src, "var jobs = chromeWork;", "function queueChromeWork(job) {");
  h.assert(drain.length > 200, "non-vacuity: bounded drain has a real body, got " + drain.length);
  contains(drain, "CHROME_CHUNK", "the slice is bounded by a named chunk size");
  contains(drain, "Math.min(", "the slice never exceeds the queue length");
  contains(drain, "chromeCursor", "the cursor advances across idle tasks");
  contains(drain, "for (;", "the bounded loop is explicit");
  contains(drain, "jobs[chromeCursor]()", "jobs are invoked from the captured slice");
  h.assertEqual(countIn(drain, "jobs[chromeCursor]()"), 1,
    "one job invocation site keeps the slice bounded");
  contains(drain, "scheduleChromeWork();", "remaining jobs coalesce onto the next task");
  contains(drain, "chromeWork = [];", "the queue resets when the slice finished it");
  contains(drain, "chromeCursor = 0;", "the cursor resets with the queue");

  var match = src.match(/CHROME_CHUNK\s*=\s*(\d+)/);
  h.assert(match, "CHROME_CHUNK must be a literal integer bound");
  var chunk = parseInt(match[1], 10);
  h.assert(chunk >= 1 && chunk <= 16,
    "the chunk must stay small enough for the <=100 ms idle budget, got " + chunk);
});

h.staticTest("#2933 no idle task reads layout then writes — geometry stays in its own batched clamp", function () {
  var src = readChrome();
  var boot = span(src, "function drainChromeWork() {", "var jobs = chromeWork;");
  var drain = span(src, "var jobs = chromeWork;", "function queueChromeWork(job) {");
  var queue = span(src, "function queueChromeWork(job) {", "WM.defineMethods(");

  // The three chrome idle tasks contain no geometry read at all, so none can
  // pair a layout read with a write. The boot task's single write is a class
  // flip on <html>, not a style/geometry write on a window.
  noLayout(boot, "the boot idle task");
  noLayout(drain, "the bounded chrome drain");
  noLayout(queue, "the chrome queue scheduler");
  absent(drain, ".style.", "the drain itself performs no inline style write");
  absent(drain, "classList.", "the drain itself performs no class write");

  // The only layout-reading idle task is the post-reveal clamp. It is still
  // scheduled on its own callback (guard 68 literal) and batches one read
  // ahead of its write phase (guard 76 literal).
  var reveal = span(src, "const reclampAfterReveal = () => {", "if (!links.length) {");
  contains(reveal, "window.requestIdleCallback(() => this._clampWindowIntoView(wid), { timeout: 800 })",
    "the clamp keeps its separate idle callback");
  contains(reveal, "window.setTimeout(() => this._clampWindowIntoView(wid), 1)",
    "the clamp keeps its timer fallback");

  var clamp = span(src, "_clampWindowIntoView(id) {", "_sanityButton(id) {");
  h.assertEqual(countIn(clamp, "getBoundingClientRect("), 1,
    "the clamp still takes exactly one geometry snapshot");
  ordered(clamp, ["// READ PHASE —", "const rect = win.getBoundingClientRect();",
    "// WRITE PHASE"], "the clamp read phase still precedes its write phase");
});

h.staticTest("#2933 guard 91 is registered, focused and mutation-probe ready", function () {
  var src = readChrome();
  h.assert(src.split("\n").length < 450,
    "03_window_chrome.js must stay under 450 lines, got " + src.split("\n").length);
  var guardSrc = h.readRepo(GUARD);
  h.assert(guardSrc.split("\n").length < 450, "guard must stay under 450 lines");
  contains(h.readRepo(RUNNER),
    'require("./webtest-office-fidelity/91_chrome_idle_task_budget");',
  "guard 91 must be registered in webtest-office-fidelity.js to run in CI");

  // Every guard 68/76 literal surrounding this work survives untouched.
  ["var chromeWork = [], chromeWorkQueued = false;",
    "if (chromeWorkQueued) return;",
    "if (remaining <= 0) queueChromeWork(finish);",
    "const timer = setTimeout(function () { queueChromeWork(finish); }, 3000);",
    "queueChromeWork(() => {",
    "// READ PHASE done",
    "body.classList.add(\"gb-window-ready\");",
    "reclampAfterReveal();",
    "_clampWindowIntoView(id) {"].forEach(function (literal) {
    contains(src, literal, "pinned guard 68/76 literal survived: <" + literal + ">");
  });
  var finish = span(src, "const finish = () => {", "const onOne = () => {");
  ordered(finish, ["body.classList.add(\"gb-window-ready\");", "reclampAfterReveal();"],
    "guard 68 reveal-before-clamp order survives");

  var hooks = guardSrc.match(/=\s*process\.env\.GB_WCHROME_PROBE_ROOT/g) || [];
  h.assertEqual(hooks.length, 1,
    "the guard exposes exactly one probe-root hook for mutation testing");
});

h.note("#2933: offline static proof only — boot flag flip, bounded coalesced idle slices, and layout reads isolated in the existing clamp task. GB_WCHROME_PROBE_ROOT=/tmp/... with a mutated 03_window_chrome.js makes the matching case FAIL. Live Chrome boot/unlock/deep-link <= 100 ms UAT and issue closure belong to the driver; no browser/server/cargo/locale change.");
