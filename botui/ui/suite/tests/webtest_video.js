/**
 * GB7 + BBSIA Web Automation — Playwright Video Evidence Suite
 *
 * Each test shows visual overlays: title → steps → result, with pauses
 * so the recorded video has meaningful content.
 *
 * Usage:  node botui/ui/suite/tests/webtest_video.js
 *
 * The suite lives in ordered modules under ./webtest-video/ (#1731):
 *   00_common.js     server, overlay helpers, harness loader
 *   10_cases_core.js TC01-TC07
 *   20_cases_ui.js   TC08-TC15
 *   30_gallery.js    evidence gallery generator
 *   40_runner.js     Playwright runner
 */
"use strict";

const TESTS = [].concat(
  require("./webtest-video/10_cases_core"),
  require("./webtest-video/20_cases_ui"),
);
const main = require("./webtest-video/40_runner");

main(TESTS).catch(e => { console.error("Fatal:", e.message); process.exit(1); });
