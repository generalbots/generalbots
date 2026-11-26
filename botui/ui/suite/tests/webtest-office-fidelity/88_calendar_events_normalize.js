/**
 * #2941 — opening Calendar logged
 *   "Failed to load CS.events: TypeError: CS.events.forEach is not a function"
 *   at calendar/modules/calendar-draw.js:56 -> drawEvents -> drawEventsWeek.
 *
 * The events endpoint can answer with a non-array body (401/500 error object).
 * calendar.js used `CS.events = data || []`, which happily stored that object,
 * and the painters then called .filter/.forEach/.some/.find on it. This guard
 * pins two layers of the fix:
 *   1. the fetch boundary normalizes with Array.isArray and surfaces the
 *      Calendar app's existing localized error state (no new strings);
 *   2. the draw layer routes every iteration through an Array.isArray gate,
 *      so even a direct CS.events mutation cannot throw, and the successful
 *      render path is unchanged.
 *
 * GB_CAL2941_PROBE_ROOT points at a /tmp tree holding mutated copies of
 * calendar.js / calendar-draw.js (matched by basename). Removing either
 * Array.isArray gate must make this guard FAIL. No server, browser, cargo or
 * commit is involved; the driver owns the live UAT and issue closure.
 */
"use strict";

var h = require("./00_harness");
var fs = require("fs");
var path = require("path");

var CAL = "botui/ui/suite/calendar/calendar.js";
var DRAW = "botui/ui/suite/calendar/modules/calendar-draw.js";
var HTML = "botui/ui/suite/calendar/calendar.html";
var RUNNER = "botui/ui/suite/tests/webtest-office-fidelity.js";
var GUARD = "botui/ui/suite/tests/webtest-office-fidelity/88_calendar_events_normalize.js";
var PROBE_ROOT = process.env.GB_CAL2941_PROBE_ROOT || "";

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

// --------------------------------------------------------------------------
// (a) The fetch boundary normalizes before drawing.
// --------------------------------------------------------------------------
h.staticTest("#2941 the events fetch normalizes non-array payloads before drawEvents", function () {
  var cal = codeOnly(read(CAL));
  h.assert(lines(cal) > 350, "non-vacuity: calendar.js is no longer a shipped module");
  contains(cal, "var valid = Array.isArray(data);", "payload shape is checked with Array.isArray");
  contains(cal, "CS.events = valid ? data : [];", "a non-array payload becomes an empty list");
  absent(cal, "CS.events = data || [];", "the original object-storing assignment is gone");
  contains(cal, "window.GBAppLifecycle.setState(valid ? null : 'error');",
    "the shipped localized state is selected from the payload shape");
  contains(cal, "CS.eventsLoadError = !valid;", "the error state is tracked for recovery");
  contains(cal, "Failed to load CS.events:", "the pinned error log literal is preserved");
  absent(cal, "throw ", "the fix never throws at the boundary");

  var load = span(cal, "function loadEvents() {", "\n    function drawEvents() {");
  var atValid = load.indexOf("var valid = Array.isArray(data);");
  var atAssign = load.indexOf("CS.events = valid ? data : [];");
  var atDraw = load.indexOf("drawEvents();", atAssign);
  h.assert(atValid !== -1 && atAssign > atValid && atDraw > atAssign,
    "order must be check -> normalize -> draw (check=" + atValid +
    " normalize=" + atAssign + " draw=" + atDraw + ")");
});

// --------------------------------------------------------------------------
// (b) The error/empty state is the existing localized one — no new strings.
// --------------------------------------------------------------------------
h.staticTest("#2941 the error state reuses the shipped localized Calendar state", function () {
  var html = read(HTML);
  contains(html, 'class="gb-state gb-state-error"', "the shipped error state element remains");
  contains(html, 'data-i18n="calendar-unavailable"', "its message uses an existing i18n key");
  contains(html, 'role="alert" aria-live="assertive"', "the error state stays announced");
  var stateKey = h.readRepo("botlib/locales/en/ui-productivity.ftl");
  h.assert(/^calendar-unavailable\s*=/m.test(stateKey),
    "calendar-unavailable must be a real catalog key");
  absent(html, 'data-gb-state="error"', "the guard does not hardcode a state attribute in markup");
  var cal = codeOnly(read(CAL));
  absent(cal, "gbT(", "no new user-facing string is introduced in the loader");
});

