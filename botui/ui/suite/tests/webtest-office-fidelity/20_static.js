/**
 * [STATIC] source assertions documenting the production facts this harness
 * measures. These read shipped source only; no runtime claim is made here.
 */
"use strict";

var h = require("./00_harness");

[
  "js/modules-export/01_csv_json_html_pdf.js",
  "js/modules-export/02_xlsx.js",
  "js/modules-export/04_docx_pptx_md.js",
  "drive/modules/08_preview_parse.js",
  "docs/docs-sanitize.js",
].forEach(function (relPath) {
  h.staticTest("shipped helper exists: " + relPath, function () {
    h.assert(h.readSuite(relPath).length > 100, "helper source must be non-empty");
  });
});

h.staticTest("XLSX writer is store-only, dependency-free [xlsx:container]", function () {
  h.sourceHas("botui/ui/suite/js/modules-export/02_xlsx.js", "makeZipStored", "ZIP writer present");
  h.sourceHas("botui/ui/suite/js/modules-export/02_xlsx.js", "0x04034b50", "local ZIP header written");
});

h.staticTest("DOCX/PPTX exporters build minimal OOXML [docx:paragraphs] [pptx:slides]", function () {
  h.sourceHas("botui/ui/suite/js/modules-export/04_docx_pptx_md.js", "word/document.xml", "DOCX main part");
  h.sourceHas("botui/ui/suite/js/modules-export/04_docx_pptx_md.js", "ppt/slides/slide", "PPTX slide parts");
});

h.staticTest("PPTX export deliberately flattens tables/charts [pptx:tables] [pptx:charts]", function () {
  h.sourceHas("botserver/crates/botslides/src/pptx/export.rs",
    "Tables/charts have no native shape writer", "degradation is documented in source");
});

h.staticTest("XLSX export preserves unsupported parts by merging [xlsx:tables]", function () {
  h.sourceHas("botserver/crates/botsheet/src/storage/xlsx_passthrough.rs",
    "merge_into_original", "passthrough merge present");
});

h.staticTest("DOCX export preserves the original package [docx:footnotes]", function () {
  h.sourceHas("botserver/crates/botdocs/src/ooxml.rs",
    "load_docx_preserving", "preserving loader present");
});

h.staticTest("Python structural comparator and bug template exist", function () {
  h.assert(h.fileExistsRepo("bottest/tools/fidelity/fx_extract.py"), "extractor present");
  h.assert(h.fileExistsRepo(".github/ISSUE_TEMPLATE/office-fidelity.md"), "bug template present");
});
