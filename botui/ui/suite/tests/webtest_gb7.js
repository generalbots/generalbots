/**
 * GB7 Web Automation Test Suite
 * Tests all GB7 features and BBSIA review fixes.
 * Run with: node botui/ui/suite/tests/webtest_gb7.js
 *
 * The suite lives in ordered modules under ./webtest-gb7/ (#1731):
 *   00_harness.js     counters, assertions, file helpers, report
 *   10_security.js    SEC-01..07 and FUNC-01..05
 *   20_features.js    file/content/BBSIA/artifacts/sharing/trash/memory/connectors
 *   30_syntax.js      syntax validation of the shipped scripts
 *   40_behaviour.js   behavioural tests (vm sandbox)
 *   50_issues.js      issue coverage, drive checks and size guards
 */
"use strict";

var harness = require("./webtest-gb7/00_harness");

require("./webtest-gb7/10_security");
require("./webtest-gb7/20_features");
require("./webtest-gb7/30_syntax");
require("./webtest-gb7/40_behaviour");
require("./webtest-gb7/50_issues");

harness.finish();
