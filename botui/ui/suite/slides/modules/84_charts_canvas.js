"use strict";
/* Slides chart canvas layer (#2486) — layer 7: materialize a chart definition
 * as an SVG chart inside a slide element and surface the local link badge.
 * The chart never claims to be a native PPTX chart part: it is SVG rendering
 * of the deck-scoped local definition. */
(function (window) {
  var C = (window.GBSlidesCharts = window.GBSlidesCharts || {});
  var V = (C.canvas = C.canvas || {});
  var U = C.ui;

  V.host = function () {
    return document.getElementById("slide-canvas") || document.querySelector(".sl-canvas");
  };

  V.renderNode = function (node) {
    var id = node.getAttribute("data-chart-id");
    var chart = id ? C.get(id) : null;
    node.textContent = "";
    if (!chart) return false;
    var width = C.num(node.getAttribute("data-chart-w"), 520);
    var height = C.num(node.getAttribute("data-chart-h"), 310);
    var svg = C.materialize(C.build(chart, { theme: C.editor ? C.editor.theme() : null,
      width: width, height: height }));
    if (svg) node.appendChild(svg);
    if (chart.link.range && chart.link.stale) {
      node.appendChild(U.badge("slides-charts-stale", "STALE", "danger"));
    }
    return true;
  };

  V.refresh = function () {
    var nodes = document.querySelectorAll("[data-chart-id]");
    var count = 0;
    Array.prototype.forEach.call(nodes, function (node) {
      if (V.renderNode(node)) count += 1;
    });
    return count;
  };

  V.insert = function (definition, slideIndex) {
    var chart = C.create(definition);
    if (!chart) return null;
    var host = V.host();
    if (!host) return chart;
    var node = document.createElement("div");
    node.className = "sl-element gbch-canvas-chart";
    node.setAttribute("data-type", "chart");
    node.setAttribute("data-id", chart.id);
    node.setAttribute("data-chart-id", chart.id);
    node.setAttribute("data-chart-w", "520");
    node.setAttribute("data-chart-h", "310");
    node.setAttribute("data-i18n-aria-label", "slides-charts-a11y-default");
    node.setAttribute("aria-label", C.tr("slides-charts-a11y-default", "Chart"));
    node.setAttribute("role", "img");
    node.style.position = "absolute";
    node.style.left = "80px";
    node.style.top = "80px";
    node.style.width = "520px";
    node.style.height = "310px";
    host.appendChild(node);
    V.renderNode(node);
    return chart;
  };

  V.removeNode = function (node) {
    var id = node.getAttribute("data-chart-id");
    if (id) C.remove(id);
    if (node.parentNode) node.parentNode.removeChild(node);
  };
})(typeof window !== "undefined" ? window : globalThis);
