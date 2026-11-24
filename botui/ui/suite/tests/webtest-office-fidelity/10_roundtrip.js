/**
 * Round-trip corpus tests: structural extraction, feature checks, text sanity
 * and measured degradation detection. Every fixture is built once from the
 * real helper host and scored with the same schema as the Python CI tool.
 */
"use strict";

var h = require("./00_harness");
var host = require("./01_host");
var zip = require("./02_zip");
var features = require("./03_features");
var corpus = require("./04_corpus");

var ctx = host.create();
var fixtures = corpus.buildAll(ctx);
var vectors = {};
fixtures.forEach(function (fixture) {
  vectors[fixture.id] = features.featureVector(fixture.kind, fixture.archive);
});

function token(fixture, tag) {
  return " [" + fixture.kind + ":" + tag + "]";
}

fixtures.forEach(function (fixture) {
  h.test("corpus " + fixture.id + " package integrity", function () {
    h.assertGreater(fixture.bytes.length, 100, "package must be non-empty");
    h.assertEqual(zip.u32(fixture.bytes, 0), zip.LOCAL_SIG, "local header signature");
    h.assertGreater(fixture.archive.entries, 0, "at least one OOXML part");
    (fixture.expect.parts || []).forEach(function (name) {
      h.assert(fixture.archive.files[name] && fixture.archive.files[name].length > 0,
        "missing or empty part " + name);
    });
  });

  fixture.tags.forEach(function (tag) {
    h.test("corpus " + fixture.id + " feature " + tag + token(fixture, tag), function () {
      var vector = vectors[fixture.id];
      var expected = fixture.expect.counts && fixture.expect.counts[tag];
      if (tag === "text") {
        h.assert(typeof vector.text === "string" && vector.text.length > 3,
          "text signature must be a non-empty string");
      } else {
        h.assert(expected === undefined ? vector[tag] > 0 : vector[tag] === expected,
          tag + " expected " + expected + " but saw " + vector[tag]);
      }
    });
  });

  h.test("corpus " + fixture.id + " text signature sane", function () {
    var text = vectors[fixture.id].text;
    h.assertGreater(text.length, 3, "text signature must be non-empty");
    h.assert(text.indexOf("undefined") < 0, "text must not contain 'undefined'");
    h.assert(text.indexOf("\u0000") < 0, "text must not contain NUL bytes");
    (fixture.expect.textHas || []).forEach(function (needle) {
      h.assert(text.indexOf(needle) >= 0, "text missing '" + needle + "'");
    });
  });

  h.test("self round-trip determinism " + fixture.id + token(fixture, "text"), function () {
    var score = features.scoreVectors(vectors[fixture.id], vectors[fixture.id], fixture.kind);
    h.assertEqual(score.overall, 1, "identical vectors must score 1.0");
  });
});

h.test("structural score detects a dropped feature [docx:tables]", function () {
  var before = vectors["hb-docx-tables"];
  var after = Object.assign({}, before);
  after.tables = 0;
  var score = features.scoreVectors(before, after, "docx");
  var row = score.rows.filter(function (r) { return r.feature === "tables"; })[0];
  h.assertEqual(row.status, "dropped", "removed table must be reported as dropped");
  h.assertEqual(row.score, 0, "removed table must score zero");
  h.assert(score.overall < 1, "overall score must fall below 1.0");
});

h.knownBug("shipped PPTX export flattens tables/charts to text [pptx:tables] [pptx:charts]", function () {
  var source = fixtures.filter(function (f) { return f.id === "hb-pptx-table-chart"; })[0];
  var before = features.featureVector("pptx", source.archive);
  // Mirrors botslides/src/pptx/export.rs: table/chart elements are rewritten
  // as text shapes. Feed the preserved text back through the REAL exporter.
  var flattenedBlob = ctx.Docs.exportPPTX([
    { title: "Quarterly results", body: before.text },
    { title: "Margin trend", body: before.text },
  ], { filename: "flattened.pptx" });
  var after = features.featureVector("pptx", zip.readStoredZipLocal(host.blobBytes(flattenedBlob)));
  var score = features.scoreVectors(before, after, "pptx");
  var tables = score.rows.filter(function (r) { return r.feature === "tables"; })[0];
  var charts = score.rows.filter(function (r) { return r.feature === "charts"; })[0];
  h.assertEqual(after.tables, 0, "flattened export has no native table");
  h.assertEqual(after.charts, 0, "flattened export has no native chart");
  h.assertEqual(tables.status, "dropped", "table loss must be scored, not hidden");
  h.assertEqual(charts.status, "dropped", "chart loss must be scored, not hidden");
  h.assert(after.text.indexOf("Quarterly results") >= 0, "data survives as text");
  h.assert(score.overall < 1, "degradation lowers the overall score");
});

h.test("shipped makeZipStored writes a readable multi-part XLSX [xlsx:container]", function () {
  var grid = host.makeGrid([["A", "B"], ["1", "2"]]);
  var bytes = host.blobBytes(ctx.XLSX.buildXLSX(grid));
  var archive = zip.readStoredZip(bytes);
  h.assertGreater(archive.entries, 1, "a workbook has several OOXML parts");
  h.assertEqual(archive.names.length, archive.entries, "every central entry is named");
  h.assert(archive.files["xl/workbook.xml"] && archive.files["xl/workbook.xml"].length > 0,
    "workbook part must be readable at its recorded local offset");
  h.note("Regression: makeZipStored must advance the central-directory offset by " +
    "local header length + data length; counting only the header made every " +
    "generated XLSX/DOCX/PPTX unreadable by standard ZIP readers.");
});

module.exports = { ctx: ctx, fixtures: fixtures, vectors: vectors };
