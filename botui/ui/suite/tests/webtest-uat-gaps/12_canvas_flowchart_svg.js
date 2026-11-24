/**
 * [UAT:CANVAS-03] "Crie um fluxograma de aprovação de compras"
 * Steps exercised: (1) nodes and connectors are drawn left-to-right and adding
 * a third node keeps both connectors in place; (3) the SVG export keeps the
 * node captions as selectable <text> and never embeds a bitmap.
 * Step 1's chat generation is LLM-driven; the diagram is drawn here with the
 * shipped canvas tools.
 * Drives the real canvas modules (01 pointer tools, 04 composeSvg/exportSvg).
 */
"use strict";

var env = require("./00_env");
var h = env.h;
var canvasStub = require("./00_canvas_stub");

h.test("[UAT:CANVAS-03] draw a left-to-right flowchart and export selectable SVG", function () {
  var labels = ["Compras", "Aprovar", "Financeiro"];
  var ctx = canvasStub.makeCanvas({
    GBModal: {
      prompt: function () { return labels.length ? labels.shift() : "Node"; },
    },
  });
  var app = ctx.app;

  // Approval flow: rect nodes with arrow connectors, drawn left to right.
  app.setFillColor("#e0f2fe");
  canvasStub.draw(ctx, "rectangle", [10, 10], [60, 40]);
  canvasStub.draw(ctx, "rectangle", [70, 10], [120, 40]);
  app.setStrokeColor("#334155");
  canvasStub.draw(ctx, "arrow", [60, 25], [70, 25]);
  canvasStub.draw(ctx, "text", [16, 20], [16, 20]);
  canvasStub.draw(ctx, "text", [76, 20], [76, 20]);

  var rectangles = app.elements.filter(function (el) { return el.type === "rectangle"; });
  var arrows = app.elements.filter(function (el) { return el.type === "arrow"; });
  h.assert(rectangles.length === 2, "expected two initial nodes, got " + rectangles.length);
  h.assert(rectangles[0].x < rectangles[1].x, "nodes are not drawn left-to-right");
  h.assert(arrows.length === 1, "the first connector is missing");

  // Adding a node must not drop the connector drawn above.
  canvasStub.draw(ctx, "rectangle", [130, 10], [180, 40]);
  canvasStub.draw(ctx, "arrow", [120, 25], [130, 25]);
  canvasStub.draw(ctx, "text", [136, 20], [136, 20]);
  rectangles = app.elements.filter(function (el) { return el.type === "rectangle"; });
  arrows = app.elements.filter(function (el) { return el.type === "arrow"; });
  h.assert(rectangles.length === 3, "expected three nodes after adding one, got " + rectangles.length);
  h.assert(
    rectangles[0].x < rectangles[1].x && rectangles[1].x < rectangles[2].x,
    "the added node broke the left-to-right order",
  );
  h.assert(arrows.length === 2, "adding a node dropped a connector: " + arrows.length);
  h.assert(
    arrows[0].x + arrows[0].width === rectangles[1].x &&
      arrows[1].x + arrows[1].width === rectangles[2].x,
    "a connector no longer joins its neighbouring node",
  );

  // Step 3 - the SVG keeps the captions as text, not as an embedded bitmap.
  app.exportSvg("flowchart", false);
  var svg = ctx.blobs[ctx.blobs.length - 1];
  h.assert(!!svg, "no SVG blob was produced");
  h.assert(svg.type === "image/svg+xml", "export is not an SVG: " + svg.type);
  var markup = svg.parts.join("");
  ["Compras", "Aprovar", "Financeiro"].forEach(function (label) {
    h.assert(
      markup.indexOf(">" + label + "</text>") !== -1,
      "caption is not selectable SVG text: " + label,
    );
  });
  h.assert((markup.match(/<line /g) || []).length === 2, "SVG lost one of the connectors: " + markup);
  h.assert(markup.indexOf("<image") === -1, "SVG embeds a bitmap instead of paths");
  var link = ctx.links[ctx.links.length - 1];
  h.assert(link && link.download === "flowchart.svg", "SVG was not offered for download");
});
