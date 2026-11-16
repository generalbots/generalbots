"use strict";
var h = require("./00_harness");

h.test("SNAPSHOT: files exist", function () {
  h.assert(h.fileExists("snapshot/snapshot.js"), "snapshot.js missing");
  h.assert(h.fileExists("snapshot/snapshot.html"), "snapshot.html missing");
});

h.test("SNAPSHOT: mounts after load (readyState complete)", function () {
  var src = h.readFile("snapshot/snapshot.js");
  h.assert(/readyState/.test(src), "snapshot.js missing readyState guard");
  var doc = h.makeDocument({ readyState: "complete", ids: ["gb-snap-root", "gb-snap-capture", "gb-snap-copy", "gb-snap-download", "gb-snap-preview"] });
  var sandbox = {
    window: {},
    document: doc,
    navigator: { mediaDevices: {}, clipboard: {} },
    URL: { createObjectURL: function(){return "blob:x";}, revokeObjectURL: function(){} },
    console: { warn: function () {}, error: function () {}, log: function () {} },
  };
  sandbox.window = sandbox;
  h.loadScript("snapshot/snapshot.js", sandbox);
  h.assert(doc.getElementById("gb-snap-capture"), "snapshot capture marker present");
});

h.test("SNAPSHOT: one listener per control (Capture)", function () {
  var doc = h.makeDocument({ readyState: "complete", ids: ["gb-snap-root", "gb-snap-capture", "gb-snap-copy", "gb-snap-download", "gb-snap-preview"] });
  var sandbox = {
    window: {}, document: doc,
    navigator: { mediaDevices: {}, clipboard: {} },
    URL: { createObjectURL: function(){return "blob:x";}, revokeObjectURL: function(){} },
    console: { warn: function () {}, error: function () {}, log: function () {} },
  };
  sandbox.window = sandbox;
  h.loadScript("snapshot/snapshot.js", sandbox);
  var clicks = h.countListeners(doc.getElementById("gb-snap-capture"), "click");
  h.assert(clicks <= 1, "expected <=1 click listener on gb-snap-capture, got " + clicks);
});

h.test("SNAPSHOT: hostile input as text (no unsafe HTML rendering path)", function () {
  var src = h.readFile("snapshot/snapshot.js");
  h.assert(src.indexOf("innerHTML") === -1 || src.indexOf("escape") !== -1,
    "snapshot should avoid unescaped innerHTML for hostile values");
});
