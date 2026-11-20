/**
 * Sheets Structured Data & What-If — Offline Test Suite (#2436).
 * Run with: node botui/ui/suite/tests/webtest-sheet-structured.js
 *
 * The shipped IIFE modules are loaded from disk into one vm sandbox with
 * stubbed window/document/gbT. Ordered modules:
 *   00_harness.js   counters, assertions, module loading, report
 *   10_structured.js  addressing, table model, slicer, sparklines, goal seek,
 *                     scenarios and labelled STATIC scope checks
 *
 * Real Excel Table objects, structured references, XLSX round-trip and the
 * browser-facing dialogs remain CI/CDP-only work.
 */
"use strict";

var harness = require("./webtest-sheet-structured/00_harness");

require("./webtest-sheet-structured/10_structured");

harness.finish();
