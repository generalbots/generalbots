/**
 * [UAT:CANVAS-06] "Gere um organograma da minha equipe de 12 pessoas"
 * Steps exercised: (1) a three-level tree of 12 named nodes (1 root, 3
 * managers, 8 reports) with 11 connectors is drawn with the real tools;
 * (2) zoomIn reaches 200%+ with a live zoom label and the names remain
 * selectable vector text (sharp), then resetZoom restores 100%.
 * Drives real canvas modules 01 (pointer tools), 03 (zoom/resetZoom) and
 * 04 (exportSvg) with the shared canvas stub.
 */
"use strict";

var env = require("./00_env");
var h = env.h;
var canvasStub = require("./00_canvas_stub");

h.test("[UAT:CANVAS-06] draw a three-level org chart of 12 people and zoom past 200%", function () {
  var names = [
    "CEO", "CFO", "COO", "CTO",
    "Ana", "Bruno", "Carla", "Diego", "Elisa", "Fabio", "Gina", "Hugo",
  ];
  var prompts = names.slice();
  var ctx = canvasStub.makeCanvas({
    GBModal: { prompt: function () { return prompts.length ? prompts.shift() : "Person"; } },
  });
  var app = ctx.app;

  // Step 1 - three levels: root, managers, reports, connected by lines.
  var nodes = [{ x: 52, y: 5, w: 16, h: 12 }];
  [6, 52, 98].forEach(function (x) { nodes.push({ x: x, y: 45, w: 16, h: 12 }); });
  [0, 14, 28, 42, 56, 70, 84, 98].forEach(function (x) {
    nodes.push({ x: x, y: 85, w: 12, h: 10 });
  });

  app.setFillColor("#dcfce7");
  nodes.forEach(function (n) {
    canvasStub.draw(ctx, "rectangle", [n.x, n.y], [n.x + n.w, n.y + n.h]);
  });
  app.setStrokeColor("#15803d");
  function center(n) { return { x: n.x + n.w / 2, y: n.y + n.h / 2 }; }
  var parent = center(nodes[0]);
  [1, 2, 3].forEach(function (i) {
    var child = center(nodes[i]);
    canvasStub.draw(ctx, "line", [parent.x, parent.y], [child.x, child.y]);
  });
  [1, 2, 3].forEach(function (managerIndex) {
    var from = center(nodes[managerIndex]);
    var first = 4 + (managerIndex - 1) * 3;
    var last = managerIndex === 3 ? 12 : first + 3;
    for (var i = first; i < last; i++) {
      var to = center(nodes[i]);
      canvasStub.draw(ctx, "line", [from.x, from.y], [to.x, to.y]);
    }
  });
  nodes.forEach(function (n) {
    var c = center(n);
    canvasStub.draw(ctx, "text", [c.x - 5, c.y], [c.x - 5, c.y]);
  });

  var rects = app.elements.filter(function (el) { return el.type === "rectangle"; });
  var lines = app.elements.filter(function (el) { return el.type === "line"; });
  var texts = app.elements.filter(function (el) { return el.type === "text"; });
  h.assert(rects.length === 12, "expected 12 org-chart nodes, got " + rects.length);
  var level1 = rects.filter(function (r) { return r.y < 20; });
  var level2 = rects.filter(function (r) { return r.y >= 20 && r.y < 60; });
  var level3 = rects.filter(function (r) { return r.y >= 60; });
  h.assert(level1.length === 1 && level2.length === 3 && level3.length === 8,
    "expected a 1/3/8 three-level tree, got " +
      [level1.length, level2.length, level3.length].join("/"));
  h.assert(lines.length === 11, "expected 11 parent/child connectors, got " + lines.length);
  h.assert(texts.length === 12 && texts[0].text === "CEO" && texts[11].text === "Hugo",
    "not all 12 people are labelled: " + JSON.stringify(texts.map(function (t) { return t.text; })));

  // Step 2 - zoom past 200%; names stay vector text, then reset to 100%.
  var label = ctx.doc.getElementById("zoom-level");
  for (var z = 0; z < 4; z++) app.zoomIn();
  h.assert(app.zoom >= 2, "zoom did not reach 200%: " + app.zoom);
  h.assert(label.textContent === Math.round(app.zoom * 100) + "%",
    "zoom label does not reflect the live zoom: " + label.textContent);
  h.assert(parseInt(label.textContent, 10) >= 200, "zoom label is below 200%: " + label.textContent);

  app.exportSvg("orgchart", false);
  var svg = ctx.blobs[ctx.blobs.length - 1].parts.join("");
  names.forEach(function (name) {
    h.assert(svg.indexOf(">" + name + "</text>") !== -1,
      "name is not sharp/selectable vector text after zoom: " + name);
  });
  h.assert(svg.indexOf("<image") === -1, "zoomed org chart was flattened to a bitmap");

  app.resetZoom();
  h.assert(app.zoom === 1 && label.textContent === "100%",
    "resetZoom did not restore 100%: " + label.textContent);
});
