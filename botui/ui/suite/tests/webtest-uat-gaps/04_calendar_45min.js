/**
 * [UAT:CALENDAR-01] "Marque uma reunião com o time amanhã às 15h por 45 minutos"
 * Steps exercised: (2) confirming saves exactly one calendar event;
 * (3) the event keeps its 45-minute duration (not the default hour).
 * The chat confirmation (step 1) is LLM-driven and stays out of this offline test.
 * Exercises the real event-form submit handler in calendar/modules/calendar-more.js.
 */
"use strict";

var env = require("./00_env");
var h = env.h;

function makeCalendar() {
  var listeners = {};
  var registry = {};

  function el(id) {
    var own = {};
    return {
      id: id || "",
      tagName: "DIV",
      className: "",
      textContent: "",
      innerHTML: "",
      value: "",
      checked: false,
      hidden: false,
      style: {},
      dataset: {},
      children: [],
      classList: {
        _set: {},
        add: function (c) { this._set[c] = true; },
        remove: function (c) { delete this._set[c]; },
        contains: function (c) { return !!this._set[c]; },
        toggle: function (c, force) {
          if (force === undefined) {
            if (this._set[c]) delete this._set[c]; else this._set[c] = true;
          } else if (force) this._set[c] = true; else delete this._set[c];
        },
      },
      __listeners: own,
      addEventListener: function (type, fn) {
        (own[type] = own[type] || []).push(fn);
      },
      removeEventListener: function () {},
      dispatchEvent: function (evt) {
        (own[evt.type] || []).slice().forEach(function (fn) { fn(evt); });
      },
      querySelector: function () { return null; },
      querySelectorAll: function () { return []; },
      setAttribute: function () {},
      getAttribute: function () { return null; },
      closest: function () { return null; },
      focus: function () {},
      reset: function () { this.__resets = (this.__resets || 0) + 1; },
      click: function () {
        this.dispatchEvent({ type: "click", target: this, preventDefault: function () {} });
      },
      appendChild: function (child) { this.children.push(child); return child; },
      getBoundingClientRect: function () {
        return { top: 0, left: 0, width: 100, height: 40 };
      },
    };
  }

  function get(id) {
    if (!registry[id]) registry[id] = el(id);
    return registry[id];
  }

  var doc = {
    readyState: "complete",
    title: "",
    documentElement: el("html"),
    body: el("body"),
    __listeners: listeners,
    getElementById: get,
    querySelector: function () { return null; },
    querySelectorAll: function () { return []; },
    createElement: function (tag) { var node = el(""); node.tagName = tag.toUpperCase(); return node; },
    addEventListener: function (type, fn) {
      (listeners[type] = listeners[type] || []).push(fn);
    },
  };

  var form = get("event-form");
  var fields = {
    'input[name="title"]': { value: "Sync com o time" },
    'input[name="start"]': { value: "2026-11-25T15:00" },
    'input[name="end"]': { value: "2026-11-25T15:45" },
    'input[name="all_day"]': { value: "", checked: false },
    'input[name="location"]': { value: "Sala 1" },
    'textarea[name="description"]': { value: "Weekly sync" },
  };
  form.tagName = "FORM";
  form.querySelector = function (sel) { return fields[sel] || null; };

  var calls = [];
  function fetchStub(url, opts) {
    calls.push({ url: url, opts: opts || {} });
    var payload = /\/conflicts$/.test(url)
      ? { has_conflicts: false }
      : (opts && opts.method === "POST" && /\/events$/.test(url)
        ? { id: "evt-1" }
        : []);
    return Promise.resolve({
      ok: true,
      json: function () { return Promise.resolve(payload); },
    });
  }

  var sandbox = env.baseSandbox(doc, env.makeStore(), {
    fetch: fetchStub,
    getComputedStyle: function () { return { display: "block" }; },
    GBModal: { confirm: function (msg, yes) { if (yes) yes(); } },
  });
  env.loadAll(sandbox, ["calendar/calendar.js", "calendar/modules/calendar-more.js"]);
  return { form: form, doc: doc, calls: calls, sandbox: sandbox };
}

function flush() {
  return new Promise(function (resolve) { setTimeout(resolve, 0); });
}

h.test("[UAT:CALENDAR-01] save one 45-minute event and refresh the view", function () {
  var ctx = makeCalendar();
  h.assert(
    typeof ctx.sandbox.CalendarModule === "object" &&
      typeof ctx.sandbox.CalendarModule.loadEvents === "function",
    "calendar module did not initialise",
  );

  ctx.form.dispatchEvent({
    type: "submit",
    target: ctx.form,
    preventDefault: function () {},
  });

  return flush().then(function () {
    var posts = ctx.calls.filter(function (c) {
      return c.opts.method === "POST" && /\/events$/.test(c.url);
    });
    h.assert(posts.length === 1, "expected one event POST, got " + posts.length);

    var body = JSON.parse(posts[0].opts.body);
    var start = new Date("2026-11-25T15:00").toISOString();
    var end = new Date("2026-11-25T15:45").toISOString();
    h.assert(body.start_time === start, "wrong start: " + body.start_time);
    h.assert(body.end_time === end, "wrong end: " + body.end_time);
    h.assert(
      (Date.parse(body.end_time) - Date.parse(body.start_time)) / 60000 === 45,
      "duration is not 45 minutes: " + body.start_time + " -> " + body.end_time,
    );

    var lists = ctx.calls.filter(function (c) {
      return c.opts.method !== "POST" && /\/calendar\/events/.test(c.url);
    });
    h.assert(lists.length >= 2, "calendar view was not refreshed after save");
    h.assert(
      ctx.doc.getElementById("event-modal").classList.contains("hidden"),
      "event modal stayed open after save",
    );
  });
});
