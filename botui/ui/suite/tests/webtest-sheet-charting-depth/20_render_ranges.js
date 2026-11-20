/**
 * RUNTIME assertions for the shipped render integration (36) and live range
 * binding (31) that feeds it. Determinism is proven by calling the REAL
 * renderSvg twice under a fixed clock and comparing the emitted SVG strings.
 */
"use strict";

var h = require("./00_harness");
var e = h.engine;
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var deep = h.deep;
var note = h.note;

function svgFor(type, options, analysis, data, labels) {
  var def = e.seriesDef(data || [{ label: "s", data: [10, -4, 6] }], labels || ["Q1", "Q2", "Q3"], options);
  def.chart_type = type;
  if (analysis) def.analysis = analysis;
  return def;
}

// ── renderSvg determinism and structure (36) ────────────────────────────
test("renderSvg emits a deterministic, accessible SVG frame", function () {
  var def = svgFor("waterfall");
  var first = e.charting.renderSvg(def, 480, 300);
  var second = e.charting.renderSvg(def, 480, 300);
  equal(first, second);
  assert(first.indexOf("<svg ") === 0 && first.endsWith("</svg>"), "not a complete SVG");
  assert(first.indexOf('role="img"') !== -1, "missing role=img");
  assert(first.indexOf('aria-label="') !== -1, "missing alt text");
  assert(first.indexOf("width=\"480\"") !== -1 && first.indexOf("height=\"300\"") !== -1, "wrong canvas");
});
test("every registered chart type renders without NaN or an exception", function () {
  Object.keys(e.renderers).forEach(function (type) {
    var svg = e.charting.renderSvg(svgFor(type), 480, 300);
    assert(svg.indexOf("<svg ") === 0 && svg.endsWith("</svg>"), type + ": bad frame");
    assert(svg.indexOf("NaN") === -1, type + ": NaN in SVG");
    equal(svg, e.charting.renderSvg(svgFor(type), 480, 300), type + ": not deterministic");
  });
  note("types rendered: " + Object.keys(e.renderers).length);
});
test("unknown chart types still produce an empty, labelled frame", function () {
  var svg = e.charting.renderSvg(svgFor("not-a-chart"), 320, 200);
  assert(svg.indexOf("<svg ") === 0 && svg.endsWith("</svg>"), "no frame for unknown type");
  assert(svg.indexOf("not-a-chart") !== -1, "type label absent from alt text");
  assert(svg.indexOf("<polyline ") === -1, "unexpected marks for unknown type");
});
test("title and data-range flow into deterministic alt text", function () {
  var plain = e.charting.renderSvg(svgFor("waterfall"), 480, 300);
  var titled = svgFor("waterfall");
  var fixtureTitle = "Board variance";
  titled.title = fixtureTitle;
  var withTitle = e.charting.renderSvg(titled, 480, 300);
  assert(withTitle.indexOf(fixtureTitle) !== -1, "title missing from frame");
  assert(withTitle !== plain, "title did not change the frame");
  var ranged = svgFor("combo", null, null, [{ label: "s", data: [1, 2, 3] }], ["a", "b", "c"]);
  ranged.data_range = "A1:B4";
  assert(e.charting.renderSvg(ranged, 480, 300).indexOf(" from A1:B4") !== -1,
    "range not reflected in alt text");
});
test("renderSvg delegates number formatting to the host gbFormatNumber stub", function () {
  var def = svgFor("waterfall");
  e.withFormat(function (value) { return "[" + value + "]"; }, function () {
    var svg = e.charting.renderSvg(def, 480, 300);
    assert(svg.indexOf("&#91;") !== -1 || svg.indexOf("[") !== -1, "formatted number missing");
    assert(svg.indexOf("NaN") === -1, "formatting broke coordinates");
  });
});
test("t() resolves gbT first, then SheetI18n, then the English fallback", function () {
  var i18n = e.window.SheetI18n.t;
  try {
    e.window.SheetI18n.t = function () { return "from-i18n"; };
    e.withTranslator(function () { return "from-gbt"; }, function () {
      equal(e.charting.t("sheet-charting-total", "Total"), "from-gbt");
    });
    e.withTranslator(null, function () {
      equal(e.charting.t("sheet-charting-total", "Total"), "from-i18n");
    });
    e.window.SheetI18n.t = null;
    e.withTranslator(null, function () {
      equal(e.charting.t("sheet-charting-total", "Total"), "Total");
    });
  } finally {
    e.window.SheetI18n.t = i18n;
  }
});

