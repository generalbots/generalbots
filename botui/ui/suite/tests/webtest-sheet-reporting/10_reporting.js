/**
 * Sheet reporting — core and print/page-setup invariants (#2438).
 * Production modules are loaded unchanged (see 00_harness.js). Every test in
 * this file is a RUNTIME test against the real shipped namespaces.
 */
"use strict";

var h = require("./00_harness");
var g = h.loadModules();
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var deep = h.deep;

// ── exported API surface ─────────────────────────────────────────────────
test("core module exports the shared helpers", function () {
  ["t", "sheetId", "storageKey", "defaultSetup", "normalizeSetup", "emptyState",
    "state", "save", "mutate", "element", "escapeHtml", "colToNumber",
    "numberToCol", "parseRange", "focusables", "setStatus", "makeDialog", "flash",
  ].forEach(function (name) {
    assert(typeof g.core[name] === "function", "missing core helper " + name);
  });
  equal(g.core.MAX_ITEMS, 200);
});

test("print/preview/dashboard/schedule/boot namespaces are all installed", function () {
  ["addArea", "clearArea", "clearAreas", "summary", "updateSetup", "paginate",
    "addBreak", "clearBreaks", "validateArea",
  ].forEach(function (name) { assert(typeof g.print[name] === "function", "missing print." + name); });
  ["variantFor", "fieldText", "lineFor", "pageCells", "repeatRows"].forEach(function (name) {
    assert(typeof g.preview[name] === "function", "missing preview." + name);
  });
  ["validateWidget", "createDashboard", "addWidget", "removeWidget", "sparklinePoints",
    "widgetNode",
  ].forEach(function (name) { assert(typeof g.dashboard[name] === "function", "missing dashboard." + name); });
  ["validatePolicy", "nextRun", "addPolicy", "removePolicy"].forEach(function (name) {
    assert(typeof g.schedule[name] === "function", "missing schedule." + name);
  });
  ["open", "close", "toggle", "switchTab", "relocalize", "rerender", "refreshPreview"].forEach(function (name) {
    assert(typeof g.boot[name] === "function", "missing boot." + name);
  });
});

test("shipped paper table keeps its declared page capacities", function () {
  deep(g.print.PAPER, {
    A4: { cols: 8, rows: 45 }, A3: { cols: 12, rows: 60 },
    Letter: { cols: 9, rows: 42 }, Legal: { cols: 9, rows: 52 },
  });
});

// ── column conversions ───────────────────────────────────────────────────
test("colToNumber is zero-based and case-insensitive", function () {
  equal(g.core.colToNumber("A"), 0);
  equal(g.core.colToNumber("z"), 25);
  equal(g.core.colToNumber("AA"), 26);
  equal(g.core.colToNumber("AB"), 27);
  equal(g.core.colToNumber("ZZ"), 701);
});

test("colToNumber rejects empty, numeric and non-letter input", function () {
  ["", "A1", "4", "A B", "AA!"].forEach(function (value) {
    equal(g.core.colToNumber(value), -1, "accepted " + value);
  });
  equal(g.core.colToNumber(null), -1);
});

test("numberToCol is the inverse of colToNumber", function () {
  equal(g.core.numberToCol(0), "A");
  equal(g.core.numberToCol(25), "Z");
  equal(g.core.numberToCol(26), "AA");
  equal(g.core.numberToCol(-5), "A");
  [0, 7, 25, 26, 51, 701].forEach(function (index) {
    equal(g.core.colToNumber(g.core.numberToCol(index)), index, "round-trip " + index);
  });
});

// ── parseRange ───────────────────────────────────────────────────────────
test("parseRange normalises A1:C2 into zero-based bounds", function () {
  deep(g.core.parseRange("A1:C2"), {
    sheet: "", startCol: 0, endCol: 2, startRow: 0, endRow: 1,
    cols: 3, rows: 2,
  });
});

test("parseRange sorts reversed corners without losing extent", function () {
  deep(g.core.parseRange("D10:B2"), {
    sheet: "", startCol: 1, endCol: 3, startRow: 1, endRow: 9,
    cols: 3, rows: 9,
  });
});

