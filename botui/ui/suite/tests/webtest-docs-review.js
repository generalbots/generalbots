/**
 * Offline verification for #2425 docs enterprise review workflow.
 * Run with: node botui/ui/suite/tests/webtest-docs-review.js
 *
 * The suite lives in ordered modules under ./webtest-docs-review/:
 *   00_harness.js          counters, FakeNode/DOM/vm loader, deterministic clock
 *   10_review.js           runtime tests of the shipped 49..57 IIFEs
 *   11_review_deferred.js  additional runtime + labelled STATIC source checks
 *
 * Real production modules are loaded unmodified into one shared Node vm
 * sandbox. STATIC assertions only cover private/DOM wiring or behaviour the
 * modules explicitly defer to the server (403 enforcement, DOCX w:ins/w:del,
 * colour PDF export, notifications); they never fabricate a runtime result.
 */
"use strict";

var harness = require("./webtest-docs-review/00_harness");

require("./webtest-docs-review/10_review");
require("./webtest-docs-review/11_review_deferred");

harness.report();
