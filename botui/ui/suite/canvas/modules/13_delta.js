"use strict";
/* #2172/#2175 — apply gb-doc-delta/v1 shape ops to canvas (SVG-like). */
(function () {
  function getSvgRoot() {
    return document.querySelector("#canvas svg, #canvasSvg, .canvas-surface svg, svg.canvas-svg");
  }
  function apply(payload) {
    var svg = getSvgRoot();
    var ops = payload.ops || [];
    ops.forEach(function (op) {
      if (!op) return;
      if (op.op === "clear" && svg) { svg.innerHTML = ""; return; }
      if (!svg) return;
      if (op.op === "add_shape" || op.op === "add_text") {
        var tag = op.shape || (op.op === "add_text" ? "text" : "rect");
        var el = document.createElementNS("http://www.w3.org/2000/svg", tag);
        if (op.id) el.setAttribute("data-delta-id", op.id);
        var attrs = op.attrs || {};
        Object.keys(attrs).forEach(function (k) { el.setAttribute(k, attrs[k]); });
        if (op.text) el.textContent = op.text;
        if (op.style) el.setAttribute("style", op.style);
        svg.appendChild(el);
      }
      if ((op.op === "update_shape" || op.op === "set_style") && op.id) {
        var el2 = svg.querySelector('[data-delta-id="' + op.id + '"]');
        if (el2) {
          var attrs2 = op.attrs || {};
          Object.keys(attrs2).forEach(function (k) { el2.setAttribute(k, attrs2[k]); });
          if (op.style) el2.setAttribute("style", op.style);
          if (op.text != null) el2.textContent = op.text;
        }
      }
      if (op.op === "delete_shape" && op.id) {
        var del = svg.querySelector('[data-delta-id="' + op.id + '"]');
        if (del) del.remove();
      }
    });
  }
  if (window.GBDocDelta) window.GBDocDelta.register("canvas", apply);
  else document.addEventListener("DOMContentLoaded", function () { if (window.GBDocDelta) window.GBDocDelta.register("canvas", apply); });
})();