test("parseRange keeps a sheet qualifier and uppercases input", function () {
  var parsed = g.core.parseRange("  data!b2:c3 ");
  equal(parsed.sheet, "DATA");
  equal(parsed.startCol, 1);
  equal(parsed.endCol, 2);
  equal(parsed.startRow, 1);
  equal(parsed.endRow, 2);
});

test("parseRange decodes multi-letter columns", function () {
  var parsed = g.core.parseRange("AA1:AB3");
  equal(parsed.startCol, 26);
  equal(parsed.endCol, 27);
  equal(parsed.cols, 2);
  equal(parsed.rows, 3);
});

test("parseRange rejects single cells, zero rows and malformed refs", function () {
  ["", "A1", "A0:B2", "1A:B2", "A1:", "AA:B2", "$A$1:$B$2", "A1:B2:C3"].forEach(function (value) {
    assert(g.core.parseRange(value) === null, "accepted " + value);
  });
  assert(g.core.parseRange(null) === null);
  assert(g.core.parseRange(42) === null);
});

// ── page setup normalisation ─────────────────────────────────────────────
test("defaultSetup is a stable workbook-scoped shape", function () {
  var setup = g.core.defaultSetup();
  equal(setup.orientation, "portrait");
  equal(setup.paper, "A4");
  deep(setup.margins, { top: 0.5, bottom: 0.5, left: 0.5, right: 0.5 });
  equal(setup.scaleMode, "fit");
  equal(setup.fitToWidth, 1);
  equal(setup.fitToHeight, 0);
  equal(setup.percent, 100);
  equal(setup.footer.center, "page");
  equal(setup.footer.right, "sheet");
  equal(setup.pageOrder, "down");
  equal(setup.gridlines, false);
});

test("normalizeSetup returns fresh defaults for absent or non-object input", function () {
  deep(g.core.normalizeSetup(undefined), g.core.defaultSetup());
  deep(g.core.normalizeSetup(null), g.core.defaultSetup());
  deep(g.core.normalizeSetup("x"), g.core.defaultSetup());
});

test("normalizeSetup clamps enums, numbers and margins to declared ranges", function () {
  var setup = g.core.normalizeSetup({
    orientation: "diagonal", paper: "A2", scaleMode: "zoom",
    fitToWidth: 99, fitToHeight: -4, percent: 900,
    margins: { top: -2, left: 12 },
    gridlines: "yes", headings: 1, blackWhite: 0,
  });
  equal(setup.orientation, "portrait");
  equal(setup.paper, "A4");
  equal(setup.scaleMode, "fit");
  equal(setup.fitToWidth, 20);
  equal(setup.fitToHeight, 0);
  equal(setup.percent, 400);
  equal(setup.margins.top, 0);
  equal(setup.margins.left, 3);
  equal(setup.margins.right, 0.5);
  equal(setup.gridlines, true);
  equal(setup.headings, true);
  equal(setup.blackWhite, false);
});

test("normalizeSetup truncates header/footer/repeat fields and coerces nulls", function () {
  var setup = g.core.normalizeSetup({
    header: { left: "abcdef", center: null },
    footer: { right: 77 },
    repeatRows: "1:9", repeatCols: "A:D",
  });
  equal(setup.header.left.length, 6);
  equal(setup.header.center, "");
  equal(setup.footer.right, "77");
  equal(setup.repeatRows, "1:9");
  equal(setup.repeatCols, "A:D");
  var long = new Array(130).join("x");
  equal(g.core.normalizeSetup({ header: { left: long } }).header.left.length, 120);
  equal(g.core.normalizeSetup({ repeatRows: long }).repeatRows.length, 40);
});

test("normalizeSetup keeps only well-formed manual breaks, capped at 50", function () {
  var setup = g.core.normalizeSetup({
    manualBreaks: [
      { axis: "col", before: 3 }, { axis: "row", before: 0 },
      { axis: "col", before: 2.5 }, { axis: "sideways", before: 4 },
      null, { axis: "row", before: -1 },
    ],
  });
  deep(setup.manualBreaks, [
    { axis: "col", before: 3 }, { axis: "row", before: 0 },
  ]);
  var many = [];
  for (var i = 0; i < 80; i++) many.push({ axis: "row", before: i });
  equal(g.core.normalizeSetup({ manualBreaks: many }).manualBreaks.length, 50);
});

