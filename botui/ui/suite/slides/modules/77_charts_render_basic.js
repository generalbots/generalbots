"use strict";
/* Slides chart SVG geometry (#2486) — layer 3a: shared scale helpers plus the
 * clustered/stacked column, horizontal bar, line and area builders. Geometry is
 * a list of plain descriptors ({tag, attrs, text}); materialize() turns them
 * into SVG nodes without innerHTML. */
(function (window) {
  var C = (window.GBSlidesCharts = window.GBSlidesCharts || {});
  var G = (C.geom = C.geom || {});
  var U = (G.util = G.util || {});

  function shape(tag, attrs, text, children) {
    var node = { tag: tag, attrs: attrs || {}, children: children || [] };
    if (text !== undefined && text !== null) node.text = String(text);
    return node;
  }
  U.shape = shape;

  U.plotArea = function (width, height) {
    return { x: 58, y: 46, w: Math.max(80, width - 80), h: Math.max(60, height - 96) };
  };

  U.extent = function (values, stacked) {
    var min = Infinity, max = -Infinity;
    if (stacked) {
      values[0].forEach(function (_, index) {
        var sum = 0;
        values.forEach(function (row) { sum += C.num(row[index], 0); });
        min = Math.min(min, sum); max = Math.max(max, sum);
      });
    } else {
      values.forEach(function (row) {
        row.forEach(function (value) { min = Math.min(min, value); max = Math.max(max, value); });
      });
    }
    if (!isFinite(min) || !isFinite(max)) { min = 0; max = 1; }
    if (max === min) { max = min + 1; }
    if (min > 0) min = 0;
    return { min: min, max: max };
  };

  U.scaleY = function (value, ctx) {
    var span = ctx.max - ctx.min || 1;
    return ctx.plot.y + ctx.plot.h - ((C.num(value, 0) - ctx.min) / span) * ctx.plot.h;
  };
  U.scaleX = function (value, ctx) {
    var span = ctx.max - ctx.min || 1;
    return ctx.plot.x + ((C.num(value, 0) - ctx.min) / span) * ctx.plot.w;
  };
  U.categoryCenters = function (ctx) {
    var count = Math.max(1, ctx.chart.categories.length);
    var step = ctx.plot.w / count;
    return ctx.chart.categories.map(function (_, index) { return ctx.plot.x + step * (index + 0.5); });
  };

  U.dataLabel = function (shapes, x, y, value, ctx) {
    if (!ctx.chart.dataLabels) return;
    shapes.push(shape("text", {
      x: Math.round(x), y: Math.round(y - 6), "text-anchor": "middle",
      "class": "gbch-datalabel", fill: "#334155", "font-size": 10
    }, ctx.format(value)));
  };

  U.trendline = function (shapes, points, ctx) {
    if (!ctx.chart.options.trendline || points.length < 2) return;
    var n = points.length;
    var sx = 0, sy = 0, sxy = 0, sxx = 0;
    points.forEach(function (point, index) {
      sx += index; sy += point.value; sxy += index * point.value; sxx += index * index;
    });
    var slope = (n * sxy - sx * sy) / (n * sxx - sx * sx || 1);
    var intercept = (sy - slope * sx) / n;
    var y0 = U.scaleY(intercept, ctx);
    var y1 = U.scaleY(intercept + slope * (n - 1), ctx);
    shapes.push(shape("line", {
      x1: Math.round(points[0].x), y1: Math.round(y0),
      x2: Math.round(points[n - 1].x), y2: Math.round(y1),
      "class": "gbch-trendline", stroke: "#0f172a", "stroke-dasharray": "5 3"
    }));
  };

  U.errorBars = function (shapes, points, ctx) {
    if (!ctx.chart.options.errorBars) return;
    var delta = Math.max(2, (ctx.max - ctx.min) * 0.05);
    points.forEach(function (point) {
      var top = point.y - delta, bottom = point.y + delta;
      shapes.push(shape("line", {
        x1: Math.round(point.x), y1: Math.round(top), x2: Math.round(point.x),
        y2: Math.round(bottom), "class": "gbch-errorbar", stroke: "#0f172a", "stroke-width": 1
      }));
      shapes.push(shape("line", {
        x1: Math.round(point.x - 3), y1: Math.round(top), x2: Math.round(point.x + 3),
        y2: Math.round(top), "class": "gbch-errorbar", stroke: "#0f172a", "stroke-width": 1
      }));
    });
  };

  function clustered(ctx, horizontal) {
    var shapes = [];
    var centers = U.categoryCenters(ctx);
    var count = Math.max(1, ctx.values.length);
    var step = (ctx.plot.w / Math.max(1, centers.length)) * 0.72 / count;
    ctx.values.forEach(function (row, seriesIndex) {
      var fill = ctx.colors[seriesIndex];
      row.forEach(function (value, catIndex) {
        var height = Math.abs(U.scaleY(value, ctx) - U.scaleY(ctx.min, ctx));
        if (horizontal) {
          var width = U.scaleX(value, ctx) - U.scaleX(ctx.min, ctx);
          var y = centers[catIndex] - (step * count) / 2 + seriesIndex * step;
          shapes.push(shape("rect", {
            x: Math.round(U.scaleX(ctx.min, ctx)), y: Math.round(y),
            width: Math.round(width), height: Math.max(1, Math.round(step * 0.82)),
            fill: fill, "class": "gbch-bar", "data-series": seriesIndex
          }));
          U.dataLabel(shapes, U.scaleX(value, ctx), y + step * 0.4, value, ctx);
        } else {
          var x = centers[catIndex] - (step * count) / 2 + seriesIndex * step;
          var top = U.scaleY(value, ctx);
          shapes.push(shape("rect", {
            x: Math.round(x), y: Math.round(top),
            width: Math.max(1, Math.round(step * 0.82)), height: Math.max(1, Math.round(height)),
            fill: fill, "class": "gbch-column", "data-series": seriesIndex
          }));
          U.dataLabel(shapes, x + step * 0.4, top, value, ctx);
        }
      });
    });
    return shapes;
  }

  function stacked(ctx) {
    var shapes = [];
    var centers = U.categoryCenters(ctx);
    var step = (ctx.plot.w / Math.max(1, centers.length)) * 0.6;
    var baseline = U.scaleY(0, ctx);
    centers.forEach(function (center, catIndex) {
      var cumulative = 0;
      ctx.values.forEach(function (row, seriesIndex) {
        var value = C.num(row[catIndex], 0);
        var start = baseline - (cumulative / (ctx.max - ctx.min || 1)) * ctx.plot.h;
        var height = (Math.abs(value) / (ctx.max - ctx.min || 1)) * ctx.plot.h;
        shapes.push(shape("rect", {
          x: Math.round(center - step / 2), y: Math.round(start - height),
          width: Math.max(1, Math.round(step)), height: Math.max(1, Math.round(height)),
          fill: ctx.colors[seriesIndex], "class": "gbch-column-stacked",
          "data-series": seriesIndex
        }));
        U.dataLabel(shapes, center, start - height, value, ctx);
        cumulative += value;
      });
    });
    return shapes;
  }

  function lineOrArea(ctx, area) {
    var shapes = [];
    var centers = U.categoryCenters(ctx);
    ctx.values.forEach(function (row, seriesIndex) {
      var points = row.map(function (value, index) {
        return { x: centers[index], y: U.scaleY(value, ctx), value: value };
      });
      var coords = points.map(function (point) {
        return Math.round(point.x) + "," + Math.round(point.y);
      }).join(" ");
      if (area) {
        var base = Math.round(U.scaleY(ctx.min, ctx));
        shapes.push(shape("polygon", {
          points: Math.round(points[0].x) + "," + base + " " + coords + " " +
            Math.round(points[points.length - 1].x) + "," + base,
          fill: ctx.colors[seriesIndex], "fill-opacity": 0.28,
          "class": "gbch-area", "data-series": seriesIndex
        }));
      }
      shapes.push(shape("polyline", {
        points: coords, fill: "none", stroke: ctx.colors[seriesIndex],
        "stroke-width": 2, "class": "gbch-line", "data-series": seriesIndex
      }));
      points.forEach(function (point) {
        shapes.push(shape("circle", {
          cx: Math.round(point.x), cy: Math.round(point.y), r: 3,
          fill: ctx.colors[seriesIndex], "class": "gbch-point", "data-series": seriesIndex
        }));
        U.dataLabel(shapes, point.x, point.y, point.value, ctx);
      });
      U.trendline(shapes, points, ctx);
      U.errorBars(shapes, points, ctx);
    });
    return shapes;
  }

  G.column = function (ctx) { return clustered(ctx, false); };
  G.bar = function (ctx) { return clustered(ctx, true); };
  G["column-stacked"] = stacked;
  G.line = function (ctx) { return lineOrArea(ctx, false); };
  G.area = function (ctx) { return lineOrArea(ctx, true); };
})(typeof window !== "undefined" ? window : globalThis);
