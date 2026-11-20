/**
 * Offline verification for #2427 docs templates, content controls, mail merge
 * and document protection. Run with:
 *   node botui/ui/suite/tests/webtest-docs-forms.js
 *
 * The suite lives in ordered modules under ./webtest-docs-forms/:
 *   00_harness.js         counters, assertions, fake DOM, vm sandbox, report
 *   10_forms.js           state, i18n, templates and content controls
 *   11_merge_protection.js mail merge, dialog shell, security, protection,
 *                         boot wiring and clearly-labelled STATIC guards
 */
"use strict";

var harness = require("./webtest-docs-forms/00_harness");

require("./webtest-docs-forms/10_forms");
require("./webtest-docs-forms/11_merge_protection");

harness.finish();
