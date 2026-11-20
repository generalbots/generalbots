/**
 * Sheets: unstructured-to-structured ingest — Offline Test Suite (#2494).
 * Run with: node botui/ui/suite/tests/webtest-sheet-ingest.js
 *
 * The nine shipped ingest IIFEs (59..65) are loaded verbatim into one shared
 * vm sandbox with stubbed window/document/gbT/localStorage/clock/uid. Pure and
 * DOM-rendered invariants are proved by EXECUTING the real modules. Behaviour
 * that needs a real parser, tenant storage or retention enforcement is covered
 * only by labelled [STATIC] source assertions; no runtime result is fabricated.
 *
 * Browser/CDP verification of the styled dialog stays a separate gate.
 */
"use strict";

var harness = require("./webtest-sheet-ingest/00_harness");

require("./webtest-sheet-ingest/10_sim_cells");
require("./webtest-sheet-ingest/11_preview_map");
require("./webtest-sheet-ingest/12_review_batch_policy");

harness.finish();
