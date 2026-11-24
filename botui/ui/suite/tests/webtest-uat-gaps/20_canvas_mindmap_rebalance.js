/**
 * [UAT:CANVAS-09] "Desenhe um mapa mental com 6 ramos sobre marketing digital"
 * Steps exercised: (1) a central node with six labelled branches and six
 * connectors is drawn with the real tools; (2) a seventh branch is added and
 * all branches/connectors are rebalanced through the real property editor,
 * with state assertions that no two branch boxes overlap and the centre is intact.
 * Drives real canvas modules 01 (pointer tools) and 03 (updateElementProperty)
 * with the shared canvas stub.
 */
"use strict";

var env = require("./00_env");
var h = env.h;
var canvasStub = require("./00_canvas_stub");

h.test("[UAT:CANVAS-09] draw six mind-map branches and rebalance after adding a seventh", function () {
  var prompts = [
    "Digital Marketing", "SEO", "Ads", "Social", "Email", "Content", "Analytics",
    "Partnerships",
  ];
  var ctx = canvasStub.makeCanvas({
    GBModal: { prompt: function () { return prompts.length ? prompts.shift() : "Branch"; } },
  });
  var app = ctx.app;
  var cx = 60;
  var cy = 40;
  var radius = 42;

  function pointOnCircle(count, index) {
    var angle = (Math.PI * 2 * index) / count;
    return { x: cx + radius * Math.cos(angle), y: cy + radius * Math.sin(angle) };
  }

  // Step 1 - central node plus six labelled branches, each with a connector.
  app.setFillColor("#fef3c7");
  canvasStub.draw(ctx, "ellipse", [cx - 10, cy - 8], [cx + 10, cy + 8]);
  canvasStub.draw(ctx, "text", [cx - 14, cy], [cx - 14, cy]);

  var six = [];
  app.setFillColor("#dbeafe");
  for (var i = 0; i < 6; i++) {
    var p = pointOnCircle(6, i);
    six.push(p);
    canvasStub.draw(ctx, "rectangle", [p.x - 11, p.y - 6], [p.x + 11, p.y + 6]);
  }
  app.setStrokeColor("#1e3a8a");
  six.forEach(function (p) { canvasStub.draw(ctx, "line", [cx, cy], [p.x, p.y]); });
  six.forEach(function (p) { canvasStub.draw(ctx, "text", [p.x - 6, p.y], [p.x - 6, p.y]); });

  var branches = app.elements.filter(function (el) { return el.type === "rectangle"; });
  var connectors = app.elements.filter(function (el) { return el.type === "line"; });
  h.assert(branches.length === 6, "expected six mind-map branches, got " + branches.length);
  h.assert(connectors.length === 6, "expected six branch connectors, got " + connectors.length);
  var centre = app.elements.filter(function (el) { return el.type === "ellipse"; })[0];
  h.assert(!!centre && centre.x === cx - 10 && centre.y === cy - 8, "central node is missing");

  // Add a seventh branch, then rebalance all seven evenly with the real editor.
  app.setFillColor("#dbeafe");
  canvasStub.draw(ctx, "rectangle", [100, 5], [122, 17]);
  app.setStrokeColor("#1e3a8a");
  canvasStub.draw(ctx, "line", [cx, cy], [111, 11]);
  canvasStub.draw(ctx, "text", [100, 8], [100, 8]);
  branches = app.elements.filter(function (el) { return el.type === "rectangle"; });
  connectors = app.elements.filter(function (el) { return el.type === "line"; });
  h.assert(branches.length === 7 && connectors.length === 7,
    "seventh branch did not produce seven branches and connectors");

  branches.forEach(function (rect, index) {
    var p = pointOnCircle(7, index);
    app.selectedElement = rect;
    app.updateElementProperty("x", Math.round(p.x - 11));
    app.updateElementProperty("y", Math.round(p.y - 6));
  });
  connectors.forEach(function (line, index) {
    var p = pointOnCircle(7, index);
    app.selectedElement = line;
    app.updateElementProperty("x", cx);
    app.updateElementProperty("y", cy);
    app.updateElementProperty("width", Math.round(p.x - cx));
    app.updateElementProperty("height", Math.round(p.y - cy));
  });

  function overlaps(a, b) {
    return !(a.x + a.width <= b.x || b.x + b.width <= a.x ||
      a.y + a.height <= b.y || b.y + b.height <= a.y);
  }
  for (var a = 0; a < branches.length; a++) {
    for (var b = a + 1; b < branches.length; b++) {
      h.assert(!overlaps(branches[a], branches[b]),
        "branches " + a + " and " + b + " overlap after rebalancing");
    }
  }
  connectors.forEach(function (line, index) {
    var p = pointOnCircle(7, index);
    h.assert(Math.abs(line.x + line.width - p.x) <= 1 &&
      Math.abs(line.y + line.height - p.y) <= 1,
      "connector " + index + " no longer meets its rebalanced branch");
  });

  var labels = app.elements.filter(function (el) { return el.type === "text"; });
  h.assert(labels.length === 8, "expected the centre and seven branch labels, got " + labels.length);
  var distinct = {};
  labels.forEach(function (el) { distinct[el.text] = true; });
  h.assert(Object.keys(distinct).length === 8 && !!distinct.Partnerships,
    "rebalancing lost or duplicated branch labels: " + JSON.stringify(Object.keys(distinct)));
  h.assert(centre.x === cx - 10 && centre.y === cy - 8, "rebalancing moved the central node");
});
