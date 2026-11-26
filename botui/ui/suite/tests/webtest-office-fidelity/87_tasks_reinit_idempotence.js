/**
 * #2940 — Tasks app reopened in the same tab threw
 *   SyntaxError: Identifier 'tasksWindow' has already been declared
 *   ReferenceError: setupEventListeners is not defined
 *
 * tasks.html is injected through HTMX, so its classic scripts are evaluated
 * again for every window instance. This guard pins the three properties that
 * make those re-evaluations safe:
 *   1. no top-level const/let redeclaration — shared state lives on the
 *      guarded window.TasksRuntime namespace and is reached through
 *      getTasksWindow();
 *   2. setupEventListeners is defined and published by the same script that
 *      owns it, and the cross-module caller waits for that publication;
 *   3. every listener registration is guarded by tasksBindOnce so a second
 *      initialization neither throws nor stacks duplicate handlers.
 *
 * Set GB_TASKS_REINIT_PROBE_ROOT to a /tmp directory holding mutated copies of
 * tasks-scoped-state.js / tasks-scoped-progress.js / tasks.html (matched by
 * basename) to prove each assertion fails when the fix is reverted. No server,
 * browser, cargo or commit is involved; the driver owns issue closure.
 */
"use strict";

var h = require("./00_harness");
var fs = require("fs");
var path = require("path");

var STATE = "botui/ui/suite/tasks/tasks-scoped-state.js";
var PROGRESS = "botui/ui/suite/tasks/tasks-scoped-progress.js";
var HTML = "botui/ui/suite/tasks/tasks.html";
var RUNNER = "botui/ui/suite/tests/webtest-office-fidelity.js";
var PROBE_ROOT = process.env.GB_TASKS_REINIT_PROBE_ROOT || "";

var BIND_KEYS = [
  "boot-afterSwap", "detail-afterSwap", "intent-beforeRequest",
  "intent-afterRequest", "intent-enter", "intent-input",
  "filter-click", "search-input", "nav-click", "log-toggle", "keydown",
];

function read(rel) {
  var candidate = PROBE_ROOT ? path.join(PROBE_ROOT, path.basename(rel)) : "";
  if (candidate && fs.existsSync(candidate)) return fs.readFileSync(candidate, "utf8");
  return h.readRepo(rel);
}

