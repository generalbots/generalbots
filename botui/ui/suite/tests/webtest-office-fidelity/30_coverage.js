/**
 * Generate the feature-coverage matrix from recorded harness results.
 * Status is derived from evidence (PASS / DEGRADED / FAIL), never hand-written
 * as a product claim. Untested features are reported as such, not as supported.
 */
"use strict";

var h = require("./00_harness");
var features = require("./03_features");

var TOKEN = /\[(docx|pptx|xlsx):([a-z_]+)\]/g;

function buildMatrix(results) {
  var buckets = Object.create(null);
  results.forEach(function (result) {
    var match;
    TOKEN.lastIndex = 0;
    while ((match = TOKEN.exec(result.name)) !== null) {
      var key = match[1] + ":" + match[2];
      var bucket = buckets[key] || (buckets[key] = { pass: 0, degraded: 0, fail: 0, evidence: [] });
      if (result.status === "PASS") bucket.pass++;
      else if (result.status === "DEGRADED") bucket.degraded++;
      else if (result.status === "FAIL") bucket.fail++;
      bucket.evidence.push(result.status);
    }
  });

  var rows = [];
  Object.keys(features.FEATURES).forEach(function (kind) {
    features.FEATURES[kind].forEach(function (feature) {
      var key = kind + ":" + feature;
      var bucket = buckets[key];
      var status = "untested";
      var evidence = "no offline check";
      if (bucket) {
        status = bucket.fail ? "failing" : bucket.degraded ? "degraded" : bucket.pass ? "supported" : "untested";
        evidence = bucket.pass + " PASS / " + bucket.degraded + " DEGRADED / " + bucket.fail + " FAIL";
      }
      rows.push({ app: kind, feature: feature, status: status, evidence: evidence });
    });
  });
  return rows;
}

h.note("Gap: licensed de-identified corpus (100+ files/format) is out of scope offline.");
h.note("Gap: visual raster diff, CI fidelity dashboard, 24h importer fuzz and four-browser smoke matrix.");
h.setCoverage(buildMatrix(h.results()));

module.exports = { buildMatrix: buildMatrix };
