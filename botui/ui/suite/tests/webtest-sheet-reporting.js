/**
 * Sheets: print, reporting and scheduled delivery — Offline Test Suite (#2438).
 * Run with: node botui/ui/suite/tests/webtest-sheet-reporting.js
 *
 * The six shipped reporting IIFEs are loaded verbatim into one shared vm
 * sandbox (see ./webtest-sheet-reporting/00_harness.js). Pure helpers are
 * exercised at runtime; private/DOM-only and backend-deferred behaviour is
 * covered by clearly-labelled [STATIC] source assertions only.
 *
 * The shipped feature is explicitly a local-only preview: no real PDF export,
 * email/chat delivery, cron registration or workbook write is claimed here.
 * Browser/CDP verification of the rendered dialog remains a separate gate.
 */
"use strict";

var harness = require("./webtest-sheet-reporting/00_harness");

require("./webtest-sheet-reporting/10_reporting");
require("./webtest-sheet-reporting/11_reporting_more");

harness.finish();
