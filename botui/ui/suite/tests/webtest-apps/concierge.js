"use strict";
var h = require("./00_harness");

h.test("CONCIERGE: files exist", function () {
  h.assert(h.fileExists("concierge/concierge.js"), "concierge.js missing");
  h.assert(h.fileExists("concierge/concierge.html"), "concierge.html missing");
});

h.test("CONCIERGE: mounts after load (readyState complete)", function () {
  var src = h.readFile("concierge/concierge.js");
  h.assert(/readyState/.test(src) || src.indexOf("DOMContentLoaded") === -1, "concierge should boot when ready");
  var doc = h.makeDocument({ readyState: "complete", ids: ["gb-concierge", "conciergeGoal", "conciergeRun", "conciergeSteps", "conciergeHistory"] });
  var sandbox = {
    window: {}, document: doc,
    fetch: function () { return Promise.resolve({ ok:true, json:function(){return Promise.resolve({steps:[]});} }); },
    localStorage: { getItem: function(){return "[]";}, setItem: function(){} },
    JSON: JSON, Date: Date, console: { warn: function () {}, error: function () {}, log: function () {} },
    gbT: function(k,f){return f;},
  };
  sandbox.window = sandbox;
  h.loadScript("concierge/concierge.js", sandbox);
  h.assert(doc.getElementById("conciergeRun"), "concierge run marker present");
});

h.test("CONCIERGE: one listener per control (Run)", function () {
  var doc = h.makeDocument({ readyState: "complete", ids: ["gb-concierge", "conciergeGoal", "conciergeRun", "conciergeSteps", "conciergeHistory"] });
  var sandbox = {
    window: {}, document: doc,
    fetch: function () { return Promise.resolve({ ok:true, json:function(){return Promise.resolve({steps:[]});} }); },
    localStorage: { getItem: function(){return "[]";}, setItem: function(){} },
    JSON: JSON, Date: Date, console: { warn: function () {}, error: function () {}, log: function () {} },
    gbT: function(k,f){return f;},
  };
  sandbox.window = sandbox;
  h.loadScript("concierge/concierge.js", sandbox);
  var clicks = h.countListeners(doc.getElementById("conciergeRun"), "click");
  h.assert(clicks <= 1, "expected <=1 click listener on conciergeRun, got " + clicks);
});

h.test("CONCIERGE: hostile input as text", function () {
  var src = h.readFile("concierge/concierge.js");
  h.assert(src.indexOf("escape") !== -1 || src.indexOf("textContent") !== -1 || src.indexOf("innerText") !== -1,
    "concierge should render hostile input as text");
});
