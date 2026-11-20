/**
 * Issue #2509 criteria that are NOT shipped. These are honest CURRENT-STATE
 * characterizations: [BASELINE-OBSERVE] executes shipped code to show what it
 * really does; [STATIC] reads shipped sources. A PASS here means the gap is
 * still recorded accurately — it is never a claim that OCR, scanning, image
 * cleanup or table extraction work.
 */
"use strict";

var h = require("./00_harness");
var g = h.loadModules();
var C = g.core;

var OCR_RS = "botserver/src/core/bot/pipeline/turn_hooks/text_recognition.rs";
var ATTACH_RS = "botserver/src/core/bot/pipeline/turn_hooks/attachments.rs";
var PDF_RS = "botserver/crates/botdocs/src/import_pdf.rs";
var BARCODE_RS = "botserver/crates/botbasic_data/src/keywords/products.rs";
var BATCH_JS = "sheet/js/modules-sheet-advanced/63_ingest_batch.js";
var PREVIEW_JS = "sheet/js/modules-sheet-advanced/60_ingest_preview.js";
var POLICY_JS = "sheet/js/modules-sheet-advanced/64_ingest_policy.js";
var SIM_JS = "sheet/js/modules-sheet-advanced/59a_ingest_sim.js";

function allSources() {
  return h.MODULES.concat([BATCH_JS, PREVIEW_JS, POLICY_JS]).map(h.read).join("\n");
}

h.observe("parse creation needs no document bytes: any filename yields a fixture", function () {
  g.reset();
  C.mutate(function (target) { g.sim.ensureParse(target, "unknown-bytes.bin"); });
  var parse = C.state().parses["unknown-bytes.bin"];
  h.assert(Boolean(parse));
  h.equal(parse.pages, 3);
  h.equal(parse.rows[0][0].value, "Date");
  h.deep(Object.keys(parse), ["name", "kind", "pages", "columns", "rows", "structures"]);
});

h.observe("the review queue contains only the three fixture cells below threshold", function () {
  g.reset();
  g.seed();
  var parse = C.state().parses.stmt;
  var cells = g.review.reviewableCells(parse, "stmt");
  var low = cells.filter(function (item) { return C.isLow(item.cell); });
  h.equal(cells.length, 3);
  h.equal(low.length, 3);
  h.deep(low.map(function (item) { return C.cellLabel(parse, item.row, item.col); }),
    ["C3", "C5", "E9"]);
  h.assert(cells.every(function (item) { return item.cell.confidence < C.THRESHOLD; }));
});

h.observe("table structures are a three-item literal, never derived from pixels", function () {
  g.reset();
  g.seed();
  var parse = C.state().parses.stmt;
  h.equal(parse.structures.length, 3);
  h.deep(parse.structures.map(function (item) { return item.kind; }),
    ["subtotal", "merged", "repeated-header"]);
  var preview = g.review.buildPreview(C.state(), "stmt");
  h.deep(Object.keys(preview), ["fileId", "rows", "createdAt"]);
});

h.observe("confidence and page/region are fixture metadata, not engine output", function () {
  g.reset();
  g.seed();
  var cell = C.cellAt(C.state().parses.stmt, 2, 2);
  h.deep(Object.keys(cell), ["value", "confidence", "page", "region", "unreadable"]);
  h.equal(cell.confidence, 0.62);
  h.deep(Object.keys(cell.region), ["x", "y", "w", "h"]);
});

h.observe("batch state is four hardcoded files with simulated statuses", function () {
  g.reset();
  var files = C.state().files;
  h.deep(files.map(function (file) { return file.name; }), [
    "march-statement.pdf", "invoice-batch.pdf", "receipt-photo.png", "corrupt-scan.pdf",
  ]);
  h.assert(files.every(function (file) { return file.status === "queued"; }));
  h.deep(Object.keys(C.state().policy), ["retainDays", "ocrStored", "ocrText", "lastRemoved"]);
});

