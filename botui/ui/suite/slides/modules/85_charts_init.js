"use strict";
/* Slides chart authoring (#2486) — layer 8: toolbar entry point and boot.
 * Adds a single local-only "Charts" button; existing chart insertion via the
 * catalog is left untouched. No server calls, no browser restart required. */
(function (window) {
  var C = (window.GBSlidesCharts = window.GBSlidesCharts || {});
  var bound = false;

  C.currentSlideIndex = function () {
    var active = document.querySelector(".sl-thumb.active, .sl-slide.active");
    if (active && active.getAttribute("data-index") !== null) {
      return C.num(active.getAttribute("data-index"), 0);
    }
    var thumbs = document.querySelectorAll(".sl-thumb, .sl-slide");
    for (var i = 0; i < thumbs.length; i += 1) {
      if (thumbs[i].classList && thumbs[i].classList.contains("active")) return i;
    }
    return 0;
  };

  C.initUI = function () {
    C.init();
    C.link.refreshOnOpen(C.state);
    if (C.canvas) C.canvas.refresh();
    var button = document.getElementById("slChartsBtn");
    if (!button || bound) return;
    bound = true;
    button.addEventListener("click", function () {
      C.editor.open(null, C.currentSlideIndex());
    });
  };

  if (typeof document !== "undefined") {
    if (document.readyState === "loading") {
      document.addEventListener("DOMContentLoaded", C.initUI);
    } else {
      C.initUI();
    }
  }
})(typeof window !== "undefined" ? window : globalThis);
