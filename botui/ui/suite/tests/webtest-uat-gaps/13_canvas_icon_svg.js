/**
 * [UAT:CANVAS-08] "Crie um ícone de carrinho de compras e exporte em SVG"
 * Steps exercised: (1) the cart icon is drawn from real shapes and the body is
 * selected with its fill shown in the properties panel; (2) the SVG download
 * contains vector paths only, never an embedded bitmap.
 * Step 1's chat generation is LLM-driven; the shapes are drawn with the
 * shipped canvas tools.
 * Drives the real canvas modules (01 pointer tools/selection, 04 exportSvg).
 */
"use strict";

var env = require("./00_env");
var h = env.h;
var canvasStub = require("./00_canvas_stub");

h.test("[UAT:CANVAS-08] draw and select a cart icon, then export vector SVG", function () {
  var ctx = canvasStub.makeCanvas();
  var app = ctx.app;

  // Shopping cart: rectangular basket, two wheels, handle and two bars.
  app.setFillColor("transparent");
  canvasStub.draw(ctx, "rectangle", [20, 10], [80, 35]);
  app.setFillColor("#111111");
  canvasStub.draw(ctx, "ellipse", [25, 45], [35, 55]);
  canvasStub.draw(ctx, "ellipse", [65, 45], [75, 55]);
  app.setStrokeColor("#111111");
  canvasStub.draw(ctx, "line", [10, 10], [20, 10]);
  canvasStub.draw(ctx, "line", [35, 14], [35, 32]);

  var body = app.elements.filter(function (el) { return el.type === "rectangle"; })[0];
  var wheels = app.elements.filter(function (el) { return el.type === "ellipse"; });
  var lines = app.elements.filter(function (el) { return el.type === "line"; });
  h.assert(!!body, "cart body was not drawn");
  h.assert(wheels.length === 2, "expected two wheels, got " + wheels.length);
  h.assert(lines.length === 2, "expected a handle and bars, got " + lines.length);

  app.selectTool("select");
  app.handleMouseDown({ button: 0, clientX: 50, clientY: 22 });
  h.assert(app.selectedElement === body, "clicking the cart body did not select it");
  h.assert(
    ctx.doc.getElementById("prop-fill").value === "transparent",
    "the selected icon's fill is not shown in the properties panel",
  );
  h.assert(
    ctx.doc.getElementById("element-properties").classList.contains("hidden") === false,
    "the properties panel stayed hidden for the selected icon",
  );

  // Step 2 - the file is a real SVG built from the drawn paths.
  app.exportSvg("cart-icon", false);
  var svg = ctx.blobs[ctx.blobs.length - 1];
  h.assert(!!svg && svg.type === "image/svg+xml", "no SVG blob was produced");
  var markup = svg.parts.join("");
  h.assert(markup.indexOf("<rect ") !== -1, "cart body is missing from the SVG");
  h.assert((markup.match(/<ellipse /g) || []).length === 2, "wheels are missing from the SVG");
  h.assert((markup.match(/<line /g) || []).length === 2, "handle/bars are missing from the SVG");
  h.assert(markup.indexOf("<image") === -1, "SVG embeds a bitmap instead of paths");
  var link = ctx.links[ctx.links.length - 1];
  h.assert(link && link.download === "cart-icon.svg", "SVG was not offered under the expected name");
});
