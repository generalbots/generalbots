/**
 * Offline verification for #2423 shipped structured document model.
 * Run with: node botui/ui/suite/tests/webtest-docs-model.js
 *
 * The real shipped IIFEs (docs/modules/68..74) are loaded, in order, into one
 * shared Node vm sandbox with stubbed window/document/gbT/storage/clock/uid.
 * Ordered modules under ./webtest-docs-model/:
 *   00_harness.js        assertions, fake DOM, vm loader, report
 *   10_model.js          runtime: schema, ids, op log, undo, projections, UI
 *   11_invariants.js     runtime invariants + labelled STATIC source checks
 */
"use strict";

var harness = require("./webtest-docs-model/00_harness");

require("./webtest-docs-model/10_model");
require("./webtest-docs-model/11_invariants");

harness.finish();