// --------------------------------------------------------------------------
// (c) The draw layer gates every iteration.
// --------------------------------------------------------------------------
h.staticTest("#2941 every draw-layer iteration is behind one Array.isArray gate", function () {
  var draw = codeOnly(read(DRAW));
  h.assert(lines(draw) > 150, "non-vacuity: calendar-draw.js is no longer a shipped module");
  contains(draw, "function events() {", "one events accessor owns normalization");
  contains(draw, "if (!Array.isArray(CS.events)) CS.events = [];", "gate normalizes a non-array");
  contains(draw, "if (!CS) return [];", "the accessor is safe before the state exists");
  contains(draw, "return CS.events;", "a valid array is returned unchanged");
  h.assert(!/\bCS\.events\s*\.\s*(?:forEach|filter|map|some|find)\s*\(/.test(draw),
    "no raw CS.events iterator survives");
  ["events().filter(", "events().forEach(", "events().some(", "events().find("].forEach(function (needle) {
    h.assert(count(draw, needle) >= 1, "iteration must go through the gate: <" + needle + ">");
  });
  var dispatcher = span(draw, "function drawEvents() {", "\n    function checkConflict");
  h.assert(dispatcher.indexOf("events()") !== -1 &&
    dispatcher.indexOf("events()") < dispatcher.indexOf("CS.currentView"),
    "drawEvents normalizes before dispatching to a painter");
  contains(draw, "__drawEvents: drawEvents", "the successful-render painter wiring is preserved");
  contains(draw, "showEventPopup: showEventPopup", "the popup wiring is preserved");
});

// --------------------------------------------------------------------------
// (d) Shipped wiring, size ceilings and guard registration.
// --------------------------------------------------------------------------
h.staticTest("#2941 script order, size ceilings and guard 88 registration", function () {
  var html = read(HTML);
  var atCal = html.indexOf("/suite/calendar/calendar.js");
  var atDraw = html.indexOf("/suite/calendar/modules/calendar-draw.js");
  var atMore = html.indexOf("/suite/calendar/modules/calendar-more.js");
  h.assert(atCal !== -1 && atDraw > atCal && atMore > atDraw,
    "calendar.html must load the loader before the gated draw module");
  [CAL, DRAW].forEach(function (rel) {
    var src = read(rel);
    h.assert(lines(src) < 450, rel + " must stay under 450 lines");
    h.assert(!/\bhttps?:\/\//i.test(src), rel + " must not reference an external CDN");
  });
  var guardSrc = h.readRepo(GUARD);
  h.assert(lines(guardSrc) < 450, "guard must stay under 450 lines");
  contains(h.readRepo(RUNNER), 'require("./webtest-office-fidelity/88_calendar_events_normalize");',
    "guard 88 must be registered in the fidelity runner");
  var hooks = codeOnly(guardSrc).match(/= process\.env\.GB_CAL2941_PROBE_ROOT/g) || [];
  h.assertEqual(hooks.length, 1, "the guard exposes exactly one probe-root hook");
});

// --------------------------------------------------------------------------
// Runtime proof: both real modules in a stubbed vm, synchronous thenables.
// --------------------------------------------------------------------------
function resolved(value) {
  return {
    then: function (fn) {
      // Flatten a thenable returned by fn so .then(r => r.json()) hands the
      // actual body — not a nested thenable — to the next handler.
      if (value && typeof value.then === 'function') return value.then(fn);
      var next = fn(value);
      return next && typeof next.then === 'function' ? next : resolved(next);
    },
    catch: function () { return this; },
  };
}
function makeEl(id) {
  return {
    id: id || "",
    tagName: "DIV",
    className: "",
    innerHTML: "",
    style: {},
    dataset: {},
    children: [],
    classList: { add: function () {}, remove: function () {}, contains: function () { return false; } },
    appendChild: function (child) { this.children.push(child); return child; },
    querySelector: function () { return null; },
    querySelectorAll: function () { return []; },
    addEventListener: function () {},
    setAttribute: function () {},
    getAttribute: function () { return null; },
  };
}
/**
 * @param payloads queue of bodies for successive r.json() resolves
 * @param dayHost when true, expose a #day-events host so a valid event chips
 */
function makeCalendar(payloads, dayHost) {
  var states = [];
  var errors = [];
  var queue = payloads.slice();
  var sandbox = {
    console: { error: function () { for (var i = 0; i < arguments.length; i++) errors.push(String(arguments[i])); }, warn: function () {} },
    Array: Array, Object: Object, String: String, Date: Date, Math: Math, JSON: JSON, Number: Number,
    GBAppLifecycle: {
      begin: function () {},
      setState: function (state) { states.push(state); },
    },
    fetch: function () {
      var payload = queue.length > 1 ? queue.shift() : queue[0];
      return resolved({ json: function () { return resolved(payload); } });
    },
    document: {
      readyState: "complete",
      getElementById: function (id) { return id === "day-events" && dayHost ? sandbox.__dayHost : null; },
      querySelector: function () { return null; },
      querySelectorAll: function () { return []; },
      createElement: function () { return makeEl(); },
      addEventListener: function () {},
    },
  };
  sandbox.window = sandbox;
  sandbox.__dayHost = makeEl("day-events");
  h.vm.createContext(sandbox);
  h.vm.runInContext(read(CAL), sandbox, { filename: CAL });
  h.vm.runInContext(read(DRAW), sandbox, { filename: DRAW });
  return { sandbox: sandbox, states: states, errors: errors };
}

h.test("#2941 a non-array events body renders empty and sets the error state without throwing", function () {
  var t = makeCalendar([{ error: "unauthorized", detail: "401" }], false);
  t.sandbox.CalendarModule.loadEvents();
  h.assert(Array.isArray(t.sandbox.__calCS.events), "events must be normalized to an array");
  h.assertEqual(t.sandbox.__calCS.events.length, 0, "the empty grid is rendered");
  h.assertEqual(t.states.length, 1, "exactly one app state transition");
  h.assertEqual(t.states[0], "error", "the existing localized error state is selected");
  // Direct painter and helper calls must stay safe even if CS is mutated later.
  t.sandbox.__calCS.events = { error: "still bad" };
  t.sandbox.CalendarModule.__drawEvents();
  h.assertEqual(t.sandbox.__calCS.events.length, 0, "the draw gate re-normalizes a bad payload");
  h.assert(t.sandbox.CalendarModule.__checkConflict(new Date(0), new Date(1)) === false,
    "conflict check no longer iterates a non-array");
});

h.test("#2941 a valid events array still renders chips and clears a prior error", function () {
  var day = new Date(2026, 10, 26, 9, 0, 0);
  var payload = [{ id: "evt-ok", title: "Standup", start_time: day.toISOString(),
    end_time: new Date(day.getTime() + 45 * 60000).toISOString() }];
  var t = makeCalendar([{ error: "unauthorized" }, payload], true);
  t.sandbox.CalendarModule.loadEvents(); // bad payload -> error state
  h.assertEqual(t.states[0], "error", "first load selected error");
  t.sandbox.__calCS.selectedDate = new Date(2026, 10, 26);
  t.sandbox.__calCS.currentView = "day";
  t.sandbox.CalendarModule.loadEvents(); // queued valid body -> recovery
  h.assertEqual(t.sandbox.__calCS.events.length, 1, "valid payload is stored unchanged");
  h.assertEqual(t.sandbox.__dayHost.children.length, 1, "the successful event chip is still rendered");
  h.assert(/Standup/.test(t.sandbox.__dayHost.children[0].innerHTML), "chip keeps its title");
  h.assertEqual(t.states[t.states.length - 1], null, "recovery clears the error state");
});

h.note("#2941: offline static + stubbed-vm proof only — no browser/server/cargo, no new i18n key. Driver owns the live Chrome UAT and issue closure.");
