/**
 * [UAT:CANVAS-10] "Undo the last three changes and export the canvas as PDF"
 * Steps exercised: (1) three undos unwind three real edits in order;
 * (2) the exported PDF is rasterised from the SVG of what is on screen.
 * Drives the real canvas modules (01 drawing, 03 undo/saveHistory,
 * 04 composeSvg/exportPdf/writePdf) with Blob/rasteriser stubs.
 */
"use strict";

var env = require("./00_env");
var h = env.h;
var canvasStub = require("./00_canvas_stub");

function flush() {
  return new Promise(function (resolve) { setTimeout(resolve, 0); });
}

h.test("[UAT:CANVAS-10] undo three edits and export the resulting canvas as PDF", function () {
  var ctx = canvasStub.makeCanvas();
  var app = ctx.app;

  // One drawn circle is the baseline; the next three actions are real edits.
  app.setFillColor("#00ff00");
  canvasStub.draw(ctx, "ellipse", [10, 10], [50, 50]);
  h.assert(app.history.length === 1, "baseline was not saved in history");
  app.selectedElement = app.elements[0];

  app.setFillColor("#ff0000");                 // edit 1: recolour
  app.duplicateSelected();                     // edit 2: duplicate
  app.setStrokeColor("#0000ff");               // edit 3: restroke
  h.assert(app.history.length === 4, "expected 4 history snapshots, got " + app.history.length);

  // Step 1 — undo must walk back through the three edits, newest first.
  app.undo();
  h.assert(app.elements.length === 2, "first undo did not remove the restroke/duplicate snapshot");
  h.assert(app.elements[0].stroke === "#000000", "first undo did not restore the black stroke");

  app.undo();
  h.assert(app.elements.length === 1, "second undo did not remove the duplicate");
  h.assert(app.elements[0].fill === "#ff0000", "second undo lost the red fill");

  app.undo();
  h.assert(app.elements.length === 1, "third undo should leave the single baseline circle");
  h.assert(app.elements[0].fill === "#00ff00", "third undo did not restore the green circle");
  h.assert(app.historyIndex === 0, "undo did not return to the baseline snapshot");
  h.assert(app.selectedElement === null, "undo left a stale selection");

  // Step 2 — the PDF is built from the current on-screen picture.
  app.exportPdf("undo-check", false);
  return flush().then(function () {
    h.assert(ctx.rasterCalls.length === 1, "PDF export did not rasterise the composed SVG");
    var svg = ctx.rasterCalls[0].svg;
    h.assert(svg.indexOf("<ellipse") !== -1, "PDF source lost the drawn ellipse: " + svg);
    h.assert(svg.indexOf("#00ff00") !== -1, "PDF source is not the post-undo picture: " + svg);
    h.assert(svg.indexOf("#ff0000") === -1, "PDF source still has the undone red fill: " + svg);

    var pdf = ctx.blobs.filter(function (b) { return b.type === "application/pdf"; })[0];
    h.assert(!!pdf, "no PDF blob was produced: " + JSON.stringify(ctx.blobs.map(function (b) { return b.type; })));
    var content = pdf.parts.join("");
    h.assert(content.indexOf("%PDF-1.3") === 0, "blob is not a PDF: " + content.slice(0, 20));
    h.assert(content.indexOf("/MediaBox [0 0 120 80]") !== -1, "PDF page size does not match the canvas");

    var link = ctx.links[ctx.links.length - 1];
    h.assert(link && link.download === "undo-check.pdf", "PDF was not offered under the expected name");
  });
});
