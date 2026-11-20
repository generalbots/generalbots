/**
 * Structured data / what-if — invariant tests (#2436).
 * Every runtime check calls a helper exported by a shipped module loaded from
 * disk by 00_harness.js. Scope gaps are labelled STATIC and never presented as
 * runtime results.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var env = h.loadModules();
var S = env.S;
var Spark = env.Spark;
var Slicer = env.Slicer;
var Goal = env.Goal;
var Scenarios = env.Scenarios;

function set(ref, data) { h.setCell(env, ref, data); }
function value(ref) {
  var pos = S.parseRef(ref);
  return S.cellValue(pos.row, pos.col);
}
function dataRows(html) { return html.split("<tr>").length - 1; }
function chips() {
  return h.findAll(env.document.body, function (node) {
    return node.className.indexOf("ss-structured-chip") !== -1;
  });
}
function tableHost() {
  return h.findAll(env.document.body, function (node) {
    return node.className === "ss-structured-table-host";
  }).pop();
}
function clearButton() {
  var footer = h.findAll(env.document.body, function (node) {
    return node.className === "ss-structured-footer";
  }).pop();
  assert(footer && footer.children.length > 1, "slicer footer not found");
  return footer.children[footer.children.length - 1];
}

var SOURCES = {};
h.MODULE_RELS.forEach(function (rel) { SOURCES[rel] = h.readFile(rel); });
var ALL_SOURCE = h.MODULE_RELS.map(function (rel) { return SOURCES[rel]; }).join("\n");
var SLICER_SRC = SOURCES["sheet/js/modules-sheet-advanced/40_structured_slicer.js"];
var SCENARIO_SRC = SOURCES["sheet/js/modules-sheet-advanced/42_structured_scenarios.js"];

// ── Exported surface ──────────────────────────────────────────────────
test("core exports the addressing, state and write helpers", function () {
  ["t", "state", "save", "mutate", "parseRef", "parseRange", "colName", "refOf",
    "cellData", "cellValue", "numeric", "writeValue", "writeValues", "escapeHtml",
    "createDialog", "setStatus", "notify"].forEach(function (name) {
    assert(typeof S[name] === "function", "core helper missing: " + name);
  });
});

test("feature modules export their documented entry points", function () {
  [["sparklines", Spark, ["open", "openEditor", "svg", "values", "repaint"]],
    ["slicer", Slicer, ["open", "table", "openSlicer"]],
    ["goal seek", Goal, ["open", "solve", "simpleModel"]],
    ["scenarios", Scenarios, ["open", "apply", "capture"]]].forEach(function (entry) {
    entry[1] && entry[2].forEach(function (name) {
      assert(typeof entry[1][name] === "function", entry[0] + " missing: " + name);
    });
  });
});

// ── Structured cell addressing (core) ─────────────────────────────────
test("parseRef resolves A1, absolute and multi-letter columns", function () {
  assert(JSON.stringify(S.parseRef("A1")) === JSON.stringify({ col: 0, row: 0 }), "A1 mismatch");
  assert(JSON.stringify(S.parseRef("$B$7")) === JSON.stringify({ col: 1, row: 6 }), "$B$7 mismatch");
  assert(JSON.stringify(S.parseRef("AA3")) === JSON.stringify({ col: 26, row: 2 }), "AA3 mismatch");
});

test("parseRef rejects malformed references without throwing", function () {
  [null, "", "A", "1A", "A-1"].forEach(function (ref) {
    assert(S.parseRef(ref) === null, "expected null for " + String(ref));
  });
});

test("RUNTIME #2436: parseRef rejects out-of-range A1 coordinates (regression)", function () {
  assert(S.parseRef("A0") === null, "row zero must be rejected");
  assert(S.parseRef("XFE1") === null, "column beyond XFD must be rejected");
  assert(S.parseRef("XFD1") !== null, "last valid column XFD must be accepted");
  assert(S.parseRef("A1") !== null, "first valid cell must be accepted");
});

test("parseRange normalises reversed endpoints and sheet prefixes", function () {
  var range = S.parseRange("Sheet!C3:A1");
  assert(range && range.start.row === 0 && range.start.col === 0, "start not normalised");
  assert(range.end.row === 2 && range.end.col === 2, "end not normalised");
  assert(S.parseRange("A1") === null, "single cell accepted as a range");
  assert(S.parseRange("nonsense") === null, "invalid range accepted");
});

test("colName and refOf round-trip across column boundaries", function () {
  [[0, "A"], [25, "Z"], [26, "AA"], [27, "AB"], [701, "ZZ"]].forEach(function (pair) {
    assert(S.colName(pair[0]) === pair[1], "colName(" + pair[0] + ") mismatch");
    assert(S.refOf(0, pair[0]) === pair[1] + "1", "refOf(" + pair[0] + ") mismatch");
  });
});

test("numeric accepts comma-formatted values and rejects non-numbers", function () {
  assert(S.numeric("1,234") === 1234, "comma value mismatch");
  assert(S.numeric(0) === 0, "zero rejected");
  ["", null, undefined, true, "abc", NaN].forEach(function (value) {
    assert(S.numeric(value) === null, "expected null for " + String(value));
  });
});

test("escapeHtml escapes markup in generated attributes", function () {
  assert(S.escapeHtml("<a & \"b\" 'c'>") === "&lt;a &amp; &quot;b&quot; &#39;c&#39;&gt;",
    "escapeHtml output mismatch");
});

// ── Local authoring state (no server schema) ───────────────────────────
test("local state starts empty and flags itself as local-only", function () {
  env.localStorage.clear();
  var initial = S.state();
  assert(initial.sparklines.length === 0 && initial.slicers.length === 0, "state not empty");
  assert(initial.scenarios.length === 0 && initial.localOnly === true, "localOnly flag missing");
});

test("mutate creates, renames and persists a local scenario entry", function () {
  env.localStorage.clear();
  S.mutate(function (state) { state.scenarios.push({ id: "sc-1", name: "Base", values: [] }); });
  S.mutate(function (state) { state.scenarios[0].name = "Renamed"; });
  assert(S.state().scenarios.length === 1, "entry not persisted");
  assert(S.state().scenarios[0].name === "Renamed", "rename not persisted");
});

test("malformed stored state falls back to the empty model", function () {
  env.localStorage.clear();
  env.localStorage.setItem("gb-sheet-structured:current", "{broken");
  assert(S.state().scenarios.length === 0, "malformed state not rejected");
});

// ── Slicer: header model, added column, filter, clear ──────────────────
function seedSlicer() {
  h.attachGrid(env);
  set("A1", { value: "Region" });
  set("B1", { value: "Amount" });
  set("A2", { value: "East" });
  set("B2", { value: "10" });
  set("A3", { value: "West" });
  set("B3", { value: "20" });
  set("A4", { value: "East" });
  set("B4", { value: "30" });
}

test("slicer table model reads headers and body rows deterministically", function () {
  seedSlicer();
  var first = Slicer.table("A1:B4");
  var second = Slicer.table("A1:B4");
  assert(first.headers.join("|") === "Region|Amount", "headers mismatch");
  assert(first.rows.length === 3 && first.rows[0].join("|") === "East|10", "rows mismatch");
  assert(JSON.stringify(first) === JSON.stringify(second), "table read is not deterministic");
});

test("slicer table model reflects an added column", function () {
  seedSlicer();
  set("C1", { value: "Owner" });
  set("C2", { value: "Ann" });
  set("C3", { value: "Bob" });
  set("C4", { value: "Cid" });
  var table = Slicer.table("A1:C4");
  assert(table.headers.length === 3 && table.headers[2] === "Owner", "new header missing");
  assert(table.rows.every(function (row) { return row.length === 3; }), "new column missing from rows");
});

test("slicer filters by a header name and clears the selection", function () {
  seedSlicer();
  Slicer.openSlicer({ id: "slicer-test", range: "A1:B4", column: "region", selected: [] });
  var found = chips();
  assert(found.length === 2, "expected two distinct values, got " + found.length);
  var east = found.filter(function (node) { return node.textContent === "East"; })[0];
  assert(east, "value chip missing");
  h.fire(east, "click");
  var filtered = tableHost().innerHTML;
  assert(filtered.indexOf("West") === -1, "unselected value still visible");
  assert(dataRows(filtered) === 3, "expected one header plus two matching rows");
  h.fire(clearButton(), "click");
  var cleared = tableHost().innerHTML;
  assert(cleared.indexOf("West") !== -1, "cleared view lost a row");
  assert(dataRows(cleared) === 4, "cleared view did not restore all rows");
  assert(chips().every(function (chip) { return chip.getAttribute("aria-pressed") === "false"; }),
    "clear left chips pressed");
});

test("slicer supports multi-select across distinct values", function () {
  seedSlicer();
  Slicer.openSlicer({ id: "slicer-multi", range: "A1:B4", column: "Region", selected: [] });
  chips().forEach(function (chip) { h.fire(chip, "click"); });
  assert(dataRows(tableHost().innerHTML) === 4, "multi-select did not restore all rows");
});

// ── Sparklines ─────────────────────────────────────────────────────────
test("sparkline svg is deterministic and encodes a numeric series", function () {
  var first = Spark.svg([1, 2, 3], "line", "series");
  assert(first === Spark.svg([1, 2, 3], "line", "series"), "svg is not deterministic");
  assert(first !== Spark.svg([1, 3, 2], "line", "series"), "different series produced identical svg");
  assert(first.indexOf("<polyline") !== -1 && first.indexOf("viewBox=\"0 0 100 24\"") !== -1,
    "line geometry missing");
  var points = /points="([^"]+)"/.exec(first);
  assert(points && points[1].split(" ").length === 3, "point count mismatch");
  var parsed = points[1].split(" ").map(function (pair) { return pair.split(",").map(Number); });
  assert(parsed[2][1] < parsed[0][1], "ascending series did not rise on screen");
});

test("sparkline svg is empty for an empty series and renders bar/winloss shapes", function () {
  assert(Spark.svg([], "line", "series") === "", "empty series produced markup");
  var bar = Spark.svg([1, 2, 3], "bar", "series");
  assert((bar.match(/<rect /g) || []).length === 3, "bar series rect count mismatch");
  var loss = Spark.svg([-1, 2], "winloss", "series");
  var fills = (loss.match(/fill="(#[0-9a-fA-F]{6})"/g) || []);
  assert(fills.length >= 2 && fills[0] !== fills[1], "win/loss colours not distinct");
});

test("sparkline svg escapes its alternative text", function () {
  var svgText = Spark.svg([1], "line", "<x>&\"'");
  assert(svgText.indexOf("&lt;x&gt;") !== -1 && svgText.indexOf("<x>") === -1,
    "alt text was not escaped");
});

test("sparkline values read only numeric cells from the source range", function () {
  h.attachGrid(env);
  set("A1", { value: "1" });
  set("A2", { value: "2.5" });
  set("A3", { value: "n/a" });
  set("A4", { value: "3" });
  assert(JSON.stringify(Spark.values({ source: "A1:A4" })) === JSON.stringify([1, 2.5, 3]),
    "numeric series mismatch");
});

// ── Goal seek ──────────────────────────────────────────────────────────
test("goal seek models compound linear formulas and rejects unsupported forms", function () {
  assert(Goal.simpleModel("=B1*2+10", "B1", 3, "") === 16, "compound product model mismatch");
  assert(Goal.simpleModel("=2*B1-5", "B1", 3, "") === 1, "leading coefficient mismatch");
  assert(Goal.simpleModel("=-B1/2+8", "B1", 4, "") === 6, "signed quotient model mismatch");
  assert(Goal.simpleModel("=B1+B1+1", "B1", 3, "") === 7, "repeated variable mismatch");
  assert(Goal.simpleModel("=B1*B1", "B1", 3, "") === null, "nonlinear form accepted");
  assert(Goal.simpleModel("=C1*2", "B1", 3, "") === null, "foreign variable accepted");
});

test("goal seek converges on a target for a linear cell formula", function () {
  h.attachGrid(env);
  set("B1", { value: "1" });
  set("C1", { formula: "=B1*2+10" });
  var visits = [];
  var result = Goal.solve("C1", "30", "B1", function (step) { visits.push(step.index); });
  assert(result.converged === true, "solve did not converge: " + result.reason);
  assert(Math.abs(result.value - 10) < 1e-4, "wrong root: " + result.value);
  assert(Math.abs(result.residual) <= 1e-6, "residual above tolerance");
  assert(visits.length > 0 && visits[0] === 0, "iteration callback not observed");
  assert(result.engine === "simple", "unexpected engine: " + result.engine);
});

test("goal seek reports non-convergence instead of inventing a root", function () {
  h.attachGrid(env);
  set("B1", { value: "1" });
  set("C1", { formula: "=5" });
  var result = Goal.solve("C1", "100", "B1");
  assert(result.converged === false, "constant formula reported convergence");
  assert(result.reason === "nobacket", "unexpected reason: " + result.reason);
  assert(Array.isArray(result.iterations), "iteration log missing");
});

test("goal seek rejects invalid references without throwing", function () {
  var result = Goal.solve("BAD", "30", "B1");
  assert(result.converged === false && result.reason === "invalid", "invalid input accepted");
});

// ── Scenarios ──────────────────────────────────────────────────────────
test("scenario capture records changing cells and apply restores them", function () {
  h.attachGrid(env);
  set("B1", { value: "10" });
  set("C1", { value: "20" });
  var snapshot = Scenarios.capture(["B1", "C1"]);
  assert(snapshot.length === 2 && snapshot[0].ref === "B1" && snapshot[0].value === "10",
    "capture mismatch");
  set("B1", { value: "99" });
  set("C1", { value: "88" });
  var undo = [];
  env.window.SheetUndo = {
    beforeBulk: function (label, keys) { undo.push(keys.slice()); },
    recordBulk: function () { undo.push("record"); },
  };
  Scenarios.apply({ id: "sc-1", name: "base", values: snapshot }, null);
  env.window.SheetUndo = null;
  assert(value("B1") === "10" && value("C1") === "20", "apply did not restore values");
  assert(undo.length === 2 && undo[0].join("|") === "0,1|0,2", "undo hooks not observed");
  assert(undo[1] === "record", "undo was not recorded");
});

// ── Clearly labelled STATIC scope checks ───────────────────────────────
test("STATIC scope gap: no Excel Table object or structured-reference parser ships", function () {
  ["[#Headers]", "[#Totals]", "[@", "createTable", "renameTable", "addColumn", "structuredRef"]
    .forEach(function (token) {
      assert(ALL_SOURCE.indexOf(token) === -1, "unexpected production token: " + token);
    });
});

test("STATIC: slicer column resolution is name-insensitive and 1-based numeric", function () {
  assert(SLICER_SRC.indexOf("header.toLowerCase() === lowered") !== -1,
    "header-name resolution pattern missing");
  assert(SLICER_SRC.indexOf("parseInt(raw, 10) - 1") !== -1,
    "numeric column resolution pattern missing");
});

test("STATIC: scenario deletion filters by stable id", function () {
  assert(SCENARIO_SRC.indexOf("item.id !== entry.id") !== -1, "id-based delete pattern missing");
});
