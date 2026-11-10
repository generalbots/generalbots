"use strict";
/* Drawing downloads — zero-dependency test (#2144)
 * (run: node tests/chat_artifact_download.test.js)
 *
 * The epic review found that a drawing produced in chat could only leave the
 * app through Save-to-Drive. `js/artifact-download.js` now prepares the markup
 * (xmlns, intrinsic size, animation leftovers, background), rasterises PNG
 * through a canvas at a sane scale and hands the blob to the shared save
 * dialog. The live preview wires that helper to SVG/PNG buttons.
 *
 * The assertions pin the contract:
 *   1. prepare() makes a drawing self-contained and safe to rasterise, even
 *      when the model has not finished streaming it.
 *   2. downloadSvg/downloadPng produce the right file name, mime and pixels.
 *   3. the live preview exposes both buttons and downloads the *current* state.
 */

const fs = require("fs");
const path = require("path");
const vm = require("vm");

const SUITE = path.join(__dirname, "..", "ui", "suite");
const EXPORT_MODULE = path.join(SUITE, "js", "artifact-download.js");
const LIVE_MODULE = path.join(SUITE, "chat", "chat-live-artifact.js");
const ARTIFACTS_MODULE = path.join(SUITE, "chat", "chat-artifacts.js");
const CANVAS_EXPORT_MODULE = path.join(
  SUITE, "canvas", "modules", "04_canvas_export_api.js"
);

let passed = 0;
let failed = 0;

function check(condition, label) {
  if (condition) passed++;
  else {
    failed++;
    console.error("FAIL " + label);
  }
}

/* ── fake DOM / browser ─────────────────────────────────────────────────── */

function fakeCanvas() {
  const canvas = {
    width: 0,
    height: 0,
    ops: [],
    getContext: function () {
      const ops = canvas.ops;
      return {
        fillStyle: "",
        fillRect: function (x, y, w, h) { ops.push({ op: "fill", w: w, h: h }); },
        drawImage: function (img, x, y, w, h) { ops.push({ op: "draw", w: w, h: h }); },
      };
    },
    toBlob: function (callback, mime) {
      callback(new Blob(["png-bytes"], { type: mime }));
    },
    toDataURL: function () { return "data:image/png;base64,AAAA"; },
  };
  return canvas;
}

function fakeDocument() {
  return {
    body: { appendChild: function () {} },
    createElement: function (tag) {
      if (tag === "canvas") return fakeCanvas();
      return {
        tagName: tag,
        style: {},
        attributes: {},
        setAttribute: function (name, value) { this.attributes[name] = value; },
        click: function () {},
        remove: function () {},
      };
    },
  };
}

