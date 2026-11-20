/**
 * Slides Diagram/Flowchart Authoring — Offline Test Suite (#2488).
 * Run with: node botui/ui/suite/tests/webtest-slides-diagram.js
 *
 * Pure, DOM-free helpers of the shipped diagram modules are loaded into a
 * single VM sandbox and verified without a browser. The ordered modules live
 * in ./webtest-slides-diagram/ (mirrors webtest_gb7.js):
 *   00_harness.js   counters, assertions, real-module loading, report
 *   10_diagram.js   presets, flow connectors, org chart, icons, determinism,
 *                   plus labelled STATIC checks for DOM-only render helpers
 *
 * Editable PPTX export, drag-based connector editing and real-slide UAT remain
 * browser/CI-only work and are not claimed by this suite.
 */
"use strict";

var harness = require("./webtest-slides-diagram/00_harness");

require("./webtest-slides-diagram/10_diagram");

// Cycle arrows must leave/enter the box edges facing the next node, not a
// hard-coded east/west pair that cuts through boxes on the ring's sides.
harness.test("cycle connectors attach to the edges facing the next node", function () {
  var gb = harness.loadModules();
  var ns = [];
  for (var i = 0; i < 6; i++) {
    ns.push({ id: "c" + i, text: "c" + i, icon: "", color: "", level: 0, parentId: null,
      x: 0, y: 0, w: gb.presets.NODE_W, h: gb.presets.NODE_H });
  }
  var cs = gb.presets.layout("cycle", ns);
  gb.flow.recompute({ nodes: ns, connectors: cs });
  cs.forEach(function (c) {
    var from = gb.presets.byId({ nodes: ns }, c.from);
    var to = gb.presets.byId({ nodes: ns }, c.to);
    var ports = gb.flow.autoPorts(from, to);
    harness.assert(JSON.stringify(c.path[0]) === JSON.stringify([
      gb.flow.portPoint(from, ports[0]).x, gb.flow.portPoint(from, ports[0]).y
    ]), "cycle source port wrong for " + c.id);
    harness.assert(JSON.stringify(c.path[c.path.length - 1]) === JSON.stringify([
      gb.flow.portPoint(to, ports[1]).x, gb.flow.portPoint(to, ports[1]).y
    ]), "cycle target port wrong for " + c.id);
  });
});

harness.finish();
