/**
 * [UAT:CANVAS-02] "Design a landing page mockup and give me a .fig file"
 * Steps exercised: (1) a framed landing-page mockup is drawn with the real
 * frame/rectangle/text tools and split across two real layers; (2) the design
 * is exported through the shipped design-file (JSON) path — a structured
 * document, not a renamed image — and re-imported, restoring frames/layers.
 *
 * The binary .fig writer itself lives botserver-side (canvas.export command),
 * so offline it is the shipped design-file round-trip that is exercised:
 * frame draw -> structured export -> re-import restores frames and layers.
 * Drives real canvas modules 01 (pointer tools), 03 (addLayer) and
 * 04 (doExport/setCanvasElements) with the shared canvas stub.
 */
"use strict";

var env = require("./00_env");
var h = env.h;
var canvasStub = require("./00_canvas_stub");

h.test("[UAT:CANVAS-02] frame a landing page, export a structured design file and re-import it", function () {
  var labels = ["Hero", "Features", "Pricing", "Contact"];
  var ctx = canvasStub.makeCanvas({
    GBModal: {
      prompt: function () { return labels.length ? labels.shift() : "Block"; },
    },
  });
  var app = ctx.app;

  // Step 1 - frames and blocks drawn through the real pointer handlers.
  app.setFillColor("#f8fafc");
  canvasStub.draw(ctx, "frame", [5, 5], [115, 30]);
  canvasStub.draw(ctx, "frame", [5, 40], [115, 70]);
  app.setFillColor("#dbeafe");
  canvasStub.draw(ctx, "rectangle", [10, 10], [110, 25]);
  app.addLayer();
  canvasStub.draw(ctx, "rectangle", [10, 45], [110, 65]);
  app.setStrokeColor("#0f172a");
  canvasStub.draw(ctx, "text", [12, 12], [12, 12]);
  canvasStub.draw(ctx, "text", [12, 47], [12, 47]);

  var frames = app.elements.filter(function (el) { return el.type === "frame"; });
  var layerIds = app.elements.map(function (el) { return el.layer; });
  h.assert(frames.length === 2, "expected two landing-page frames, got " + frames.length);
  h.assert(
    frames[0].y < frames[1].y && frames[0].width === 110 && frames[1].width === 110,
    "frames were not drawn in the design canvas",
  );
  h.assert(app.layers.length === 2, "expected two layers, got " + app.layers.length);
  h.assert(layerIds.indexOf("layer_1") !== -1 && layerIds.indexOf("layer_2") !== -1,
    "blocks were not split across the two layers");

  // Step 2 - the shipped design-file export is structured JSON, then re-imported.
  ctx.doc.getElementById("export-format").value = "json";
  ctx.doc.getElementById("canvas-name").value = "landing";
  app.doExport();
  var blob = ctx.blobs[ctx.blobs.length - 1];
  h.assert(!!blob && blob.type === "application/json",
    "design file was not exported as structured JSON: " + (blob && blob.type));
  var design = JSON.parse(blob.parts.join(""));
  h.assert(Array.isArray(design.elements) && design.elements.length === 6,
    "exported design file lost elements: " + JSON.stringify(Object.keys(design)));
  var exportedFrames = design.elements.filter(function (el) { return el.type === "frame"; });
  h.assert(exportedFrames.length === 2, "exported design file lost the frames");
  h.assert(blob.parts.join("").indexOf("QUJD") === -1,
    "design file is a renamed image instead of a structured document");

  var original = JSON.stringify(design.elements);
  ctx.sandbox.setCanvasElements(design.elements);
  h.assert(JSON.stringify(app.elements) === original, "re-import did not restore the design");
  var restoredFrames = app.elements.filter(function (el) { return el.type === "frame"; });
  h.assert(restoredFrames.length === 2 && restoredFrames[0].width === 110,
    "re-import did not restore the frames");
  var restoredLayers = app.elements.map(function (el) { return el.layer; });
  h.assert(restoredLayers.indexOf("layer_2") !== -1, "re-import did not restore the layers");

  var link = ctx.links[ctx.links.length - 1];
  h.assert(link && link.download === "landing.json", "design file was not offered for download");
});
