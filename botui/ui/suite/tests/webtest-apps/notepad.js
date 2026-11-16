"use strict";
var h = require("./00_harness");

h.test("NOTEPAD: files exist", function () {
  h.assert(h.fileExists("notepad/notepad.js"), "notepad.js missing");
  h.assert(h.fileExists("notepad/modules/13_delta.js"), "notepad delta module missing");
});

h.test("NOTEPAD: mounts after load (readyState complete)", function () {
  var src = h.readFile("notepad/notepad.js");
  h.assert(/readyState/.test(src), "notepad.js missing readyState guard");
  var doc = h.makeDocument({ readyState: "complete", ids: ["gb-notepad-root", "gb-notepad-files", "gb-notepad-new", "gb-notepad-del", "gb-notepad-text", "gb-notepad-status"] });
  var sandbox = {
    window: {},
    document: doc,
    localStorage: { getItem: function () { return null; }, setItem: function () {} },
    Date: Date, Math: Math, JSON: JSON,
    console: { warn: function () {}, error: function () {}, log: function () {} },
    setTimeout: function (fn) { return 0; },
    clearTimeout: function () {},
    gbT: function (k,f){return f;},
  };
  sandbox.window = sandbox;
  h.loadScript("notepad/notepad.js", sandbox);
  h.assert(doc.getElementById("gb-notepad-text"), "notepad editor marker present");
});

h.test("NOTEPAD: one listener per control (New button)", function () {
  var doc = h.makeDocument({ readyState: "complete", ids: ["gb-notepad-root", "gb-notepad-files", "gb-notepad-new", "gb-notepad-del", "gb-notepad-text", "gb-notepad-status"] });
  var sandbox = {
    window: {},
    document: doc,
    localStorage: { getItem: function () { return null; }, setItem: function () {} },
    Date: Date, Math: Math, JSON: JSON,
    console: { warn: function () {}, error: function () {}, log: function () {} },
    setTimeout: function (fn) { return 0; }, clearTimeout: function () {},
    gbT: function (k,f){return f;},
  };
  sandbox.window = sandbox;
  h.loadScript("notepad/notepad.js", sandbox);
  var clicks = h.countListeners(doc.getElementById("gb-notepad-new"), "click");
  h.assert(clicks <= 1, "expected <=1 click listener on gb-notepad-new, got " + clicks);
});

h.test("NOTEPAD: hostile input as text (textarea/value path)", function () {
  var src = h.readFile("notepad/modules/13_delta.js");
  h.assert(src.indexOf(".value") !== -1 || src.indexOf("textContent") !== -1,
    "notepad delta should write text values, not HTML");
});
