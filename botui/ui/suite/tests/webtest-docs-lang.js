/**
 * Offline verification for #2484 docs language helpers.
 * Run with: node botui/ui/suite/tests/webtest-docs-lang.js
 *
 * The suite lives in ordered modules under ./webtest-docs-lang/:
 *   00_harness.js  counters, assertions, vm loader, report
 *   10_lang.js     real-module runtime tests + labelled STATIC guards
 */
"use strict";

var harness = require("./webtest-docs-lang/00_harness");

require("./webtest-docs-lang/10_lang");

harness.finish();
