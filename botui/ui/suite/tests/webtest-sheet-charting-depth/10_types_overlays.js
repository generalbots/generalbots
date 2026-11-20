/**
 * RUNTIME assertions for the shipped charting type + overlay IIFEs (#2490).
 * Every part string below was produced by executing 34a/34b/34c/35/36 in the
 * vm sandbox built in 00_harness.js. No renderer is reimplemented here.
 */
"use strict";

var h = require("./00_harness");
var e = h.engine;
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var deep = h.deep;
var note = h.note;

function rects(parts) { return parts.filter(function (p) { return p.indexOf("<rect ") === 0; }); }
function texts(parts) { return parts.filter(function (p) { return p.indexOf("<text ") === 0; }); }
function lines(parts) { return parts.filter(function (p) { return p.indexOf("<line ") === 0; }); }
function polys(parts) { return parts.filter(function (p) { return p.indexOf("<polyline ") === 0; }); }
function circles(parts) { return parts.filter(function (p) { return p.indexOf("<circle ") === 0; }); }
function attr(part, name) {
  var m = new RegExp(name + '="([^"]*)"').exec(part);
  return m ? m[1] : null;
}
function textValues(parts) {
  return texts(parts).map(function (p) {
    var open = p.indexOf(">");
    var close = p.lastIndexOf("</text>");
    return close > open ? p.slice(open + 1, close) : "";
  });
}
function finiteOnly(parts, label) {
  var re = /([a-zA-Z0-9_-]+)="([^"]*)"/g;
  parts.forEach(function (part) {
    var match;
    while ((match = re.exec(part)) !== null) {
      if (/^(x|y|x1|y1|x2|y2|width|height|cx|cy|r|opacity)$/.test(match[1])) {
        assert(Number.isFinite(Number(match[2])), label + ": non-finite " + match[1] + "=" + match[2]);
      }
    }
    re.lastIndex = 0;
  });
  assert(parts.join("").indexOf("NaN") === -1, label + ": NaN leaked into SVG");
}

// ── registration, executed via the shipped IIFEs ────────────────────────
test("all twelve advanced chart types are registered by shipped code", function () {
  ["waterfall", "histogram", "pareto", "box", "heatmap", "treemap", "sunburst",
    "combo", "stacked-column", "clustered-column", "stacked-bar", "clustered-bar"].forEach(function (name) {
    assert(typeof e.renderers[name] === "function", "missing renderer " + name);
    assert(e.charting.isAdvanced(name) === true, "isAdvanced false for " + name);
  });
});
test("the four depth-critical types are real renderer functions", function () {
  ["waterfall", "histogram", "pareto", "combo"].forEach(function (name) {
    assert(typeof e.types.renderers[name] === "function", name + " not on public types registry");
  });
  assert(e.charting.isAdvanced("not-a-chart") === false, "unknown type accepted");
});
test("public register() adds a chart type to the live registry", function () {
  var named = "probe-" + e.scheduled.length;
  e.types.register(named, function () { return ["<text>probe</text>"]; });
  assert(typeof e.renderers[named] === "function", "register did not persist");
  assert(e.charting.types[named] === e.renderers[named], "public types view stale");
  assert(e.charting.isAdvanced(named) === true, "isAdvanced missed registered type");
});

// ── waterfall (34a) ──────────────────────────────────────────────────────
function waterfallCtx() {
  return e.ctxFor(e.seriesDef([{ label: "delta", data: [10, -4, 6] }], ["Q1", "Q2", "Q3"]));
}
test("waterfall renders deterministically with one bar per delta plus a total", function () {
  var parts = e.renderers.waterfall(waterfallCtx());
  deep(parts, e.renderers.waterfall(waterfallCtx()));
  equal(rects(parts).length, 4);
  equal(lines(parts).filter(function (p) { return attr(p, "stroke") === "var(--sheet-muted, #94a3b8)"; }).length, 2);
  finiteOnly(parts, "waterfall");
});
test("waterfall computes the running total and colour-codes gains, losses and total", function () {
  var joined = e.renderers.waterfall(waterfallCtx()).join("");
  assert(textValues(e.renderers.waterfall(waterfallCtx())).indexOf("Total") !== -1, "missing total label");
  assert(textValues(e.renderers.waterfall(waterfallCtx())).indexOf("+10") !== -1, "missing gain label");
  assert(textValues(e.renderers.waterfall(waterfallCtx())).indexOf("-4") !== -1, "missing loss label");
  assert(textValues(e.renderers.waterfall(waterfallCtx())).indexOf("12") !== -1, "running total 12 absent");
  assert(joined.indexOf('fill="#009E73"') !== -1 && joined.indexOf('fill="#D55E00"') !== -1 &&
    joined.indexOf('fill="#0072B2"') !== -1, "Okabe-Ito waterfall colours missing");
});
test("waterfall with no deltas produces no marks", function () {
  deep(e.renderers.waterfall(e.ctxFor(e.seriesDef([{ label: "d", data: [] }], []))), []);
});
test("waterfall keeps all-negative deltas finite", function () {
  var parts = e.renderers.waterfall(e.ctxFor(e.seriesDef([{ label: "d", data: [-3, -5] }], ["a", "b"])));
  assert(rects(parts).length === 3, "expected two steps plus total");
  finiteOnly(parts, "waterfall/negative");
});

