/**
 * Ingest core, simulation and cell/provenance logic (#2494).
 * Every non-STATIC test executes the real shipped modules in the vm sandbox.
 */
"use strict";

var h = require("./00_harness");
var g = h.loadModules();
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var deep = h.deep;
var C = g.core;

test("all nine shipped ingest namespaces install into one sandbox", function () {
  ["SheetIngestCore", "SheetIngestSim", "SheetIngestPreview", "SheetIngestMap",
    "SheetIngestReview", "SheetIngestBatch", "SheetIngestPolicy", "SheetIngest",
  ].forEach(function (name) {
    assert(g.window[name] && typeof g.window[name] === "object", "missing namespace " + name);
  });
  ["t", "sheetId", "storageKey", "state", "save", "mutate", "emptyState", "normalize",
    "oneOf", "clampDays", "element", "escapeHtml", "focusables", "setStatus", "makeDialog",
    "colName", "cellLabel", "cellAt", "isLow", "isUnreadable", "cellKey", "overrideFor",
    "isConfirmed", "isResolved", "effectiveValue", "formulaBlocked", "toNumber",
    "columnTypes", "detectProfile", "isDataRow", "runningBalance",
  ].forEach(function (name) {
    assert(typeof C[name] === "function", "missing core helper " + name);
  });
  equal(C.THRESHOLD, 0.85);
  equal(C.MAX_FILES, 12);
  equal(C.MAX_OVERRIDES, 200);
});

test("default state is workbook scoped, local-only and deterministic", function () {
  g.reset();
  var state = C.emptyState();
  equal(state.localOnly, true);
  equal(state.version, 1);
  equal(state.activeFileId, "stmt");
  deep(state.files.map(function (file) { return file.id; }), ["stmt", "inv", "photo", "bad"]);
  assert(state.files.every(function (file) { return file.status === "queued" && file.error === ""; }));
  deep(state.overrides, []);
  equal(state.preview, null);
  equal(state.policy.retainDays, 30);
  equal(state.policy.ocrStored, true);
  assert(state.policy.ocrText.indexOf("OPENING BALANCE") > 0);
  equal(C.storageKey(), "gb-sheet-ingest:sheet-A");
});

test("normalize clamps enums, lengths and policy values from untrusted state", function () {
  var files = [];
  for (var i = 0; i < 20; i++) files.push({ id: "f" + i, status: "bogus", name: null });
  var overrides = [];
  for (var j = 0; j < 300; j++) overrides.push({ key: "k" + j });
  var result = C.normalize({
    files: files,
    overrides: overrides,
    policy: { retainDays: 99999, ocrStored: "yes", ocrText: new Array(5000).join("x") },
    activeFileId: 42,
  });
  equal(result.files.length, 12);
  equal(result.files[0].status, "queued");
  equal(result.files[0].name, "f0");
  equal(result.overrides.length, 200);
  equal(result.policy.retainDays, 3650);
  equal(result.policy.ocrStored, true);
  equal(result.policy.ocrText.length, 4000);
  equal(result.activeFileId, "stmt");
  equal(C.clampDays("abc", 7), 7);
  equal(C.clampDays(-4, 7), 1);
});

test("normalize returns fresh safe defaults for malformed input", function () {
  deep(C.normalize(null), C.emptyState());
  deep(C.normalize("bad"), C.emptyState());
  deep(C.normalize(7), C.emptyState());
});

test("simulateParse builds a three-page bank fixture with structure tags", function () {
  var parse = g.sim.simulateParse("march-statement.pdf");
  equal(parse.kind, "bank");
  equal(parse.pages, 3);
  equal(parse.rows.length, 10);
  deep(parse.columns.map(function (column) { return column.header; }),
    ["Date", "Description", "Debit", "Credit", "Balance"]);
  deep(parse.columns.map(function (column) { return column.target; }),
    ["date", "text", "currency", "currency", "currency"]);
  deep(parse.structures, [
    { kind: "subtotal", row: 6, page: 2, col: 0, colSpan: 1 },
    { kind: "merged", row: 6, page: 2, col: 1, colSpan: 2 },
    { kind: "repeated-header", row: 7, page: 3, col: 0, colSpan: 5 },
  ]);
  var pages = {};
  parse.rows.forEach(function (row) { row.forEach(function (cellValue) { pages[cellValue.page] = true; }); });
  deep(Object.keys(pages).sort(), ["1", "2", "3"]);
});

