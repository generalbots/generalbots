/**
 * Deterministic hand-built + generated OOXML corpus fixture set (#2457).
 *
 * The licensed 100-files-per-format corpus is out of scope offline; this is a
 * small, de-identified, fully deterministic seed whose parts exercise the same
 * feature tags. Generated fixtures come from the REAL shipped export helpers;
 * hand-built fixtures are minimal XML packaged by the REAL makeZipStored()
 * writer and then read back by the independent ZIP reader.
 */
"use strict";

var host = require("./01_host");
var zip = require("./02_zip");

function decl(body) {
  return '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' + body;
}

function paragraph(text, inner) {
  return "<w:p>" + (inner || "") + "<w:r><w:t xml:space=\"preserve\">" + text + "</w:t></w:r></w:p>";
}

function tableRow(cellText) {
  return "<w:tr><w:tc>" + paragraph(cellText) + "</w:tc><w:tc>" + paragraph(cellText + " value") + "</w:tc></w:tr>";
}

function docParts() {
  var document = decl('<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" ' +
    'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:body>' +
    paragraph("Fidelity corpus", '<w:pPr><w:pStyle w:val="Heading1"/></w:pPr>') +
    "<w:tbl>" + tableRow("Metric") + tableRow("Score") + "</w:tbl>" +
    '<w:p><w:r><w:rPr><w:b/><w:i/><w:u w:val="single"/></w:rPr>' +
      "<w:t>Formatted text</w:t></w:r>" +
      '<w:hyperlink r:id="rId2"><w:r><w:t>Reference</w:t></w:r></w:hyperlink></w:p>' +
    paragraph("List item", '<w:pPr><w:numPr><w:ilvl w:val="0"/></w:numPr></w:pPr>') +
    paragraph("Deletion kept for audit", '<w:del><w:r><w:t>old</w:t></w:r></w:del>') +
    '<w:ins><w:r><w:t>inserted</w:t></w:r></w:ins>' +
    "</w:body></w:document>");
  var footnotes = decl('<w:footnotes xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">' +
    '<w:footnote w:id="1"><w:p><w:r><w:t>Footnote one</w:t></w:r></w:p></w:footnote>' +
    '<w:footnote w:id="2"><w:p><w:r><w:t>Footnote two</w:t></w:r></w:p></w:footnote></w:footnotes>');
  var comments = decl('<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">' +
    '<w:comment w:id="1"><w:p><w:r><w:t>Review note</w:t></w:r></w:p></w:comment></w:comments>');
  return {
    "word/document.xml": document,
    "word/footnotes.xml": footnotes,
    "word/comments.xml": comments,
  };
}

function docChartParts() {
  var parts = docParts();
  parts["word/document.xml"] = parts["word/document.xml"].replace("</w:body>",
    '<w:p><w:r><w:drawing><c:chart xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart" ' +
    'r:id="rId3"/></w:drawing></w:r></w:p></w:body>');
  parts["word/charts/chart1.xml"] = decl('<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart">' +
    '<c:chart><c:plotArea><c:barChart/></c:plotArea></c:chart></c:chartSpace>');
  return parts;
}

function pptParts() {
  var slide1 = decl('<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" ' +
    'xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" ' +
    'xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" ' +
    'xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><p:cSld><p:spTree>' +
    '<p:sp><p:txBody><a:p><a:r><a:rPr b="1"/><a:t>Quarterly results</a:t></a:r></a:p></p:txBody></p:sp>' +
    '<a:tbl><a:tr><a:tc><a:p><a:r><a:t>Region</a:t></a:r></a:p></a:tc></a:tr></a:tbl>' +
    '<p:pic><p:blipFill><a:blip r:embed="rId9"/></p:blipFill></p:pic>' +
    "</p:spTree></p:cSld></p:sld>");
  var slide2 = decl('<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" ' +
    'xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" ' +
    'xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><p:cSld><p:spTree>' +
    '<p:sp><p:txBody><a:p><a:r><a:rPr i="1" u="sng"/><a:t>Margin trend</a:t></a:r></a:p></p:txBody></p:sp>' +
    '<c:chart xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" r:id="rId7"/>' +
    "</p:spTree></p:cSld></p:sld>");
  return {
    "ppt/presentation.xml": decl('<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"/>'),
    "ppt/slides/slide1.xml": slide1,
    "ppt/slides/slide2.xml": slide2,
    "ppt/slideMasters/slideMaster1.xml": decl('<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"/>'),
    "ppt/notesSlides/notesSlide1.xml": decl('<p:notes xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"><a:t>Speaker note</a:t></p:notes>'),
    "ppt/charts/chart1.xml": decl('<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"><c:chart/></c:chartSpace>'),
  };
}

