/**
 * Cycle detection, error classification, formula text and dependency maps
 * (#2491). Every assertion executes the shipped 25a_audit_reports.js code.
 */
"use strict";

var h = require("./00_harness");
var e = h.createEngine();
var G = e.graph;
var assert = h.assert;
var equal = h.equal;
var deep = h.deep;

function graphOf(book) {
  return G.buildGraph(G.normalizeSnapshot(book, 0));
}

h.test("findCycles returns an empty report for an acyclic graph", function () {
  var book = h.makeBook();
  book.worksheets[0].data = {
    "0,0": { value: 1, formula: "" },
    "0,1": { value: 2, formula: "=A1+1" },
  };
  var found = G.findCycles(graphOf(book));
  deep(found.cycles, [], "no cycles");
  equal(found.iterative, false, "non-iterative");
  equal(found.truncated, false, "not truncated");
});

h.test("findCycles reports exact membership for a three-cell cycle", function () {
  var book = h.makeBook();
  book.worksheets[0].data = {
    "0,0": { value: "", formula: "=B1+1" },
    "0,1": { value: "", formula: "=C1+1" },
    "0,2": { value: "", formula: "=A1+1" },
    "0,3": { value: 9, formula: "=A1+2" },
  };
  var found = G.findCycles(graphOf(book));
  equal(found.cycles.length, 1, "one cycle");
  var ids = found.cycles[0].ids;
  equal(ids.length, 3, "three members");
  [0, 1, 2].forEach(function (col) {
    assert(ids.indexOf(G.cellId(0, 0, col)) >= 0, "A1/B1/C1 member");
  });
  assert(ids.indexOf(G.cellId(0, 0, 3)) < 0,
    "D1 depends on the cycle but is not a member");
  equal(found.cycles[0].refs.length, 3, "human-readable refs are reported");
});

h.test("findCycles detects a self-reference", function () {
  var book = h.makeBook();
  book.worksheets[0].data["0,0"] = { value: "", formula: "=A1+1" };
  var found = G.findCycles(graphOf(book));
  equal(found.cycles.length, 1, "self-cycle found");
  deep(found.cycles[0].ids, [G.cellId(0, 0, 0)], "only A1 is in the cycle");
});

h.test("findCycles propagates the workbook iterative flag", function () {
  var book = h.makeBook();
  book.calculation.iterate = true;
  equal(G.findCycles(graphOf(book)).iterative, true, "iterative setting is reported");
});

h.test("inspectErrors classifies every documented error type", function () {
  var codes = ["#DIV/0!", "#REF!", "#VALUE!", "#N/A", "#NUM!", "#NULL!", "#CYCLE!"];
  var book = h.makeBook();
  codes.forEach(function (code, row) {
    book.worksheets[0].data[row + ",0"] = { value: code, formula: "=1" };
  });
  var errors = G.inspectErrors(graphOf(book), {});
  equal(errors.length, codes.length, "one row per cached error");
  errors.forEach(function (error) {
    assert(error.causeKey.indexOf("sheet-audit-cause-") === 0, error.type + " has a cause key");
    assert(error.fixKey.indexOf("sheet-audit-fix-") === 0, error.type + " has a fix key");
    assert(error.causeFallback && error.fixFallback, error.type + " has English fallbacks");
    assert(error.ref && error.id, error.type + " has a location");
  });
  var byType = {};
  errors.forEach(function (error) { byType[error.type] = error; });
  equal(byType["#DIV/0!"].causeKey, "sheet-audit-cause-div0", "division cause key");
  equal(byType["#CYCLE!"].fixKey, "sheet-audit-fix-cycle", "cycle fix key");
});

h.test("inspectErrors finds #REF! in formula text and deduplicates", function () {
  var book = h.makeBook();
  book.worksheets[0].data["0,0"] = { value: "#REF!", formula: "=#REF!+1" };
  var errors = G.inspectErrors(graphOf(book), {});
  equal(errors.length, 1, "cached value and formula produce one row");
  equal(errors[0].type, "#REF!", "the type is preserved");
});

h.test("inspectErrors consumes evaluator issues and falls back for unknown types", function () {
  var book = h.makeBook();
  book.worksheets[0].data["0,0"] = { value: 5, formula: "=A2/0" };
  book.worksheets[0].data["1,0"] = { value: "#WEIRD!", formula: "=2" };
  var graph = graphOf(book);
  var issues = {};
  issues[G.cellId(0, 0, 0)] = "#DIV/0!";
  var errors = G.inspectErrors(graph, issues);
  equal(errors.length, 2, "one evaluator issue plus one cached error");
  var byType = {};
  errors.forEach(function (error) { byType[error.type] = error; });
  equal(byType["#DIV/0!"].causeKey, "sheet-audit-cause-div0", "evaluator issue classified");
  equal(byType["#WEIRD!"].causeKey, "sheet-audit-cause-other", "unknown type uses the other key");
});

