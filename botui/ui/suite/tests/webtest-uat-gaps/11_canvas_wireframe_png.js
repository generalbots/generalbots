/**
 * [UAT:CANVAS-01] "Gere um wireframe de site de restaurante em canvas"
 * Steps exercised: (2) the drawn blocks are selectable and movable through the
 * real pointer handlers; (3) PNG export rasterises the composed picture at 1x
 * and 2x and downloads both files.
 * Step 1 (the chat phrase producing the wireframe) is LLM-driven; the blocks
 * themselves are drawn here with the shipped canvas tools.
 * Drives the real canvas modules (01 pointer/selection, 04 doExport/composeSvg).
 */
"use strict";

var env = require("./00_env");
var h = env.h;
var canvasStub = require("./00_canvas_stub");

function flush() {
  return new Promise(function (resolve) { setTimeout(resolve, 0); });
}

h.test("[UAT:CANVAS-01] move wireframe blocks and export PNG at 1x and 2x", function () {
  var rasterCalls = [];
  var blobs = [];
  var saved = [];
  var exporter = {
    rasterise: function (svg, options) {
      rasterCalls.push({ svg: svg, options: options });
      return Promise.resolve({
        width: 120,
        height: 80,
        toDataURL: function () { return "data:image/jpeg;base64,QUJD"; },
      });
    },
    toBlob: function (out, type) {
      var blob = { type: type, parts: [out.toDataURL()] };
      blobs.push(blob);
      return Promise.resolve(blob);
    },
    saveBlob: function (blob, name) {
      saved.push({ type: blob.type, name: name });
      return Promise.resolve();
    },
  };
  var labels = ["Hero", "Menu", "Contact"];
  var ctx = canvasStub.makeCanvas({
    GBArtifactExport: exporter,
    GBModal: {
      prompt: function () {
        return labels.length ? labels.shift() : "Block";
      },
    },
  });
  var app = ctx.app;

  // Three hero/menu/contact blocks plus their labels, all drawn with tools.
  app.setFillColor("#fff3c4");
  canvasStub.draw(ctx, "rectangle", [10, 5], [110, 25]);
  canvasStub.draw(ctx, "rectangle", [10, 30], [110, 55]);
  canvasStub.draw(ctx, "rectangle", [10, 60], [110, 78]);
  app.setStrokeColor("#222222");
  canvasStub.draw(ctx, "text", [12, 8], [12, 8]);
  canvasStub.draw(ctx, "text", [12, 33], [12, 33]);
  canvasStub.draw(ctx, "text", [12, 63], [12, 63]);

  var blocks = app.elements.filter(function (el) { return el.type === "rectangle"; });
  var texts = app.elements.filter(function (el) { return el.type === "text"; });
  h.assert(blocks.length === 3, "expected three wireframe blocks, got " + blocks.length);
  h.assert(texts.length === 3, "expected three block labels, got " + texts.length);
  h.assert(
    texts.map(function (t) { return t.text; }).join(",") === "Hero,Menu,Contact",
    "block labels are missing: " + JSON.stringify(texts.map(function (t) { return t.text; })),
  );
  h.assert(
    blocks[0].y < blocks[1].y && blocks[1].y < blocks[2].y,
    "hero/menu/contact blocks were not laid out top to bottom",
  );

  // Step 2 - select the hero block at a point clear of its label, then drag it.
  var startX = blocks[0].x;
  var startY = blocks[0].y;
  app.selectTool("select");
  app.handleMouseDown({ button: 0, clientX: 90, clientY: 20 });
  h.assert(app.selectedElement === blocks[0], "clicking the hero block did not select it");
  h.assert(
    String(ctx.doc.getElementById("prop-x").value) === String(Math.round(startX)),
    "properties panel does not show the selected block",
  );
  app.handleMouseMove({ clientX: 100, clientY: 28 });
  h.assert(blocks[0].x === startX + 10, "horizontal drag did not move the selected block");
  h.assert(blocks[0].y === startY + 8, "vertical drag did not move the selected block");

  // Step 3 - export the same picture at 1x and at 2x.
  function exportAt(scale) {
    ctx.doc.getElementById("export-format").value = "png";
    ctx.doc.getElementById("export-scale").value = String(scale);
    ctx.doc.getElementById("export-bg").checked = true;
    ctx.doc.getElementById("canvas-name").value = "wireframe";
    app.doExport();
  }
  exportAt(1);
  exportAt(2);
  return flush().then(function () {
    return flush();
  }).then(function () {
    h.assert(rasterCalls.length === 2, "expected two PNG rasterisations, got " + rasterCalls.length);
    h.assert(rasterCalls[0].options.scale === 1, "first export is not 1x: " + rasterCalls[0].options.scale);
    h.assert(rasterCalls[1].options.scale === 2, "second export is not 2x: " + rasterCalls[1].options.scale);
    ["Hero", "Menu", "Contact"].forEach(function (label) {
      h.assert(rasterCalls[1].svg.indexOf(label) !== -1, "exported PNG lost the " + label + " block");
    });
    h.assert(saved.length === 2, "expected two PNG downloads, got " + saved.length);
    h.assert(
      saved.every(function (s) { return s.name === "wireframe.png" && s.type === "image/png"; }),
      "PNG downloads are missing or misnamed: " + JSON.stringify(saved),
    );
    h.assert(blobs.length === 2, "no PNG blobs were created");
  });
});