// ── local chart store (36) ───────────────────────────────────────────────
test("addLocal uses the fixed clock/random and marks the chart local-only", function () {
  var before = e.charting.localCharts().length;
  var added = e.charting.addLocal(svgFor("pareto"));
  equal(added.localOnly, true);
  equal(added.id, "local-" + h.FIXED_NOW + "-" + Math.floor(h.FIXED_RANDOM * 1000));
  equal(e.charting.localCharts().length, before + 1);
  assert(e.charting.isAdvanced(added.chart_type), "added type not recognised");
});
test("local charts are keyed by sheet id and worksheet index", function () {
  var baseline = e.charting.localCharts().length;
  e.withSheetId("another-sheet-2490", function () {
    equal(e.charting.localCharts().length, 0, "sheet store leaked");
  });
  e.withIndex(7, function () {
    equal(e.charting.localCharts().length, 0, "worksheet store leaked");
  });
  equal(e.charting.localCharts().length, baseline, "default store changed");
  assert(e.scheduled.length >= 1 && e.scheduled[0].ms === 0, "boot applyAll not scheduled");
  assert(typeof e.handlers["gb-sheet-locale"] === "function", "locale listener not registered");
  assert(typeof e.bodyHandlers["htmx:afterSwap"] === "function", "htmx listener not registered");
});

// ── dynamic range binding (31) ───────────────────────────────────────────
function gridRows(rows) {
  var cells = e.makeMap([]);
  Object.keys(rows).forEach(function (key) { cells.set(key, { value: rows[key] }); });
  return { cells: cells };
}
function rangeDef(range, options) {
  return { chart_type: "combo", data_range: range, options: options || {} };
}

test("parseRange normalises sheet-qualified, absolute and reversed references", function () {
  deep(e.data.parseRange("Data!$C$2:$A$1"), {
    sheet: "Data", start: { row: 0, col: 0 }, end: { row: 1, col: 2 },
  });
  assert(e.data.parseRange("A1:B2:C3") === null, "three-part range accepted");
});
test("bindRange reads a live vm-native Map grid with labels and a header", function () {
  e.setGrid(gridRows({
    "0,0": "Region", "0,1": "Sales",
    "1,0": "North", "1,1": 10,
    "2,0": "South", "2,1": 20,
  }));
  var bound = e.data.bindRange("A1:B3");
  deep(bound.labels, ["North", "South"]);
  deep(bound.series[0].data, [10, 20]);
  equal(bound.series[0].label, "Sales");
});
test("a range expands when rows are added without changing the definition", function () {
  var cells = e.makeMap([
    ["0,0", { value: "Region" }], ["0,1", { value: "Sales" }],
    ["1,0", { value: "North" }], ["1,1", { value: 10 }],
    ["2,0", { value: "South" }], ["2,1", { value: 20 }],
    ["3,0", { value: "East" }], ["3,1", { value: 30 }],
    ["4,0", { value: "West" }], ["4,1", { value: 40 }],
  ]);
  e.setGrid({ cells: cells });
  var def = rangeDef("A1:B6");
  var before = e.data.model(def);
  /* The still-empty fifth row inside the range gets a synthetic row-number
   * label and null data; filling it later expands the bound chart in place. */
  deep(before.labels, ["North", "South", "East", "West", "6"]);
  deep(before.series[0].data, [10, 20, 30, 40, null]);
  cells.set("5,0", { value: "Central" });
  cells.set("5,1", { value: 50 });
  var after = e.data.model(def);
  deep(after.labels, ["North", "South", "East", "West", "Central"]);
  deep(after.series[0].data, [10, 20, 30, 40, 50]);
  equal(def.data_range, "A1:B6", "definition was mutated");
});
test("renderSvg re-reads expanded range data on the next call", function () {
  var cells = e.makeMap([
    ["0,0", { value: "Region" }], ["0,1", { value: "Sales" }],
    ["1,0", { value: "North" }], ["1,1", { value: 10 }],
    ["2,0", { value: "South" }], ["2,1", { value: 20 }],
  ]);
  e.setGrid({ cells: cells });
  var def = rangeDef("A1:B3");
  var before = e.charting.renderSvg(def, 480, 300);
  cells.set("2,1", { value: 99 });
  var after = e.charting.renderSvg(def, 480, 300);
  assert(before !== after, "render did not observe live grid change");
  deep(e.data.model(def).series[0].data, [10, 99]);
  equal(after, e.charting.renderSvg(def, 480, 300), "render not deterministic after change");
});
test("range values are clamped to the definition's min/max options", function () {
  e.setGrid(gridRows({ "0,0": "Q", "0,1": "V", "1,0": "a", "1,1": -5, "2,0": "b", "2,1": 20 }));
  var model = e.data.model(rangeDef("A1:B3", { min: 0, max: 15 }));
  deep(model.series[0].data, [0, 15]);
  equal(model.min, 0);
  equal(model.max, 15);
});
test("empty ranges stay finite end to end", function () {
  e.setGrid({ cells: e.makeMap([]) });
  var def = rangeDef("A1:B3");
  var model = e.data.model(def);
  /* Observed: an all-empty label column is treated as data, so bindRange
   * synthesises row-number labels; no series values are produced. */
  deep(model.labels, ["1", "2", "3"]);
  deep(model.values, []);
  var svg = e.charting.renderSvg(def, 480, 300);
  assert(svg.indexOf("NaN") === -1, "empty range produced NaN");
  assert(svg.indexOf('aria-label="') !== -1, "empty range lost alt text");
});
