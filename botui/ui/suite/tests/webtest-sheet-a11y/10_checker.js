/**
 * Sheet accessibility checker — pure-helper invariant tests (#2493).
 * Exercises the DOM-free collectFindings() helper loaded from the shipped
 * production module; no production logic is modified or duplicated here.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var checker = h.loadChecker();

function collect(workbook, index) {
  return checker.collectFindings(workbook, index || 0);
}
function ofType(findings, type) {
  return findings.filter(function (f) { return f.type === type; });
}
function rangesOf(findings) {
  return findings.map(function (f) { return f.params.range; });
}
function sheet(name, extra) {
  var result = {
    name: name,
    data: {},
    merged_cells: [],
    conditional_formats: [],
    images: [],
    drawings: [],
  };
  Object.keys(extra || {}).forEach(function (key) { result[key] = extra[key]; });
  return result;
}
function book(sheets, images) {
  return { worksheets: sheets, images: images || [] };
}
function merge(r, c, r2, c2) {
  return { start_row: r, start_col: c, end_row: r2, end_col: c2 };
}
function colourRule(r, c, r2, c2, style) {
  var rule = merge(r, c, r2, c2);
  rule.style = style;
  return rule;
}

var MERGED_KEY = "sheet-a11y-finding-merged";
var COLOUR_KEY = "sheet-a11y-finding-color-only";
var NOALT_KEY = "sheet-a11y-finding-no-alt";
var HEADER_KEY = "sheet-a11y-finding-no-header";

// ── API surface ──────────────────────────────────────────────────────
test("checker exposes collectFindings as a function", function () {
  assert(typeof checker.collectFindings === "function", "collectFindings missing");
});

test("checker exposes the panel/format entry points", function () {
  ["run", "runAndRender", "togglePanel", "formatFinding"].forEach(function (name) {
    assert(typeof checker[name] === "function", name + " missing");
  });
});

// ── Merged cells ─────────────────────────────────────────────────────
test("[UAT:SHEET-09] merged range is listed with sheet name and A1 address", function () {
  var results = collect(book([sheet("Merged", {
    merged_cells: [merge(0, 0, 2, 3)],
  })]));
  var found = ofType(results, "merged");
  assert(found.length === 1, "expected one merged finding");
  assert(found[0].sheetName === "Merged", "sheet name not carried on finding");
  assert(found[0].params.range === "A1:D3", "expected A1:D3, got " + found[0].params.range);
  assert(found[0].key === MERGED_KEY, "wrong finding key");
});

test("a single-cell merge is reported as one A1 address", function () {
  var results = collect(book([sheet("Merged", {
    merged_cells: [merge(5, 1, 5, 1)],
  })]));
  assert(rangesOf(ofType(results, "merged"))[0] === "B6", "expected B6");
});

test("each merged range is listed separately", function () {
  var results = collect(book([sheet("Merged", {
    merged_cells: [merge(0, 0, 2, 3), merge(5, 1, 5, 1), merge(9, 2, 10, 4)],
  })]));
  var ranges = rangesOf(ofType(results, "merged"));
  assert(ranges.length === 3, "expected three merged findings");
  assert(ranges.indexOf("A1:D3") !== -1, "A1:D3 missing");
  assert(ranges.indexOf("B6") !== -1, "B6 missing");
  assert(ranges.indexOf("C10:E11") !== -1, "C10:E11 missing");
});

test("merged range beyond column Z uses multi-letter A1 columns", function () {
  var results = collect(book([sheet("Merged", {
    merged_cells: [merge(0, 26, 0, 27)],
  })]));
  assert(rangesOf(ofType(results, "merged"))[0] === "AA1:AB1", "expected AA1:AB1");
});

// ── Colour-only conditional formats ──────────────────────────────────
test("colour-only conditional format is flagged with its range", function () {
  var results = collect(book([sheet("Formats", {
    conditional_formats: [colourRule(1, 0, 4, 0, { background: "#f00" })],
  })]));
  var found = ofType(results, "color-only");
  assert(found.length === 1, "expected one colour-only finding");
  assert(found[0].params.range === "A2:A5", "expected A2:A5, got " + found[0].params.range);
  assert(found[0].key === COLOUR_KEY, "wrong finding key");
});

test("colour rule with an icon alternative is not flagged", function () {
  var results = collect(book([sheet("Formats", {
    conditional_formats: [colourRule(0, 0, 0, 0, { background: "#f00", icon: "!" })],
  })]));
  assert(ofType(results, "color-only").length === 0, "icon alternative ignored");
});

test("colour rule with a pattern alternative is not flagged", function () {
  var results = collect(book([sheet("Formats", {
    conditional_formats: [colourRule(0, 0, 0, 0, { fill: "#00f", pattern: "diagonal" })],
  })]));
  assert(ofType(results, "color-only").length === 0, "pattern alternative ignored");
});

test("non-colour style change is not flagged as colour-only", function () {
  var results = collect(book([sheet("Formats", {
    conditional_formats: [colourRule(0, 0, 0, 0, { font_weight: "bold" })],
  })]));
  assert(ofType(results, "color-only").length === 0, "non-colour style flagged");
});

// ── Images without alternative text ──────────────────────────────────
test("image without alt text is flagged with its name", function () {
  var results = collect(book([sheet("Pictures", {
    images: [{ name: "chart1", alt: "" }],
  })]));
  var found = ofType(results, "no-alt");
  assert(found.length === 1, "expected one image finding");
  assert(found[0].params.name === "chart1", "image name not carried");
  assert(found[0].key === NOALT_KEY, "wrong finding key");
});

test("image with alt text is not flagged", function () {
  var results = collect(book([sheet("Pictures", {
    images: [{ name: "chart1", alt: "Quarterly revenue" }],
  })]));
  assert(ofType(results, "no-alt").length === 0, "alt text ignored");
});

test("drawing without alt text is flagged", function () {
  var results = collect(book([sheet("Pictures", {
    drawings: [{ name: "logo" }],
  })]));
  assert(ofType(results, "no-alt").length === 1, "drawing not scanned");
});

test("workbook-level image without alt text is flagged", function () {
  var results = collect(book([sheet("Pictures")], [{ id: "img9" }]));
  var found = ofType(results, "no-alt");
  assert(found.length === 1, "workbook image not scanned");
  assert(found[0].params.name === "img9", "workbook image name not carried");
});

test("unnamed image still produces a non-empty name parameter", function () {
  var results = collect(book([sheet("Pictures", { images: [{}] })]));
  var found = ofType(results, "no-alt");
  assert(found.length === 1, "unnamed image not flagged");
  assert(typeof found[0].params.name === "string" && found[0].params.name.length > 0,
    "name parameter must not be empty");
});

// ── Missing header rows ──────────────────────────────────────────────
test("data below row zero without a header is flagged", function () {
  var results = collect(book([sheet("Data", {
    data: { "1,0": { value: "x" }, "2,0": { formula: "=1+1" } },
  })]));
  var found = ofType(results, "no-header");
  assert(found.length === 1, "expected one header finding");
  assert(found[0].key === HEADER_KEY, "wrong finding key");
  assert(found[0].sheetName === "Data", "sheet name not carried on header finding");
});

test("worksheet with a header row is not flagged", function () {
  var results = collect(book([sheet("Data", {
    data: { "0,0": { value: "Name" }, "1,0": { value: "Ada" } },
  })]));
  assert(ofType(results, "no-header").length === 0, "header row ignored");
});

test("empty worksheet is not flagged for a missing header", function () {
  var results = collect(book([sheet("Data")]));
  assert(ofType(results, "no-header").length === 0, "empty data flagged");
});

// ── Clean workbook ───────────────────────────────────────────────────
test("clean workbook reports zero issues", function () {
  var results = collect(book([sheet("Clean", {
    data: { "0,0": { value: "Name" }, "1,0": { value: "Ada" } },
    conditional_formats: [colourRule(1, 0, 1, 0, { background: "#0f0", icon: "ok" })],
    images: [{ name: "logo", alt: "Company logo" }],
  })]));
  assert(results.length === 0, "expected zero findings, got " + results.length);
});

test("empty workbook reports zero issues", function () {
  assert(collect({}).length === 0, "empty workbook produced findings");
});

// ── Determinism and robustness ───────────────────────────────────────
var mixed = book([sheet("Mixed", {
  merged_cells: [merge(0, 0, 0, 1)],
  conditional_formats: [colourRule(1, 0, 1, 0, { background: "#f00" })],
  images: [{ name: "chart" }],
  data: { "1,0": { value: "x" } },
})]);

test("report is a deterministic array across repeated calls", function () {
  var first = JSON.stringify(collect(mixed));
  var second = JSON.stringify(collect(mixed));
  assert(first === second, "repeated calls differed");
});

test("report uses a stable category order for reproducibility", function () {
  var types = collect(mixed).map(function (f) { return f.type; });
  assert(types.join(",") === "merged,color-only,no-alt,no-header",
    "unexpected category order: " + types.join(","));
});

test("malformed entries are skipped without throwing", function () {
  var results = null;
  try {
    results = collect(book([sheet("Bad", {
      merged_cells: [null, 7],
      conditional_formats: [null],
      images: [null],
    })]));
  } catch (e) {
    throw new Error("collectFindings threw on malformed input: " + e.message);
  }
  assert(Array.isArray(results), "expected an array result");
});

test("out-of-range sheet index falls back to the first sheet", function () {
  var results = collect(book([sheet("First", {
    merged_cells: [merge(0, 0, 0, 0)],
  })]), 99);
  assert(results.length === 1 && results[0].sheetName === "First",
    "invalid index did not fall back to sheet zero");
});

test("every finding carries an identifier, params object and severity", function () {
  collect(mixed).forEach(function (f) {
    assert(typeof f.key === "string" && f.key.length > 0, "finding key missing");
    assert(f.params && typeof f.params === "object", "finding params missing");
    assert(typeof f.severity === "string" && f.severity.length > 0, "severity missing");
  });
});
