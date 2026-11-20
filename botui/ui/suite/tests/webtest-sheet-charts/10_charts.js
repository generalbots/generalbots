/**
 * Sheet charting — pure-helper invariant tests (#2490).
 * Production modules are loaded unchanged (see 00_harness.js). Tests marked
 * [STATIC] assert source structure only, for helpers that are private to an
 * IIFE or need a DOM; they never claim a fabricated runtime result.
 */
"use strict";

var h = require("./00_harness");
var g = h.loadModules();
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var deep = h.deep;
var source = h.readFile;

function useGrid(cells) {
  g.window.SheetCore = { getGrid: function () { return { cells: cells }; } };
}

function labelledGrid() {
  return {
    "0,0": { value: "Region" }, "0,1": { value: "Sales" },
    "1,0": { value: "North" }, "1,1": { value: 10 },
    "2,0": { value: "South" }, "2,1": { value: 20 },
  };
}

function seriesDef(series, labels, options) {
  return {
    labels: labels,
    datasets: series.map(function (s) { return { label: s.label, color: s.color, data: s.data }; }),
    options: options || {},
  };
}

function makeCtx(def, labels) {
  var model = g.data.model(def);
  return {
    model: model,
    geo: g.svg.geometry(400, 240),
    colors: g.palette.colors(model.series),
    def: def,
    t: g.window.gbT,
    w: 400,
    h: 240,
  };
}

function rectAttrs(parts) {
  return parts.filter(function (p) { return p.indexOf("<rect ") === 0; }).map(function (p) {
    function attr(name) {
      var m = new RegExp(name + '="([^"]*)"').exec(p);
      return m ? Number(m[1]) : NaN;
    }
    return { x: attr("x"), y: attr("y"), w: attr("width"), h: attr("height") };
  });
}

function assertFiniteParts(parts, label) {
  parts.forEach(function (part) {
    var re = /([a-zA-Z0-9_-]+)="([^"]*)"/g;
    var match;
    while ((match = re.exec(part)) !== null) {
      if (/^(x|y|x1|y1|x2|y2|width|height|cx|cy|r|opacity)$/.test(match[1])) {
        assert(Number.isFinite(Number(match[2])), label + ": non-finite " + match[1]);
      }
    }
  });
}

function assertNonNegativeRects(parts, label) {
  rectAttrs(parts).forEach(function (r) {
    assert(r.w >= 0 && r.h >= 0, label + ": negative rect dimension " + JSON.stringify(r));
  });
}

// ── API surface ────────────────────────────────────────────────────────
test("data module exports its pure helpers", function () {
  ["parseRef", "parseRange", "clamp", "bindRange", "model", "stats", "fiveNumber",
    "regression", "exponential", "movingAverage", "cellValue"].forEach(function (name) {
    assert(typeof g.data[name] === "function", "missing " + name);
  });
});
test("svg module exports primitives and geometry", function () {
  ["esc", "n", "fmt", "open", "rect", "circle", "polyline", "path", "frame",
    "geometry", "verticalScale", "axes"].forEach(function (name) {
    assert(typeof g.svg[name] === "function", "missing " + name);
  });
});
test("variant and hierarchy renderers are registered", function () {
  ["combo", "stacked-column", "clustered-column", "stacked-bar", "clustered-bar",
    "heatmap", "treemap", "sunburst"].forEach(function (name) {
    assert(typeof g.renderers[name] === "function", "missing renderer " + name);
  });
});

// ── parseRef ───────────────────────────────────────────────────────────
test("parseRef('B3') is zero-based (col 1, row 2)", function () {
  deep(g.data.parseRef("B3"), { col: 1, row: 2 });
});
test("parseRef is case-insensitive", function () {
  deep(g.data.parseRef("b3"), g.data.parseRef("B3"));
});
test("parseRef strips absolute markers and whitespace", function () {
  deep(g.data.parseRef(" $B$3 "), { col: 1, row: 2 });
});
test("parseRef decodes multi-letter columns", function () {
  deep(g.data.parseRef("AA1"), { col: 26, row: 0 });
});
test("parseRef rejects malformed and non-string input", function () {
  assert(g.data.parseRef("") === null, "empty accepted");
  assert(g.data.parseRef("A") === null, "missing row accepted");
  assert(g.data.parseRef(42) === null, "number accepted");
});

