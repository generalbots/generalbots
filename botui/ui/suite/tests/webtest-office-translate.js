/**
 * Offline verification for #2511 office translation across the trio.
 * Run with: node botui/ui/suite/tests/webtest-office-translate.js
 *
 * The suite lives in ordered modules under ./webtest-office-translate/:
 *   00_harness.js                 vm/stub harness, counters, report
 *   10_translate_runtime.js       REAL docs/17_translate.js executed in vm
 *   20_glossary_variant_runtime.js REAL docs language + quick-parts helpers
 *   30_static_gaps.js             labelled [STATIC] not-shipped assertions
 *
 * RUNTIME tests execute shipped code. [STATIC] tests only document source
 * facts and never claim runtime coverage for features that are not shipped.
 */
"use strict";

var harness = require("./webtest-office-translate/00_harness");

require("./webtest-office-translate/10_translate_runtime");
require("./webtest-office-translate/20_glossary_variant_runtime");
require("./webtest-office-translate/30_static_gaps");

harness.finish();
