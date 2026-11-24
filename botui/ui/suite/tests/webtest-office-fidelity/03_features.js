/**
 * Structural feature extraction and scoring for OOXML packages (#2457).
 *
 * Mirrors the fixed feature schema of bottest/tools/fidelity/fx_extract.py so
 * this offline browser-side harness and the Python CI comparator reason about
 * the same features. Elements are matched by local name, never by namespace
 * prefix, exactly as the Python extractor does.
 */
"use strict";

var zip = require("./02_zip");

var KINDS = ["docx", "pptx", "xlsx"];

var FEATURES = {
  docx: ["paragraphs", "tables", "text", "bold", "italic", "underline",
    "hyperlinks", "images", "charts", "footnotes", "tracked_changes",
    "comments", "numbering"],
  pptx: ["slides", "shapes", "tables", "text", "bold", "italic", "underline",
    "images", "charts", "masters"],
  xlsx: ["sheets", "rows", "cells", "formulas", "shared_strings", "tables",
    "text", "bold", "italic", "underline", "comments"],
};

var PART_PATTERNS = {
  docx: ["word/document.xml", "word/footnotes.xml", "word/comments.xml", "word/charts/chart"],
  pptx: ["ppt/slides/slide", "ppt/slideMasters/", "ppt/notesSlides/", "ppt/charts/chart"],
  xlsx: ["xl/workbook.xml", "xl/worksheets/sheet", "xl/sharedStrings.xml",
    "xl/tables/", "xl/comments", "xl/styles.xml"],
};

function matchingParts(names, kind) {
  return names.filter(function (name) {
    return PART_PATTERNS[kind].some(function (pattern) {
      return name.indexOf(pattern) === 0;
    });
  }).sort();
}

function allXml(files, names) {
  return names.map(function (name) { return zip.readXml(files, name); }).filter(Boolean);
}

function countAcross(xmlParts, local) {
  return xmlParts.reduce(function (sum, xml) { return sum + zip.countTag(xml, local); }, 0);
}

function featureVector(kind, archive) {
  if (KINDS.indexOf(kind) < 0) throw new Error("unknown-kind:" + kind);
  var names = matchingParts(archive.names, kind);
  var xmlParts = allXml(archive.files, names);
  var counts = {};
  FEATURES[kind].forEach(function (name) { counts[name] = 0; });

  if (kind === "docx") {
    counts.paragraphs = countAcross(xmlParts, "p");
    counts.tables = countAcross(xmlParts, "tbl");
    counts.bold = countAcross(xmlParts, "b");
    counts.italic = countAcross(xmlParts, "i");
    counts.underline = countAcross(xmlParts, "u");
    counts.hyperlinks = countAcross(xmlParts, "hyperlink");
    counts.images = countAcross(xmlParts, "drawing") + countAcross(xmlParts, "pict");
    counts.charts = xmlParts.reduce(function (n, xml) { return n + zip.countTag(xml, "chart"); }, 0);
    counts.footnotes = countAcross(xmlParts, "footnote");
    counts.tracked_changes = countAcross(xmlParts, "ins") + countAcross(xmlParts, "del");
    counts.comments = countAcross(xmlParts, "comment");
    counts.numbering = countAcross(xmlParts, "numPr");
  } else if (kind === "pptx") {
    counts.slides = names.filter(function (n) { return n.indexOf("ppt/slides/slide") === 0; }).length;
    counts.shapes = countAcross(xmlParts, "sp");
    counts.tables = countAcross(xmlParts, "tbl");
    counts.bold = xmlParts.reduce(function (n, xml) { return n + zip.countAttr(xml, "b", "1"); }, 0);
    counts.italic = xmlParts.reduce(function (n, xml) { return n + zip.countAttr(xml, "i", "1"); }, 0);
    counts.underline = xmlParts.reduce(function (n, xml) { return n + zip.countAttr(xml, "u", "sng"); }, 0);
    counts.images = countAcross(xmlParts, "pic");
    counts.charts = xmlParts.reduce(function (n, xml) { return n + zip.countTag(xml, "chart"); }, 0);
    counts.masters = names.filter(function (n) { return n.indexOf("ppt/slideMasters/") === 0; }).length;
  } else {
    counts.sheets = countAcross(xmlParts, "sheet");
    counts.rows = countAcross(xmlParts, "row");
    counts.cells = countAcross(xmlParts, "c");
    counts.formulas = countAcross(xmlParts, "f");
    counts.shared_strings = countAcross(xmlParts, "si");
    counts.tables = countAcross(xmlParts, "table");
    counts.bold = countAcross(xmlParts, "b");
    counts.italic = countAcross(xmlParts, "i");
    counts.underline = countAcross(xmlParts, "u");
    counts.comments = countAcross(xmlParts, "comment");
  }
  counts.text = zip.textOf(xmlParts);
  counts.parts = names;
  return counts;
}

function scoreNumber(before, after) {
  if (before === after) return 1;
  var ceiling = Math.max(before, after);
  if (ceiling === 0) return 1;
  return 1 - Math.abs(after - before) / ceiling;
}

function scoreText(before, after) {
  if (before === after) return 1;
  var a = before.toLowerCase().split(/\s+/).filter(Boolean);
  var b = after.toLowerCase().split(/\s+/).filter(Boolean);
  if (!a.length && !b.length) return 1;
  var setB = Object.create(null);
  b.forEach(function (word) { setB[word] = true; });
  var hits = 0;
  a.forEach(function (word) { if (setB[word]) hits++; });
  var union = a.length + b.length - hits;
  return union ? (hits / union) : 0;
}

/** Deterministic per-feature score; 'dropped' is never silently ignored. */
function scoreVectors(before, after, kind) {
  var features = FEATURES[kind] || [];
  var rows = [];
  var total = 0;
  features.forEach(function (name) {
    var a = before[name];
    var b = after[name];
    var score = name === "text" ? scoreText(a, b) : scoreNumber(a, b);
    var status = score === 1 ? "kept" : (b < a ? "dropped" : "added");
    if (name === "text" && score < 1) status = "changed";
    rows.push({ feature: name, before: a, after: b, score: score, status: status });
    total += score;
  });
  return { overall: rows.length ? total / rows.length : 0, rows: rows };
}

module.exports = {
  KINDS: KINDS, FEATURES: FEATURES, PART_PATTERNS: PART_PATTERNS,
  matchingParts: matchingParts, featureVector: featureVector,
  scoreNumber: scoreNumber, scoreText: scoreText, scoreVectors: scoreVectors,
};
