/**
 * Confidence gate (#2509), proved by EXECUTING the shipped 59b ingest-cells
 * IIFE. Nothing here claims that an OCR engine produced the confidence values:
 * they come from the shipped deterministic fixture (59a_ingest_sim.js).
 */
"use strict";

var h = require("./00_harness");
var g = h.loadModules();
var C = g.core;

h.runtime("confidence threshold is 0.85 and the gate is strictly less-than", function () {
  h.equal(C.THRESHOLD, 0.85);
  h.equal(C.isLow({ value: "120.50", confidence: 0.85 }), false);
  h.equal(C.isLow({ value: "120.50", confidence: 0.849 }), true);
  h.equal(C.isLow({ value: "120.50", confidence: 0.851 }), false);
});

h.runtime("unreadable is the source flag, independent of the numeric gate", function () {
  h.equal(C.isUnreadable({ value: "x", confidence: 0.96, unreadable: true }), true);
  h.equal(C.isUnreadable({ value: "x", confidence: 0.30, unreadable: true }), true);
  h.equal(C.isUnreadable({ value: "x", confidence: 0.96 }), false);
  h.equal(C.isUnreadable({ value: "", confidence: 0.96 }), false);
  h.equal(C.isUnreadable(null), false);
});

h.runtime("the review queue selects low OR unreadable cells and skips headers", function () {
  g.reset();
  g.seed();
  var parse = C.state().parses.stmt;
  var cells = g.review.reviewableCells(parse, "stmt");
  h.equal(cells.length, 3);
  h.assert(cells.every(function (item) { return C.isLow(item.cell); }));
  h.deep(cells.map(function (item) {
    return C.cellLabel(parse, item.row, item.col);
  }), ["C3", "C5", "E9"]);
  h.assert(!cells.some(function (item) { return item.row === 0; }));
  h.deep(g.review.reviewableCells(null, "stmt"), []);
});

h.runtime("cell keys and labels are deterministic A1-style provenance", function () {
  g.reset();
  g.seed();
  h.equal(C.cellKey("stmt", 2, 2), "stmt:2:2");
  h.equal(C.cellLabel(C.state().parses.stmt, 0, 0), "A1");
  h.equal(C.cellLabel(C.state().parses.stmt, 8, 4), "E9");
  h.equal(C.cellLabel(null, 0, 0), "?");
  h.equal(C.cellAt(C.state().parses.stmt, 99, 0), null);
});

h.runtime("a confirmation resolves a low cell and clears its formula block", function () {
  g.reset();
  g.seed();
  var state = C.state();
  h.equal(C.formulaBlocked(state, "stmt", 2, 2), true);
  C.mutate(function (target) {
    target.confirmations[C.cellKey("stmt", 2, 2)] = true;
  });
  state = C.state();
  h.equal(C.isConfirmed(state, "stmt", 2, 2), true);
  h.equal(C.isResolved(state, "stmt", 2, 2), true);
  h.equal(C.formulaBlocked(state, "stmt", 2, 2), false);
  h.equal(C.effectiveValue(state, "stmt", 2, 2), "120.50");
});

h.runtime("an override resolves the gate without a confirmation and wins on value", function () {
  g.reset();
  g.seed();
  C.mutate(function (target) {
    target.overrides.push({
      key: "stmt:4:2", fileId: "stmt", row: 4, col: 2, cell: "C5",
      original: "", corrected: "77.70", author: "Local reviewer", timestamp: g.fixedIso,
    });
  });
  var state = C.state();
  var found = C.overrideFor(state, "stmt", 4, 2);
  h.equal(found.corrected, "77.70");
  h.equal(C.isConfirmed(state, "stmt", 4, 2), false);
  h.equal(C.isResolved(state, "stmt", 4, 2), true);
  h.equal(C.formulaBlocked(state, "stmt", 4, 2), false);
  h.equal(C.effectiveValue(state, "stmt", 4, 2), "77.70");
  h.equal(C.overrideFor(state, "stmt", 2, 2), null);
});

h.runtime("numeric parsing accepts a grouped decimal value and rejects junk", function () {
  h.equal(C.toNumber("1,234.56"), 1234.56);
  h.equal(C.toNumber("120.50"), 120.5);
  h.equal(C.toNumber(""), null);
  h.equal(C.toNumber("abc"), null);
});

h.runtime("bank profile detection identifies the five statement columns", function () {
  g.reset();
  g.seed();
  h.deep(C.detectProfile(C.state().parses.stmt), {
    kind: "bank",
    columns: { date: 0, description: 1, debit: 2, credit: 3, balance: 4 },
  });
  h.equal(C.detectProfile(null).kind, "generic");
});

h.runtime("structural rows are excluded from the data-row gate", function () {
  g.reset();
  g.seed();
  var parse = C.state().parses.stmt;
  h.equal(C.isDataRow(parse, 0), false);
  h.equal(C.isDataRow(parse, 2), true);
  h.equal(C.isDataRow(parse, 6), false);
  h.equal(C.isDataRow(parse, 7), false);
});

h.runtime("column types use a saved mapping, else the fixture target", function () {
  g.reset();
  g.seed();
  h.deep(C.columnTypes(C.state(), "stmt"), ["date", "text", "currency", "currency", "currency"]);
  C.mutate(function (target) {
    target.mappings.stmt = ["date", "text", "number", "currency", "currency"];
  });
  h.deep(C.columnTypes(C.state(), "stmt"), ["date", "text", "number", "currency", "currency"]);
});

h.runtime("the running-balance gate blocks exactly the unconfirmed low rows", function () {
  g.reset();
  g.seed();
  var result = C.runningBalance(C.state(), "stmt");
  h.equal(result.kind, "bank");
  h.equal(result.blocked, 3);
  h.equal(result.mismatch, 0);
  h.equal(result.available, 6);
  h.deep(result.rows.map(function (item) { return item.row; }), [2, 4, 8]);
});

h.runtime("resolving flagged cells unblocks rows and surfaces arithmetic mismatch", function () {
  g.reset();
  g.seed();
  C.mutate(function (target) {
    target.confirmations[C.cellKey("stmt", 2, 2)] = true;
    target.overrides.push({
      key: "stmt:4:2", fileId: "stmt", row: 4, col: 2, cell: "C5",
      original: "", corrected: "77.70", author: "Local reviewer", timestamp: g.fixedIso,
    });
  });
  var result = C.runningBalance(C.state(), "stmt");
  h.equal(result.blocked, 1);
  h.equal(result.mismatch, 1);
  h.equal(result.available, 7);
  h.equal(result.rows[0].row, 8);
  h.equal(result.rows[1].row, 9);
  h.equal(result.rows[1].expected, 1244.8);
  h.equal(result.rows[1].stated, 1322.5);
});
