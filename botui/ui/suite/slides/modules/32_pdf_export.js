// botui/ui/suite/slides/modules/32_pdf_export.js
// Export the current presentation to PDF.
//
// Strategy: use the browser's native print-to-PDF capability, but
// pre-format the document with print-friendly styles via a hidden
// <iframe> that loads a synthesized print view. The iframe contains
// one slide per page, scaled to fit, with all interactive chrome
// (toolbars, sidebars) removed.
//
// For headless environments where window.print is not available
// (e.g. server-side rendering), falls back to a server-side
// /api/pdf endpoint via fetch and blob download.
//
// API:
//   window.SlidesPdfExport.print()                  // opens browser print dialog
//   window.SlidesPdfExport.exportViaIframe()        // creates hidden iframe with print view
//   window.SlidesPdfExport.exportViaServer(slides)  // POSTs to /api/pdf, returns Promise<Blob>
"use strict";

(function () {
  function buildSlideHtml(slide) {
    const elements = (slide && slide.elements) || [];
    const sorted = elements.slice().sort(function (a, b) {
      return (a.z_index || 0) - (b.z_index || 0);
    });
    let body = "";
    for (let i = 0; i < sorted.length; i++) {
      const el = sorted[i];
      body += renderElement(el);
    }
    return body;
  }

  function renderElement(el) {
    if (!el) return "";
    const style = el.style || {};
    const pos =
      "position:absolute;left:" + (style.left || 0) + "px;top:" + (style.top || 0) +
      "px;width:" + (style.width || 200) + "px;height:" + (style.height || 100) + "px;";
    const color = style.color ? "color:" + style.color + ";" : "";
    const bg = style.backgroundColor ? "background-color:" + style.backgroundColor + ";" : "";
    const font = style.fontSize ? "font-size:" + style.fontSize + "px;" : "";
    const family = style.fontFamily ? "font-family:" + style.fontFamily + ";" : "";
    const weight = style.fontWeight ? "font-weight:" + style.fontWeight + ";" : "";
    const align = style.textAlign ? "text-align:" + style.textAlign + ";" : "";

    if (el.element_type === "text") {
      return '<div style="' + pos + color + bg + font + family + weight + align + '">' +
        escapeHtml(el.content || "") + '</div>';
    }
    if (el.element_type === "shape") {
      const shape = el.shape || "rectangle";
      if (shape === "circle" || shape === "ellipse") {
        return '<div style="' + pos + bg + 'border-radius:50%;"></div>';
      }
      return '<div style="' + pos + bg + '"></div>';
    }
    if (el.element_type === "image" && el.src) {
      return '<img src="' + escapeAttr(el.src) + '" style="' + pos + 'object-fit:cover;" alt="" />';
    }
    if (el.element_type === "line") {
      return '<svg style="' + pos + 'overflow:visible;"><line x1="0" y1="0" x2="' +
        (style.width || 100) + '" y2="' + (style.height || 100) +
        '" stroke="' + (style.stroke || "#000") + '" stroke-width="' + (style.strokeWidth || 1) + '"/></svg>';
    }
    return "";
  }

  function buildPrintHtml(slides, options) {
    const opts = options || {};
    const title = opts.title || "Presentation";
    let pages = "";
    for (let i = 0; i < slides.length; i++) {
      pages += '<section class="print-slide">' + buildSlideHtml(slides[i]) + "</section>";
    }
    return (
      '<!DOCTYPE html><html><head><meta charset="utf-8"><title>' +
      escapeHtml(title) + "</title>" +
      "<style>" +
      "@page { size: 960px 540px; margin: 0; }" +
      "html, body { margin:0; padding:0; background:#fff; font-family:Arial,sans-serif; }" +
      ".print-slide { position:relative; width:960px; height:540px; page-break-after:always; overflow:hidden; background:#fff; }" +
      ".print-slide:last-child { page-break-after:auto; }" +
      "img { display:block; }" +
      "</style></head><body>" +
      pages +
      "</body></html>"
    );
  }

  function escapeHtml(s) {
    return String(s == null ? "" : s)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
  }

  function escapeAttr(s) {
    return String(s == null ? "" : s)
      .replace(/&/g, "&amp;")
      .replace(/"/g, "&quot;")
      .replace(/</g, "&lt;");
  }

  function exportViaIframe(slides, options) {
    return new Promise(function (resolve, reject) {
      const html = buildPrintHtml(slides || [], options);
      const iframe = document.createElement("iframe");
      iframe.style.position = "fixed";
      iframe.style.right = "0";
      iframe.style.bottom = "0";
      iframe.style.width = "0";
      iframe.style.height = "0";
      iframe.style.border = "0";
      iframe.setAttribute("aria-hidden", "true");
      iframe.setAttribute("title", "Print view");
      document.body.appendChild(iframe);
      let loaded = false;
      function cleanup() {
        if (iframe.parentNode) iframe.parentNode.removeChild(iframe);
      }
      iframe.onload = function () {
        if (loaded) return;
        loaded = true;
        try {
          const win = iframe.contentWindow;
          const doc = iframe.contentDocument || (win && win.document);
          if (win && win.focus) win.focus();
          if (win && win.print) {
            setTimeout(function () {
              try { win.print(); } catch (e) { /* user cancelled */ }
              cleanup();
              resolve({ method: "print", ok: true });
            }, 100);
          } else {
            cleanup();
            reject(new Error("Print API not available"));
          }
        } catch (e) {
          cleanup();
          reject(e);
        }
      };
      iframe.onerror = function () {
        cleanup();
        reject(new Error("Failed to load print iframe"));
      };
      iframe.srcdoc = html;
      setTimeout(function () {
        if (!loaded) {
          cleanup();
          reject(new Error("Print iframe load timeout"));
        }
      }, 10000);
    });
  }

  function exportViaServer(slides, options) {
    if (!window.fetch) {
      return Promise.reject(new Error("Fetch API not available"));
    }
    const opts = options || {};
    const body = {
      title: opts.title || "Presentation",
      slides: slides || [],
      format: "pdf",
    };
    return fetch("/api/pdf", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
      credentials: "same-origin",
    }).then(function (r) {
      if (!r.ok) throw new Error("HTTP " + r.status);
      return r.blob();
    });
  }

  function download(blob, filename) {
    if (!blob) return;
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename || "presentation.pdf";
    document.body.appendChild(a);
    a.click();
    setTimeout(function () {
      if (a.parentNode) a.parentNode.removeChild(a);
      URL.revokeObjectURL(url);
    }, 100);
  }

  function print() {
    const slides = (window.slidesApp && window.slidesApp.getSlidesForExport) ?
      window.slidesApp.getSlidesForExport() : [];
    if (slides.length === 0) {
      return Promise.reject(new Error("No slides to export"));
    }
    return exportViaIframe(slides, {
      title: (document.getElementById("presentationName") || {}).value || "Presentation",
    });
  }

  window.SlidesPdfExport = {
    print: print,
    exportViaIframe: exportViaIframe,
    exportViaServer: exportViaServer,
    download: download,
    buildPrintHtml: buildPrintHtml,
  };
})();
