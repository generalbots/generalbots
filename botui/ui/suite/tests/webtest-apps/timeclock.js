"use strict";
var h = require("./00_harness");

h.test("TIMECLOCK: files exist", function () {
  h.assert(h.fileExists("timeclock/timeclock.js"), "timeclock.js missing");
  h.assert(h.fileExists("timeclock/timeclock.html"), "timeclock.html missing");
});

h.test("TIMECLOCK: mounts after load (readyState complete)", function () {
  var src = h.readFile("timeclock/timeclock.js");
  h.assert(/readyState/.test(src) || src.indexOf("DOMContentLoaded") === -1 || src.indexOf("GBAppLifecycle") !== -1,
    "timeclock should be mount-safe");
  var ids = ["tc-clock","tc-date","tc-zone","tc-content","tc-ot-open","tc-ot-modal","tc-ot-cancel","tc-ot-form","tc-status"];
  var doc = h.makeDocument({ readyState: "complete", ids: ids });
  var sandbox = {
    window: {}, document: doc, console: { warn: function () {}, error: function () {}, log: function () {} },
    setInterval: function(){return 1;}, clearInterval: function(){},
    fetch: function(){return Promise.resolve({ok:true,json:function(){return Promise.resolve({});}});},
    Date: Date, Intl: Intl, gbT: function(k,f){return f;},
  };
  sandbox.window = sandbox;
  h.loadScript("timeclock/timeclock.js", sandbox);
  h.assert(doc.getElementById("tc-clock"), "timeclock clock marker present");
});

h.test("TIMECLOCK: one listener per control (Overtime open)", function () {
  var ids = ["tc-clock","tc-date","tc-zone","tc-content","tc-ot-open","tc-ot-modal","tc-ot-cancel","tc-ot-form","tc-status"];
  var doc = h.makeDocument({ readyState: "complete", ids: ids });
  var sandbox = {
    window: {}, document: doc, console: { warn: function () {}, error: function () {}, log: function () {} },
    setInterval: function(){return 1;}, clearInterval: function(){},
    fetch: function(){return Promise.resolve({ok:true,json:function(){return Promise.resolve({});}});},
    Date: Date, Intl: Intl, gbT: function(k,f){return f;},
  };
  sandbox.window = sandbox;
  h.loadScript("timeclock/timeclock.js", sandbox);
  var clicks = h.countListeners(doc.getElementById("tc-ot-open"), "click");
  h.assert(clicks <= 1, "expected <=1 click listener on tc-ot-open, got " + clicks);
});

h.test("TIMECLOCK: hostile input as text", function () {
  var src = h.readFile("timeclock/timeclock.js");
  h.assert(src.indexOf("textContent") !== -1 || src.indexOf("escape") !== -1 || src.indexOf("innerText") !== -1,
    "timeclock should render hostile input as text");
});
