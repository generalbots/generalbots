/**
 * Sheets: charting depth — offline runtime harness (#2490).
 * Run with: node botui/ui/suite/tests/webtest-sheet-charting-depth.js
 *
 * ONE Node vm sandbox loads the REAL shipped charting IIFEs (31/32/33/34a/
 * 34b/34c/35/36) with stubbed window/document/gbT/localStorage and a fixed
 * clock + Math.random. Every RUNTIME assertion executes shipped code; nothing
 * is reimplemented. XLSX-native chart parts, server named ranges, templates,
 * axis formats, print/vector export are labelled [STATIC] source assertions
 * because they are deliberately NOT shipped (local-only charts). Observed
 * defects are pinned as KNOWN BUG tests and reported, never patched here.
 *
 * Ordered modules under ./webtest-sheet-charting-depth/:
 *   00_harness.js         vm sandbox, stubs, assertion + report helpers
 *   10_types_overlays.js  registered types, waterfall/histogram/Pareto/box, overlays
 *   20_render_ranges.js   renderSvg determinism, local store, dynamic range binding
 *   30_static_deferred.js [STATIC] deferred criteria and KNOWN BUG observations
 *
 * The pre-existing webtest-sheet-charts harness is left untouched; this suite
 * covers the cartesian/overlay/render modules that harness does not load.
 */
"use strict";

var harness = require("./webtest-sheet-charting-depth/00_harness");

require("./webtest-sheet-charting-depth/10_types_overlays");
require("./webtest-sheet-charting-depth/20_render_ranges");
require("./webtest-sheet-charting-depth/30_static_deferred");

harness.finish();
