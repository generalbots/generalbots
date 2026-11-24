/**
 * Runtime checks on the shipped office preview/render helpers themselves:
 * store-only ZIP writer, XML escaping, XLSX grid build, DOCX/PPTX/Markdown
 * export and the shipped Drive preview + Docs sanitizer helpers.
 */
"use strict";

var h = require("./00_harness");
var host = require("./01_host");
var zip = require("./02_zip");

var ctx = require("./10_roundtrip").ctx;

h.test("makeZipStored emits deterministic stored ZIP [xlsx:container]", function () {
  var files = { "a.xml": "<a/>", "b.xml": "<b>text</b>" };
  var first = ctx.XLSX.makeZipStored(files);
  var second = ctx.XLSX.makeZipStored(files);
  h.assertEqual(Buffer.compare(Buffer.from(first), Buffer.from(second)), 0, "byte-identical output");
  h.assertEqual(zip.u32(first, 0), zip.LOCAL_SIG, "local signature");
  var parsed = zip.readStoredZipLocal(first);
  h.assertEqual(parsed.entries, 2, "two stored entries");
  h.assertEqual(zip.decode(parsed.files["b.xml"]), "<b>text</b>", "round-tripped content");
});

h.test("escapeXml handles all five markup characters [docx:text]", function () {
  var out = ctx.XLSX.escapeXml('<a & "b"> c\'d');
  h.assertEqual(out, "&lt;a &amp; &quot;b&quot;&gt; c&apos;d", "escaped output");
});

h.test("column helpers agree [xlsx:cells]", function () {
  h.assertEqual(ctx.XLSX.colLetter(0), "A", "first column");
  h.assertEqual(ctx.XLSX.colLetter(26), "AA", "column 27");
  h.assertEqual(ctx.XLSX.colToIdx("AA"), 26, "AA round trip");
});

h.test("[UAT:SHEET-01] buildXLSX emits non-empty workbook parts and deduped strings [xlsx:shared_strings]", function () {
  var grid = host.makeGrid([["Name", "Value"], ["alpha", "1"], ["beta", "2"]]);
  var blob = ctx.XLSX.buildXLSX(grid);
  var parsed = zip.readStoredZipLocal(host.blobBytes(blob));
  h.assert(parsed.names.indexOf("xl/workbook.xml") >= 0, "workbook part");
  h.assert(parsed.names.indexOf("xl/worksheets/sheet1.xml") >= 0, "sheet part");
  h.assert(parsed.names.indexOf("xl/sharedStrings.xml") >= 0, "shared strings part");
  var sheet = zip.readXml(parsed.files, "xl/worksheets/sheet1.xml");
  h.assertEqual(zip.countTag(sheet, "row"), 3, "three rows");
  h.assertEqual(zip.countTag(sheet, "c"), 6, "six cells");
  h.assertEqual(blob.type, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", "MIME");
});

h.test("exportDOCX uses paragraph styles and preserves text [docx:paragraphs]", function () {
  var element = host.makeDocElement([
    { tag: "h1", text: "Title" },
    { tag: "p", text: "Body sentence." },
    { tag: "li", text: "Bullet" },
  ]);
  var blob = ctx.Docs.exportDOCX(element, {});
  var parsed = zip.readStoredZipLocal(host.blobBytes(blob));
  var document = zip.readXml(parsed.files, "word/document.xml");
  h.assert(document.indexOf("Heading1") >= 0, "heading style emitted");
  h.assertEqual(zip.countTag(document, "p"), 3, "three paragraphs");
  h.assert(document.indexOf("Body sentence.") >= 0, "body text preserved");
  h.assertEqual(blob.type, "application/vnd.openxmlformats-officedocument.wordprocessingml.document", "MIME");
});

h.test("exportDOC fallback produces non-empty HTML [docx:text]", function () {
  var html = ctx.Docs.exportDOC(host.makeDocElement([{ tag: "p", text: "Fallback" }]), {});
  h.assert(html.indexOf("<!DOCTYPE html>") === 0, "HTML document");
  h.assert(html.indexOf("Fallback") >= 0, "content present");
});

h.test("[UAT:SLIDE-01] exportPPTX writes one slide part per slide [pptx:slides]", function () {
  var blob = ctx.Docs.exportPPTX([{ title: "One", body: "First" }, { title: "Two", body: "Second" }], {});
  var parsed = zip.readStoredZipLocal(host.blobBytes(blob));
  h.assert(parsed.names.indexOf("ppt/presentation.xml") >= 0, "presentation part");
  h.assertEqual(parsed.names.filter(function (n) { return /^ppt\/slides\/slide\d+\.xml$/.test(n); }).length,
    2, "two slide parts");
  var slide = zip.readXml(parsed.files, "ppt/slides/slide1.xml");
  h.assert(slide.indexOf("One") >= 0 && slide.indexOf("First") >= 0, "slide text preserved");
  h.assertEqual(blob.type, "application/vnd.openxmlformats-officedocument.presentationml.presentation", "MIME");
});

h.test("exportMarkdown renders headings and emphasis [docx:text]", function () {
  var markdown = ctx.Docs.exportMarkdown(host.makeDocElement([
    { tag: "h1", text: "Report" }, { tag: "p", text: "Summary line." },
  ]), {});
  h.assert(markdown.indexOf("# Report") === 0, "heading rendered");
  h.assert(markdown.indexOf("Summary line.") >= 0, "paragraph rendered");
});

h.test("shipped Drive preview parser yields non-empty output [xlsx:tables]", function () {
  h.assert(ctx.Preview, "DrivePreviewParse loaded");
  var parsed = ctx.Preview.parseCsv("Name,Score\nalpha,10\nbeta,20", {});
  h.assertEqual(parsed.rows.length, 3, "three CSV rows");
  h.assertEqual(parsed.columns, 2, "two columns");
  h.assert(ctx.Preview.escapeHtml("<b>x</b>") === "&lt;b&gt;x&lt;/b&gt;", "escaped preview");
  h.assert(ctx.Preview.highlightJson('{"a":1}').indexOf("gb-tok-key") >= 0, "JSON tokens");
});

h.test("shipped Docs sanitizer fails closed without a DOM [docx:text]", function () {
  h.assert(ctx.Sanitize, "DocsSanitize loaded");
  var out = ctx.Sanitize.sanitize("<script>alert(1)</script><p>ok</p>");
  h.assert(out.length > 0, "non-empty output");
  h.assert(out.indexOf("<script") < 0, "no active markup");
  h.assert(out.indexOf("ok") >= 0, "text retained");
});