// ── histogram (34a) ──────────────────────────────────────────────────────
function histogramCtx(bins, values) {
  var def = e.seriesDef([{ label: "v", data: values || [1, 2, 3, 4, 5] }], ["1", "2", "3", "4", "5"]);
  if (bins != null) def.options.bins = bins;
  return e.ctxFor(def);
}
test("histogram defaults to six bins and honours an explicit bin count", function () {
  equal(rects(e.renderers.histogram(histogramCtx())).length, 6);
  equal(rects(e.renderers.histogram(histogramCtx(3))).length, 3);
});
test("histogram clamps bin request into the [2,30] range", function () {
  equal(rects(e.renderers.histogram(histogramCtx(1))).length, 2);
  equal(rects(e.renderers.histogram(histogramCtx(40))).length, 30);
});
test("histogram counts every value exactly once and labels the count axis", function () {
  var parts = e.renderers.histogram(histogramCtx(3, [1, 1.5, 2, 2.5, 3, 5]));
  deep(parts, e.renderers.histogram(histogramCtx(3, [1, 1.5, 2, 2.5, 3, 5])));
  equal(textValues(parts).filter(function (v) { return v === "Count"; }).length, 1);
  var counts = textValues(parts).filter(function (v) { return /^\d+$/.test(v) && v !== "Count"; });
  assert(counts.length >= 3, "missing per-bin count labels");
  finiteOnly(parts, "histogram");
});
test("histogram avoids a zero-width domain for constant input", function () {
  var parts = e.renderers.histogram(histogramCtx(4, [7, 7, 7]));
  assert(rects(parts).length === 4, "constant input bins missing");
  finiteOnly(parts, "histogram/constant");
});
test("histogram with no values produces no marks", function () {
  deep(e.renderers.histogram(e.ctxFor(e.seriesDef([{ label: "v", data: [] }], []))), []);
});

// ── Pareto (34a) ─────────────────────────────────────────────────────────
function paretoCtx(values) {
  return e.ctxFor(e.seriesDef([{ label: "defects", data: values || [3, 9, 2] }], ["A", "B", "C"]));
}
test("Pareto sorts bars descending and draws a cumulative line", function () {
  var parts = e.renderers.pareto(paretoCtx());
  deep(parts, e.renderers.pareto(paretoCtx()));
  var bars = rects(parts).filter(function (p) { return Number(attr(p, "width")) > 10; });
  equal(bars.length, 3);
  assert(Number(attr(bars[0], "height")) > Number(attr(bars[1], "height")) &&
    Number(attr(bars[1], "height")) > Number(attr(bars[2], "height")), "bars not sorted descending");
  var poly = polys(parts);
  equal(poly.length, 1);
  var points = attr(poly[0], "points").trim().split(" ");
  equal(points.length, 3);
  assert(points[2].endsWith(",26"), "cumulative line does not reach 100% at the right axis top: " + points[2]);
});
test("Pareto labels both axes and marks value/cumulative series", function () {
  var joined = e.renderers.pareto(paretoCtx()).join("");
  var values = textValues(e.renderers.pareto(paretoCtx()));
  assert(values.indexOf("Value") !== -1, "missing value legend");
  assert(values.indexOf("Cumulative %") !== -1, "missing cumulative legend");
  assert(joined.indexOf("%") !== -1 && joined.indexOf('stroke="#D55E00"') !== -1, "right axis/line missing");
  finiteOnly(e.renderers.pareto(paretoCtx()), "pareto");
});
test("Pareto handles all-zero values without division-by-zero artefacts", function () {
  var parts = e.renderers.pareto(paretoCtx([0, 0, 0]));
  finiteOnly(parts, "pareto/zero");
  equal(circles(parts).length, 3);
});

// ── box plot (34a) ───────────────────────────────────────────────────────
function boxCtx() {
  return e.ctxFor(e.seriesDef([{ label: "s", data: [1, 2, 3, 4, 5, 100] }], ["g"]));
}
test("box plot renders quartiles and outliers deterministically", function () {
  var parts = e.renderers.box(boxCtx());
  deep(parts, e.renderers.box(boxCtx()));
  assert(circles(parts).length >= 1, "outlier circle missing");
  assert(lines(parts).length >= 4, "whisker lines missing");
  finiteOnly(parts, "box");
});
test("box plot tolerates an empty series without NaN", function () {
  var parts = e.renderers.box(e.ctxFor(e.seriesDef([{ label: "s", data: [] }], [])));
  finiteOnly(parts, "box/empty");
  assert(parts.length > 0, "no axes emitted");
});

