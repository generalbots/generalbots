"use strict";

/**
 * Module 21: Print scaling for Sheet.
 * Reads the printScale dropdown (100%, fit to width, fit to page, 75%,
 * 50%) and applies a CSS transform on the print content. Updates
 * updatePrintPreview() to honor the chosen scale and triggers a
 * re-render of the preview iframe.
 *
 * Public API: window.SheetPrintScale = { setScale, getScale,
 *   updatePreview, fitToWidth, fitToPage }.
 */

(function () {
  let currentScale = 100;

  function getScale() { return currentScale; }

  function setScale(value) {
    if (typeof value === "string") {
      if (value === "fit_to_width") return fitToWidth();
      if (value === "fit_to_page") return fitToPage();
      const n = parseInt(value);
      if (!isNaN(n)) currentScale = n;
    } else if (typeof value === "number") {
      currentScale = value;
    }
    applyScale();
    document.dispatchEvent(new CustomEvent("sheetPrintScaleChange", { detail: { scale: currentScale } }));
    return currentScale;
  }

  function applyScale() {
    const preview = document.getElementById("printPreviewContent") || document.querySelector(".print-preview");
    if (!preview) return;
    const ratio = currentScale / 100;
    preview.style.transform = "scale(" + ratio + ")";
    preview.style.transformOrigin = "top left";
  }

  function fitToWidth() {
    const preview = document.getElementById("printPreviewContent") || document.querySelector(".print-preview");
    if (!preview) return currentScale;
    const parent = preview.parentElement;
    if (!parent) return currentScale;
    const parentWidth = parent.clientWidth - 32;
    const natural = preview.scrollWidth || preview.offsetWidth || 1;
    const ratio = Math.min(1, parentWidth / natural);
    currentScale = Math.round(ratio * 100);
    applyScale();
    return currentScale;
  }

  function fitToPage() {
    const preview = document.getElementById("printPreviewContent") || document.querySelector(".print-preview");
    if (!preview) return currentScale;
    const parent = preview.parentElement;
    if (!parent) return currentScale;
    const w = parent.clientWidth - 32;
    const h = parent.clientHeight - 32;
    const naturalW = preview.scrollWidth || preview.offsetWidth || 1;
    const naturalH = preview.scrollHeight || preview.offsetHeight || 1;
    const ratio = Math.min(w / naturalW, h / naturalH, 1);
    currentScale = Math.round(ratio * 100);
    applyScale();
    return currentScale;
  }

  function updatePreview() {
    const sel = document.getElementById("printScale");
    if (sel) {
      setScale(sel.value);
    } else {
      applyScale();
    }
  }

  function attach() {
    const sel = document.getElementById("printScale");
    if (sel) {
      sel.addEventListener("change", function () { setScale(sel.value); });
    }
    const btn = document.getElementById("printPreviewBtn");
    if (btn) {
      btn.addEventListener("click", function () { setTimeout(updatePreview, 50); });
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 50);
  }

  window.SheetPrintScale = { setScale, getScale, fitToWidth, fitToPage, updatePreview, applyScale };
})();
