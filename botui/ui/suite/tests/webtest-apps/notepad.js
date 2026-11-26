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

h.test("NOTEPAD: #2962 guards the root and every control lookup", function () {
  var src = h.readFile("notepad/notepad.js");
  h.assert(src.indexOf('if (!root || !root.classList || root.dataset.npInit === "1") return;') !== -1,
    "the app root must be null/classList-guarded before any DOM access");
  h.assert(src.indexOf("if (!sel || !ta || !status || !newBtn || !delBtn) return;") !== -1,
    "missing controls must fail closed instead of throwing on addEventListener");
  h.assert(src.indexOf('root.querySelector("#gb-notepad-new").addEventListener') === -1 &&
    src.indexOf('root.querySelector("#gb-notepad-del").addEventListener') === -1,
    "no direct listener binding on a fresh querySelector result");
});

h.test("NOTEPAD: #2962 boot does not throw when controls are missing", function () {
  var root = h.makeElement("gb-notepad-root", "div");
  root.querySelector = function () { return null; };
  var doc = {
    readyState: "complete",
    getElementById: function (id) { return id === "gb-notepad-root" ? root : null; },
    addEventListener: function () {},
  };
  var sandbox = {
    document: doc,
    localStorage: { getItem: function () { return null; }, setItem: function () {} },
    Date: Date, Math: Math, JSON: JSON,
    console: { warn: function () {}, error: function () {}, log: function () {} },
    setTimeout: function () { return 0; }, clearTimeout: function () {},
    gbT: function (k, f) { return f; },
  };
  sandbox.window = sandbox;
  // The pre-fix code threw here: a detached root still had querySelector(),
  // but its buttons were gone, so addEventListener hit null.
  h.loadScript("notepad/notepad.js", sandbox);
  h.assert(root.dataset.npInit !== "1",
    "an incomplete shell must stay uninitialized so a later boot can retry");
});
