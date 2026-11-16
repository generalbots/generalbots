/**
 * GB Suite Small Apps Web Test Suite (#2418)
 * Run with: node botui/ui/suite/tests/webtest_apps.js
 *
 * One file per app under ./webtest-apps/ asserting:
 * 1) Mounts after load (readyState === "complete")
 * 2) One listener per control (no duplicate action on repeated click)
 * 3) Hostile input rendered as text (no injected element)
 */
"use strict";

var harness = require("./webtest-apps/00_harness");

[
  "timer",
  "notes",
  "notepad",
  "recycle",
  "concierge",
  "snapshot",
  "calculator",
  "weather",
  "store",
  "timeclock",
  "lists",
  "photos",
].forEach(function (name) {
  require("./webtest-apps/" + name);
});

harness.finish();
