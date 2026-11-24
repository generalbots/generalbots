/**
 * UAT gap step-level suite (#2031).
 * Run with: node botui/ui/suite/tests/webtest_uat_gaps.js
 *
 * Each module carries a real [UAT:<ID>] label and drives shipped app code
 * (not "app mounts" assertions) through at least two concrete scenario steps.
 */
"use strict";

var harness = require("./webtest-apps/00_harness");

[
  "01_notes_reopen",
  "02_goals_30km",
  "03_goals_12books",
  "04_calendar_45min",
  "05_tasks_complete_delete",
].forEach(function (name) {
  require("./webtest-uat-gaps/" + name);
});

harness.finish();
