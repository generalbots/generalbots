/**
 * [UAT:CANVAS-04] "Open the canvas and draw a red circle with a blue label below it"
 * Steps exercised: (1) a red ellipse and a blue text label are drawn through
 * the real pointer handlers and the ellipse is selected with its fill shown;
 * (2) changing the fill recolours only the selected ellipse.
 * Drives the real canvas modules (01 pointer tools, 02 properties/selection,
 * 03 setFillColor/saveHistory).
 */
"use strict";

var env = require("./00_env");
var h = env.h;
var canvasStub = require("./00_canvas_stub");

h.test("[UAT:CANVAS-04] draw a red circle and label, then recolour only the circle", function () {
  var ctx = canvasStub.makeCanvas({
    GBModal: { prompt: function () { return "Label"; } },
  });
  var app = ctx.app;

  // Step 1 — draw the circle, then a text label below it, using real tools.
  app.setFillColor("#ff0000");
  canvasStub.draw(ctx, "ellipse", [20, 20], [40, 50]);
  app.setStrokeColor("#0000ff");
  canvasStub.draw(ctx, "text", [20, 60], [60, 80]);

  h.assert(app.elements.length === 2, "expected circle + label, got " + app.elements.length + " elements");
  h.assert(app.elements[0].type === "ellipse", "circle was not drawn as an ellipse");
  h.assert(app.elements[0].fill === "#ff0000", "circle is not red: " + app.elements[0].fill);
  h.assert(app.elements[1].type === "text" && app.elements[1].text === "Label", "blue label was not drawn");
  h.assert(app.elements[1].stroke === "#0000ff", "label is not blue: " + app.elements[1].stroke);

  app.selectTool("select");
  app.handleMouseDown({ button: 0, clientX: 30, clientY: 35 });
  h.assert(app.selectedElement === app.elements[0], "clicking the circle did not select it");
  h.assert(
    ctx.doc.getElementById("prop-fill").value === "#ff0000",
    "selected circle's red fill is not shown in the properties panel",
  );

  // Step 2 — changing the fill affects only the selected circle.
  var labelFill = app.elements[1].fill;
  var historyBefore = app.history.length;
  app.setFillColor("#0000ff");

  h.assert(app.elements[0].fill === "#0000ff", "circle was not recoloured blue");
  h.assert(app.elements[1].fill === labelFill, "fill change leaked to the label");
  h.assert(app.history.length === historyBefore + 1, "the fill change was not saved in history");
});