/**
 * Structured slide fixture for the #2457 table/chart regression: a real table
 * grid and a chart with cached series. This is the shape the server sends and
 * the browser exporter (04_docx_pptx_md.js) must turn into native OOXML nodes.
 */
function structuredPptxSlides() {
  return [
    {
      title: "Quarterly results",
      body: "Regional performance",
      elements: [{
        element_type: "table",
        x: 0, y: 1500000, width: 800, height: 160,
        table_data: {
          cells: [
            ["Region", "Q1", "Q2"],
            ["North", "12", "15"],
            ["South", "8", "11"],
          ],
        },
      }],
    },
    {
      title: "Margin trend",
      body: "Quarterly margin",
      elements: [{
        element_type: "chart",
        x: 0, y: 1500000, width: 800, height: 480, title: "Margin trend",
        chart_data: {
          chart_type: "line",
          labels: ["Q1", "Q2", "Q3", "Q4"],
          datasets: [{ label: "Margin", data: [10, 14, 13, 18] }],
        },
      }],
    },
  ];
}

function xlsxParts() {
  var sheet = decl('<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>' +
    '<row r="1"><c r="A1" t="s"><v>0</v></c><c r="B1" t="s"><v>1</v></c></row>' +
    '<row r="2"><c r="A2"><v>10</v></c><c r="B2"><v>20</v></c><c r="C2"><f>SUM(A2:B2)</f><v>30</v></c></row>' +
    "</sheetData></worksheet>");
  return {
    "xl/workbook.xml": decl('<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">' +
      '<sheets><sheet name="Sheet1" sheetId="1"/></sheets></workbook>'),
    "xl/worksheets/sheet1.xml": sheet,
    "xl/sharedStrings.xml": decl('<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">' +
      "<si><t>Product</t></si><si><t>Total</t></si></sst>"),
    "xl/tables/table1.xml": decl('<table xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" name="Totals"/>'),
    "xl/comments1.xml": decl('<comments xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">' +
      "<comment><text>Check total</text></comment></comments>"),
    "xl/styles.xml": decl('<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">' +
      "<fonts><font><b/><i/></font><font><u/></font></fonts></styleSheet>"),
  };
}

function packageFixture(ctx, id, kind, tags, origin, files, expect) {
  var bytes = ctx.XLSX.makeZipStored(files);
  return {
    id: id, kind: kind, tags: tags, origin: origin, bytes: bytes,
    archive: zip.readStoredZipLocal(bytes), expect: expect || {},
  };
}

function fromGenerated(ctx, id, kind, tags, blob, expect) {
  var bytes = host.blobBytes(blob);
  return {
    id: id, kind: kind, tags: tags, origin: "generated", bytes: bytes,
    archive: zip.readStoredZipLocal(bytes), expect: expect || {},
  };
}

