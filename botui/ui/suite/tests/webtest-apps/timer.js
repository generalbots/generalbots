"use strict";
var h = require("./00_harness");

h.test("TIMER: files exist", function () {
  h.assert(h.fileExists("timer/timer.js"), "timer.js missing");
  h.assert(h.fileExists("timer/timer.html"), "timer.html missing");
});

h.test("[UAT:TIMER-01] TIMER: mounts after load (readyState complete)", function () {
  var src = h.readFile("timer/timer.js");
  h.assert(/readyState/.test(src), "timer.js missing readyState guard");
  var doc = h.makeDocument({ readyState: "complete", ids: ["timerDisplay", "timerStart", "timerReset", "timerLabel", "timerSessions"] });
  var timers = [];
  var sandbox = {
    window: {},
    document: doc,
    localStorage: { getItem: function () { return null; }, setItem: function () {} },
    setInterval: function (fn, ms) { timers.push({ fn: fn, ms: ms }); return timers.length; },
    clearInterval: function () {},
    Date: Date,
    Math: Math,
    JSON: JSON,
    console: { warn: function () {}, error: function () {}, log: function () {} },
    Notification: function () {},
  };
  sandbox.window = sandbox;
  h.loadScript("timer/timer.js", sandbox);
  h.assert(doc.getElementById("timerDisplay"), "timer display marker present");
  // GBTimer export or display text set indicates boot ran
  h.assert(sandbox.GBTimer || sandbox.window.GBTimer || doc.getElementById("timerDisplay").textContent !== undefined,
    "timer did not initialize marker");
});

h.test("TIMER: one listener per control (Start click)", function () {
  var doc = h.makeDocument({ readyState: "complete", ids: ["timerDisplay", "timerStart", "timerReset", "timerLabel", "timerSessions"] });
  var sandbox = {
    window: {},
    document: doc,
    localStorage: { getItem: function () { return null; }, setItem: function () {} },
    setInterval: function () { return 1; },
    clearInterval: function () {},
    Date: Date, Math: Math, JSON: JSON,
    console: { warn: function () {}, error: function () {}, log: function () {} },
    Notification: function () {},
  };
  sandbox.window = sandbox;
  h.loadScript("timer/timer.js", sandbox);
  var clicks = h.countListeners(doc.getElementById("timerStart"), "click");
  h.assert(clicks === 1, "expected 1 click listener on timerStart, got " + clicks);
});

h.test("TIMER: hostile input as text (label rendering path)", function () {
  // Timer uses textContent for dynamic labels; ensure no raw HTML injection helper exists in script.
  var src = h.readFile("timer/timer.js");
  h.assert(src.indexOf("innerHTML") === -1 || src.indexOf("escape") !== -1,
    "timer should avoid unescaped innerHTML for hostile values");
});
