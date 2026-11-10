"use strict";
/* Canvas export includes the background layer — zero-dependency test (#2144)
 * (run: node tests/canvas_export_layer.test.js)
 *
 * A drawing sent from chat to Canvas lives in `#canvas-bg-svg`, a DOM layer
 * *under* the bitmap. The SVG/PNG/PDF exporters used to read only `C.elements`,
 * so exporting a canvas that showed a chat drawing produced a blank picture.
 * `C.composeSvg()` now serialises that layer into the exported document,
 * stretched to the canvas box exactly like the CSS layer.
 */

const fs = require("fs");
const path = require("path");
const vm = require("vm");

const SUITE = path.join(__dirname, "..", "ui", "suite");
const EXPORT_MODULE = path.join(SUITE, "js", "artifact-download.js");
const CANVAS_EXPORT_MODULE = path.join(
  SUITE, "canvas", "modules", "04_canvas_export_api.js"
);

const LAYER_SVG = '<svg viewBox="0 0 200 100"><circle cx="50" cy="50" r="20"/></svg>';

let passed = 0;
let failed = 0;

function check(condition, label) {
  if (condition) passed++;
  else {
    failed++;
    console.error("FAIL " + label);
  }
}

function loadCanvasExport(layer) {
  const window = {};
  const blobs = [];
  const links = [];
  const C = (window._canvasApp = {
    canvas: { width: 800, height: 600 },
    elements: [],
    layers: [{ id: "layer_1", hidden: false }],
    canvasBg: "#ffffff",
    strokeColor: "#000000",
    strokeWidth: 2,
    boot: function () {},
  });
  const sandbox = {
    window: window,
    document: {
      getElementById: function (id) {
        if (id !== "canvas-bg-svg") return null;
        if (!layer) return { querySelector: function () { return null; }, innerHTML: "" };
        return {
          querySelector: function (selector) {
            return selector === "svg" ? { outerHTML: layer } : null;
          },
          innerHTML: layer,
        };
      },
      createElement: function (tag) {
        if (tag !== "a") return { getContext: function () { return null; } };
        const link = {
          click: function () { links.push(link); },
          download: "",
          href: "",
        };
        return link;
      },
    },
    Blob: Blob,
    URL: {
      createObjectURL: function (blob) {
        blobs.push(blob);
        return "blob:test";
      },
      revokeObjectURL: function () {},
    },
    atob: function (value) { return Buffer.from(value, "base64").toString("binary"); },
    setTimeout: setTimeout,
    Promise: Promise,
    console: console,
  };
  vm.createContext(sandbox);
  vm.runInContext(fs.readFileSync(EXPORT_MODULE, "utf8"), sandbox, {
    filename: EXPORT_MODULE,
  });
  vm.runInContext(fs.readFileSync(CANVAS_EXPORT_MODULE, "utf8"), sandbox, {
    filename: CANVAS_EXPORT_MODULE,
  });
  return { C: C, blobs: blobs, links: links };
}

async function main() {
  const loaded = loadCanvasExport(LAYER_SVG);
  const C = loaded.C;
  check(typeof C.composeSvg === "function", "the canvas export composes an SVG document");
  check(typeof C.backgroundLayerSvg === "function", "the background layer is serialised");

  // 1. The layer is stretched to the canvas box, like the CSS layer.
  const layer = C.backgroundLayerSvg();
  check(layer.indexOf("<circle") !== -1, "the background drawing reaches the export");
  check(layer.indexOf('width="100%"') !== -1 && layer.indexOf('height="100%"') !== -1,
    "the layer fills the canvas box instead of keeping its own pixel size");
  check(layer.indexOf('xmlns="http://www.w3.org/2000/svg"') !== -1,
    "the layer is a self-contained SVG");

  // 2. The composed document carries the canvas size, the background and the
  //    layer, in that order.
  const composed = C.composeSvg(true);
  check(composed.indexOf('width="800"') !== -1 && composed.indexOf('height="600"') !== -1,
    "the composed document uses the canvas size");
  const background = composed.indexOf('fill="#ffffff"');
  const drawing = composed.indexOf("<circle");
  check(background !== -1, "the background fill is exported when asked for");
  check(drawing > background, "the drawing sits on top of the background");

  // 3. Without the background option the drawing still exports.
  const transparent = C.composeSvg(false);
  check(transparent.indexOf("<circle") !== -1, "the layer survives without the background fill");
  check(transparent.indexOf('fill="#ffffff"') === -1, "no fill rect when the option is off");

  // 4. Elements drawn on the canvas still come after the layer.
  C.elements = [{ type: "rectangle", x: 10, y: 10, width: 20, height: 20, stroke: "#ff0000" }];
  const withElements = C.composeSvg(true);
  check(withElements.indexOf("<circle") < withElements.indexOf('stroke="#ff0000"'),
    "canvas elements are drawn above the imported drawing");

  // 5. The SVG download is the composed document.
  C.exportSvg("board", true);
  check(loaded.links.length === 1 && loaded.links[0].download === "board.svg",
    "exportSvg() downloads a .svg file");
  const svgText = await loaded.blobs[loaded.blobs.length - 1].text();
  check(svgText.indexOf("<circle") !== -1, "the exported SVG contains the background layer");

  // 6. An empty layer adds nothing (no stray empty <svg> in the export).
  const empty = loadCanvasExport("");
  check(empty.C.backgroundLayerSvg() === "", "an empty background layer is skipped");
  check(empty.C.composeSvg(true).indexOf("<svg width=\"100%\"") === -1,
    "no empty nested SVG is emitted");

  // 7. The bitmap path is kept as the fallback for a tainted/unsupported raster.
  check(typeof C.pngFromBitmap === "function", "the bitmap PNG export stays available");
  check(typeof C.writePdf === "function", "the PDF writer is shared by both paths");

  console.log("canvas_export_layer: " + passed + " passed, " + failed + " failed");
  if (failed) process.exit(1);
}

main().catch(function (error) {
  console.error(error);
  process.exit(1);
});
