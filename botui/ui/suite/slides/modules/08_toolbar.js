"use strict";
/* #2184 — formatting row of the two-row slides toolbar.
 *
 * The canvas renders elements from the server, so a style change has to go to
 * `/api/slides/element/update` (the handler merges the `style` object into the
 * element) AND be applied locally for instant feedback. Every control acts on
 * the element currently selected on the slide.
 */
(function () {
  function selectedElement() {
    var id = window.SlideCanvas && window.SlideCanvas.selectedId;
    if (!id) return null;
    return document.querySelector('#slide-canvas [data-id="' + id + '"]');
  }

  function applyLocal(el, style) {
    if (style.font_weight) el.style.fontWeight = style.font_weight;
    if (style.font_style) el.style.fontStyle = style.font_style;
    if (style.text_align) el.style.textAlign = style.text_align;
    if (style.color) el.style.color = style.color;
    if (style.fill) el.style.background = style.fill;
    if (style.font_size) el.style.fontSize = style.font_size + "px";
  }

  function persist(style) {
    var el = selectedElement();
    if (!el) return;
    applyLocal(el, style);
    var canvas = document.getElementById("slide-canvas");
    var slide = canvas ? parseInt(canvas.dataset.slideId, 10) || 0 : 0;
    fetch("/api/slides/element/update", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        presentation_id: (window.getSlidesPresentationId && window.getSlidesPresentationId()) || "current",
        slide_index: slide,
        element_id: el.dataset.id,
        style: style,
      }),
    }).catch(function () {});
    if (window.recordSlidesEdit) window.recordSlidesEdit();
    if (window.recordSlidesSnapshot) window.recordSlidesSnapshot();
  }

  // Toggle helper: bold/italic flip between on and off instead of only setting.
  function toggle(current, on, off) {
    return current === on ? off : on;
  }

  function bindToggle(button, key, on, off) {
    button.addEventListener("click", function (event) {
      event.preventDefault();
      var el = selectedElement();
      if (!el) return;
      var current = key === "font_weight" ? el.style.fontWeight : el.style.fontStyle;
      var next = toggle(current, on, off);
      button.setAttribute("aria-pressed", next === on ? "true" : "false");
      var style = {};
      style[key] = next;
      persist(style);
    });
  }

  function initToolbar() {
    var bold = document.getElementById("slBoldBtn");
    if (bold) bindToggle(bold, "font_weight", "bold", "normal");
    var italic = document.getElementById("slItalicBtn");
    if (italic) bindToggle(italic, "font_style", "italic", "normal");

    Array.prototype.forEach.call(
      document.querySelectorAll("[data-slides-align]"),
      function (button) {
        button.addEventListener("click", function (event) {
          event.preventDefault();
          persist({ text_align: button.getAttribute("data-slides-align") || "left" });
        });
      },
    );

    Array.prototype.forEach.call(
      document.querySelectorAll("[data-slides-style]"),
      function (button) {
        button.addEventListener("click", function (event) {
          event.preventDefault();
          var spec = (button.getAttribute("data-slides-style") || "").split(":");
          if (spec.length !== 2) return;
          var style = {};
          style[spec[0]] = spec[1];
          persist(style);
        });
      },
    );

    var size = document.getElementById("slFontSize");
    if (size) {
      size.addEventListener("change", function () {
        var value = parseFloat(size.value);
        if (!value) return;
        persist({ font_size: value });
      });
    }

    var textColor = document.getElementById("slTextColor");
    if (textColor) {
      textColor.addEventListener("input", function () {
        persist({ color: textColor.value });
      });
    }

    var fillColor = document.getElementById("slFillColor");
    if (fillColor) {
      fillColor.addEventListener("input", function () {
        persist({ fill: fillColor.value });
      });
    }

    var remove = document.getElementById("slDeleteBtn");
    if (remove) {
      remove.addEventListener("click", function (event) {
        event.preventDefault();
        var el = selectedElement();
        if (el && window.SlideCanvas && window.SlideCanvas.deleteElement) {
          window.SlideCanvas.deleteElement(el);
        }
      });
    }
  }

  window.initSlidesToolbar = initToolbar;
  if (document.readyState === "loading") {
    window.addEventListener("DOMContentLoaded", initToolbar);
  } else {
    initToolbar();
  }
})();

// Export handler (toolbar action). SLIDE-06 (#2155): PDF exports one slide per
// page by printing the server's full-deck HTML export in a paged print view.
window.GBSlidesExport = function (event) {
  event.preventDefault();
  var form = event.target;
  var formatSelect = form.querySelector('[name="format"]');
  var format = formatSelect ? formatSelect.value : "pptx";
  var deck = (form.dataset && form.dataset.presentationId) ||
    window.__SLIDES_PRESENTATION_ID || "current";
  var token = (typeof getGBAccessToken === "function" && getGBAccessToken()) ||
    window.localStorage.getItem("gb-access-token") ||
    window.sessionStorage.getItem("gb-access-token") || "";
  var headers = { "Content-Type": "application/json" };
  if (token) headers.Authorization = "Bearer " + token;

  function closeExportModal() {
    var modal = form.closest ? form.closest(".sl-modal") : null;
    if (modal) modal.remove();
    else {
      var container = form.closest ? form.closest("#slides-modal-container") : null;
      if (container) container.innerHTML = "";
    }
  }

  if (format === "pdf") {
    var printWindow = window.open("", "_blank");
    if (!printWindow) {
      if (window.GBToasts && GBToasts.show) {
        GBToasts.show("Export", "Pop-up bloqueado. Permita pop-ups para exportar PDF.", "error");
      }
      return false;
    }
    fetch("/api/slides/export", {
      method: "POST",
      headers: headers,
      body: JSON.stringify({ presentation_id: deck, format: "html" }),
    })
      .then(function (response) {
        if (!response.ok) throw new Error("HTTP " + response.status);
        return response.text();
      })
      .then(function (html) {
        var printCss =
          "<style>@page{size:960px 540px;margin:0;}" +
          "html,body{margin:0!important;padding:0!important;background:#fff!important;}" +
          ".slide{margin:0!important;box-shadow:none!important;" +
          "page-break-after:always;break-after:page;page-break-inside:avoid;break-inside:avoid;}" +
          ".slide:last-child{page-break-after:auto;break-after:auto;}</style>";
        var doc = html.replace(/<\/head>/i, printCss + "</head>");
        printWindow.document.open();
        printWindow.document.write(doc);
        printWindow.document.close();
        printWindow.focus();
        printWindow.onload = function () { printWindow.print(); };
        closeExportModal();
      })
      .catch(function (error) {
        printWindow.close();
        if (window.GBToasts && GBToasts.show) GBToasts.show("Export", error.message, "error");
      });
    return false;
  }

  fetch("/api/slides/export", {
    method: "POST",
    headers: headers,
    body: JSON.stringify({ presentation_id: deck, format: format }),
  })
    .then(function (response) {
      if (!response.ok) throw new Error("HTTP " + response.status);
      return response.blob();
    })
    .then(function (blob) {
      var link = document.createElement("a");
      link.href = URL.createObjectURL(blob);
      link.download = "presentation." + format;
      document.body.appendChild(link);
      link.click();
      link.remove();
      setTimeout(function () { URL.revokeObjectURL(link.href); }, 4000);
      closeExportModal();
    })
    .catch(function (error) {
      if (window.GBToasts && GBToasts.show) GBToasts.show("Export", error.message, "error");
    });
  return false;
};