function loadExportModule() {
  const window = {};
  const saved = [];
  window.GBShared = {
    saveBlob: function (blob, name) {
      saved.push({ blob: blob, name: name });
      return Promise.resolve(name);
    },
  };
  window.GBToasts = { show: function () {} };
  const sandbox = {
    window: window,
    document: fakeDocument(),
    Blob: Blob,
    URL: {
      createObjectURL: function () { return "blob:test"; },
      revokeObjectURL: function () {},
    },
    Image: class {
      set src(value) {
        this._src = value;
        const image = this;
        setTimeout(function () { if (image.onload) image.onload(); }, 0);
      }
      get src() { return this._src; }
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
  return { exporter: window.GBArtifactExport, saved: saved };
}

/* ── tiny DOM for the live-preview wiring ───────────────────────────────── */

class FakeElement {
  constructor(tag) {
    this.tagName = String(tag || "div").toUpperCase();
    this.children = [];
    this.className = "";
    this.textContent = "";
    this.title = "";
    this.attributes = {};
    this.listeners = {};
    this.dataset = {};
    this.hidden = false;
    this.isConnected = true;
    this.outerHTML = "";
  }

  appendChild(child) {
    this.children.push(child);
    child.parentNode = this;
    return child;
  }

  setAttribute(name, value) {
    this.attributes[name] = value;
  }

  getAttribute(name) {
    return this.attributes[name];
  }

  addEventListener(name, handler) {
    if (!this.listeners[name]) this.listeners[name] = [];
    this.listeners[name].push(handler);
  }

  click() {
    (this.listeners.click || []).forEach(function (handler) { handler({}); });
  }

  remove() {
    this.removed = true;
    this.isConnected = false;
  }

  querySelector(selector) {
    return findByClass(this, selector.replace(/^\./, ""));
  }
}

function findByClass(node, className) {
  for (const child of node.children) {
    if (String(child.className || "").split(/\s+/).indexOf(className) >= 0) return child;
    const nested = findByClass(child, className);
    if (nested) return nested;
  }
  return null;
}

function findAllByClass(node, className, found) {
  const list = found || [];
  for (const child of node.children) {
    if (String(child.className || "").split(/\s+/).indexOf(className) >= 0) list.push(child);
    findAllByClass(child, className, list);
  }
  return list;
}

function loadLiveModule() {
  const window = {};
  const saved = [];
  window.GBShared = {
    saveBlob: function (blob, name) {
      saved.push({ blob: blob, name: name });
      return Promise.resolve(name);
    },
  };
  window.GBToasts = { show: function () {} };
  // The real exporter runs inside the same sandbox: the wiring test exercises
  // the actual serialisation, not a stub.
  window.GBLiveSvg = {
    render: function (state, block) {
      state.root = {
        outerHTML: '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10">' +
          '<circle cx="5" cy="5" r="4"/></svg>',
      };
      state.badge.textContent = block.closed ? "1 parts" : "drawing… 1 parts";
    },
  };
  const sandbox = {
    window: window,
    document: {
      createElement: function (tag) {
        if (tag === "canvas") return fakeCanvas();
        if (tag === "a") {
          return {
            attributes: {},
            setAttribute: function () {},
            click: function () {},
            remove: function () {},
          };
        }
        return new FakeElement(tag);
      },
      dispatchEvent: function () {},
      addEventListener: function () {},
      body: { appendChild: function () {} },
    },
    Blob: Blob,
    URL: {
      createObjectURL: function () { return "blob:test"; },
      revokeObjectURL: function () {},
    },
    Image: class {
      set src(value) {
        this._src = value;
        const image = this;
        setTimeout(function () { if (image.onload) image.onload(); }, 0);
      }
      get src() { return this._src; }
    },
    atob: function (value) { return Buffer.from(value, "base64").toString("binary"); },
    CustomEvent: function (name, options) { this.type = name; this.detail = options && options.detail; },
    setInterval: function () { return 1; },
    clearInterval: function () {},
    setTimeout: setTimeout,
    Promise: Promise,
    console: console,
  };
  vm.createContext(sandbox);
  vm.runInContext(fs.readFileSync(EXPORT_MODULE, "utf8"), sandbox, {
    filename: EXPORT_MODULE,
  });
  vm.runInContext(fs.readFileSync(LIVE_MODULE, "utf8"), sandbox, {
    filename: LIVE_MODULE,
  });
  return { live: window.GBLiveArtifact, exporter: window.GBArtifactExport, saved: saved };
}

function flush() {
  return new Promise(function (resolve) { setTimeout(resolve, 25); });
}

/* ── assertions ─────────────────────────────────────────────────────────── */

async function main() {
  const loaded = loadExportModule();
  const exporter = loaded.exporter;
  check(!!exporter, "the export module registers window.GBArtifactExport");

  // 1. A bare drawing becomes a self-contained document.
  const bare = exporter.prepare('<svg viewBox="0 0 512 512"><circle r="10"/></svg>', {
    background: null,
  });
  check(bare.indexOf('xmlns="http://www.w3.org/2000/svg"') !== -1,
    "prepare() adds the SVG namespace");
  check(bare.indexOf('width="512"') !== -1 && bare.indexOf('height="512"') !== -1,
    "prepare() derives width/height from the viewBox");

  // 2. A drawing with pixels gets a viewBox-friendly size, not a stretch.
  const sized = exporter.prepare('<svg width="320" height="200"></svg>', { background: null });
  check(sized.indexOf('width="320"') !== -1 && sized.indexOf('height="200"') !== -1,
    "prepare() keeps explicit pixel sizes");
  const percent = exporter.prepare(
    '<svg width="100%" height="100%" viewBox="0 0 240 120"></svg>', { background: null }
  );
  check(percent.indexOf('width="240"') !== -1 && percent.indexOf('height="120"') !== -1,
    "prepare() replaces percentage sizes with the viewBox extent");

  // 3. Background is painted by default and can be suppressed.
  const withBg = exporter.prepare('<svg viewBox="0 0 10 10"></svg>');
  check(/<rect x="0" y="0" width="100%" height="100%" fill="#ffffff"\/>/.test(withBg),
    "prepare() paints the preview background by default");
  const transparent = exporter.prepare('<svg viewBox="0 0 10 10"></svg>', { background: null });
  check(transparent.indexOf("<rect") === -1, "background: null keeps the drawing transparent");

  // 4. The draw-on animation must not leak into the file.
  const animated = exporter.prepare(
    '<svg viewBox="0 0 10 10"><path style="stroke-dasharray: 42 42; stroke-dashoffset: 42; transform-box: fill-box; fill: red"/></svg>',
    { background: null }
  );
  check(animated.indexOf("stroke-dasharray") === -1 && animated.indexOf("stroke-dashoffset") === -1,
    "prepare() removes the draw-on animation leftovers");
  check(animated.indexOf("fill: red") !== -1, "prepare() keeps authored styles");
  check(animated.indexOf("transform-box") === -1, "prepare() drops the animation transform box");

  // 5. Streaming: an unclosed drawing still exports what has arrived.
  const streaming = exporter.prepare(
    '<svg viewBox="0 0 100 100"><circle cx="50" cy="50" r="20"/>', { background: null }
  );
  check(streaming.indexOf("<circle") !== -1, "an unfinished drawing keeps the shapes drawn so far");
  check(streaming.indexOf('width="100"') !== -1, "an unfinished drawing still carries a size");

  // 6. Sizing helper reads pixels before the viewBox.
  const dims = exporter.dimensions('<svg width="200" height="100" viewBox="0 0 10 10"></svg>');
  check(dims.width === 200 && dims.height === 100, "dimensions() prefers explicit pixels");
  const boxed = exporter.dimensions('<svg viewBox="0 0 640 480"></svg>');
  check(boxed.width === 640 && boxed.height === 480, "dimensions() falls back to the viewBox");

  // 7. File names are slugs with the right extension.
  check(exporter.fileName("Meu Relatório de Vendas", "svg") === "meu-relatorio-de-vendas.svg",
    "fileName() transliterates and slugifies");
  check(exporter.fileName("", "png") === "drawing.png", "fileName() falls back to 'drawing'");

  // 8. downloadSvg writes the prepared markup.
  await exporter.downloadSvg("robot", '<svg viewBox="0 0 10 10"><rect/></svg>');
  const svgSave = loaded.saved[loaded.saved.length - 1];
  check(svgSave.name === "robot.svg", "downloadSvg() names the file .svg");
  const svgText = await svgSave.blob.text();
  check(svgText.indexOf('xmlns="http://www.w3.org/2000/svg"') !== -1,
    "downloadSvg() writes a self-contained SVG");

  // 9. downloadPng rasterises at 2x through a canvas element.
  const before = loaded.saved.length;
  await exporter.downloadPng("robot", '<svg viewBox="0 0 512 512"><circle r="10"/></svg>');
  const pngSave = loaded.saved[loaded.saved.length - 1];
  check(loaded.saved.length === before + 1, "downloadPng() saves one file");
  check(pngSave.name === "robot.png", "downloadPng() names the file .png");
  check(pngSave.blob.type === "image/png", "downloadPng() writes a PNG blob");
  const drawn = pngSave.blob && pngSave.blob.size > 0;
  check(drawn, "downloadPng() produced bytes");

  // The raster canvas is not exposed by downloadPng(), so pin the scale and
  // the safety cap through rasterise() directly.
  const capture = loadExportModule();
  const capped = await capture.exporter.rasterise('<svg viewBox="0 0 4000 4000"></svg>', { scale: 2 });
  check(capped.width === 4096 && capped.height === 4096,
    "rasterise() caps the canvas side at 4096px");
  const huge = await capture.exporter.rasterise('<svg viewBox="0 0 10000 10000"></svg>', { scale: 2 });
  check(huge.width === 4096 && huge.height === 4096,
    "rasterise() shrinks a drawing larger than the cap instead of ignoring it");

  const scaled = await capture.exporter.rasterise('<svg viewBox="0 0 512 512"></svg>', { scale: 2 });
  check(scaled.width === 1024 && scaled.height === 1024, "rasterise() honours the 2x scale");
  check(scaled.ops.length === 1 && scaled.ops[0].op === "draw",
    "rasterise() draws the image once");

  // 10. A non-drawing is rejected instead of producing an empty file.
  let rejected = false;
  await exporter.downloadSvg("nope", "just prose").catch(function () { rejected = true; });
  check(rejected, "downloadSvg() refuses markup that is not a drawing");

  // 11. Live preview wiring: both buttons exist and download the current state.
  const live = loadLiveModule();
  check(!!live.live, "the live artifact module loads");
  check(!!live.exporter, "the live preview runs against the real exporter");
  const container = new FakeElement("div");
  live.live.render(container, '```svg\n<svg viewBox="0 0 10 10"><circle r="1"/></svg>\n```');
  const buttons = findAllByClass(container, "gb-live-artifact-download");
  check(buttons.length === 2, "the live preview offers an SVG and a PNG download");
  check(buttons[0].hidden === false && buttons[1].hidden === false,
    "download buttons are visible for a drawing");
  buttons[0].click();
  buttons[1].click();
  await flush();
  check(live.saved.length === 2, "both buttons trigger a download");
  const liveSvg = live.saved[0];
  const livePng = live.saved[1];
  check(/^drawing-\d{8}-\d{4}\.svg$/.test(liveSvg.name),
    "the live SVG download is named after the moment it was taken: " + liveSvg.name);
  check(/^drawing-\d{8}-\d{4}\.png$/.test(livePng.name),
    "the live PNG download is named after the moment it was taken: " + livePng.name);
  check(livePng.blob.type === "image/png", "the live PNG download is a PNG blob");
  const liveSvgText = await liveSvg.blob.text();
  check(liveSvgText.indexOf("<circle") !== -1,
    "the live download serialises the drawing drawn so far");
  check(liveSvgText.indexOf('xmlns="http://www.w3.org/2000/svg"') !== -1,
    "the live download is a self-contained SVG");

  // 12. An HTML artifact has no SVG to export: the buttons hide.
  const htmlContainer = new FakeElement("div");
  live.live.render(htmlContainer, "```html\n<!doctype html><html><body>hi</body></html>\n```");
  const htmlButtons = findAllByClass(htmlContainer, "gb-live-artifact-download");
  check(htmlButtons.length === 2 && htmlButtons.every(function (b) { return b.hidden; }),
    "download buttons are hidden for a generated HTML app");

  // 13. The final artifact panel and the canvas export offer the same actions.
  const artifacts = fs.readFileSync(ARTIFACTS_MODULE, "utf8");
  check(artifacts.indexOf("gb-artifact-download-svg") !== -1 &&
    artifacts.indexOf("gb-artifact-download-png") !== -1,
  "the artifact panel offers SVG and PNG downloads");
  check(artifacts.indexOf("downloadText(") !== -1,
    "the artifact panel downloads text artifacts too");
  check(artifacts.indexOf("GBArtifactExport.download(") !== -1,
    "the artifact panel uses the shared exporter");

  const canvasExport = fs.readFileSync(CANVAS_EXPORT_MODULE, "utf8");
  check(canvasExport.indexOf("backgroundLayerSvg") !== -1,
    "the canvas export serialises the background layer");
  check(/composeSvg\(bg\)/.test(canvasExport),
    "PNG/PDF export the composed document, not just the bitmap");
  check(canvasExport.indexOf("pngFromBitmap") !== -1,
    "the bitmap export stays as a fallback");

  console.log("chat_artifact_download: " + passed + " passed, " + failed + " failed");
  if (failed) process.exit(1);
}

main().catch(function (error) {
  console.error(error);
  process.exit(1);
});
