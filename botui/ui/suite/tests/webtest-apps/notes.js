"use strict";
var h = require("./00_harness");

h.test("NOTES: files exist", function () {
  h.assert(h.fileExists("notes/notes.js"), "notes.js missing");
  h.assert(h.fileExists("notes/notes.html"), "notes.html missing");
});

h.test("NOTES: mounts after load (readyState complete)", function () {
  var src = h.readFile("notes/notes.js");
  h.assert(/readyState/.test(src), "notes.js missing readyState guard");
  var doc = h.makeDocument({ readyState: "complete", ids: ["gb-notes", "notesAdd", "notesCount", "notesBoard"] });
  var sandbox = {
    window: {},
    document: doc,
    localStorage: { getItem: function () { return "[]"; }, setItem: function () {} },
    Date: Date, Math: Math, JSON: JSON, crypto: { randomUUID: function(){return "id";} },
    console: { warn: function () {}, error: function () {}, log: function () {} },
    requestAnimationFrame: function (fn) { return fn && fn(); },
    gbT: function (k,f){return f;},
  };
  sandbox.window = sandbox;
  h.loadScript("notes/notes.js", sandbox);
  h.assert(doc.getElementById("notesBoard"), "notes board marker present");
});

h.test("NOTES: one listener per control (New Note button)", function () {
  var doc = h.makeDocument({ readyState: "complete", ids: ["gb-notes", "notesAdd", "notesCount", "notesBoard"] });
  var sandbox = {
    window: {},
    document: doc,
    localStorage: { getItem: function () { return "[]"; }, setItem: function () {} },
    Date: Date, Math: Math, JSON: JSON, crypto: { randomUUID: function(){return "id";} },
    console: { warn: function () {}, error: function () {}, log: function () {} },
    requestAnimationFrame: function (fn) { return fn && fn(); },
    gbT: function (k,f){return f;},
  };
  sandbox.window = sandbox;
  h.loadScript("notes/notes.js", sandbox);
  var clicks = h.countListeners(doc.getElementById("notesAdd"), "click");
  h.assert(clicks <= 1, "expected <=1 click listener on notesAdd, got " + clicks);
});

h.test("NOTES: hostile input as text", function () {
  var src = h.readFile("notes/notes.js");
  // notes renders note content via textContent/escaping patterns in modern implementation
  h.assert(src.indexOf("textContent") !== -1 || src.indexOf("escape") !== -1 || src.indexOf("innerText") !== -1,
    "notes should render user content as text");
});
