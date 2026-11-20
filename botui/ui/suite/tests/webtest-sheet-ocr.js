/**
 * Office OCR / scan-to-document pipeline — OFFLINE characterization (#2509).
 * Run with: node botui/ui/suite/tests/webtest-sheet-ocr.js
 *
 * The shipped confidence-review chain (59 core, 59a fixture, 59b cells, 62
 * review) is loaded verbatim into one Node vm sandbox with stubbed host globals.
 * Real gate/treatment behaviour is PROVED BY EXECUTION. OCR engine, scanning,
 * image cleanup, language detection and table extraction are NOT shipped here:
 * those criteria are locked as labelled [BASELINE-OBSERVE]/[STATIC] current
 * state, never as runtime PASS. Defects are proved as [BUG] and reported only.
 *
 * The sibling webtest-sheet-ingest harness is deliberately untouched, as are
 * production JS, locales, guards, baselines and CI.
 */
"use strict";

var harness = require("./webtest-sheet-ocr/00_harness");

require("./webtest-sheet-ocr/10_confidence_gate");
require("./webtest-sheet-ocr/20_confidence_treatment");
require("./webtest-sheet-ocr/30_pipeline_baseline");
require("./webtest-sheet-ocr/40_bug_proofs");

harness.finish();
