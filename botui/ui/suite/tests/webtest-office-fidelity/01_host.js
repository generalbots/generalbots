/**
 * Deterministic Node host for the real shipped office export IIFEs.
 *
 * Modules 01 (CSV base), 02 (XLSX/store-ZIP) and 04 (DOCX/PPTX/Markdown) are
 * loaded UNMODIFIED into one vm sandbox. The stubs only emulate the browser
 * surface those modules touch (window/document/Blob/URL/clock/timer). Nothing
 * here reimplements production export logic; fixtures merely feed the helpers
 * and observe the bytes they emit.
 */
"use strict";

var h = require("./00_harness");

var MODULES = [
  "js/modules-export/01_csv_json_html_pdf.js",
  "js/modules-export/02_xlsx.js",
  "js/modules-export/04_docx_pptx_md.js",
  "drive/modules/08_preview_parse.js",
  "docs/docs-sanitize.js",
];

var FIXED_NOW = Date.UTC(2026, 0, 15, 12, 0, 0);

function FixedDate() {}
FixedDate.now = function () { return FIXED_NOW; };
FixedDate.prototype.toISOString = function () { return "2026-01-15T12:00:00.000Z"; };
FixedDate.prototype.getHours = function () { return 12; };
FixedDate.prototype.getMinutes = function () { return 0; };
FixedDate.prototype.getSeconds = function () { return 0; };
FixedDate.prototype.getFullYear = function () { return 2026; };
FixedDate.prototype.getMonth = function () { return 0; };
FixedDate.prototype.getDate = function () { return 15; };

function FakeBlob(parts, options) {
  this.parts = parts || [];
  this.type = (options && options.type) || "";
  this.size = this.parts.reduce(function (total, part) {
    return total + (part && part.length !== undefined ? part.length : String(part).length);
  }, 0);
}

/** Blob constructor that records every object the helpers create. */
function observingBlob(sink) {
  function Observed(parts, options) {
    var blob = new FakeBlob(parts, options);
    sink.push(blob);
    return blob;
  }
  Observed.prototype = FakeBlob.prototype;
  return Observed;
}

/** First byte-bearing part of a FakeBlob (exports always pass a Uint8Array). */
function blobBytes(blob) {
  for (var i = 0; i < blob.parts.length; i++) {
    if (blob.parts[i] && blob.parts[i].length !== undefined &&
        typeof blob.parts[i].subarray === "function") {
      return blob.parts[i];
    }
  }
  return null;
}

function makeDocument(ctx) {
  var body = {
    childNodes: [],
    appendChild: function (node) { body.childNodes.push(node); return node; },
    removeChild: function (node) {
      var i = body.childNodes.indexOf(node);
      if (i >= 0) body.childNodes.splice(i, 1);
      return node;
    },
  };
  return {
    body: body,
    createElement: function (tag) {
      return { tagName: String(tag).toUpperCase(), href: "", download: "",
        click: function () { ctx.clicks.push(this); } };
    },
  };
}

/** Fresh vm context with the real modules already executed. */
function create() {
  var ctx = { blobs: [], clicks: [], timers: [] };
  var win = { crcTable: null };
  var doc = makeDocument(ctx);
  var sandbox = {
    window: win,
    document: doc,
    Blob: observingBlob(ctx.blobs),
    Date: FixedDate,
    TextEncoder: TextEncoder,
    URL: {
      createObjectURL: function () { return "blob:offline"; },
      revokeObjectURL: function () {},
    },
    setTimeout: function (fn) { ctx.timers.push(fn); return ctx.timers.length; },
    clearTimeout: function () {},
    gbPrintHtml: function () {},
    console: { log: function () {}, warn: function () {}, error: function () {} },
  };
  ctx.sandbox = sandbox;
  ctx.win = win;
  ctx.doc = doc;
  ctx.ctx = h.loadModules(MODULES, sandbox);
  ctx.XLSX = win.ExportImportXLSX;
  ctx.Docs = win.ExportImportDocs;
  ctx.EI = win.ExportImportCSV;
  ctx.Preview = win.DrivePreviewParse;
  ctx.Sanitize = win.DocsSanitize;
  return ctx;
}

/** Minimal grid stub understood by the shipped readGridAsArray(). */
function makeGrid(rows) {
  var cells = [];
  var width = rows.reduce(function (max, row) { return Math.max(max, row.length); }, 0);
  rows.forEach(function (row, r) {
    row.forEach(function (value, c) {
      cells.push({ dataset: { row: String(r), col: String(c) }, textContent: String(value) });
    });
  });
  return {
    querySelectorAll: function (selector) {
      return selector === "[data-row][data-col]" ? cells.slice() : [];
    },
    querySelector: function (selector) {
      var m = /\[data-row='(\d+)'\]\[data-col='(\d+)'\]/.exec(selector);
      if (!m) return null;
      var index = Number(m[1]) * width + Number(m[2]);
      return cells[index] || null;
    },
  };
}

/** Minimal contenteditable stub understood by exportDOCX/exportMarkdown. */
function makeDocElement(blocks) {
  var nodes = blocks.map(function (block) {
    return {
      nodeType: 1,
      tagName: String(block.tag || "p").toUpperCase(),
      textContent: String(block.text || ""),
      childNodes: [{ nodeType: 3, textContent: String(block.text || "") }],
      children: [],
      getAttribute: function (name) {
        return name === "href" ? (block.href || "") : null;
      },
    };
  });
  var element = {
    nodeType: 1,
    tagName: "ARTICLE",
    childNodes: nodes.slice(),
    children: nodes.slice(),
    textContent: nodes.map(function (n) { return n.textContent; }).join(" "),
    querySelectorAll: function () { return nodes.slice(); },
    getAttribute: function () { return null; },
  };
  element.outerHTML = "<article>" + nodes.map(function (n) {
    return "<" + n.tagName.toLowerCase() + ">" + n.textContent + "</" + n.tagName.toLowerCase() + ">";
  }).join("") + "</article>";
  return element;
}

module.exports = {
  MODULES: MODULES, FIXED_NOW: FIXED_NOW,
  create: create, makeGrid: makeGrid, makeDocElement: makeDocElement,
  blobBytes: blobBytes, FakeBlob: FakeBlob,
};
