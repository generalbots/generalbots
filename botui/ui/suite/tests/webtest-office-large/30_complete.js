/**
 * Completeness gate: every (app x capability) pair must be registered exactly
 * once, so an unprobed cell can never masquerade as a silent pass.
 */
"use strict";

var H = require("./00_harness");
var M = require("./01_matrix");

var rows = H.matrix();
var seen = Object.create(null);
var problems = [];

M.APPS.forEach(function (app) {
  M.CAPABILITIES.forEach(function (cap) {
    var key = app.id + "|" + cap.id;
    var matches = rows.filter(function (r) {
      return r.app === app.id && r.capability === cap.id;
    });
    if (matches.length !== 1) {
      problems.push(key + " registered " + matches.length + " times (expected 1)");
    }
    if (matches.length === 1 && ["PASS", "GAP", "FAIL"].indexOf(matches[0].status) < 0) {
      problems.push(key + " has invalid status " + matches[0].status);
    }
    seen[key] = true;
  });
});

rows.forEach(function (row) {
  if (!seen[row.app + "|" + row.capability]) {
    problems.push("unexpected matrix row " + row.app + "|" + row.capability);
  }
});

H.runtime("matrix completeness: " + M.APPS.length + " apps x " + M.CAPABILITIES.length +
  " capabilities = " + rows.length + " cells", function () {
  H.assert(problems.length === 0, problems.join("; "));
});