// ── parseRange ─────────────────────────────────────────────────────────
test("parseRange('A1:C2') returns normalised bounds", function () {
  deep(g.data.parseRange("A1:C2"), {
    sheet: null, start: { row: 0, col: 0 }, end: { row: 1, col: 2 },
  });
});
test("parseRange keeps sheet name and normalises reversed refs", function () {
  deep(g.data.parseRange("Data!C2:A1"), {
    sheet: "Data", start: { row: 0, col: 0 }, end: { row: 1, col: 2 },
  });
});
test("parseRange rejects non-range input", function () {
  assert(g.data.parseRange("A1") === null, "single cell accepted");
  assert(g.data.parseRange("A1:zz") === null, "bad end accepted");
  assert(g.data.parseRange(null) === null, "null accepted");
});

// ── clamp / boundsOf ───────────────────────────────────────────────────
test("clamp enforces low and high bounds", function () {
  equal(g.data.clamp(5, 0, 3), 3);
  equal(g.data.clamp(-1, 0, 3), 0);
  equal(g.data.clamp(2, 0, 3), 2);
});
test("clamp ignores non-finite bounds", function () {
  equal(g.data.clamp(5, NaN, 3), 3);
  equal(g.data.clamp(-5, 0, NaN), 0);
});
test("[STATIC] boundsOf treats empty/null min-max as NaN", function () {
  var src = source(h.DATA_REL);
  assert(/function boundsOf\(options\)/.test(src), "boundsOf not found");
  assert(/src\.min === "" \|\| src\.min == null \? NaN : Number\(src\.min\)/.test(src), "min handling changed");
  assert(/src\.max === "" \|\| src\.max == null \? NaN : Number\(src\.max\)/.test(src), "max handling changed");
});

// ── isNumeric / numbers / stats / quantile ─────────────────────────────
test("stats computes count/mean/sd/min/max on a fixed array", function () {
  deep(g.data.stats([2, 4, 4, 4, 5, 5, 7, 9]), {
    count: 8, mean: 5, sd: 2, min: 2, max: 9,
  });
});
test("stats drops NaN-coercing values and empty arrays stay zeroed", function () {
  equal(g.data.stats([1, "x", undefined, 3]).count, 2);
  equal(g.data.stats([1, "x", undefined, 3]).mean, 2);
  deep(g.data.stats([]), { count: 0, mean: 0, sd: 0, min: 0, max: 0 });
});
test("stats coerces numeric strings to numbers and counts them", function () {
  var out = g.data.stats(["2", 4, "6"]);
  assert(out.count === 3 && out.mean === 4 && out.min === 2 && out.max === 6 &&
    Math.abs(out.sd - Math.sqrt(8 / 3)) < 1e-12, JSON.stringify(out));
});
test("stats treats null as a gap, not zero (regression #2490)", function () {
  var out = g.data.stats([1, null, 3]);
  assert(out.count === 2 && out.min === 1 && out.max === 3 &&
    Math.abs(out.mean - 2) < 1e-12 && Math.abs(out.sd - 1) < 1e-12, JSON.stringify(out));
});
test("[STATIC] isNumeric accepts numeric strings and rejects text/booleans", function () {
  var src = source(h.DATA_REL);
  assert(/function isNumeric\(value\)/.test(src), "isNumeric not found");
  assert(/value === "" \|\| value === null \|\| typeof value === "boolean"/.test(src), "reject list changed");
  assert(/Number\.isFinite\(Number\(value\)\)/.test(src), "numeric coercion changed");
});
test("[STATIC] numbers maps non-finite coercion to null", function () {
  var src = source(h.DATA_REL);
  assert(/function numbers\(values\)/.test(src), "numbers not found");
  assert(/Number\.isFinite\(numeric\) \? numeric : null/.test(src), "null mapping changed");
});
test("fiveNumber median exercises quantile at 0.5", function () {
  equal(g.data.fiveNumber([5, 1, 3, 2, 4]).median, 3);
  equal(g.data.fiveNumber([1, 2, 3, 4]).median, 2.5);
});
test("fiveNumber quartile interpolation is deterministic", function () {
  deep(g.data.fiveNumber([1, 2, 3, 4, 5]), {
    min: 1, q1: 2, median: 3, q3: 4, max: 5, outliers: [],
  });
});
test("[STATIC] quantile clamps probability and interpolates floor/ceil", function () {
  var src = source(h.DATA_REL);
  assert(/function quantile\(sorted, probability\)/.test(src), "quantile not found");
  assert(/Math\.max\(0, Math\.min\(1, probability\)\)/.test(src), "probability not clamped");
  assert(/sorted\[lower\] \+ \(sorted\[upper\] - sorted\[lower\]\) \* \(position - lower\)/.test(src), "interpolation changed");
});

