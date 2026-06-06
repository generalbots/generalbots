// botui/ui/suite/slides/modules/32_pdf_export.js
// PDF export — refactored to delegate rendering to botserver via
// window.SlidesAPI.exportPresentation. The iframe + window.print
// hack is removed entirely; the backend uses umya-spreadsheet and
// rust_xlsxwriter to produce proper Office Open XML, then optionally
// converts to PDF via the server's /api/pdf endpoint.
//
// For environments where the server is unreachable (e.g. static
// preview), a minimal print-fallback is kept (browser's native
// print dialog) but the canonical path is the API call.
"use strict";

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
    if (!API) {
      return exportViaPrintFallback().catch(function (e) {
        return { ok: false, error: { message: e.message || "Export failed" } };
      });
    }
    if (!presId) {
      return Promise.resolve({ ok: false, error: { message: "No presentation loaded" } });
    }
    return API.exportPresentation(presId, "pdf").then(function (r) {
      if (!r.ok) {
        return exportViaPrintFallback().then(function () {
          return { ok: true, method: "print-fallback" };
        }).catch(function (e) {
          return { ok: false, error: { message: e.message || "Export failed" } };
        });
      }
      const blob = (r.data instanceof Blob) ? r.data : new Blob([JSON.stringify(r.data)], { type: "application/json" });
      return download(blob, getTitle() + ".pdf").then(function () {
        return { ok: true, method: "api", size: blob.size };
      });
    });
  }

  function exportViaPrintFallback() {
    return new Promise(function (resolve, reject) {
      if (!window.print) {
        reject(new Error("Print API not available"));
        return;
      }
      try {
        window.print();
        resolve(true);
      } catch (e) {
        reject(e);
      }
    });
  }

  function exportViaServer(slides, options) {
    const API = getAPI();
    if (!API) return Promise.reject(new Error("SlidesAPI not loaded"));
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
