"use strict";
/* #2172 — apply gb-doc-delta/v1 ops to the live deck. */
(function () {
  function apply(payload) {
    var ops = payload.ops || [];
    ops.forEach(function (op) {
      if (!op) return;
      if (op.op === "add_slide" && window.SlidesEditor && typeof window.SlidesEditor.addSlide === "function") {
        window.SlidesEditor.addSlide(op);
      } else if (op.op === "add_slide" && window.addSlide) {
        window.addSlide(op);
      } else if (op.op === "set_text" && window.SlidesEditor && typeof window.SlidesEditor.setText === "function") {
        window.SlidesEditor.setText(op);
      } else if (op.op === "finalize" && window.GBToasts) {
        window.GBToasts.show("Slides", "Generation complete", "success");
      }
    });
  }
  if (window.GBDocDelta) window.GBDocDelta.register("slides", apply);
  else document.addEventListener("DOMContentLoaded", function () { if (window.GBDocDelta) window.GBDocDelta.register("slides", apply); });
})();