// ── regression helpers (exported, DOM-free) ───────────────────────────
test("regression fits a linear series", function () {
  var fit = g.data.regression([1, 3, 5, 7]);
  equal(fit.slope, 2);
  equal(fit.intercept, 1);
  equal(fit.predict(3), 7);
});
test("exponential predict stays positive for positive inputs", function () {
  assert(g.data.exponential([1, 2, 4, 8]).predict(2) > 0, "non-positive prediction");
});
test("movingAverage smooths with the requested period", function () {
  deep(g.data.movingAverage([1, 2, 3, 4], 2), [1, 1.5, 2.5, 3.5]);
});

// ── cellValue / bindRange / model ──────────────────────────────────────
test("cellValue reads object-form grid cells", function () {
  useGrid(labelledGrid());
  equal(g.data.cellValue(1, 1), 10);
  equal(g.data.cellValue(0, 0), "Region");
});
test("cellValue reads vm-native Map cells", function () {
  useGrid(g.makeMap([["0,0", { value: 7 }]]));
  equal(g.data.cellValue(0, 0), 7);
  equal(g.data.cellValue(3, 3), "");
});
test("bindRange reads labels, header and series from a fake sheet", function () {
  useGrid(labelledGrid());
  deep(g.data.bindRange("A1:B3"), {
    range: "A1:B3", labels: ["North", "South"],
    series: [{ label: "Sales", data: [10, 20] }],
  });
});
test("bindRange clamps cell values to the requested bounds", function () {
  useGrid(labelledGrid());
  deep(g.data.bindRange("A1:B3", { min: 0, max: 15 }).series[0].data, [10, 15]);
});
test("bindRange treats an all-numeric first column as data (isNumeric path)", function () {
  useGrid({
    "0,0": { value: "1" }, "0,1": { value: "2" },
    "1,0": { value: "3" }, "1,1": { value: "4" },
  });
  var bound = g.data.bindRange("A1:B2");
  deep(bound.labels, ["1", "2"]);
  equal(bound.series.length, 2);
  deep(bound.series[0].data, [1, 3]);
});
test("bindRange returns an empty structure for an invalid range", function () {
  deep(g.data.bindRange("nope"), { range: "nope", labels: [], series: [] });
});
test("model builds series structure and value extent from datasets", function () {
  var def = seriesDef([
    { label: "Rev", color: "#0072B2", data: [10, 20] },
    { label: "Cost", color: "#E69F00", data: [4, 8] },
  ], ["Q1", "Q2"]);
  var m = g.data.model(def);
  deep(m.labels, ["Q1", "Q2"]);
  equal(m.series[0].color, "#0072B2");
  deep(m.values, [10, 20, 4, 8]);
  equal(m.min, 4);
  equal(m.max, 20);
  equal(m.range, "");
});
test("model applies min/max options to dataset values", function () {
  var m = g.data.model(seriesDef([
    { label: "Rev", data: [10, 20, 4] },
  ], ["a", "b", "c"], { min: 5, max: 15 }));
  deep(m.series[0].data, [10, 15, 5]);
  equal(m.max, 15);
});
test("model synthesises labels and tolerates an empty definition", function () {
  equal(g.data.model(seriesDef([{ label: "s", data: [3, 6] }], [])).labels.length, 2);
  var empty = g.data.model({});
  deep(empty.labels, []);
  deep(empty.series, []);
  equal(empty.min, 0);
  equal(empty.max, 0);
});
test("model delegates data_range to bindRange", function () {
  useGrid(labelledGrid());
  var m = g.data.model({ data_range: "A1:B3" });
  equal(m.range, "A1:B3");
  deep(m.labels, ["North", "South"]);
  deep(m.series[0].data, [10, 20]);
});