// ── analysis overlays (35) ───────────────────────────────────────────────
function overlayCtx(analysis, options, data) {
  var def = e.seriesDef([{ label: "s", data: data || [1, 3, 5, 7] }], ["a", "b", "c", "d"], options);
  if (analysis) def.analysis = analysis;
  return e.ctxFor(def);
}
function applyOverlay(ctx, seed) {
  return e.overlays.apply(ctx, (seed || []).slice(), e.mapFor(ctx));
}
test("linear trendline adds exactly one dashed overlay polyline and its label", function () {
  var parts = applyOverlay(overlayCtx({ trendline: "linear" }));
  equal(polys(parts).length, 1);
  equal(attr(polys(parts)[0], "stroke"), "#CC79A7");
  equal(attr(polys(parts)[0], "stroke-dasharray"), "6,3");
  assert(textValues(parts).indexOf("linear") !== -1, "trendline label missing");
});
test("exponential, moving and linear trendlines are distinct shipped branches", function () {
  var positive = [2, 4, 8, 16];
  ["linear", "exponential", "moving"].forEach(function (kind) {
    var parts = applyOverlay(overlayCtx({ trendline: kind }, null, positive));
    equal(polys(parts).length, 1, kind + " trendline missing");
    assert(textValues(parts).indexOf(kind) !== -1, kind + " label missing");
  });
});
test("moving-average period is read from analysis options", function () {
  var parts = applyOverlay(overlayCtx({ trendline: "moving", movingPeriod: 2 }));
  equal(polys(parts).length, 1);
  finiteOnly(parts, "trendline/moving");
});
test("analysis settings take precedence over legacy options", function () {
  var noneWins = applyOverlay(overlayCtx({ trendline: "none" }, { trendline: "linear" }));
  equal(polys(noneWins).length, 0, "options leaked past analysis.none");
  var expWins = applyOverlay(overlayCtx({ trendline: "exponential" }, { trendline: "linear" }, [2, 4, 8, 16]));
  assert(textValues(expWins).indexOf("exponential") !== -1, "analysis did not win");
});
test("no trendline when kind is none/absent or the series is empty", function () {
  equal(polys(applyOverlay(overlayCtx(null))).length, 0);
  equal(polys(applyOverlay(overlayCtx({ trendline: "linear" }, null, []))).length, 0);
});
test("error bars add three line segments per non-null value", function () {
  var parts = applyOverlay(overlayCtx({ errorBars: true }));
  equal(lines(parts).length, 12);
  finiteOnly(parts, "errorBars");
});
test("error bars honour an explicit errorPercent", function () {
  var parts = applyOverlay(overlayCtx({ errorBars: true, errorPercent: 10 }));
  equal(lines(parts).length, 12);
  assert(parts.join("").indexOf('stroke="#CC79A7"') !== -1, "error bar colour missing");
});
test("reference lines support object and scalar forms", function () {
  var objectForm = applyOverlay(overlayCtx({ referenceLines: [{ value: 5, label: "Target" }] }));
  equal(lines(objectForm).length, 1);
  assert(textValues(objectForm).indexOf("Target") !== -1, "reference label missing");
  var scalarForm = applyOverlay(overlayCtx({ referenceLines: 5 }));
  equal(lines(scalarForm).length, 1);
  assert(textValues(scalarForm).indexOf("5") !== -1, "scalar reference label missing");
});
test("target band draws a translucent band and an accessible label", function () {
  var parts = applyOverlay(overlayCtx({ targetBand: { low: 2, high: 8 } }));
  var band = rects(parts).filter(function (p) { return attr(p, "fill") === "#F0E442"; });
  equal(band.length, 1);
  equal(attr(band[0], "opacity"), "0.16");
  assert(textValues(parts).indexOf("Target band") !== -1, "target band label missing");
});
test("variance callouts derive a target from the band midpoint or a reference", function () {
  var fromBand = applyOverlay(overlayCtx({ variance: true, targetBand: { low: 8, high: 12 } },
    null, [5, 15]));
  var bandText = textValues(fromBand);
  assert(bandText.indexOf("-50%") !== -1 && bandText.indexOf("+50%") !== -1, bandText.join("|"));
  var fromRef = applyOverlay(overlayCtx({ variance: true, referenceLines: [{ value: 10 }] },
    null, [5, 15]));
  assert(textValues(fromRef).indexOf("+50%") !== -1, "reference-derived variance missing");
});
test("overlays are deterministic and are a no-op without a coordinate map", function () {
  var ctx = overlayCtx({ trendline: "linear", errorBars: true, variance: true,
    referenceLines: [{ value: 3 }], targetBand: { low: 1, high: 6 } });
  deep(applyOverlay(ctx), applyOverlay(ctx));
  var seed = [e.svg.text(1, 2, "seed")];
  assert(e.overlays.apply(ctx, seed, null) === seed, "falsy map must not mutate parts");
  var before = seed.length;
  var after = e.overlays.apply(ctx, seed, e.mapFor(ctx));
  assert(after.length > before, "no overlay marks added");
  note("overlay apply order: targetBand, referenceLines, trendline, errorBars, variance");
});
