"use strict";
/* ExportImport — unified facade.
 * Each module is loaded as a separate <script> tag, and this facade exposes
 * a single window.ExportImport object that delegates to all submodules.
 */
(function (window) {
  const EI = window.ExportImportCSV;
  const EIX = window.ExportImportXLSX;
  const EII = window.ExportImportImage;
  const EID = window.ExportImportDocs;
  if (!EI) console.error("ExportImport: 01_csv_json_html_pdf.js must be loaded first");

  const ExIm = {
    csv: { export: EI.exportCSV, import: EI.importCSV },
    json: { export: EI.exportJSON, import: EI.importJSON },
    html: { export: EI.exportHTML },
    pdf: { export: EI.exportPDF },
    xlsx: { export: EIX.buildXLSX },
    png: { export: EII.exportPNG },
    svg: { export: EII.exportSVG },
    doc: { export: EID.exportDOC },
    docx: { export: EID.exportDOCX },
    pptx: { export: EID.exportPPTX },
    md: { export: EID.exportMarkdown },
    helpers: {
      readGrid: EI.readGridAsArray,
      writeGrid: EI.populateGridFromArray,
      readFile: EI.readFile,
      colToIdx: EI.colToIdx,
      download: EI.download
    }
  };

  ExIm.export = function (format, grid, opts) {
    const fn = ExIm[(format || "").toLowerCase()];
    if (fn && fn.export) return fn.export(grid, opts);
    throw new Error("Unknown export format: " + format);
  };
  ExIm.import = function (format, grid, data, opts) {
    const fn = ExIm[(format || "").toLowerCase()];
    if (fn && fn.import) return fn.import(grid, data, opts);
    throw new Error("Unknown import format: " + format);
  };

  window.ExportImport = ExIm;
})(window);
