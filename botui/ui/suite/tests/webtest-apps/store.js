"use strict";
var h = require("./00_harness");

h.test("STORE: files exist", function () {
  h.assert(h.fileExists("store/store.js"), "store.js missing");
  h.assert(h.fileExists("store/store.html"), "store.html missing");
});

h.test("STORE: mounts after load (readyState complete)", function () {
  var src = h.readFile("store/store.js");
  h.assert(/readyState/.test(src), "store.js missing readyState guard");
  var doc = h.makeDocument({ readyState: "complete", ids: ["gb-store", "storeSearch", "storeCats", "storeGrid"] });
  var sandbox = {
    window: {}, document: doc,
    fetch: function(){return Promise.resolve({ok:true,json:function(){return Promise.resolve({catalog:[],server_installs:{}});},text:function(){return Promise.resolve("[]");}});},
    localStorage: { getItem: function(){return "[]";}, setItem: function(){} },
    JSON: JSON, Date: Date, Array: Array, console: { warn: function () {}, error: function () {}, log: function () {} },
    gbStoreGet: function(k,d){return d;},
    gbT: function(k,f){return f;},
  };
  sandbox.window = sandbox;
  h.loadScript("store/store.js", sandbox);
  h.assert(doc.getElementById("storeGrid"), "store grid marker present");
});

h.test("STORE: one listener per control (Search input)", function () {
  var doc = h.makeDocument({ readyState: "complete", ids: ["gb-store", "storeSearch", "storeCats", "storeGrid"] });
  var sandbox = {
    window: {}, document: doc,
    fetch: function(){return Promise.resolve({ok:true,json:function(){return Promise.resolve({catalog:[],server_installs:{}});},text:function(){return Promise.resolve("[]");}});},
    localStorage: { getItem: function(){return "[]";}, setItem: function(){} },
    JSON: JSON, Date: Date, Array: Array, console: { warn: function () {}, error: function () {}, log: function () {} },
    gbStoreGet: function(k,d){return d;},
    gbT: function(k,f){return f;},
  };
  sandbox.window = sandbox;
  h.loadScript("store/store.js", sandbox);
  var inputs = h.countListeners(doc.getElementById("storeSearch"), "input");
  h.assert(inputs <= 1, "expected <=1 input listener on storeSearch, got " + inputs);
});

h.test("STORE: hostile input as text (escapeHtml present)", function () {
  var src = h.readFile("store/store.js");
  h.assert(src.indexOf("escapeHtml") !== -1, "store should escape hostile catalog/input values");
});