h.knownBug("cached #NAME? values are not classified by inspectErrors", function () {
  var book = h.makeBook();
  book.worksheets[0].data["0,0"] = { value: "#NAME?", formula: "=1" };
  var errors = G.inspectErrors(graphOf(book), {});
  equal(errors.length, 0, "characterize the miss: trailing-! regex excludes #NAME?");
  h.sourceHas("25a_audit_reports.js", "value === \"#N/A\"", "only #N/A is special-cased");
});

h.test("formulaAsText returns only formulas inside the requested range", function () {
  var book = h.makeBook();
  book.worksheets[0].data = {
    "0,0": { value: 2, formula: "=1+1" },
    "0,1": { value: 8, formula: "" },
    "1,0": { value: 3, formula: "=1+2" },
    "9,9": { value: 4, formula: "=2+2" },
  };
  var result = G.formulaAsText(G.normalizeSnapshot(book, 0), 0,
    { startRow: 0, endRow: 1, startCol: 0, endCol: 0 });
  equal(result.rows.length, 2, "two formulas in A1:A2");
  equal(result.truncated, false, "not truncated");
  assert(result.rows.every(function (row) { return !!row.formula; }), "no value-only cells");
  assert(result.rows.some(function (row) { return row.ref.indexOf("A2") >= 0; }), "A2 present");
  assert(!result.rows.some(function (row) { return row.ref.indexOf("J10") >= 0; }), "J10 excluded");
});

h.test("formulaAsText handles an empty range and a missing worksheet", function () {
  var book = h.makeBook();
  var snapshot = G.normalizeSnapshot(book, 0);
  deep(G.formulaAsText(snapshot, 0, { startRow: 5, endRow: 5, startCol: 0, endCol: 0 }).rows,
    [], "empty range");
  deep(G.formulaAsText(snapshot, 99, {}).rows, [], "missing sheet");
});

h.knownBug("formulaAsText sorts cell keys lexicographically, not numerically", function () {
  var book = h.makeBook();
  book.worksheets[0].data = {
    "2,0": { value: 2, formula: "=1+1" },
    "10,0": { value: 4, formula: "=2+2" },
  };
  var rows = G.formulaAsText(G.normalizeSnapshot(book, 0), 0,
    { startRow: 0, endRow: 20, startCol: 0, endCol: 0 }).rows;
  equal(rows.length, 2, "both formulas returned");
  assert(rows[0].ref.indexOf("A11") >= 0 && rows[1].ref.indexOf("A3") >= 0,
    "characterize the defect: string sort places row 10 before row 2");
});

h.test("dependencyMap counts edges and orders by dependents then precedents", function () {
  var book = h.makeBook();
  book.worksheets[0].data = {
    "0,0": { value: 1, formula: "" },
    "0,1": { value: 2, formula: "=A1+1" },
    "0,2": { value: 3, formula: "=A1+A2" },
    "0,3": { value: 4, formula: "=A2" },
  };
  var map = G.dependencyMap(graphOf(book), 0);
  equal(map.formulas, 3, "three formulas on sheet 0");
  equal(map.links, 4, "four precedent edges");
  equal(map.rows.length, 3, "three rows");
  assert(map.rows.every(function (row) { return typeof row.precedents === "number"; }),
    "counts are numeric");
  assert(map.rows[0].dependents >= map.rows[1].dependents, "sorted by dependents descending");
  deep(G.dependencyMap(graphOf(book), 1).rows, [], "second sheet has no formulas");
});

h.test("columnFitHint produces bounded widths and grows for formula view", function () {
  var book = h.makeBook();
  book.worksheets[0].data = {
    "0,0": { value: 12, formula: "=1+1" },
    "1,2": { value: "abc", formula: "=SUM(A1:A9)" },
  };
  var snapshot = G.normalizeSnapshot(book, 0);
  var values = G.columnFitHint(snapshot, 0, false);
  var formulas = G.columnFitHint(snapshot, 0, true);
  equal(values.length, 2, "two occupied columns");
  equal(values[0].col, 0, "sorted by column index");
  assert(values.every(function (hint) { return hint.width >= 64 && hint.width <= 260; }),
    "widths are clamped");
  assert(formulas[1].width >= values[1].width,
    "formula view never yields a narrower hint for a formula cell");
  equal(formulas[1].letter, "C", "column letter");
});