test("simulateParse selects the invoice fixture by name and keeps its flags", function () {
  var parse = g.sim.simulateParse("invoice-batch.pdf");
  equal(parse.kind, "invoice");
  equal(parse.pages, 1);
  equal(parse.rows.length, 3);
  deep(parse.structures, []);
  deep(parse.columns.map(function (column) { return column.target; }),
    ["text", "number", "currency", "currency"]);
  equal(parse.rows[2][3].confidence, 0.2);
  equal(parse.rows[2][3].unreadable, true);
});

test("ensureParse caches a parse per file id and is idempotent", function () {
  g.reset();
  var state = C.emptyState();
  var first = g.sim.ensureParse(state, "stmt");
  assert(first === g.sim.ensureParse(state, "stmt"));
  assert(first !== g.sim.ensureParse(state, "inv"));
  equal(first.name, "march-statement.pdf");
  equal(g.sim.ensureParse(state, "inv").kind, "invoice");
});

test("colName and cellLabel produce spreadsheet addresses", function () {
  deep([0, 25, 26, 27, 701].map(C.colName), ["A", "Z", "AA", "AB", "ZZ"]);
  equal(C.colName(-5), "A");
  var parse = g.sim.simulateParse("stmt");
  equal(C.cellLabel(parse, 0, 0), "A1");
  equal(C.cellLabel(parse, 8, 4), "E9");
  equal(C.cellLabel(null, 0, 0), "?");
});

test("cellAt and isLow classify confidence against the shipped threshold", function () {
  g.reset();
  g.seed();
  var state = C.state();
  equal(C.cellAt(state.parses.stmt, 2, 2).value, "120.50");
  equal(C.isLow(C.cellAt(state.parses.stmt, 2, 2)), true);
  equal(C.isLow(C.cellAt(state.parses.stmt, 1, 0)), false);
  equal(C.cellAt(state.parses.stmt, 99, 0), null);
  equal(C.cellAt(null, 0, 0), null);
});

test("[KNOWN BUG] isUnreadable conflates intentional blanks with OCR failures", function () {
  g.reset();
  g.seed();
  var parse = C.state().parses.stmt;
  // 59b:35-37 — any empty string is 'unreadable', even at confidence 0.96.
  equal(C.isUnreadable(C.cellAt(parse, 4, 2)), true);
  equal(C.cellAt(parse, 4, 2).unreadable, true);
  equal(C.cellAt(parse, 1, 2).unreadable, false);
  equal(C.isUnreadable(C.cellAt(parse, 1, 2)), true);
  equal(C.cellAt(parse, 1, 2).confidence, 0.96);
  // The reviewable picker therefore lists high-confidence intentional blanks too.
  assert(g.review.reviewableCells(parse, "stmt").some(function (item) {
    return item.row === 1 && item.col === 2;
  }));
});

test("cellKey, confirmations and overrides gate formula use", function () {
  g.reset();
  g.seed();
  equal(C.cellKey("stmt", 2, 2), "stmt:2:2");
  equal(C.isConfirmed(C.state(), "stmt", 2, 2), false);
  equal(C.isResolved(C.state(), "stmt", 2, 2), false);
  equal(C.formulaBlocked(C.state(), "stmt", 2, 2), true);
  C.mutate(function (target) { target.confirmations["stmt:2:2"] = true; });
  equal(C.isConfirmed(C.state(), "stmt", 2, 2), true);
  equal(C.isResolved(C.state(), "stmt", 2, 2), true);
  equal(C.formulaBlocked(C.state(), "stmt", 2, 2), false);
});

test("effectiveValue prefers an override and never invents an unreadable value", function () {
  g.reset();
  g.seed();
  equal(C.effectiveValue(C.state(), "stmt", 4, 2), "");
  equal(C.effectiveValue(C.state(), "stmt", 2, 2), "120.50");
  C.mutate(function (target) {
    target.overrides.push({ key: "stmt:4:2", fileId: "stmt", row: 4, col: 2,
      original: "", corrected: "77.70", author: "<a>", timestamp: "<t>" });
  });
  equal(C.effectiveValue(C.state(), "stmt", 4, 2), "77.70");
  equal(C.overrideFor(C.state(), "stmt", 4, 2).corrected, "77.70");
  equal(C.overrideFor(C.state(), "stmt", 4, 3), null);
});

