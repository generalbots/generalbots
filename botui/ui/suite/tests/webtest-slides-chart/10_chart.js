/**
 * Slides chart helpers — invariant tests (#2486).
 * Exercises the DOM-free helpers of the three SHIPPED modules loaded by
 * 00_harness.js; no production logic is modified or duplicated here.
 *
 * Two helpers (strArray in 75, patternMark in 76) are declared but NOT exported
 * on their namespaces. They are verified two honest ways: RUNTIME through the
 * exported consumer that calls them (normalize / patternDefs), and a clearly
 * labelled STATIC source assertion at the end of this file. No runtime result
 * is fabricated for a symbol that is not actually reachable.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var C = h.C;
var reset = h.reset;
var readFile = h.readFile;

function chart(patch) {
  return C.normalize(patch || {});
}
function js(value) {
  return JSON.stringify(value);
}

// ── API surface ────────────────────────────────────────────────────────
test("chart namespace exposes state, style and link surfaces", function () {
  assert(typeof C.fresh === "function" && typeof C.num === "function", "state helpers missing");
  assert(C.style && typeof C.style.tokens === "function", "style surface missing");
  assert(C.link && typeof C.link.parseRange === "function", "link surface missing");
});

// ── State helpers (75_charts_state.js) ────────────────────────────────
test("fresh() returns a deterministic version/charts/order shape", function () {
  var a = C.fresh();
  var b = C.fresh();
  assert(a.version === 1, "version must be 1, got " + a.version);
  assert(a.charts && typeof a.charts === "object" && !Array.isArray(a.charts), "charts must be an object");
  assert(Array.isArray(a.order) && a.order.length === 0, "order must be an empty array");
  assert(Object.keys(a.charts).length === 0, "charts must start empty");
  assert(js(a) === js(b), "fresh() is not deterministic");
});

test("num() returns finite values and falls back for NaN/Infinity/junk", function () {
  assert(C.num(5, 9) === 5, "finite number not preserved");
  assert(C.num("3", 9) === 3, "numeric string not coerced");
  assert(C.num(NaN, 9) === 9, "NaN did not fall back");
  assert(C.num(Infinity, 9) === 9, "Infinity did not fall back");
  assert(C.num(-Infinity, 9) === 9, "-Infinity did not fall back");
  assert(C.num("junk", 9) === 9, "junk did not fall back");
  assert(C.num(undefined, 9) === 9, "undefined did not fall back");
});

test("num(null, fallback) is a finite zero (Number coercion, documented)", function () {
  // Number(null) === 0: the code intentionally coerces, it does not fall back.
  var value = C.num(null, 9);
  assert(isFinite(value) && value === 0, "expected finite 0 for null, got " + value);
});

test("num() is deterministic across repeated calls", function () {
  assert(C.num("2.5", 0) === C.num("2.5", 0), "num() not deterministic");
});

test("strArray coercion observable via normalize(): null/undefined -> empty string", function () {
  var categories = chart({ categories: [1, "2", null, undefined, true] }).categories;
  assert(js(categories) === js(["1", "2", "", "", "true"]),
    "categories not coerced safely: " + js(categories));
});

test("strArray coercion observable via normalize(): non-array -> []", function () {
  assert(js(chart({ categories: "not-an-array" }).categories) === "[]", "non-array categories accepted");
  assert(js(chart({}).categories) === "[]", "missing categories accepted");
});

test("normalize() pads series values to the category length", function () {
  var values = chart({ categories: ["A", "B", "C"], series: [{ values: [1] }] }).series[0].values;
  assert(js(values) === js([1, 0, 0]), "series not padded: " + js(values));
});

test("normalize() supplies a placeholder series when none is given", function () {
  var result = chart({ categories: ["A", "B"] });
  assert(result.series.length === 1, "expected one placeholder series");
  assert(result.series[0].id === "s1", "placeholder id wrong");
  assert(js(result.series[0].values) === js([0, 0]), "placeholder values wrong");
});

test("normalize() clamps holeSize into [0.15, 0.85]", function () {
  assert(chart({ options: { holeSize: 5 } }).options.holeSize === 0.85, "upper clamp failed");
  assert(chart({ options: { holeSize: -1 } }).options.holeSize === 0.15, "lower clamp failed");
  assert(chart({}).options.holeSize === 0.55, "default holeSize wrong");
});

test("normalize() validates type, axis and numberFormat against declared enums", function () {
  var result = chart({
    type: "bogus",
    numberFormat: "bogus",
    series: [{ axis: "bogus" }],
  });
  assert(result.type === "column", "invalid type not defaulted");
  assert(result.numberFormat === "0", "invalid numberFormat not defaulted");
  assert(result.series[0].axis === "primary", "invalid axis not defaulted");
  assert(C.TYPES.indexOf("combo") >= 0, "combo type missing from enum");
});

test("serialize()/deserialize() round-trip a definition deterministically", function () {
  var source = chart({
    id: "c1", type: "combo", categories: ["Q1", "Q2"],
    series: [{ id: "a", values: [1, 2], axis: "primary" }, { id: "b", values: [3, 4], axis: "secondary" }],
  });
  var reopened = C.deserialize(C.serialize(source));
  assert(js(source) === js(reopened), "round-trip changed the definition");
});

test("deserialize() returns null for malformed JSON (safe)", function () {
  assert(C.deserialize("{not json") === null, "malformed JSON did not return null");
});

test("selfCheck() verifies the combo secondary-axis round-trip", function () {
  var result = C.selfCheck();
  assert(result.pass === true, "selfCheck failed: " + js(result.failed));
  assert(result.failed.length === 0, "selfCheck reported failures: " + js(result.failed));
  assert(result.checks.some(function (item) { return item.name === "axis-margin"; }),
    "secondary-axis check missing");
});

test("verifyRoundTrip() reports per-check detail", function () {
  var source = chart({ categories: ["A"], series: [{ id: "s", values: [1], axis: "primary" }] });
  var result = C.verifyRoundTrip(source, C.deserialize(C.serialize(source)));
  assert(result.pass === true, "identical definitions must compare equal");
  assert(result.checks.length >= 6, "expected detailed checks, got " + result.checks.length);
});

test("list()/forSlide()/get() scope charts by slide", function () {
  reset();
  C.create({ id: "a", slideIndex: 0, categories: ["A"] });
  C.create({ id: "b", slideIndex: 1, categories: ["B"] });
  assert(C.list().length === 2, "expected two charts");
  assert(C.forSlide(0).length === 1 && C.forSlide(0)[0].id === "a", "slide 0 scope wrong");
  assert(C.forSlide(1).length === 1 && C.forSlide(1)[0].id === "b", "slide 1 scope wrong");
  assert(C.get("a") !== null && C.get("missing") === null, "get() lookup wrong");
});

// ── Style helpers (76_charts_style.js) ────────────────────────────────
test("tokens(theme) returns a stable palette for a fixed theme", function () {
  var flat = { "color.accent1": "#111111", "font.major": "Georgia" };
  var theme = { flat: flat };
  assert(js(C.style.tokens(theme)) === js(flat), "tokens() did not return theme.flat");
  assert(C.style.tokens(theme) === flat, "tokens() must return the same flat object");
  assert(js(C.style.tokens(theme)) === js(C.style.tokens(theme)), "tokens() not stable");
});

test("tokens() returns {} for null and flatless themes", function () {
  assert(js(C.style.tokens(null)) === "{}", "null theme did not yield {}");
  assert(js(C.style.tokens({})) === "{}", "flatless theme did not yield {}");
});

test("parseColor() returns a deterministic RGB triple for #abc and #aabbcc", function () {
  assert(js(C.style.parseColor("#abc")) === js([170, 187, 204]), "short hex wrong");
  assert(js(C.style.parseColor("#AABBCC")) === js([170, 187, 204]), "uppercase hex wrong");
  assert(js(C.style.parseColor("#aabbcc")) === js([170, 187, 204]), "long hex wrong");
  assert(js(C.style.parseColor("#abc")) === js(C.style.parseColor("#abc")), "parseColor not deterministic");
});

test("parseColor() parses rgb()/rgba() into a triple", function () {
  assert(js(C.style.parseColor("rgb(1, 2, 3)")) === js([1, 2, 3]), "rgb wrong");
  assert(js(C.style.parseColor("rgba(10,20,30,0.5)")) === js([10, 20, 30]), "rgba wrong");
});

test("parseColor() returns null for invalid input (safe)", function () {
  assert(C.style.parseColor("nonsense") === null, "junk did not return null");
  assert(C.style.parseColor("") === null, "empty string did not return null");
  assert(C.style.parseColor(null) === null, "null did not return null");
});

test("luminance() of white > black and deterministic", function () {
  var white = C.style.luminance([255, 255, 255]);
  var black = C.style.luminance([0, 0, 0]);
  assert(white > black, "white must be brighter than black");
  assert(Math.abs(white - 1) < 1e-9, "white luminance should be 1, got " + white);
  assert(Math.abs(black) < 1e-9, "black luminance should be 0, got " + black);
  assert(C.style.luminance([128, 128, 128]) === C.style.luminance([128, 128, 128]),
    "luminance not deterministic");
});

test("contrast() and palette() fallbacks are deterministic", function () {
  assert(C.style.contrast("#000000", "#ffffff") >= 20, "black/white contrast too low");
  var fallback = C.style.palette({});
  assert(fallback.length === 6 && fallback[0] === "#3b82f6", "fallback palette wrong: " + js(fallback));
});

test("palette() reads accent tokens from a fixed theme", function () {
  var palette = C.style.palette({ flat: {
    "color.accent1": "#111", "color.accent2": "#222", "color.accent3": "#333",
  } });
  assert(js(palette) === js(["#111", "#222", "#333"]), "theme accents ignored: " + js(palette));
});

test("seriesColor() honours a per-series override", function () {
  var overridden = chart({ series: [{ color: "#ff0000" }] });
  assert(C.style.seriesColor(overridden, 0, {}) === "#ff0000", "override ignored");
  var themed = chart({ series: [{}] });
  assert(C.style.seriesColor(themed, 0, { flat: { "color.accent1": "#abcdef" } }) === "#abcdef",
    "theme colour ignored");
});

test("seriesPattern()/fillDescriptor() produce deterministic descriptors", function () {
  var safe = chart({ a11y: { colorBlindSafe: true } });
  var plain = chart({ a11y: { colorBlindSafe: false } });
  assert(C.style.seriesPattern(plain, 0) === "solid", "plain series must be solid");
  assert(C.style.seriesPattern(safe, 1) === "diagonal", "pattern order wrong");
  assert(js(C.style.fillDescriptor(plain, 0, {})) === js({ fill: C.style.palette({})[0], pattern: "solid" }),
    "solid descriptor wrong");
  var patterned = C.style.fillDescriptor(safe, 1, {});
  assert(patterned.pattern === "diagonal" && patterned.patternId === "gbch-pat-diagonal",
    "patternId missing for non-solid pattern");
});

test("formatNumber() applies each number format deterministically", function () {
  assert(C.style.formatNumber(1234567, "0") === "1,234,567", "grouping wrong");
  assert(C.style.formatNumber(0.125, "0%") === "13%", "percent wrong");
  assert(C.style.formatNumber(5, "currency") === "$5.00", "currency wrong");
  assert(C.style.formatNumber(1.2, "0.0") === "1.2", "decimal wrong");
});

test("contrastWarnings() flags low-contrast series and respects the opt-out", function () {
  var low = chart({ series: [{ color: "#ffffff" }], a11y: { contrastCheck: true } });
  var warnings = C.style.contrastWarnings(low, {}, "#ffffff");
  assert(warnings.length === 1 && warnings[0].seriesIndex === 0, "low contrast not flagged");
  var off = chart({ series: [{ color: "#ffffff" }], a11y: { contrastCheck: false } });
  assert(C.style.contrastWarnings(off, {}, "#ffffff").length === 0, "opt-out ignored");
});

test("patternMark geometry observable via patternDefs(): diagonal/cross/solid", function () {
  var defs = C.style.patternDefs(chart({ a11y: { colorBlindSafe: true } }), {});
  assert(defs.length === 6, "expected six pattern defs, got " + defs.length);
  var byId = {};
  defs.forEach(function (def) { byId[def.attrs.id] = def; });
  assert(byId["gbch-pat-diagonal"].children[1].attrs.d === "M0 8 L8 0", "diagonal geometry wrong");
  assert(byId["gbch-pat-cross"].children[1].attrs.d === "M0 4 H8 M4 0 V8", "cross geometry wrong");
  assert(byId["gbch-pat-solid"].children[1].tag === "circle", "dots geometry wrong");
  assert(C.style.patternDefs(chart({ a11y: { colorBlindSafe: false } }), {}).length === 0,
    "patterns emitted when colour-blind-safe is off");
});

// ── Sheet linking helpers (80_charts_linking.js) ──────────────────────
test("colIndex() maps A1 column letters deterministically", function () {
  assert(C.link.colIndex("A") === 1, "A must be 1");
  assert(C.link.colIndex("C") === 3, "C must be 3");
  assert(C.link.colIndex("AA") === 27, "AA must be 27");
});

test("parseRange() parses Sheet1!A1:C12 into bounds", function () {
  assert(js(C.link.parseRange("Sheet1!A1:C12")) === js({
    sheet: "Sheet1", startCol: 1, startRow: 1, endCol: 3, endRow: 12,
  }), "range bounds wrong");
});

test("parseRange() returns null for invalid ranges (safe)", function () {
  assert(C.link.parseRange("not-a-range") === null, "junk did not return null");
  assert(C.link.parseRange("Sheet1!A1") === null, "half range did not return null");
});

test("validRange() rejects reversed and malformed ranges", function () {
  assert(C.link.validRange("Sheet1!A1:C12") === true, "valid range rejected");
  assert(C.link.validRange("Sheet1!C12:A1") === false, "reversed range accepted");
  assert(C.link.validRange("bad") === false, "malformed range accepted");
});

function linkedChart(id, range, refreshOnOpen) {
  reset();
  C.create({ id: id, slideIndex: 0, categories: ["Q1"], series: [{ id: "s1", values: [1] }] });
  return C.link.setLink(id, range, refreshOnOpen);
}

test("setLink() binds a valid sheet range deterministically", function () {
  var a = linkedChart("c1", "Sheet1!A1:C12", true);
  assert(a !== null && a.link.range === "Sheet1!A1:C12", "range not bound");
  assert(a.link.refreshOnOpen === true, "refreshOnOpen not set");
  assert(a.link.stale === false && a.link.lastRefreshed === 0, "initial link state wrong");
  var b = linkedChart("c1", "Sheet1!A1:C12", true);
  assert(js(C.link.status(a)) === js(C.link.status(b)), "setLink() not deterministic");
});

test("setLink() returns null for an invalid range (safe)", function () {
  reset();
  C.create({ id: "c1", categories: ["Q1"] });
  assert(C.link.setLink("c1", "not-a-range", true) === null, "invalid range accepted");
  assert(C.get("c1").link.range === "", "invalid range mutated the chart");
});

test("markSourceChanged() -> STALE; refresh() clears STALE with a timestamp", function () {
  linkedChart("c1", "Sheet1!A1:C12", true);
  C.link.markSourceChanged("c1");
  assert(C.link.status(C.get("c1")).stale === true, "source change did not mark stale");
  var result = C.link.refresh("c1", undefined, 12345);
  assert(result.lastRefreshed === 12345 && result.stale === false, "refresh result wrong");
  var status = C.link.status(C.get("c1"));
  assert(status.stale === false && status.lastRefreshed === 12345, "stored link not refreshed");
});

test("columnsFromMatrix() binds a sheet matrix deterministically", function () {
  var matrix = [["Category", "Revenue", "Margin"], ["Q1", 10, 2], ["Q2", 20, 4]];
  var a = C.link.columnsFromMatrix(matrix, true);
  var b = C.link.columnsFromMatrix(matrix, true);
  assert(js(a) === js(b), "matrix binding not deterministic");
  assert(js(a.categories) === js(["Q1", "Q2"]), "categories wrong: " + js(a.categories));
  assert(a.series.length === 2 && a.series[0].name === "Revenue" && a.series[1].name === "Margin",
    "series names wrong: " + js(a.series));
  assert(js(a.series[0].values) === js([10, 20]), "series values wrong");
});

test("columnsFromMatrix() handles empty/missing input safely", function () {
  assert(js(C.link.columnsFromMatrix([], true)) === js({ categories: [], series: [] }), "empty matrix wrong");
  assert(js(C.link.columnsFromMatrix(null, true)) === js({ categories: [], series: [] }), "null matrix wrong");
  var noHeader = C.link.columnsFromMatrix([["Q1", 10], ["Q2", 20]], false);
  assert(js(noHeader.categories) === js(["Q1", "Q2"]), "headerless categories wrong");
  assert(noHeader.series[0].name === "Column 1", "headerless series name wrong");
});

test("applyMatrix() refreshes categories/series with an explicit timestamp", function () {
  reset();
  C.create({ id: "c1", categories: ["old"], series: [{ id: "s1", color: "#abcdef", axis: "secondary" }] });
  C.link.setLink("c1", "Sheet1!A1:C12", true);
  C.link.markSourceChanged("c1");
  var matrix = [["Category", "Revenue", "Margin"], ["Q1", 10, 2], ["Q2", 20, 4]];
  var result = C.link.applyMatrix("c1", matrix, true, undefined, 999);
  assert(js(result.categories) === js(["Q1", "Q2"]), "categories not refreshed");
  assert(result.lastRefreshed === 999 && result.stale === false, "refresh state wrong");
  assert(result.series[0].color === "#abcdef" && result.series[0].axis === "secondary",
    "series override not preserved");
  assert(C.link.status(C.get("c1")).lastRefreshed === 999, "stored timestamp wrong");
});

test("status() and clearLink() reflect the binding", function () {
  linkedChart("c1", "Sheet1!A1:C12", true);
  var status = C.link.status(C.get("c1"));
  assert(status.linked === true && status.range === "Sheet1!A1:C12", "linked status wrong");
  assert(C.link.clearLink("c1") !== null, "clearLink failed");
  var cleared = C.link.status(C.get("c1"));
  assert(cleared.linked === false && cleared.range === "", "clearLink did not unbind");
});

test("refreshOnOpen() refreshes only opted-in charts", function () {
  reset();
  C.create({ id: "c1", categories: ["Q1"] });
  C.create({ id: "c2", categories: ["Q2"] });
  C.link.setLink("c1", "Sheet1!A1:C12", true);
  C.link.setLink("c2", "Sheet2!A1:B5", false);
  assert(js(C.link.refreshOnOpen(C.state)) === js(["c1"]), "wrong charts refreshed on open");
});

// ── Regression guards for locally reproducible crashes ────────────────
test("palette() terminates when the default master theme has no accents", function () {
  var win = h.sandbox.window;
  var prior = win.GBSlidesMasters;
  var cleared = { id: "theme_default", flat: { "color.accent1": "" } };
  win.GBSlidesMasters = { state: {
    themes: { theme_default: cleared }, master: { defaultThemeId: "theme_default" },
  } };
  try {
    var palette = C.style.palette(cleared);
    assert(js(palette) === js(C.style.palette({})), "self-referential theme did not fall back");
    assert(palette.length === 6 && palette[0] === "#3b82f6", "fallback palette wrong: " + js(palette));
  } finally {
    if (prior === undefined) delete win.GBSlidesMasters; else win.GBSlidesMasters = prior;
  }
});

test("applyMatrix() ignores a malformed stored range (no null dereference)", function () {
  reset();
  C.create({ id: "badlink", categories: ["A"], series: [{ id: "s1", values: [1] }],
    link: { range: "not-a-range" } });
  var before = js(C.get("badlink"));
  var result = C.link.applyMatrix("badlink", [["Cat", "Val"], ["Q1", 5]], true, undefined, 77);
  assert(result === null, "malformed range must not be applied");
  assert(js(C.get("badlink")) === before, "chart mutated by invalid applyMatrix");
});

// ── STATIC source assertions (labelled; not runtime results) ──────────
// These two helpers are private (no C.strArray / S.patternMark export), so the
// runtime checks above exercise them through their exported consumers. The
// source assertions below only record the declaration/consumption boundary.
test("STATIC: strArray is declared and consumed by normalize", function () {
  var source = readFile("slides/modules/75_charts_state.js");
  assert(/function\s+strArray\s*\(/.test(source), "strArray declaration missing");
  assert(/strArray\(raw\.categories\)/.test(source), "strArray not consumed by normalize");
});

test("STATIC: patternMark is declared and consumed by patternDefs", function () {
  var source = readFile("slides/modules/76_charts_style.js");
  assert(/function\s+patternMark\s*\(/.test(source), "patternMark declaration missing");
  assert(/patternMark\(name,\s*S\.seriesColor/.test(source), "patternMark not consumed by patternDefs");
});

test("STATIC: linking module contains no real Sheet fetch primitive", function () {
  var source = readFile("slides/modules/80_charts_linking.js");
  assert(!/\bfetch\s*\(|XMLHttpRequest/.test(source),
    "linking module unexpectedly performs a real network fetch");
});
