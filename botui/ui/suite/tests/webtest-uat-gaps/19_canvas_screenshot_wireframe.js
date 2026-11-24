/**
 * [UAT:CANVAS-07] "Turn this screenshot into an editable wireframe" (attach PNG)
 * Steps exercised: (1) the screenshot is loaded as the non-interactive
 * background layer and real tools draw editable wireframe shapes over it;
 * (2) selecting and dragging a shape moves only that shape — the background
 * image and the other shapes stay put — and the exported SVG keeps both.
 * Drives real canvas modules 01 (pointer/selection), 04 (backgroundLayerSvg,
 * composeSvg/exportSvg) with the shared canvas stub.
 */
"use strict";

var env = require("./00_env");
var h = env.h;
var canvasStub = require("./00_canvas_stub");

h.test("[UAT:CANVAS-07] overlay editable wireframe shapes on a screenshot and move one", function () {
  var labels = ["Header", "Sidebar", "Card"];
  var prompts = labels.slice();
  var ctx = canvasStub.makeCanvas({
    GBModal: { prompt: function () { return prompts.length ? prompts.shift() : "Shape"; } },
  });
  var app = ctx.app;
  var shot = '<svg xmlns="http://www.w3.org/2000/svg">' +
    '<rect x="0" y="0" width="120" height="80" fill="#94a3b8"/>' +
    '<text x="4" y="12">screenshot</text></svg>';
  var bg = ctx.doc.getElementById("canvas-bg-svg");
  bg.innerHTML = shot;

  // Step 1 - screenshot is the background; real tools draw shapes over it.
  var layer = app.backgroundLayerSvg();
  h.assert(layer.indexOf("#94a3b8") !== -1 && layer.indexOf("screenshot") !== -1,
    "screenshot was not loaded as the background layer: " + layer);
  app.setStrokeColor("#2563eb");
  canvasStub.draw(ctx, "rectangle", [5, 5], [115, 20]);
  canvasStub.draw(ctx, "rectangle", [5, 25], [60, 70]);
  canvasStub.draw(ctx, "text", [8, 28], [8, 28]);

  var rects = app.elements.filter(function (el) { return el.type === "rectangle"; });
  h.assert(rects.length === 2, "expected two wireframe shapes, got " + rects.length);
  var composed = app.composeSvg(true);
  h.assert(composed.indexOf("#94a3b8") !== -1, "composed picture lost the screenshot background");
  h.assert(composed.indexOf("Header") !== -1 && composed.indexOf("#2563eb") !== -1,
    "editable wireframe shapes are missing from the composed picture");
  h.assert(composed.indexOf("screenshot") !== -1, "background text was dropped from the export");

  // Step 2 - dragging one shape leaves the background and its sibling untouched.
  var header = rects[0];
  var card = rects[1];
  var otherBefore = { x: header.x, y: header.y };
  var bgBefore = bg.innerHTML;
  app.selectTool("select");
  app.handleMouseDown({ button: 0, clientX: 30, clientY: 60 });
  h.assert(app.selectedElement === card, "clicking the card shape did not select it");
  app.handleMouseMove({ clientX: 45, clientY: 70 });
  app.handleMouseUp({ clientX: 45, clientY: 70 });
  h.assert(card.x === 20 && card.y === 35,
    "selected shape was not moved to the drag position: " + card.x + "," + card.y);
  h.assert(header.x === otherBefore.x && header.y === otherBefore.y,
    "moving one shape also moved its sibling");
  h.assert(bg.innerHTML === bgBefore, "moving a shape also moved the background image");

  app.exportSvg("screenshot-wireframe", false);
  var svg = ctx.blobs[ctx.blobs.length - 1].parts.join("");
  h.assert(svg.indexOf("#94a3b8") !== -1 && svg.indexOf("screenshot") !== -1,
    "exported wireframe lost the screenshot background");
  h.assert(svg.indexOf('x="20" y="35"') !== -1, "export shows the shape at its old position");
  h.assert(svg.indexOf('x="5" y="5"') !== -1, "export lost the unmoved sibling shape");
});