h.observe("no per-page language field exists in the ingest data model", function () {
  g.reset();
  g.seed();
  var parse = C.state().parses.stmt;
  h.assert(!Object.prototype.hasOwnProperty.call(parse, "language"));
  h.assert(parse.rows.every(function (row) {
    return row.every(function (cell) {
      return !Object.prototype.hasOwnProperty.call(cell, "language") &&
        !Object.prototype.hasOwnProperty.call(cell, "locale");
    });
  }));
});

h.statik("the ingest surface imports no OCR, PDF parser, file or network primitive", function () {
  h.MODULES.concat([BATCH_JS, PREVIEW_JS, POLICY_JS]).forEach(function (rel) {
    var src = h.read(rel);
    h.assert(!/Tesseract|tesseract|pdfjs|PDFJS|pdf_extract|OcrEngine/i.test(src),
      rel + " references an OCR/PDF engine");
    h.assert(!/FileReader|Blob|ImageData|getImageData|createImageBitmap|<canvas/i.test(src),
      rel + " reads or manipulates image bytes");
    h.assert(!/\bfetch\s*\(|XMLHttpRequest|new\s+WebSocket|Worker\(/.test(src),
      rel + " performs I/O");
  });
});

h.statik("shipped headers explicitly declare simulation and local-only limits", function () {
  h.assert(/SIMULATED in-browser[\s\S]*no OCR is/.test(h.read(h.MODULES[0])));
  h.assert(/No document bytes are read[\s\S]*without a parser or OCR engine/.test(h.read(SIM_JS)));
  h.assert(/Pure logic: no DOM, no[\s\S]*persistence side effects/.test(h.read(h.MODULES[2])));
  h.assert(/Nothing here writes to the workbook or to a server/.test(h.read(h.MODULES[3])));
  h.assert(/All transitions are simulated/.test(h.read(BATCH_JS)));
  h.assert(/never uploads, processes in a tenant[\s\S]*real original/.test(h.read(POLICY_JS)));
});

h.statik("no image cleanup, duplicate detection or table detection primitive is present", function () {
  var src = allSources();
  h.assert(!/deskew|dewarp|grayscale|contrast|rotateImage|auto.?crop|duplicate.?detect/i.test(src));
  h.assert(!/table.?detect|detect.?table|extract.?table|header.?infer/i.test(src));
});

h.statik("batch processing is not queue-backed and exposes no retry", function () {
  var src = h.read(BATCH_JS);
  h.assert(/All transitions are simulated/.test(src));
  h.assert(!/retry|backoff|setTimeout|Worker\(|\bfetch\s*\(/i.test(src));
});

h.statik("the real OCR engine is Windows-only and wired to attachments, not Sheets", function () {
  var ocr = h.readRepo(OCR_RS);
  var attachments = h.readRepo(ATTACH_RS);
  h.assert(/Windows\.Media\.Ocr/.test(ocr));
  h.assert(/#\[cfg\(windows\)\]/.test(ocr));
  h.assert(/#\[cfg\(not\(windows\)\)\]/.test(ocr));
  h.assert(/Ok\(String::new\(\)\)/.test(ocr));
  h.assert(/text_recognition::recognize/.test(attachments));
  allSources().split("\n").forEach(function (line) {
    h.assert(!/text_recognition|Windows\.Media\.Ocr|OcrEngine/.test(line));
  });
});

h.statik("Docs PDF import and the SCAN BARCODE keyword are different features", function () {
  var pdf = h.readRepo(PDF_RS);
  var barcode = h.readRepo(BARCODE_RS);
  h.assert(/pdf_extract::extract_text_from_mem/.test(pdf));
  h.assert(!/OcrEngine|Tesseract|deskew/i.test(pdf));
  h.assert(/SCAN BARCODE/.test(barcode));
  h.assert(/api\/barcode\/scan/.test(barcode));
});