// ── SVG escaping, rounding, formatting ─────────────────────────────────
test("esc escapes all five XML/attribute characters", function () {
  equal(g.svg.esc('&<>"\''), "&amp;&lt;&gt;&quot;&apos;");
});
test("esc is idempotent for clean text and stringifies nullish values", function () {
  equal(g.svg.esc("plain text"), "plain text");
  equal(g.svg.esc(g.svg.esc("plain text")), "plain text");
  equal(g.svg.esc(null), "");
  equal(g.svg.esc(undefined), "");
});
test("n rounds to two decimals", function () {
  equal(g.svg.n(1.234), 1.23);
  equal(g.svg.n(1.236), 1.24);
  equal(g.svg.n("3.14159"), 3.14);
});
test("n maps NaN and both infinities to zero", function () {
  equal(g.svg.n(NaN), 0);
  equal(g.svg.n(Infinity), 0);
  equal(g.svg.n(-Infinity), 0);
});
test("fmt delegates to gbFormatNumber when available and escapes its output", function () {
  g.withFormat(function (value) { return "<b>" + value + "</b>"; }, function () {
    equal(g.svg.fmt(1.234), "&lt;b&gt;1.23&lt;/b&gt;");
  });
});
test("fmt falls back to the escaped rounded number", function () {
  g.withFormat(null, function () {
    equal(g.svg.fmt(1.236), "1.24");
  });
});

// ── SVG primitives and geometry ────────────────────────────────────────
test("rect clamps negative width and height to zero", function () {
  var out = g.svg.rect(1, 2, -5, -9, { fill: "#0072B2" });
  assert(out.indexOf('width="0"') !== -1 && out.indexOf('height="0"') !== -1, out);
});
test("open emits an accessible SVG root with escaped alt text", function () {
  var out = g.svg.open(10.126, 20, 'a & "b"');
  assert(out.indexOf('role="img"') !== -1, "missing role");
  assert(out.indexOf('aria-label="a &amp; &quot;b&quot;"') !== -1, out);
  assert(out.indexOf('width="10.13"') !== -1, out);
});
test("geometry returns deterministic plot bounds and band helpers", function () {
  var geo = g.svg.geometry(400, 240);
  equal(geo.plotW, 342);
  equal(geo.plotH, 182);
  equal(geo.bandWidth(3), 114);
  equal(geo.x(0, 3), geo.pad.l);
  equal(geo.x(2, 3), geo.pad.l + geo.plotW);
  equal(geo.bandX(0, 3), geo.pad.l + 57);
});
test("verticalScale maps extent to the plot bottom/top", function () {
  var geo = g.svg.geometry(400, 240);
  var scale = g.svg.verticalScale(0, 10, geo, 4);
  equal(scale.y(0), geo.pad.t + geo.plotH);
  equal(scale.y(10), geo.pad.t);
  equal(scale.values().length, 5);
});
test("frame wraps parts and closes the SVG", function () {
  var out = g.svg.frame(400, 240, "t", "alt", [g.svg.circle(1, 2, 3)]);
  assert(out.indexOf("<svg ") === 0, "missing root");
  assert(out.indexOf("<circle ") !== -1 && out.slice(-6) === "</svg>", out.slice(-40));
});

// ── combo and stacked/clustered variants ───────────────────────────────
function variantCtx(stacked) {
  return makeCtx(seriesDef([
    { label: "A", color: "#0072B2", data: [10, 20] },
    { label: "B", color: "#E69F00", data: [4, 8] },
  ], ["Q1", "Q2"]), stacked);
}