/** Build the whole deterministic corpus from one real module host. */
function buildAll(ctx) {
  var corpus = [];
  var docElement = host.makeDocElement([
    { tag: "h1", text: "Quarterly Report" },
    { tag: "p", text: "Revenue increased year over year." },
    { tag: "p", text: "Costs stayed flat." },
    { tag: "li", text: "Next: regional review." },
  ]);
  var grid = host.makeGrid([
    ["Product", "Qty", "Price"],
    ["Widget", "3", "9.99"],
    ["Gadget", "12", "4.50"],
  ]);

  corpus.push(fromGenerated(ctx, "gen-xlsx", "xlsx", ["sheets", "rows", "cells", "shared_strings"],
    ctx.XLSX.buildXLSX(grid), {
      parts: ["xl/workbook.xml", "xl/worksheets/sheet1.xml", "xl/sharedStrings.xml"],
      counts: { sheets: 1, rows: 3 }, textHas: ["Product"],
    }));
  corpus.push(fromGenerated(ctx, "gen-docx", "docx", ["paragraphs", "text"],
    ctx.Docs.exportDOCX(docElement, { filename: "gen.docx" }), {
      parts: ["word/document.xml"], counts: { paragraphs: 4 }, textHas: ["Quarterly Report"],
    }));
  corpus.push(fromGenerated(ctx, "gen-pptx", "pptx", ["slides", "text"],
    ctx.Docs.exportPPTX([
      { title: "Q3 Results", body: "Revenue up 12%" },
      { title: "Q4 Plan", body: "Expand to two regions" },
    ], { filename: "gen.pptx" }), {
      parts: ["ppt/presentation.xml", "ppt/slides/slide1.xml", "ppt/slides/slide2.xml"],
      counts: { slides: 2 }, textHas: ["Q3 Results", "Q4 Plan"],
    }));

  corpus.push(fromGenerated(ctx, "gen-pptx-table-chart", "pptx",
    ["slides", "tables", "charts"],
    ctx.Docs.exportPPTX(structuredPptxSlides(), { filename: "gen-pptx-table-chart.pptx" }), {
      parts: ["ppt/presentation.xml", "ppt/slides/slide1.xml", "ppt/slides/slide2.xml",
        "ppt/charts/chart1.xml"],
      counts: { slides: 2, tables: 1, charts: 2 }, textHas: ["Quarterly results", "North"],
    }));

  corpus.push(packageFixture(ctx, "hb-docx-tables", "docx",
    ["tables", "bold", "italic", "underline", "hyperlinks", "numbering", "tracked_changes", "comments"],
    "handbuilt", docParts(), {
      parts: ["word/document.xml", "word/footnotes.xml", "word/comments.xml"],
      counts: { tables: 1, bold: 1, italic: 1, underline: 1, hyperlinks: 1, numbering: 1,
        tracked_changes: 2, comments: 1 }, textHas: ["Fidelity corpus", "Formatted text"],
    }));
  corpus.push(packageFixture(ctx, "hb-docx-charts", "docx", ["charts", "images"],
    "handbuilt", docChartParts(), {
      parts: ["word/document.xml", "word/charts/chart1.xml"],
      counts: { charts: 2, images: 1 }, textHas: ["Fidelity corpus"],
    }));
  corpus.push(packageFixture(ctx, "hb-pptx-table-chart", "pptx",
    ["slides", "shapes", "tables", "charts", "images", "masters", "bold", "italic", "underline"],
    "handbuilt", pptParts(), {
      parts: ["ppt/presentation.xml", "ppt/slides/slide1.xml", "ppt/slides/slide2.xml",
        "ppt/slideMasters/slideMaster1.xml", "ppt/notesSlides/notesSlide1.xml", "ppt/charts/chart1.xml"],
      counts: { slides: 2, shapes: 2, tables: 1, charts: 2, images: 1, masters: 1,
        bold: 1, italic: 1, underline: 1 },
      textHas: ["Quarterly results", "Margin trend"],
    }));
  corpus.push(packageFixture(ctx, "hb-xlsx-formulas", "xlsx",
    ["rows", "formulas", "tables", "comments", "shared_strings", "bold", "italic", "underline"],
    "handbuilt", xlsxParts(), {
      parts: ["xl/workbook.xml", "xl/worksheets/sheet1.xml", "xl/sharedStrings.xml",
        "xl/tables/table1.xml", "xl/comments1.xml", "xl/styles.xml"],
      counts: { sheets: 1, rows: 2, formulas: 1, tables: 1, shared_strings: 2,
        bold: 1, italic: 1, underline: 1 }, textHas: ["Product", "Total"],
    }));
  return corpus;
}

module.exports = {
  buildAll: buildAll, decl: decl,
  docParts: docParts, pptParts: pptParts, xlsxParts: xlsxParts,
  structuredPptxSlides: structuredPptxSlides,
};
