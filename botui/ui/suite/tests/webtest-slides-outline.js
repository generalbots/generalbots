/**
 * Slides outline, sections, find/replace, notes and accessibility — offline
 * test suite (#2433). Run with:
 *   node botui/ui/suite/tests/webtest-slides-outline.js
 *
 * The ordered cases live in ./webtest-slides-outline/:
 *   00_dom.js       deterministic DOM stub (no production logic)
 *   00_harness.js   counters, real-IIFE vm loader, shared fixtures, report
 *   10_outline.js   shared helpers, undo, outline view
 *   11_find_sections.js  find/replace and sections
 *   12_notes_a11y.js     notes, accessibility, properties, review bootstrap
 *
 * Browser/CDP UAT, PPTX core-property round-trip and CI remain separate gates.
 */
"use strict";

var harness = require("./webtest-slides-outline/00_harness");

require("./webtest-slides-outline/10_outline");
require("./webtest-slides-outline/11_find_sections");
require("./webtest-slides-outline/12_notes_a11y");

harness.finish();
