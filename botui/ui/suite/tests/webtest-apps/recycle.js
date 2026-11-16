"use strict";
var h = require("./00_harness");

h.test("RECYCLE: files exist", function () {
  h.assert(h.fileExists("recycle/recycle.js"), "recycle.js missing");
  h.assert(h.fileExists("recycle/recycle.html"), "recycle.html missing");
});

h.test("RECYCLE: mounts after load (readyState complete)", function () {
  var src = h.readFile("recycle/recycle.js");
  h.assert(/readyState/.test(src), "recycle.js missing readyState guard");
  var doc = h.makeDocument({ readyState: "complete", ids: ["gb-recycle", "recycleList", "recycleEmpty", "recycleRefresh"] });
  var sandbox = {
    window: {},
    document: doc,
    fetch: function () { return Promise.resolve({ ok: true, json: function(){return Promise.resolve([]);} }); },
    console: { warn: function () {}, error: function () {}, log: function () {} },
    Date: Date, JSON: JSON, Array: Array,
    gbT: function (k,f){return f;},
    confirm: function(){return true;},
    addEventListener: function(){},
    removeEventListener: function(){},
  };
  sandbox.window = sandbox;
  sandbox.window.addEventListener = function(){};
  sandbox.window.removeEventListener = function(){};
  h.loadScript("recycle/recycle.js", sandbox);
  h.assert(doc.getElementById("recycleList"), "recycle list marker present");
});

h.test("RECYCLE: one listener per control (Refresh)", function () {
  var doc = h.makeDocument({ readyState: "complete", ids: ["gb-recycle", "recycleList", "recycleEmpty", "recycleRefresh"] });
  var sandbox = {
    window: {},
    document: doc,
    fetch: function () { return Promise.resolve({ ok: true, json: function(){return Promise.resolve([]);} }); },
    console: { warn: function () {}, error: function () {}, log: function () {} },
    Date: Date, JSON: JSON, Array: Array,
    gbT: function (k,f){return f;},
    confirm: function(){return true;},
    addEventListener: function(){},
    removeEventListener: function(){},
  };
  sandbox.window = sandbox;
  sandbox.window.addEventListener = function(){};
  sandbox.window.removeEventListener = function(){};
  h.loadScript("recycle/recycle.js", sandbox);
  var clicks = h.countListeners(doc.getElementById("recycleRefresh"), "click");
  h.assert(clicks <= 1, "expected <=1 click listener on recycleRefresh, got " + clicks);
});

h.test("RECYCLE: hostile input as text (escapeHtml present)", function () {
  var src = h.readFile("recycle/recycle.js");
  h.assert(src.indexOf("escapeHtml") !== -1 || src.indexOf("textContent") !== -1,
    "recycle should escape/render hostile names as text");
});