/** Prose must never satisfy (or spoil) a code pin. */
function codeOnly(src) {
  return src.replace(/\/\*[\s\S]*?\*\//g, "")
    .split("\n").filter(function (line) { return line.trim().indexOf("//") !== 0; })
    .join("\n");
}

function contains(src, needle, msg) {
  h.assert(src.indexOf(needle) !== -1, (msg || "source must contain") + " <" + needle + ">");
}
function absent(src, needle, msg) {
  h.assert(src.indexOf(needle) === -1, (msg || "source must not contain") + " <" + needle + ">");
}
function count(src, needle) { return src.split(needle).length - 1; }
function span(src, startMarker, endMarker) {
  var start = src.indexOf(startMarker);
  h.assert(start !== -1, "missing start marker <" + startMarker + ">");
  var end = src.indexOf(endMarker, start + startMarker.length);
  h.assert(end !== -1 && end > start, "missing/empty span after <" + startMarker + ">");
  return src.slice(start, end);
}
function lines(src) { return src.split("\n").length; }

h.staticTest("#2940 no top-level const/let survives a Tasks script re-evaluation", function () {
  var stateRaw = read(STATE);
  var progressRaw = read(PROGRESS);
  var state = codeOnly(stateRaw);
  var progress = codeOnly(progressRaw);
  [state, progress].forEach(function (src, i) {
    var name = i === 0 ? STATE : PROGRESS;
    h.assert(lines(src) > 300, "non-vacuity: " + name + " is no longer a shipped module");
    h.assert(!/^(?:const|let)\s/m.test(src), name + " still has a top-level const/let");
    absent(src, "const tasksWindow", name + " redeclares tasksWindow");
    absent(src, "let tasksWindow", name + " redeclares tasksWindow");
  });
  contains(state, "function getTasksWindow()", "the window is resolved per call");
  contains(progress, "getTasksWindow()", "the listener module resolves the window the same way");
  contains(state, "window.TasksRuntime", "shared state is namespaced on window");
});

h.staticTest("#2940 one guarded initializer owns window state and listener uniqueness", function () {
  var raw = read(STATE);
  var state = codeOnly(raw);
  h.assert(count(state, "window.TasksRuntime = {") === 1,
    "TasksRuntime must be created by exactly one guarded initializer");
  contains(state, "if (!window.TasksRuntime) {", "namespace creation is guarded");
  contains(state, "new WeakMap()", "per-element bindings use a leak-free WeakMap");
  var bind = span(raw, "function tasksBindOnce(el, key)", "\nfunction parseTaskStatsFragment");
  h.assert(bind.length > 250, "non-vacuity: tasksBindOnce body is substantial, got " + bind.length);
  contains(bind, "var seen = window.TasksRuntime.bound.get(el);", "bindings are looked up per element");
  contains(bind, "if (seen[key]) return false;", "a bound purpose is never bound twice");
  contains(bind, "seen[key] = true;", "the purpose is recorded before binding");
  contains(bind, "return true;", "first binding is allowed");
  h.assert(count(state, "function scheduleTasksInit()") === 1,
    "there must be exactly one guarded scheduler");
  var schedule = span(state, "function scheduleTasksInit()", "\nsetTimeout(scheduleTasksInit, 100)");
  contains(schedule, "tasksBindOnce(w, \"boot-afterSwap\")", "the boot swap listener is bound once per window");
  contains(schedule, "typeof rt.setupListeners !== \"function\"", "caller waits for the listener module");
  contains(schedule, "rt.initRetries++ < 40", "readiness retries are bounded");
  contains(schedule, "setTimeout(scheduleTasksInit, 50)", "readiness is retried, not skipped");
  contains(schedule, "initTasksApp();", "initialization runs once listeners are ready");
  contains(state, "setTimeout(scheduleTasksInit, 100);", "boot still starts from the original 100ms timer");
});

h.staticTest("#2940 setupEventListeners is module-local, published, then called via the namespace", function () {
  var stateRaw = read(STATE);
  var state = codeOnly(stateRaw);
  var progress = codeOnly(read(PROGRESS));
  contains(progress, "function setupEventListeners()", "the listener setup is defined in its own module");
  contains(progress, "window.TasksRuntime.setupListeners = setupEventListeners;",
    "the owner publishes setup before any caller can run");
  var definedAt = progress.indexOf("function setupEventListeners()");
  var publishedAt = progress.indexOf("window.TasksRuntime.setupListeners = setupEventListeners;");
  h.assert(definedAt !== -1 && publishedAt > definedAt,
    "publication must follow the definition in the same script");
  var init = span(stateRaw, "function initTasksApp()", "\n// Helper to find pending manifest");
  h.assert(init.length > 300, "non-vacuity: initTasksApp body must be substantial, got " + init.length);
  contains(init, "typeof window.TasksRuntime.setupListeners !== \"function\"",
    "the cross-module call is gated even for direct init calls");
  contains(init, "window.TasksRuntime.setupListeners();", "the published setup is invoked");
  absent(init, "setupEventListeners(", "no bare cross-module identifier call");
  absent(state, "setupEventListeners(", "the state module never calls the bare identifier");
  var atGate = init.indexOf("typeof window.TasksRuntime.setupListeners");
  var atCall = init.indexOf("window.TasksRuntime.setupListeners();");
  h.assert(atGate !== -1 && atCall > atGate, "readiness gate must precede the first call");
});

h.staticTest("#2940 every re-init listener registration is guarded by tasksBindOnce", function () {
  var stateRaw = read(STATE);
  var progressRaw = read(PROGRESS);
  var state = codeOnly(stateRaw);
  var progress = codeOnly(progressRaw);
  BIND_KEYS.forEach(function (key) {
    var inProgress = key === "filter-click" || key === "search-input" || key === "nav-click" ||
      key === "log-toggle" || key === "keydown";
    var src = inProgress ? progress : state;
    var re = new RegExp("tasksBindOnce\\([^\"\\n]*\"" + key + "\"");
    h.assert(re.test(src), "listener key " + key + " must be bound once in its owning module");
  });
  h.assert(count(state, "tasksBindOnce(") >= 6, "state module must guard all of its listeners");
  h.assert(count(progress, "tasksBindOnce(") >= 5, "listener module must guard all of its listeners");
  var listeners = span(progressRaw, "function setupEventListeners()", "\n// #2940");
  contains(listeners, "var tasksWindow = getTasksWindow();", "setup resolves the current window");
  contains(listeners, "if (!tasksWindow) return;", "setup is a no-op before the window exists");
  var keyboard = span(progressRaw, "function setupKeyboardShortcuts()", "\n// TASK SELECTION & FILTERING");
  contains(keyboard, "if (tasksWindow && tasksBindOnce(tasksWindow, \"keydown\")) {",
    "keyboard shortcut registration is guarded");
});

h.staticTest("#2940 shipped wiring, size ceilings and guard registration", function () {
  var html = read(HTML);
  var stateAt = html.indexOf("/suite/tasks/tasks-scoped-state.js");
  var progressAt = html.indexOf("/suite/tasks/tasks-scoped-progress.js");
  h.assert(stateAt !== -1 && progressAt !== -1 && stateAt < progressAt,
    "tasks.html must load state before the listener module");
  h.assert(count(html, "/suite/tasks/tasks-scoped-state.js") === 1,
    "tasks.html must not inject the state script twice");
  [STATE, PROGRESS].forEach(function (rel) {
    var src = read(rel);
    h.assert(lines(src) < 450, rel + " must stay under 450 lines");
    h.assert(lines(src) > 300, rel + " must remain a shipped module");
    h.assert(!/\bhttps?:\/\//i.test(src), rel + " must not reference an external CDN");
  });
  contains(h.readRepo(RUNNER), 'require("./webtest-office-fidelity/87_tasks_reinit_idempotence")',
    "this guard must be required by the fidelity runner");
});