test("toNumber normalizes separators and rejects non-numeric input", function () {
  equal(C.toNumber("1000.00"), 1000);
  equal(C.toNumber("1,234.56"), 1234.56);
  equal(C.toNumber(-12.5), -12.5);
  ["", "-", "abc", "12-34", null].forEach(function (value) {
    equal(C.toNumber(value), null, "accepted " + value);
  });
});

test("toNumber treats a trailing decimal comma as the decimal mark (regression #2494)", function () {
  equal(C.toNumber("45,00"), 45);
  equal(C.toNumber("R$ 45,00"), 45);
  equal(C.toNumber("1.234,50"), 1234.5);
});

test("columnTypes reads the saved mapping and falls back per column", function () {
  g.reset();
  g.seed();
  deep(C.columnTypes(C.state(), "stmt"),
    ["date", "text", "currency", "currency", "currency"]);
  C.mutate(function (target) {
    target.mappings.stmt = ["text", "date", "number", "currency", "bogus"];
  });
  deep(C.columnTypes(C.state(), "stmt"),
    ["text", "date", "number", "currency", "currency"]);
});

test("detectProfile finds all bank columns and falls back to generic", function () {
  g.reset();
  g.seed();
  deep(C.detectProfile(C.state().parses.stmt), {
    kind: "bank", columns: { date: 0, description: 1, debit: 2, credit: 3, balance: 4 },
  });
  equal(C.detectProfile(C.state().parses.inv).kind, "generic");
  equal(C.detectProfile(null).kind, "generic");
});

test("isDataRow excludes header, subtotal and repeated-header rows", function () {
  g.reset();
  g.seed();
  var parse = C.state().parses.stmt;
  [1, 2, 3, 4, 5, 8, 9].forEach(function (row) { equal(C.isDataRow(parse, row), true); });
  [0, 6, 7].forEach(function (row) { equal(C.isDataRow(parse, row), false); });
});

test("runningBalance seeds from opening balance and flags unconfirmed rows", function () {
  g.reset();
  g.seed();
  var result = C.runningBalance(C.state(), "stmt");
  equal(result.kind, "bank");
  equal(result.available, 6);
  equal(result.blocked, 2);
  equal(result.mismatch, 0);
  deep(result.rows.map(function (item) { return item.row; }), [2, 8]);
  equal(result.rows[1].stated, 1300);
});

test("[KNOWN BUG] runningBalance skips a blocked row whose amount cells are blank", function () {
  g.reset();
  g.seed();
  var state = C.state();
  var parse = state.parses.stmt;
  equal(C.isDataRow(parse, 4), true);
  equal(C.formulaBlocked(state, "stmt", 4, 2), true);
  // 59b:147 — `continue` fires before blocked is counted when all three
  // numeric values are blank, so the unreadable charge disappears from the check.
  var result = C.runningBalance(state, "stmt");
  assert(!result.rows.some(function (item) { return item.row === 4; }));
  equal(result.blocked, 2);
  equal(g.review.reviewableCells(parse, "stmt").some(function (item) { return item.row === 4 && item.col === 2; }), true);
});

test("confirming the low-confidence debit removes its formula block", function () {
  g.reset();
  g.seed();
  C.mutate(function (target) { target.confirmations["stmt:2:2"] = true; });
  var result = C.runningBalance(C.state(), "stmt");
  equal(result.blocked, 1);
  equal(result.mismatch, 0);
  equal(result.rows[0].row, 8);
});

test("escapeHtml and setStatus keep error feedback safe and accessible", function () {
  equal(C.escapeHtml("<b>&\"'</b>"), "&lt;b&gt;&amp;&quot;&#39;&lt;/b&gt;");
  var region = g.document.createElement("p");
  var dialog = { querySelector: function () { return region; } };
  C.setStatus(dialog, "problem", true);
  equal(region.textContent, "problem");
  equal(region.classList.contains("ss-ingest-status-error"), true);
  C.setStatus(dialog, "fine", false);
  equal(region.classList.contains("ss-ingest-status-error"), false);
});
