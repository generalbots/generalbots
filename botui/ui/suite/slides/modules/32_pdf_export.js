"use strict";

// botui/ui/suite/slides/modules/32_pdf_export.js
// PDF export — SERVER-ONLY. Delegates rendering to botserver via
// window.SlidesAPI.exportPresentation. The botserver uses
// umya-spreadsheet / rust_xlsxwriter to produce proper Office Open
// XML, then optionally converts to PDF. No client-side rendering.
// If the server is unreachable, print() rejects with a clear error.
(function () {
  function getAPI() {
    return window.SlidesAPI || null;
  }

  function download(blob, filename) {
    if (!blob) return Promise.reject(new Error("No blob to download"));
    return new Promise(function (resolve, reject) {
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = filename || "presentation.pdf";
      document.body.appendChild(a);
      a.click();
      setTimeout(function () {
        if (a.parentNode) a.parentNode.removeChild(a);
        URL.revokeObjectURL(url);
        resolve(true);
      }, 100);
    });
  }

  function getTitle() {
    const el = document.getElementById("presentationName");
    return (el && el.value) ? el.value : "Presentation";
  }

  function getPresId() {
    const el = document.getElementById("presentationName");
    return (el && el.value) ? el.value : null;
  }

  function print() {
    const API = getAPI();
    const presId = getPresId();
    if (!API) return Promise.resolve({ ok: false, error: { message: "SlidesAPI not loaded; server required for export" } });
    if (!presId) return Promise.resolve({ ok: false, error: { message: "No presentation loaded" } });
    return API.exportPresentation(presId, "pdf").then(function (r) {
      if (!r.ok) {
        return Promise.resolve({ ok: false, error: r.error || { message: "Server rejected export" } });
      }
      const blob = (r.data instanceof Blob) ? r.data : new Blob([JSON.stringify(r.data)], { type: "application/json" });
      return download(blob, getTitle() + ".pdf").then(function () {
        return { ok: true, method: "api", size: blob.size };
      });
    });
  }

  function exportViaServer(slides, options) {
    const API = getAPI();
    if (!API) return Promise.reject(new Error("SlidesAPI not loaded; server required for export"));
    const opts = options || {};
    const body = {
      pres_id: opts.presId || getPresId(),
      title: opts.title || getTitle(),
      slides: slides || [],
      format: "pdf",
    };
    if (!body.pres_id) return Promise.reject(new Error("No pres_id"));
    return API.exportPresentation(body.pres_id, "pdf").then(function (r) {
      if (!r.ok) return r;
      const blob = (r.data instanceof Blob) ? r.data : new Blob([JSON.stringify(r.data)], { type: "application/json" });
      return download(blob, (opts.filename || body.title) + ".pdf");
    });
  }

  window.SlidesPdfExport = {
    print: print,
    exportViaServer: exportViaServer,
    download: download,
  };
})();
