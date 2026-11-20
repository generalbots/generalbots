/**
 * Dependency graph, reference parsing and precedent/dependent tracing (#2491).
 * Every assertion below executes the shipped 25_audit_graph.js / 25a IIFEs.
 */
"use strict";

var h = require("./00_harness");
var e = h.createEngine();
var G = e.graph;
var I = e.graphInternals;
var assert = h.assert;
var equal = h.equal;
var deep = h.deep;

h.test("all nine shipped audit IIFEs are present in one shared sandbox", function () {
  equal(h.ORDER.length, 9, "expected nine shipped modules");
  [
    "SheetAuditGraph", "SheetAuditEval", "SheetAuditUI", "SheetAuditSections",
    "SheetAuditOverlay", "SheetAdvancedAudit", "SheetAuditT",
    "__SheetAuditGraphInternals", "__SheetAuditEvalInternals",
  ].forEach(function (name) {
    assert(e.window[name], name + " was not exported by its real module");
  });
  equal(typeof G.trace, "function", "25a did not extend SheetAuditGraph");
  equal(typeof G.findCycles, "function", "25a did not add findCycles");
  equal(typeof e.eval.evaluate, "function", "26a did not add SheetAuditEval.evaluate");
  assert(e.auditButton.dataset.auditWired === "true", "30_audit_boot did not wire the toolbar button");
});

h.test("column index/name helpers round-trip across A..AAA boundaries", function () {
  [[0, "A"], [25, "Z"], [26, "AA"], [51, "AZ"], [52, "BA"], [701, "ZZ"], [702, "AAA"]]
    .forEach(function (pair) {
      equal(G.colName(pair[0]), pair[1], "colName(" + pair[0] + ")");
      equal(G.colIdx(pair[1]), pair[0], "colIdx(" + pair[1] + ")");
    });
  equal(G.colIdx("$C$4"), 2, "absolute markers must be ignored");
});

h.test("cellId / parseId round-trip with sheet, row and column", function () {
  var id = G.cellId(2, 4, 7);
  equal(id, "2:4,7", "cellId format");
  deep(G.parseId(id), { sheetIdx: 2, row: 4, col: 7 }, "parseId inverse");
});

h.test("parseRef resolves local, absolute, quoted and cross-sheet references", function () {
  var sheets = e.book().worksheets;
  deep(G.parseRef("A1", sheets),
    { sheetName: "", sheetIdx: 0, row: 0, col: 0, ref: "A1" }, "plain local ref");
  deep(G.parseRef("$B$3", sheets),
    { sheetName: "", sheetIdx: 0, row: 2, col: 1, ref: "B3" }, "absolute local ref");
  equal(G.parseRef("Sheet2!C4", sheets).sheetIdx, 1, "named sheet index");
  equal(G.parseRef("'Sheet2'!D5", sheets).sheetIdx, 1, "quoted sheet index");
  equal(G.parseRef("Nope!A1", sheets).sheetIdx, -1, "unknown sheet must stay unresolved");
  equal(G.parseRef("not-a-ref", sheets), null, "invalid reference rejected");
});

h.test("extractRefs ignores string literals and function-like tokens", function () {
  deep(G.extractRefs("=A1+B2").map(function (r) { return r.text; }), ["A1", "B2"], "two refs");
  var range = G.extractRefs("=SUM(A1:A3)")[0];
  equal(range.text, "A1:A3", "range text");
  equal(range.endRow, 2, "range end row");
  equal(range.endCol, 0, "range end col");
  deep(G.extractRefs("=\"A1\"+B2").map(function (r) { return r.text; }), ["B2"],
    "references inside string literals are masked");
  deep(G.extractRefs("=LOG10(2)+A1").map(function (r) { return r.text; }), ["A1"],
    "LOG10( is not a cell reference");
  equal(G.extractRefs("=A1+A1").length, 1, "duplicate references are collapsed");
  equal(G.extractRefs("=Sheet2!A1")[0].sheetName, "Sheet2", "cross-sheet prefix preserved");
});

h.test("iterativeFlag recognises the documented calculation keys", function () {
  equal(G.iterativeFlag(make({ calculation: {} })), false, "default is non-iterative");
  ["iterate", "iterative", "iterativeCalculation", "iterative_calculation", "enableIterative"]
    .forEach(function (key) {
      var book = { calculation: {} };
      book.calculation[key] = true;
      equal(G.iterativeFlag(book), true, "flag key " + key);
    });
  var top = { iterative: true };
  equal(G.iterativeFlag(top), true, "top-level flag");
});

function make(book) { return book; }

h.test("normalizeSnapshot copies cell data instead of aliasing the workbook", function () {
  var snapshot = G.normalizeSnapshot(e.book(), 0);
  snapshot.worksheets[0].data["0,0"] = { value: "mutated" };
  assert(!e.book().worksheets[0].data["0,0"], "snapshot write leaked into the workbook");
  equal(snapshot.sheetIndex, 0, "sheet index");
  equal(snapshot.iterative, false, "iterative flag");
});

h.test("normalizeSnapshot folds a read-only copy of freshest grid cells", function () {
  e.setCell(0, 0, 0, { value: 1, formula: "" });
  e.grid.cells.set("0,0", { value: 9, formula: "=9", comment: "live" });
  var snapshot = G.normalizeSnapshot(e.book(), 0);
  equal(snapshot.worksheets[0].data["0,0"].value, 9, "grid value must win");
  equal(e.book().worksheets[0].data["0,0"].value, 1, "workbook value must not change");
  e.grid.cells.clear();
});

