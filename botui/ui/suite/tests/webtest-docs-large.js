/**
 * Offline verification for #2533 docs large .docx load path.
 * Run with: node botui/ui/suite/tests/webtest-docs-large.js
 *
 * The shipped production IIFEs docs/modules/84_large_state.js through
 * 91_large_init.js are loaded UNMODIFIED into one shared Node vm sandbox with
 * stubbed window/document/gbT/localStorage/fixed clock/uid. Every RUNTIME test
 * executes that shipped code; no production logic is reimplemented here.
 *
 * Ordered modules under ./webtest-docs-large/:
 *   00_harness.js            counters, assertions, shared-vm loader, report
 *   01_dom.js                minimal but faithful FakeNode DOM stub
 *   02_fixture.js            deterministic sandbox factory (fixture)
 *   10_fixture_window.js     fixture generation + windowed read/mount
 *   11_save.js               save queue, ETag, 2 MB ceiling, commit
 *   12_degradation_baseline.js  read-only gating + baseline table
 *   13_inspector.js          dialog, status, inspector, boot wiring
 *   14_deferred.js           labelled STATIC scope assertions + KNOWN BUG
 *
 * Local-only / backend-deferred behaviour is covered by clearly-labelled
 * [STATIC] source assertions, never by a fabricated runtime PASS.
 */
"use strict";

var harness = require("./webtest-docs-large/00_harness");

require("./webtest-docs-large/01_dom");
require("./webtest-docs-large/02_fixture");
require("./webtest-docs-large/10_fixture_window");
require("./webtest-docs-large/11_save");
require("./webtest-docs-large/12_degradation_baseline");
require("./webtest-docs-large/13_inspector");
require("./webtest-docs-large/14_deferred");

harness.finish();
