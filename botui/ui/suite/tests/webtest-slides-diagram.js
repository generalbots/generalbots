/**
 * Slides Diagram/Flowchart Authoring — Offline Test Suite (#2488).
 * Run with: node botui/ui/suite/tests/webtest-slides-diagram.js
 *
 * Pure, DOM-free helpers of the shipped diagram modules are loaded into a
 * single VM sandbox and verified without a browser. The ordered modules live
 * in ./webtest-slides-diagram/ (mirrors webtest_gb7.js):
 *   00_harness.js   counters, assertions, real-module loading, report
 *   10_diagram.js   presets, flow connectors, org chart, icons, determinism,
 *                   plus labelled STATIC checks for DOM-only render helpers
 *
 * Editable PPTX export, drag-based connector editing and real-slide UAT remain
 * browser/CI-only work and are not claimed by this suite.
 */
"use strict";

var harness = require("./webtest-slides-diagram/00_harness");

require("./webtest-slides-diagram/10_diagram");

harness.finish();
