/**
 * Offline readiness audit for #2534 — are Docs, Sheets and Slides ready for
 * large files on slow, low-memory machines?
 *
 * Run with: node botui/ui/suite/tests/webtest-office-large.js
 *
 * Deterministic, read-only Node harness: plain scripts, a stubbed vm sandbox,
 * no CDN, no server, no browser and no cargo. It scans the shipped Office UI
 * plus the XLSX import guards, executes the shipped Docs large-file preview
 * modules in a vm, and prints a Docs/Sheets/Slides x capability matrix.
 * Present capabilities are asserted PASS with file:line evidence; each missing
 * capability is reported as an explicit GAP with a concrete file:line anchor
 * rather than being silently passed. Timing/RSS/wire-byte/OOM interaction
 * claims are deliberately left to a real browser + CI and are NOT fabricated.
 *
 * Ordered modules under ./webtest-office-large/:
 *   00_harness.js      counters (PASS/GAP/FAIL), source readers, anchors, report
 *   01_matrix.js       capability/app definitions and absence patterns
 *   10_docs.js         Docs probes (windowed read + virtualized render PASS)
 *   11_sheet.js        Sheets probes (virtualized grid PASS, rest GAP)
 *   12_slides.js       Slides probes (all GAP)
 *   20_vm.js           shipped Docs large modules executed in a stubbed vm
 *   30_complete.js     every matrix cell probed exactly once
 *
 * Nothing is committed by this suite; the driver owns issue closure/deploy.
 */
"use strict";

var harness = require("./webtest-office-large/00_harness");

require("./webtest-office-large/01_matrix");
require("./webtest-office-large/10_docs");
require("./webtest-office-large/11_sheet");
require("./webtest-office-large/12_slides");
require("./webtest-office-large/20_vm");
require("./webtest-office-large/30_complete");

harness.finish();
