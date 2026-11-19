"use strict";
/* Slides chart SVG geometry (#2486) — layer 3c: scatter, bubble, combo with a
 * secondary axis and waterfall, plus the shared build/materialize wrappers that
 * add gridlines, titles, legend, data labels and accessibility descriptors. */
(function (window) {
  var C = (window.GBSlidesCharts = window.GBSlidesCharts || {});
  var G = C.geom;
  var U = G.util;
  var S = C.style;
  var SVG_NS = "http://www.w3.org/2000/svg";

  function pointShapes(ctx, bubble) {
    var shapes = [];
    var xs = ctx.values[0] || [];
    var ys = ctx.values[1] || xs;
    var sizes = ctx.values[ctx.chart.options.bubbleIndex] || [];
    xs.forEach(function (xValue, index) {
      var x = U.scaleX(xValue, ctx);
      var y = U.scaleY(ys[index], ctx);
      var radius = bubble ? 4 + Math.sqrt(Math.abs(C.num(sizes[index], 0))) * 2 : 4;
      shapes.push(U.shape("circle", {
        cx: Math.round(x), cy: Math.round(y), r: Math.round(radius),
        fill: ctx.colors[0], "class": bubble ? "gbch-bubble" : "gbch-scatter",
        "data-category": index
      }));
      U.dataLabel(shapes, x, y - (bubble ? radius : 0), ys[index], ctx);
    });
    U.trendline(shapes, xs.map(function (value, index) {
      return { x: U.scaleX(value, ctx), y: U.scaleY(ys[index], ctx), value: C.num(ys[index], 0) };
    }), ctx);
    U.errorBars(shapes, xs.map(function (value, index) {
      return { x: U.scaleX(value, ctx), y: U.scaleY(ys[index], ctx), value: C.num(ys[index], 0) };
    }), ctx);
    return shapes;
  }
  G.scatter = function (ctx) { return pointShapes(ctx, false); };
  G.bubble = function (ctx) { return pointShapes(ctx, true); };

  G.combo = function (ctx) {
    var shapes = [];
    var centers = U.categoryCenters(ctx);
    var baseline = U.scaleY(0, ctx);
    var step = (ctx.plot.w / Math.max(1, centers.length)) * 0.46;
    ctx.values.forEach(function (row, seriesIndex) {
      var secondary = ctx.chart.series[seriesIndex] && ctx.chart.series[seriesIndex].axis === "secondary";
      if (!secondary || !ctx.chart.options.comboSecondary) {
        row.forEach(function (value, catIndex) {
          var top = U.scaleY(value, ctx);
          shapes.push(U.shape("rect", {
            x: Math.round(centers[catIndex] - step / 2), y: Math.round(top),
            width: Math.max(1, Math.round(step)), height: Math.max(1, Math.round(baseline - top)),
            fill: ctx.colors[seriesIndex], "class": "gbch-combo-bar", "data-series": seriesIndex
          }));
          U.dataLabel(shapes, centers[catIndex], top, value, ctx);
        });
        return;
      }
      var points = row.map(function (value, index) {
        return { x: centers[index], y: U.scaleY(value, ctx), value: value };
      });
      shapes.push(U.shape("polyline", {
        points: points.map(function (p) { return Math.round(p.x) + "," + Math.round(p.y); }).join(" "),
        fill: "none", stroke: ctx.colors[seriesIndex], "stroke-width": 2,
        "class": "gbch-combo-line", "data-series": seriesIndex
      }));
      points.forEach(function (point) {
        shapes.push(U.shape("circle", {
          cx: Math.round(point.x), cy: Math.round(point.y), r: 3,
          fill: ctx.colors[seriesIndex], "class": "gbch-combo-point", "data-series": seriesIndex
        }));
      });
      U.trendline(shapes, points, ctx);
    });
    var axisX = ctx.plot.x + ctx.plot.w;
    shapes.push(U.shape("g", { "class": "gbch-secondary-axis" }, undefined, [
      U.shape("line", {
        x1: Math.round(axisX), y1: ctx.plot.y, x2: Math.round(axisX),
        y2: Math.round(ctx.plot.y + ctx.plot.h), stroke: "#64748b", "class": "gbch-axis-line"
      }),
      U.shape("text", {
        x: Math.round(axisX - 4), y: ctx.plot.y + 10, "text-anchor": "end",
        "class": "gbch-axis-title gbch-axis-title-secondary", fill: "#64748b", "font-size": 10
      }, ctx.chart.axisTitles.y2)
    ]));
    return shapes;
  };

  G.waterfall = function (ctx) {
    var shapes = [];
    var centers = U.categoryCenters(ctx);
    var step = (ctx.plot.w / Math.max(1, centers.length)) * 0.55;
    var cumulative = 0;
    var baseline = U.scaleY(0, ctx);
    centers.forEach(function (center, index) {
      var value = C.num((ctx.values[0] || [])[index], 0);
      var start = cumulative;
      cumulative += value;
      var y = U.scaleY(Math.max(start, start + value), ctx);
      var height = Math.max(1, Math.abs(U.scaleY(start, ctx) - U.scaleY(start + value, ctx)));
      shapes.push(U.shape("rect", {
        x: Math.round(center - step / 2), y: Math.round(y),
        width: Math.max(1, Math.round(step)), height: Math.round(height),
        fill: value < 0 ? "#ef4444" : ctx.colors[0], "class": "gbch-waterfall",
        "data-category": index
      }));
      if (index < centers.length - 1) {
        shapes.push(U.shape("line", {
          x1: Math.round(center + step / 2), y1: Math.round(U.scaleY(cumulative, ctx)),
          x2: Math.round(centers[index + 1] - step / 2), y2: Math.round(U.scaleY(cumulative, ctx)),
          stroke: "#94a3b8", "stroke-dasharray": "3 2", "class": "gbch-waterfall-connector"
        }));
      }
      U.dataLabel(shapes, center, y, value, ctx);
    });
    shapes.push(U.shape("line", {
      x1: ctx.plot.x, y1: Math.round(baseline), x2: ctx.plot.x + ctx.plot.w,
      y2: Math.round(baseline), stroke: "#94a3b8", "class": "gbch-baseline"
    }));
    return shapes;
  };

  C.build = function (definition, opts) {
    opts = opts || {};
    var def = C.normalize(definition);
    var width = C.num(opts.width, 640);
    var height = C.num(opts.height, 380);
    var theme = opts.theme || null;
    var plot = U.plotArea(width, height);
    var values = def.series.map(function (series) { return series.values; });
    var stacked = def.options.stacked || def.type === "column-stacked";
    var extent = def.type === "pie" || def.type === "donut" ? { min: 0, max: 1 } :
      U.extent(values, stacked);
    var ctx = {
      chart: def, plot: plot, values: values, min: extent.min, max: extent.max,
      colors: def.series.map(function (_, index) {
        if (def.a11y.colorBlindSafe && S.seriesPattern(def, index) !== "solid") {
          return "url(#gbch-pat-" + S.seriesPattern(def, index) + ")";
        }
        return S.seriesColor(def, index, theme);
      }),
      format: function (value) { return S.formatNumber(value, def.numberFormat); }
    };
    var builder = G[def.type] || G.column;
    var shapes = builder(ctx);
    var root = U.shape("svg", {
      xmlns: SVG_NS, viewBox: "0 0 " + width + " " + height,
      role: "img", "class": "gbch-svg",
      "aria-label": def.a11y.altText || def.title || C.tr("slides-charts-a11y-default", "Chart")
    });
    root.children.push(U.shape("title", {}, def.title || def.a11y.altText || C.tr("slides-charts-a11y-default", "Chart")));
    root.children.push(U.shape("desc", {}, def.a11y.altText));
    if (def.a11y.colorBlindSafe) {
      root.children.push(U.shape("defs", {}, undefined, S.patternDefs(def, theme)));
    }
    if (def.gridlines && def.type !== "pie" && def.type !== "donut") {
      for (var i = 0; i <= 4; i += 1) {
        var y = plot.y + (plot.h / 4) * i;
        root.children.push(U.shape("line", {
          x1: plot.x, y1: Math.round(y), x2: plot.x + plot.w, y2: Math.round(y),
          stroke: "#e2e8f0", "class": "gbch-gridline"
        }));
      }
    }
    if (def.type !== "pie" && def.type !== "donut") {
      root.children.push(U.shape("line", {
        x1: plot.x, y1: plot.y, x2: plot.x, y2: plot.y + plot.h,
        stroke: "#94a3b8", "class": "gbch-axis-line"
      }));
      root.children.push(U.shape("text", {
        x: plot.x + plot.w / 2, y: height - 8, "text-anchor": "middle",
        "class": "gbch-axis-title", fill: "#475569", "font-size": 11
      }, def.axisTitles.x));
      root.children.push(U.shape("text", {
        x: 12, y: plot.y + plot.h / 2, "text-anchor": "middle",
        "class": "gbch-axis-title", fill: "#475569", "font-size": 11,
        transform: "rotate(-90 12 " + (plot.y + plot.h / 2) + ")"
      }, def.axisTitles.y));
    }
    if (def.title) {
      root.children.push(U.shape("text", {
        x: width / 2, y: 24, "text-anchor": "middle", "class": "gbch-title",
        fill: "#0f172a", "font-size": 15, "font-family": S.font(theme, "title")
      }, def.title));
    }
    root.children.push(U.shape("g", { "class": "gbch-plot" }, undefined, shapes));
    if (def.legend) {
      var legend = U.shape("g", { "class": "gbch-legend" });
      def.series.forEach(function (series, index) {
        var x = plot.x + index * 110;
        legend.children.push(U.shape("rect", {
          x: x, y: height - 28, width: 10, height: 10,
          fill: ctx.colors[index], "class": "gbch-legend-swatch"
        }));
        legend.children.push(U.shape("text", {
          x: x + 14, y: height - 19, "class": "gbch-legend-label",
          fill: "#334155", "font-size": 10
        }, series.name));
      });
      root.children.push(legend);
    }
    return root;
  };

  C.materialize = function (descriptor, doc, html) {
    var owner = doc || (typeof document !== "undefined" ? document : null);
    if (!owner) return null;
    var node = html ? owner.createElement(descriptor.tag) :
      owner.createElementNS(SVG_NS, descriptor.tag);
    Object.keys(descriptor.attrs || {}).forEach(function (name) {
      node.setAttribute(name, String(descriptor.attrs[name]));
    });
    if (descriptor.text !== undefined) node.textContent = descriptor.text;
    (descriptor.children || []).forEach(function (child) {
      node.appendChild(C.materialize(child, owner, html));
    });
    return node;
  };

  C.dataTable = function (definition) {
    var def = C.normalize(definition);
    var table = U.shape("table", { "class": "gbch-data-table" });
    var head = U.shape("tr", {});
    head.children.push(U.shape("th", {}, C.tr("slides-charts-category", "Category")));
    def.series.forEach(function (series) { head.children.push(U.shape("th", {}, series.name)); });
    table.children.push(U.shape("thead", {}, undefined, [head]));
    var body = U.shape("tbody", {});
    def.categories.forEach(function (category, index) {
      var row = U.shape("tr", {});
      row.children.push(U.shape("th", {}, category));
      def.series.forEach(function (series) {
        row.children.push(U.shape("td", {}, S.formatNumber(series.values[index], def.numberFormat)));
      });
      body.children.push(row);
    });
    table.children.push(body);
    return table;
  };
})(typeof window !== "undefined" ? window : globalThis);
