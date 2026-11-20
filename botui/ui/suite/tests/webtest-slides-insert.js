/**
 * Slides Insert hub — Offline Test Suite (#2428).
 * Run with: node botui/ui/suite/tests/webtest-slides-insert.js
 *
 * The four shipped modules (10_insert_catalog, 11_insert_dialog,
 * 12_insert_actions, 13_insert_place) are loaded into one shared Node vm
 * sandbox with stubbed window/document/gbT/storage/clock/uid. Every RUNTIME
 * check calls a genuine shipped function. STATIC checks only assert source and
 * locale-catalog facts. Ordered modules live in ./webtest-slides-insert/:
 *   00_harness.js       counters, assertions, fake DOM, vm loader, report
 *   10_insert.js        catalog, search, dialog, keyboard, placement payloads
 *   11_insert_async.js  insertion dispatch, Drive, reuse, drag/drop, failures
 *
 * Browser paint, real dataTransfer objects and the legacy server modals are
 * explicitly out of scope and are NOT claimed by this harness.
 */
"use strict";

var harness = require("./webtest-slides-insert/00_harness");

require("./webtest-slides-insert/10_insert");
require("./webtest-slides-insert/11_insert_async");

harness.finish();
