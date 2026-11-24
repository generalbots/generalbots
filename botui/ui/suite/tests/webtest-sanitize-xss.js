/**
 * #2743 — standalone, CI-runnable entry point for the sanitizer XSS corpus.
 * Run with: node botui/ui/suite/tests/webtest-sanitize-xss.js
 */
"use strict";

const harness = require("./webtest-gb7/00_harness");

require("./webtest-gb7/55_sanitize_xss");

harness.finish();
