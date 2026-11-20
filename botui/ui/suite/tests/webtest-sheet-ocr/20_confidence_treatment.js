/**
 * Confidence treatment (#2509), proved by EXECUTING the shipped 62 ingest-review
 * IIFE: what the review UI shows, blanks, blocks and records. Bug proofs live
 * in 40_bug_proofs.js so a production fix can retire them without disturbing
 * the behavioural characterization below.
 */
"use strict";

var h = require("./00_harness");
var g = h.loadModules();
var C = g.core;

function byClass(parent, className) {
  return parent.children.filter(function (node) {
    return node.classList.contains(className);
  });
}

function firstByClass(parent, className) {
  return byClass(parent, className)[0];
}

h.runtime("the local preview is deterministic and workbook-scoped", function () {
  g.reset();
  g.seed();
  var preview = g.review.buildPreview(C.state(), "stmt");
  h.equal(preview.fileId, "stmt");
  h.equal(preview.createdAt, g.fixedIso);
  h.equal(preview.rows.length, 7);
  h.assert(preview.rows.every(function (row) { return row.cells.length === 5; }));
  h.equal(preview.rows[0].cells[0].label, "A2");
  h.equal(preview.rows[0].cells[4].formula, "=E2");
  h.equal(preview.rows[1].cells[4].formula, "=E2+D3-C3");
});

h.runtime("high-confidence values render; low-confidence cells are blanked and blocked", function () {
  g.reset();
  g.seed();
  var preview = g.review.buildPreview(C.state(), "stmt");
  var row2 = preview.rows[1].cells;
  h.equal(row2[0].value, "03/03/2026");
  h.equal(row2[0].status, "ok");
  h.equal(row2[2].label, "C3");
  h.equal(row2[2].status, "blocked");
  h.equal(row2[2].value, "");
  var row8 = preview.rows[5].cells[4];
  h.equal(row8.label, "E9");
  h.equal(row8.status, "blocked");
  h.equal(row8.value, "");
});

h.runtime("confirming a flagged value flips its preview cell to ok", function () {
  g.reset();
  g.seed();
  var before = g.review.buildPreview(C.state(), "stmt");
  h.equal(before.rows[1].cells[2].status, "blocked");
  C.mutate(function (target) {
    target.confirmations[C.cellKey("stmt", 2, 2)] = true;
  });
  var after = g.review.buildPreview(C.state(), "stmt");
  h.equal(after.rows[1].cells[2].status, "ok");
  h.equal(after.rows[1].cells[2].value, "120.50");
});

h.runtime("an override clears the gate for the affected cell only", function () {
  g.reset();
  g.seed();
  C.mutate(function (target) {
    target.overrides.push({
      key: "stmt:4:2", fileId: "stmt", row: 4, col: 2, cell: "C5",
      original: "", corrected: "77.70", author: "Local reviewer", timestamp: g.fixedIso,
    });
  });
  var preview = g.review.buildPreview(C.state(), "stmt");
  h.equal(preview.rows[3].cells[2].status, "ok");
  h.equal(preview.rows[5].cells[4].status, "blocked");
  h.equal(preview.rows[1].cells[2].status, "blocked");
});

h.runtime("the correction form records provenance and never edits the source cell", function () {
  g.reset();
  g.seed();
  var body = g.dom.makeNode("div");
  g.review.render(body, g.ctx);
  var section = body.children[0];
  var form = firstByClass(section, "ss-ingest-correction");
  h.assert(Boolean(form));
  var picker = firstByClass(form, "ss-ingest-select");
  var input = firstByClass(form, "ss-ingest-input");
  var save = firstByClass(form, "ss-ingest-btn");
  h.equal(picker.children.length, g.review.reviewableCells(C.state().parses.stmt, "stmt").length);
  picker.value = "4:2";
  input.value = "  77.70  ";
  save.dispatch("click");
  var overrides = C.state().overrides;
  h.equal(overrides.length, 1);
  h.equal(overrides[0].key, "stmt:4:2");
  h.equal(overrides[0].cell, "C5");
  h.equal(overrides[0].original, "");
  h.equal(overrides[0].corrected, "77.70");
  h.equal(overrides[0].timestamp, g.fixedIso);
  h.equal(C.cellAt(C.state().parses.stmt, 4, 2).value, "");
  h.equal(input.value, "");
  h.assert(g.counters.rerender > 0);
});

h.runtime("a blank correction is rejected without writing an override", function () {
  g.reset();
  g.seed();
  var body = g.dom.makeNode("div");
  g.review.render(body, g.ctx);
  var form = firstByClass(body.children[0], "ss-ingest-correction");
  var picker = firstByClass(form, "ss-ingest-select");
  var input = firstByClass(form, "ss-ingest-input");
  var save = firstByClass(form, "ss-ingest-btn");
  picker.value = "4:2";
  input.value = "   ";
  save.dispatch("click");
  h.deep(C.state().overrides, []);
  h.equal(g.statuses.length, 1);
  h.equal(g.statuses[0].isError, true);
});

h.runtime("the provenance log renders the override beside the untouched original", function () {
  g.reset();
  g.seed();
  C.mutate(function (target) {
    target.overrides.push({
      key: "stmt:2:2", fileId: "stmt", row: 2, col: 2, cell: "C3",
      original: "120.50", corrected: "125.00", author: "Local reviewer", timestamp: g.fixedIso,
    });
  });
  var body = g.dom.makeNode("div");
  g.review.render(body, g.ctx);
  var log = firstByClass(body.children[0], "ss-ingest-log");
  h.assert(Boolean(log));
  var lines = byClass(log, "ss-ingest-detail");
  h.equal(lines.length, 1);
  h.assert(lines[0]._text.indexOf("C3") === 0);
  h.assert(lines[0]._text.indexOf("125.00") > 0);
  h.equal(C.cellAt(C.state().parses.stmt, 2, 2).value, "120.50");
});

h.runtime("rendered preview markup exposes blocked status to assistive tech and tests", function () {
  g.reset();
  g.seed();
  var built = g.review.buildPreview(C.state(), "stmt");
  C.mutate(function (target) { target.preview = built; });
  var body = g.dom.makeNode("div");
  g.review.render(body, g.ctx);
  h.equal(body.querySelectorAll(".ss-ingest-status-blocked").length, 3);
  h.equal(body.querySelectorAll(".ss-ingest-status-ok").length, 32);
  h.equal(body.querySelectorAll(".ss-ingest-formula-cell").length, 35);
});
