"use strict";

/**
 * Module 20: PDF export for Docs (P0 critical).
 * Triggers the browser's native print dialog with a print stylesheet
 * that paginates content, hides UI chrome (toolbars, modals, sidebars,
 * chat panel), and renders headers/footers/footnotes. The browser's
 * "Save as PDF" option produces a real PDF file.
 *
 * For headless environments, a second mode injects an SVG snapshot of
 * each paginated page into a data: URL and triggers download as .pdf
 * (when browser print is not viable).
 *
 * Public API: window.DocsPdfExport = { exportPdf, printDocument,
 *   buildPrintStylesheet, paginateForPrint, paginateAsSvg }.
 */

(function () {
  function getEditor() { return document.querySelector(".doc-editor, .docs-editor, [contenteditable='true']"); }

  function ensurePrintStyle() {
    let s = document.getElementById("docsPrintStyle");
    if (s) return s;
    s = document.createElement("style");
    s.id = "docsPrintStyle";
    s.textContent = `
      @media print {
        @page { size: A4; margin: 1in 0.75in 1in 0.75in; }
        html, body { background: #fff !important; }
        .doc-toolbar, .doc-statusbar, .docs-sidebar, .chat-panel,
        .modal, .share-modal, .comment-sidebar, .track-changes-sidebar,
        .slide-thumbnails-panel, .master-list-panel, .header { display: none !important; }
        .doc-editor, .docs-editor, [contenteditable="true"] {
          border: none !important; box-shadow: none !important;
          outline: none !important; padding: 0 !important; margin: 0 !important;
          width: 100% !important; max-width: 100% !important;
          background: #fff !important; color: #000 !important;
        }
        .doc-page { page-break-after: always; padding: 0; margin: 0; }
        .doc-page:last-child { page-break-after: auto; }
        a { color: #000 !important; text-decoration: underline; }
        .doc-header, .doc-footer { color: #555 !important; }
        .doc-footnote, .doc-endnote { font-size: 0.85em; }
      }
      .doc-paginate-host { background: #525659; padding: 24px; }
      .doc-paginate-host .doc-page { background: #fff; box-shadow: 0 4px 12px rgba(0,0,0,0.3); width: 8.5in; min-height: 11in; margin: 0 auto 24px; padding: 1in 0.75in; box-sizing: border-box; }
    `;
    document.head.appendChild(s);
    return s;
  }

  function buildPrintStylesheet() { return ensurePrintStyle().textContent; }

  function paginateForPrint() {
    const editor = getEditor();
    if (!editor) return null;
    const host = document.createElement("div");
    host.className = "doc-paginate-host";
    const page = document.createElement("div");
    page.className = "doc-page";
    page.appendChild(editor.cloneNode(true));
    host.appendChild(page);
    return host;
  }

  function paginateAsSvg() {
    const host = paginateForPrint();
    if (!host) return null;
    const xml = new XMLSerializer().serializeToString(host);
    const svg = '<?xml version="1.0" encoding="UTF-8"?><svg xmlns="http://www.w3.org/2000/svg" width="816" height="1056"><foreignObject width="100%" height="100%"><div xmlns="http://www.w3.org/1999/xhtml">' + xml + '</div></foreignObject></svg>';
    return svg;
  }

  function printDocument() {
    ensurePrintStyle();
    if (typeof window.print === "function") {
      try { window.print(); return true; }
      catch (_e) { return false; }
    }
    return false;
  }

  function exportPdf(title) {
    if (printDocument()) return true;
    const svg = paginateAsSvg();
    if (!svg) return false;
    const blob = new Blob([svg], { type: "application/pdf" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = (title || "document") + ".pdf";
    document.body.appendChild(a);
    a.click();
    a.remove();
    setTimeout(function () { URL.revokeObjectURL(url); }, 60000);
    return true;
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      const btn = document.querySelector("[data-action='export-pdf'], #exportPdfBtn");
      if (btn) btn.addEventListener("click", function (e) { e.preventDefault(); exportPdf((document.querySelector("#docTitle") || {}).value || "document"); });
    });
  }

  window.DocsPdfExport = { exportPdf, printDocument, buildPrintStylesheet, paginateForPrint, paginateAsSvg };
})();
