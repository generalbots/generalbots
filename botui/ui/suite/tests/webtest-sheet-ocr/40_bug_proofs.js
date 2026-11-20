/**
 * Former defects (#2509), now fixed and kept as RUNTIME regressions.
 * Each case EXECUTES shipped code and asserts the corrected behaviour, so a
 * future regression in the confidence gate, the missing-cell accessor or the
 * correction preview fails this harness instead of silently returning.
 */
"use strict";

var h = require("./00_harness");
var g = h.loadModules();
var C = g.core;

h.runtime("isLow fails closed on non-numeric confidence (regression #2509)", function () {
  h.equal(C.isLow({ value: "120.50", confidence: 0.62 }), true);
  h.equal(C.isLow({ value: "120.50", confidence: "unknown" }), true);
  h.equal(C.isLow({ value: "120.50", confidence: undefined }), true);
  h.equal(C.isLow({ value: "120.50", confidence: 0.86 }), false);
  h.equal(C.isLow(null), false);
});

h.runtime("effectiveValue is null-safe for a missing cell (regression #2509)", function () {
  g.reset();
  g.seed();
  var outcome = h.capture(function () {
    return C.effectiveValue(C.state(), "stmt", 99, 0);
  });
  h.equal(outcome.threw, false);
  h.equal(outcome.value, "");
});

h.runtime("a blocked balance cell carries no derived formula (regression #2494)", function () {
  g.reset();
  g.seed();
  var blocked = g.review.buildPreview(C.state(), "stmt").rows[5].cells[4];
  h.equal(blocked.label, "E9");
  h.equal(blocked.status, "blocked");
  h.equal(blocked.value, "");
  h.equal(blocked.formula, "");
});

h.runtime("a correction for an unreadable cell is shown in preview (regression #2509)", function () {
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
  h.equal(cell.value, "77.70");
  h.equal(C.effectiveValue(state, "stmt", 4, 2), "77.70");
});