test("[KNOWN QUIRK] numberIn coerces null to 0 via Number(null)", function () {
  equal(g.core.normalizeSetup({ fitToWidth: null }).fitToWidth, 0);
  equal(g.core.normalizeSetup({ percent: undefined }).percent, 100);
  equal(g.core.normalizeSetup({ percent: "nope" }).percent, 100);
});

// ── workbook-scoped state ────────────────────────────────────────────────
test("storageKey is scoped to the active sheet id", function () {
  equal(g.core.storageKey(), "gb-sheet-reporting:sheet-A");
  equal(g.core.sheetId(), "sheet-A");
});

test("empty state is local-only and versioned", function () {
  var empty = g.core.emptyState();
  equal(empty.version, 1);
  equal(empty.localOnly, true);
  deep(empty.printAreas, []);
  deep(empty.dashboards, []);
  deep(empty.schedules, []);
});

test("state() falls back to empty state on missing or corrupt storage", function () {
  g.resetState();
  deep(g.core.state(), g.core.emptyState());
  g.storage.setItem(g.core.storageKey(), "{not-json");
  deep(g.core.state(), g.core.emptyState());
  g.resetState();
});

test("save/state round-trip persists and renormalises stored data", function () {
  g.resetState();
  var next = g.core.emptyState();
  next.printAreas = [{ id: "a", name: "<n>", range: "A1:B2" }];
  next.pageSetup.orientation = "landscape";
  assert(g.core.save(next) === true);
  equal(g.core.state().printAreas.length, 1);
  equal(g.core.state().pageSetup.orientation, "landscape");
  g.resetState();
});

test("mutate persists changes and returns the callback result", function () {
  g.resetState();
  var result = g.core.mutate(function (state) {
    state.pageSetup.paper = "A3";
    return "done";
  });
  equal(result, "done");
  equal(g.core.state().pageSetup.paper, "A3");
  g.resetState();
});

test("escapeHtml covers the five HTML-sensitive characters", function () {
  equal(g.core.escapeHtml('<a href="x">&\'</a>'),
    "&lt;a href=&quot;x&quot;&gt;&amp;&#39;&lt;/a&gt;");
  equal(g.core.escapeHtml(null), "");
  equal(g.core.escapeHtml(undefined), "");
});

// ── print areas ──────────────────────────────────────────────────────────
test("validateArea rejects missing fields and invalid ranges", function () {
  var missing = g.print.validateArea({ name: "<n>", range: "" });
  equal(missing.ok, false);
  equal(missing.key, "sheet-rpt-needs-fields");
  var bad = g.print.validateArea({ name: "<n>", range: "A1" });
  equal(bad.ok, false);
  equal(bad.key, "sheet-rpt-invalid-range");
});

test("validateArea uppercases a valid range and caps the name", function () {
  var valid = g.print.validateArea({ name: new Array(90).join("n"), range: " a1:h60 " });
  equal(valid.ok, true);
  equal(valid.range, "A1:H60");
  equal(valid.name.length, 80);
});

test("addArea/clearArea/clearAreas maintain the local print-area list", function () {
  g.resetState();
  var added = g.print.addArea({ name: "<one>", range: "A1:H60" });
  equal(added.ok, true);
  assert(/^area-[a-z0-9]+-[a-z0-9]+$/.test(g.core.state().printAreas[0].id));
  g.print.addArea({ name: "<two>", range: "J1:M20" });
  equal(g.print.summary().count, 2);
  g.print.clearArea(g.core.state().printAreas[0].id);
  equal(g.print.summary().count, 1);
  equal(g.core.state().printAreas[0].name, "<two>");
  g.print.clearAreas();
  equal(g.print.summary().count, 0);
  g.resetState();
});

// ── pagination math ──────────────────────────────────────────────────────
test("paginate reports no-area deterministically for invalid input", function () {
  var plan = g.print.paginate("nope", g.core.defaultSetup());
  equal(plan.pages, 0);
  equal(plan.reason, "no-area");
  deep(plan.colBreaks, []);
  deep(plan.rowBreaks, []);
  equal(plan.colPages, 0);
  equal(plan.rowPages, 0);
});