h.test("buildGraph links five levels of precedents and dependents", function () {
  e.setCell(0, 0, 0, { value: 1 });
  for (var col = 1; col <= 5; col++) {
    e.setCell(0, 0, col, { formula: G.colName(col - 1) + "1+1", value: col + 1 });
  }
  var graph = G.buildGraph(G.normalizeSnapshot(e.book(), 0));
  equal(Object.keys(graph.cells).length, 6, "six cells in the chain");
  equal(graph.links, 5, "five edges");
  equal(graph.formulaCount, 5, "five formulas");
  var pre = G.trace(graph, G.cellId(0, 0, 5), "precedents", 5);
  equal(pre.nodes.length, 6, "root plus five precedents");
  equal(pre.levels, 5, "five levels");
  deep(pre.nodes.map(function (n) { return n.ref.split("!")[1]; }),
    ["F1", "E1", "D1", "C1", "B1", "A1"], "precedent order");
  var dep = G.trace(graph, G.cellId(0, 0, 0), "dependents", 5);
  deep(dep.nodes.map(function (n) { return n.ref.split("!")[1]; }),
    ["A1", "B1", "C1", "D1", "E1", "F1"], "dependent order");
  equal(dep.levels, 5, "dependent depth");
});

h.test("trace depth is clamped into 1..12", function () {
  var book = h.makeBook();
  book.worksheets[0].name = "S";
  book.worksheets = [book.worksheets[0]];
  for (var row = 0; row < 14; row++) {
    var value = row === 0 ? "1" : "A" + row + "+1";
    book.worksheets[0].data[row + ",0"] = { value: value, formula: row === 0 ? "" : "=" + value };
  }
  var graph = G.buildGraph(G.normalizeSnapshot(book, 0));
  var top = G.cellId(0, 13, 0);
  equal(G.trace(graph, top, "precedents", 1).levels, 1, "minimum depth");
  equal(G.trace(graph, top, "precedents", 99).levels, 12, "maximum depth");
  equal(G.trace(graph, top, "precedents", 0).levels, 5, "zero falls back to the default depth");
});

h.test("diamond graph deduplicates the shared precedent", function () {
  var book = h.makeBook();
  book.worksheets[0].data = {
    "0,0": { value: 1, formula: "" },
    "0,1": { value: 2, formula: "=A1+1" },
    "0,2": { value: 2, formula: "=A1+1" },
    "0,3": { value: 4, formula: "=B1+C1" },
  };
  var graph = G.buildGraph(G.normalizeSnapshot(book, 0));
  var trace = G.trace(graph, G.cellId(0, 0, 3), "precedents", 6);
  equal(trace.nodes.filter(function (n) { return n.ref.indexOf("A1") >= 0; }).length, 1,
    "A1 is visited once");
  equal(trace.levels, 2, "B1/C1 at level 1, A1 at level 2");
});

h.test("range references expand to every covered precedent cell", function () {
  var book = h.makeBook();
  book.worksheets[0].data["1,0"] = { value: 6, formula: "=SUM(A1:A3)" };
  var graph = G.buildGraph(G.normalizeSnapshot(book, 0));
  var pre = graph.precedents[G.cellId(0, 1, 0)];
  equal(pre.length, 3, "A1, A2, A3");
  assert(pre.indexOf(G.cellId(0, 0, 0)) >= 0, "A1 present");
  assert(pre.indexOf(G.cellId(0, 2, 0)) >= 0, "A3 present");
});

h.test("cross-sheet precedents resolve to the named worksheet", function () {
  var book = h.makeBook();
  book.worksheets[0].data["0,0"] = { value: 7, formula: "=Sheet2!B2*2" };
  var graph = G.buildGraph(G.normalizeSnapshot(book, 0));
  var pre = graph.precedents[G.cellId(0, 0, 0)];
  equal(pre.length, 1, "one cross-sheet precedent");
  equal(pre[0], G.cellId(1, 1, 1), "Sheet2!B2 id");
  equal(graph.cells[pre[0]].missing, true, "absent target is marked missing, not fabricated as data");
});

h.test("buildGraph reports truncation for a range wider than the cell budget", function () {
  var book = h.makeBook();
  book.worksheets[0].data["0,0"] = { value: 0, formula: "=SUM(A1:HA100)" };
  var graph = G.buildGraph(G.normalizeSnapshot(book, 0));
  equal(graph.truncated, true, "wide range must set the truncated flag");
  assert(graph.formulaCount === 1, "only the formula cell is counted as a formula");
});

h.test("buildGraph does not mutate the normalized snapshot", function () {
  var snapshot = G.normalizeSnapshot(e.book(), 0);
  var before = JSON.stringify(snapshot.worksheets[0].data);
  G.buildGraph(snapshot);
  equal(JSON.stringify(snapshot.worksheets[0].data), before, "graph build is read-only");
});

h.test("constants expose a bounded graph/cycle budget", function () {
  equal(I.MAX_GRAPH_CELLS, 250000, "graph cell budget");
  equal(I.MAX_CYCLES, 40, "cycle report budget");
  equal(I.CYCLE_BUDGET, 200000, "cycle step budget");
});
