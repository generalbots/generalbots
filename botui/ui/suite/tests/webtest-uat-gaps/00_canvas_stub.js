/**
 * Shared DOM/browser stubs for the canvas UAT-gap step tests.
 * Loads the real canvas modules 01-04 into one vm context and exposes the
 * live _canvasApp plus capture hooks for Blob-backed SVG/PDF downloads.
 */
"use strict";

var env = require("./00_env");
var h = env.h;

var CANVAS_MODULES = [
  "canvas/modules/01_canvas_state_events.js",
  "canvas/modules/02_canvas_render.js",
  "canvas/modules/03_canvas_tools.js",
  "canvas/modules/04_canvas_export_api.js",
];

/** Make a canvas sandbox whose boot is bypassed (no HTMX root in this shell). */
function makeCanvas(extra) {
  var blobs = [];
  var links = [];
  var rasterCalls = [];
  var doc = h.makeDocument({ readyState: "complete" });
  var originalGet = doc.getElementById;
  doc.getElementById = function (id) {
    // Keep the shipped boot()/mini-map paths out of the way; the test drives
    // the real drawing/undo/export functions directly on the loaded app.
    if (id === "canvas-app-root" || id === "mini-map-canvas") return null;
    return originalGet(id);
  };
  var originalCreate = doc.createElement;
  doc.createElement = function (tag) {
    var el = originalCreate(tag);
    if (String(tag).toLowerCase() === "a") links.push(el);
    return el;
  };

  // A 2D context stand-in: every drawing call is a no-op, assigned state sticks.
  var ctx = new Proxy({}, {
    get: function (target, prop) {
      if (prop in target) return target[prop];
      return function () {};
    },
    set: function (target, prop, value) {
      target[prop] = value;
      return true;
    },
  });

  function FakeBlob(parts, options) {
    this.parts = parts;
    this.type = options && options.type;
  }

  var stubs = {
    addEventListener: function () {},
    removeEventListener: function () {},
    Blob: FakeBlob,
    Uint8Array: Uint8Array,
    atob: function (value) { return Buffer.from(value, "base64").toString("binary"); },
    URL: {
      createObjectURL: function (blob) { blobs.push(blob); return "blob:captured"; },
      revokeObjectURL: function () {},
    },
    GBArtifactExport: {
      rasterise: function (svg, options) {
        rasterCalls.push({ svg: svg, options: options });
        return Promise.resolve({
          width: 120,
          height: 80,
          toDataURL: function () { return "data:image/jpeg;base64,QUJD"; },
        });
      },
    },
  };
  if (extra) Object.keys(extra).forEach(function (key) { stubs[key] = extra[key]; });

  var sandbox = env.baseSandbox(doc, env.makeStore(), stubs);
  // Module 01 adopts the existing window._canvasApp as its private state holder
  // and module 04 later replaces the global with a trimmed toolbar API. Keep a
  // handle on that original object so the tests can drive its real state.
  var app = {};
  sandbox._canvasApp = app;
  env.loadAll(sandbox, CANVAS_MODULES);
  h.assert(!!app, "canvas modules did not initialise _canvasApp");

  var canvas = originalGet("main-canvas");
  canvas.width = 120;
  canvas.height = 80;
  app.canvas = canvas;
  app.ctx = ctx;

  return {
    app: app,
    doc: doc,
    sandbox: sandbox,
    blobs: blobs,
    links: links,
    rasterCalls: rasterCalls,
  };
}

/** Draw one shape through the real pointer handlers (not by mutating elements). */
function draw(ctx, tool, from, to) {
  ctx.app.selectTool(tool);
  ctx.app.handleMouseDown({ button: 0, clientX: from[0], clientY: from[1] });
  ctx.app.handleMouseUp({ clientX: to[0], clientY: to[1] });
}

module.exports = {
  makeCanvas: makeCanvas,
  draw: draw,
};