test("[STATIC] valueAt coerces data and falls back to zero", function () {
  var src = source(h.VARIANTS_REL);
  assert(/const value = Number\(series && series\.data \? series\.data\[index\] : 0\);/.test(src), "coercion changed");
  assert(/return Number\.isFinite\(value\) \? value : 0;/.test(src), "fallback changed");
});
test("combo is deterministic for a fixed ctx", function () {
  deep(g.renderers.combo(variantCtx()), g.renderers.combo(variantCtx()));
});
test("combo emits one marker per label and non-negative rect widths", function () {
  var parts = g.renderers.combo(variantCtx());
  equal(parts.filter(function (p) { return p.indexOf("<circle ") === 0; }).length, 2);
  equal(parts.filter(function (p) { return p.indexOf("<polyline ") === 0; }).length, 1);
  assertNonNegativeRects(parts, "combo");
  assertFiniteParts(parts, "combo");
});
test("combo tolerates non-numeric values without NaN coordinates", function () {
  var ctx = variantCtx();
  ctx.model.series[0].data = ["x", null];
  ctx.model.series[1].data = [NaN, "7"];
  var parts = g.renderers.combo(ctx);
  assert(parts.length > 0, "no output");
  assertFiniteParts(parts, "combo/impure-data");
  assert(parts.join("").indexOf("NaN") === -1, "NaN leaked into SVG");
});
test("[STATIC] barGeometry accumulates positive/negative stacks", function () {
  var src = source(h.VARIANTS_REL);
  assert(/if \(value >= 0\) up \+= value; else down \+= value;/.test(src), "stack accumulation changed");
  assert(/positive = Math\.max\(positive, up\);/.test(src), "positive extent changed");
  assert(/negative = Math\.min\(negative, down\);/.test(src), "negative extent changed");
  assert(/low: negative < 0 \? negative \* 1\.1 : 0/.test(src), "low extent changed");
});
test("clustered-column geometry is deterministic and non-negative", function () {
  var parts = g.renderers["clustered-column"](variantCtx(false));
  deep(parts, g.renderers["clustered-column"](variantCtx(false)));
  assertNonNegativeRects(parts, "clustered-column");
  assertFiniteParts(parts, "clustered-column");
});
test("stacked-column geometry is deterministic and non-negative", function () {
  var parts = g.renderers["stacked-column"](variantCtx(true));
  deep(parts, g.renderers["stacked-column"](variantCtx(true)));
  assertNonNegativeRects(parts, "stacked-column");
  assertFiniteParts(parts, "stacked-column");
});
test("clustered-bar and stacked-bar geometry are deterministic", function () {
  ["clustered-bar", "stacked-bar"].forEach(function (name) {
    var parts = g.renderers[name](variantCtx(name === "stacked-bar"));
    deep(parts, g.renderers[name](variantCtx(name === "stacked-bar")));
    assertNonNegativeRects(parts, name);
    assertFiniteParts(parts, name);
  });
});
test("stacked and clustered variants produce different geometry", function () {
  assert(JSON.stringify(g.renderers["stacked-column"](variantCtx(true))) !==
    JSON.stringify(g.renderers["clustered-column"](variantCtx(false))), "variants identical");
});
test("bar extents handle mixed-sign input without negative dimensions", function () {
  var ctx = variantCtx();
  ctx.model.series[1].data = [-6, 3];
  ["stacked-column", "clustered-column", "stacked-bar", "clustered-bar"].forEach(function (name) {
    assertNonNegativeRects(g.renderers[name](ctx), name + "/mixed-sign");
  });
});

// ── hierarchy layout helpers ───────────────────────────────────────────
function hierarchyCtx() {
  return makeCtx(seriesDef([
    { label: "S1", color: "#0072B2", data: [25, 75] },
  ], ["A", "B"]));
}

test("treemap layout is deterministic for a fixed ctx", function () {
  deep(g.renderers.treemap(hierarchyCtx()), g.renderers.treemap(hierarchyCtx()));
});
test("treemap column widths follow the 1:3 input weights", function () {
  var rects = rectAttrs(g.renderers.treemap(hierarchyCtx()));
  equal(rects.length, 2);
  var ratio = (rects[1].w + 1) / (rects[0].w + 1);
  assert(Math.abs(ratio - 3) < 0.001, "expected 3, got " + ratio);
  assert(rects[1].x > rects[0].x, "columns not laid out left-to-right");
});
test("heatmap emits one cell per label/series pair, deterministically", function () {
  var parts = g.renderers.heatmap(hierarchyCtx());
  deep(parts, g.renderers.heatmap(hierarchyCtx()));
  equal(rectAttrs(parts).length, 2 * 1 + 2);
  assertFiniteParts(parts, "heatmap");
});
test("sunburst arc layout is deterministic and NaN-free", function () {
  var parts = g.renderers.sunburst(hierarchyCtx());
  deep(parts, g.renderers.sunburst(hierarchyCtx()));
  assert(parts.length > 0, "no arcs");
  assert(parts.join("").indexOf("NaN") === -1, "NaN in arc path");
  equal(parts.filter(function (p) { return p.indexOf("<path ") === 0; }).length, 4);
});
