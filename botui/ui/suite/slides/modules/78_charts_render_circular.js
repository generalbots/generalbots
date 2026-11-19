"use strict";
/* Slides chart SVG geometry (#2486) — layer 3b: pie and donut. Both derive a
 * wedge path from the same data model; the donut adds an inner-radius arc so
 * its geometry is observably distinct from the pie. */
(function (window) {
  var C = (window.GBSlidesCharts = window.GBSlidesCharts || {});
  var G = (C.geom = C.geom || {});
  var U = G.util;
  var TAU = Math.PI * 2;
  var START = -Math.PI / 2;

  function point(cx, cy, radius, angle) {
    return { x: cx + radius * Math.cos(angle), y: cy + radius * Math.sin(angle) };
  }
  function round(value) { return Math.round(value * 100) / 100; }

  function wedgePath(cx, cy, radius, from, to) {
    var a = point(cx, cy, radius, from);
    var b = point(cx, cy, radius, to);
    var large = to - from > Math.PI ? 1 : 0;
    return "M " + round(cx) + " " + round(cy) +
      " L " + round(a.x) + " " + round(a.y) +
      " A " + round(radius) + " " + round(radius) + " 0 " + large + " 1 " +
      round(b.x) + " " + round(b.y) + " Z";
  }

  function donutPath(cx, cy, outer, inner, from, to) {
    var a = point(cx, cy, outer, from);
    var b = point(cx, cy, outer, to);
    var ib = point(cx, cy, inner, to);
    var ia = point(cx, cy, inner, from);
    var large = to - from > Math.PI ? 1 : 0;
    return "M " + round(a.x) + " " + round(a.y) +
      " A " + round(outer) + " " + round(outer) + " 0 " + large + " 1 " +
      round(b.x) + " " + round(b.y) +
      " L " + round(ib.x) + " " + round(ib.y) +
      " A " + round(inner) + " " + round(inner) + " 0 " + large + " 0 " +
      round(ia.x) + " " + round(ia.y) + " Z";
  }

  function slices(ctx, donut) {
    var shapes = [];
    var cx = ctx.plot.x + ctx.plot.w / 2;
    var cy = ctx.plot.y + ctx.plot.h / 2;
    var radius = Math.max(20, Math.min(ctx.plot.w, ctx.plot.h) / 2 - 6);
    var inner = radius * (donut ? ctx.chart.options.holeSize : 0);
    var sum = 0;
    ctx.values.forEach(function (row) {
      row.forEach(function (value) { sum += Math.max(0, C.num(value, 0)); });
    });
    if (sum <= 0) {
      shapes.push(U.shape("circle", {
        cx: Math.round(cx), cy: Math.round(cy), r: Math.round(radius),
        fill: "#e2e8f0", "class": "gbch-empty-slice"
      }));
      return shapes;
    }
    var angle = START;
    ctx.values.forEach(function (row, seriesIndex) {
      row.forEach(function (value, catIndex) {
        var portion = Math.max(0, C.num(value, 0)) / sum;
        var next = angle + portion * TAU;
        var path = donut ? donutPath(cx, cy, radius, inner, angle, next) :
          wedgePath(cx, cy, radius, angle, next);
        shapes.push(U.shape("path", {
          d: path, fill: ctx.colors[seriesIndex], "class": donut ? "gbch-donut" : "gbch-pie",
          "data-category": catIndex, "data-series": seriesIndex
        }));
        if (ctx.chart.dataLabels) {
          var labelRadius = donut ? (radius + inner) / 2 : radius * 0.68;
          var mid = point(cx, cy, labelRadius, (angle + next) / 2);
          shapes.push(U.shape("text", {
            x: Math.round(mid.x), y: Math.round(mid.y), "text-anchor": "middle",
            "class": "gbch-datalabel", fill: "#0f172a", "font-size": 10
          }, ctx.format(value)));
        }
        angle = next;
      });
    });
    if (donut) {
      shapes.push(U.shape("circle", {
        cx: Math.round(cx), cy: Math.round(cy), r: 1, fill: "none", "class": "gbch-donut-hole"
      }));
    }
    return shapes;
  }

  G.pie = function (ctx) { return slices(ctx, false); };
  G.donut = function (ctx) { return slices(ctx, true); };
})(typeof window !== "undefined" ? window : globalThis);