test("paginate A1:H60 on default A4 portrait into 1 x 2 pages", function () {
  var plan = g.print.paginate("A1:H60", g.core.defaultSetup());
  equal(plan.reason, "ok");
  equal(plan.colsPerPage, 8);
  equal(plan.rowsPerPage, 43);
  equal(plan.colPages, 1);
  equal(plan.rowPages, 2);
  equal(plan.pages, 2);
  deep(plan.colBreaks, [{ start: 0, end: 7, manual: false }]);
  deep(plan.rowBreaks, [
    { start: 0, end: 42, manual: false },
    { start: 43, end: 59, manual: false },
  ]);
});

test("paginate landscape widens columns and shortens rows", function () {
  var setup = g.core.defaultSetup();
  setup.orientation = "landscape";
  var plan = g.print.paginate("A1:H60", setup);
  // 45 * 0.7 is 31.499999999999996 in JS, so Math.round yields 31 before margins.
  equal(plan.colsPerPage, 10);
  equal(plan.rowsPerPage, 29);
  equal(plan.colPages, 1);
  equal(plan.rowPages, 3);
  equal(plan.pages, 3);
});

test("paginate A3 expands a wide row-major area without splitting columns", function () {
  var setup = g.core.defaultSetup();
  setup.paper = "A3";
  var plan = g.print.paginate("A1:Z100", setup);
  equal(plan.colsPerPage, 26);
  equal(plan.rowsPerPage, 58);
  equal(plan.colPages, 1);
  equal(plan.rowPages, 2);
  equal(plan.pages, 2);
});

test("paginate percent scaling produces a 3 x 3 grid for a 50% scale", function () {
  var setup = g.core.defaultSetup();
  setup.scaleMode = "percent";
  setup.percent = 50;
  var plan = g.print.paginate("A1:H60", setup);
  equal(plan.colsPerPage, 3);
  equal(plan.rowsPerPage, 22);
  equal(plan.colPages, 3);
  equal(plan.rowPages, 3);
  equal(plan.pages, 9);
});

test("paginate applies manual column breaks and flags the split page", function () {
  g.resetState();
  var added = g.print.addBreak("col", "D");
  equal(added.ok, true);
  var plan = g.print.paginate("A1:H60", g.core.state().pageSetup);
  deep(plan.colBreaks, [
    { start: 0, end: 2, manual: false },
    { start: 3, end: 7, manual: true },
  ]);
  equal(plan.colPages, 2);
  equal(plan.pages, 4);
  g.resetState();
});

test("manual breaks are sorted, deduplicated and reversible", function () {
  g.resetState();
  g.print.addBreak("row", "30");
  g.print.addBreak("row", "10");
  g.print.addBreak("row", "10");
  deep(g.core.state().pageSetup.manualBreaks, [
    { axis: "row", before: 9 }, { axis: "row", before: 29 },
  ]);
  var plan = g.print.paginate("A1:H60", g.core.state().pageSetup);
  deep(plan.rowBreaks[1], { start: 9, end: 28, manual: true });
  deep(plan.rowBreaks[2], { start: 29, end: 59, manual: true });
  g.print.clearBreaks();
  deep(g.core.state().pageSetup.manualBreaks, []);
  equal(g.print.paginate("A1:H60", g.core.state().pageSetup).rowPages, 2);
  g.resetState();
});

test("addBreak rejects malformed column letters and row numbers", function () {
  g.resetState();
  equal(g.print.addBreak("col", "4").ok, false);
  equal(g.print.addBreak("row", "0").ok, false);
  equal(g.print.addBreak("row", "nope").ok, false);
  deep(g.core.state().pageSetup.manualBreaks, []);
  g.resetState();
});

test("updateSetup persists normalised page setup patches", function () {
  g.resetState();
  var setup = g.print.updateSetup({ orientation: "landscape", paper: "A3", percent: 900 });
  equal(setup.orientation, "landscape");
  equal(setup.paper, "A3");
  equal(setup.percent, 400);
  equal(g.core.state().pageSetup.paper, "A3");
  g.resetState();
});

test("fixed uid/clock makes identical addArea calls produce identical state", function () {
  g.resetState();
  function snapshot() {
    g.print.addArea({ name: "<same>", range: "A1:B2" });
    g.print.addArea({ name: "<other>", range: "C3:D4" });
    return JSON.stringify(g.core.state());
  }
  var first = g.withFixedIds(snapshot);
  g.resetState();
  var second = g.withFixedIds(snapshot);
  equal(first, second);
  g.resetState();
});
