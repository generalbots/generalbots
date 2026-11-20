/**
 * Real defects found while characterizing the confidence-review path (#2509).
 * Every case EXECUTES shipped code and asserts the current faulty behaviour so
 * it is evidence, not speculation. [BUG] never means "acceptable": these are
 * report-only findings for the driver; production code is not patched here.
 */
"use strict";

var h = require("./00_harness");
var g = h.loadModules();
var C = g.core;

h.bug("isLow fails open on non-numeric confidence (59b_ingest_cells.js:32)", function () {
  h.equal(C.isLow({ value: "120.50", confidence: 0.62 }), true);
  h.equal(C.isLow({ value: "120.50", confidence: "unknown" }), false);
  h.equal(C.isLow({ value: "120.50", confidence: undefined }), false);
});

h.bug("effectiveValue throws for a missing cell (59b_ingest_cells.js:63)", function () {
  g.reset();
  g.seed();
  var outcome = h.capture(function () {
    return C.effectiveValue(C.state(), "stmt", 99, 0);
  });
  h.equal(outcome.threw, true);
  h.assert(/value/.test(outcome.message));
});

h.bug("a blocked balance cell still carries a formula (62_ingest_review.js:151-157)", function () {
  g.reset();
  g.seed();
  var blocked = g.review.buildPreview(C.state(), "stmt").rows[5].cells[4];
  h.equal(blocked.label, "E9");
  h.equal(blocked.status, "blocked");
  h.equal(blocked.value, "");
  h.equal(blocked.formula, "=E8+D9-C9");
});

h.bug("a correction for an unreadable cell is blanked in preview (62_ingest_review.js:161)", function () {
  g.reset();
  g.seed();
  C.mutate(function (target) {
    target.overrides.push({
      key: "stmt:4:2", fileId: "stmt", row: 4, col: 2, cell: "C5",
      original: "", corrected: "77.70", author: "Local reviewer", timestamp: g.fixedIso,
    });
  });
  var state = C.state();
  var cell = g.review.buildPreview(state, "stmt").rows[3].cells[2];
  h.equal(cell.status, "ok");
  h.equal(cell.value, "");
  h.equal(C.effectiveValue(state, "stmt", 4, 2), "77.70");
});
