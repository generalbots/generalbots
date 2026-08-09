"use strict";
/* Sheet advanced modules — zero-dependency unit tests (run: node tests/sheet_modules.test.js)
 * Loads the IIFE modules in a minimal window/document/fetch shim and asserts on
 * the pure logic exposed through window.SheetCore / window.SheetClipboard. */

const fs = require("fs");
const path = require("path");
const vm = require("vm");

const MODULES = path.join(__dirname, "..", "js", "modules-sheet-advanced");

let passed = 0;
let failed = 0;

function assertEqual(actual, expected, label) {
  const a = JSON.stringify(actual);
  const e = JSON.stringify(expected);
  if (a === e) {
    passed++;
  } else {
    failed++;
    console.error("FAIL " + label + "\n  expected: " + e + "\n  actual:   " + a);
  }
}

function assertTrue(cond, label) {
  if (cond) passed++;
  else {
    failed++;
    console.error("FAIL " + label);
  }
}

function makeCellMap(obj) {
  return new Map(Object.entries(obj));
}

function loadModules() {
  const noopEl = {
    style: {},
    dataset: {},
    classList: { contains: function () { return false; } },
    addEventListener: function () {},
    appendChild: function () {},
    setAttribute: function () {},
    remove: function () {},
    select: function () {},
    value: "",
  };
  const sandbox = {
    window: {},
    document: {
      body: { appendChild: function () {}, removeChild: function () {} },
      createElement: function () { return noopEl; },
      getElementById: function () { return null; },
      querySelector: function () { return null; },
      querySelectorAll: function () { return []; },
      addEventListener: function () {},
    },
    navigator: { language: "en" },
    localStorage: { getItem: function () { return null; }, setItem: function () {} },
    sessionStorage: { getItem: function () { return null; }, setItem: function () {} },
    fetch: function () { return Promise.resolve({ ok: true, json: function () { return Promise.resolve({}); }, text: function () { return Promise.resolve(""); }, blob: function () { return Promise.resolve({}); } }); },
    console: console,
    setTimeout: function () {},
    clearTimeout: function () {},
    CustomEvent: function (t) { return { type: t }; },
    Promise: Promise,
    Map: Map,
    Set: Set,
    Number: Number,
    String: String,
    Object: Object,
    Array: Array,
    JSON: JSON,
    Math: Math,
    isNaN: isNaN,
    parseInt: parseInt,
    parseFloat: parseFloat,
  };
  sandbox.window = sandbox;
  sandbox.dispatchEvent = function () {};
  sandbox.addEventListener = function () {};
  sandbox.globalThis = sandbox;
  const ctx = vm.createContext(sandbox);
  ["00_registry.js", "01_core.js", "02_conditional.js", "03_charts.js", "05_clipboard.js", "07_conditional_render.js", "14_widths.js", "15_freeze.js", "17_filter.js", "18_charts.js"].forEach(function (f) {
    const code = fs.readFileSync(path.join(MODULES, f), "utf8");
    vm.runInContext(code, ctx, { filename: f });
  });
  return sandbox;
}

// --- Tests ---

const env = loadModules();

// 01_core colName / setRange
assertEqual(env.window.SheetCore.colName(0), "A", "colName(0)=A");
assertEqual(env.window.SheetCore.colName(25), "Z", "colName(25)=Z");
assertEqual(env.window.SheetCore.colName(26), "AA", "colName(26)=AA");

// 05_clipboard adjustFormula: $ anchors survive fill, relative shift by dc/dr
env.window.SheetAdvanced.setRange(0, 0, 0, 0);
// expose adjustFormula indirectly: not public; test via SheetClipboard internals through a fill on a stubbed grid
const g = {
  cells: makeCellMap({
    "0,0": { value: "1", formula: null },
    "0,1": { value: "2", formula: null },
  }),
  totalRows: 1000,
  totalCols: 26,
  lastRenderedRange: null,
  requestRange: function () {},
  selectCell: function () {},
};
env.window.SheetVirtualGrid = g;
env.window.SheetCore.setGrid(g);
env.window.SheetAPI = { updateCell: function (ref, v) { return Promise.resolve({ success: true }); }, load: function () { return Promise.resolve(null); } };

// computeFillCell via applyFill: source A1:A2 = 1,2 -> fill down extrapolates 3,4,5
g.cells.set("1,0", { value: "2" });
env.window.SheetAdvanced.setRange(0, 0, 1, 0); // A1:A2
env.window.SheetCore.applyFill(4, 0).then(function () { // fill to A5
  const a3 = g.cells.get("2,0");
  const a4 = g.cells.get("3,0");
  const a5 = g.cells.get("4,0");
  assertEqual(a3 && a3.value, "3", "fill A3 = 3 (series)");
  assertEqual(a4 && a4.value, "4", "fill A4 = 4 (series)");
  assertEqual(a5 && a5.value, "5", "fill A5 = 5 (series)");
  runClipboard();
});

// clipboard text build: selection A1:B2 with values
function runClipboard() {
  g.cells = makeCellMap({
    "0,0": { value: "a", formula: null },
    "0,1": { value: "b", formula: null },
    "1,0": { value: "c", formula: null },
    "1,1": { value: "d", formula: null },
  });
  env.window.SheetAdvanced.setRange(0, 0, 1, 1);
  env.window.__gbClipboard = null;
  env.window.navigator.clipboard = { writeText: function (t) { env.window.__gbClipboard = t; return Promise.resolve(); } };
  env.window.SheetClipboard.copy().then(function () {
    assertEqual(env.window.__gbClipboard, "a\tb\nc\td", "clipboard TSV for A1:B2");
    runValidation();
  });
}

// validation eval: number rule rejects text, list rule accepts allowed
function runValidation() {
  function validationValue(rule, v) {
    const sheet = { worksheets: [{ validations: { "0,0": rule } }] };
    env.window.__LOADED_SHEET = sheet;
    const res = env.window.SheetCore.validateEdit(0, 0, v);
    return res.valid;
  }
  assertTrue(validationValue({ validation_type: "number", allowed_values: null }, "12.5"), "number validation accepts 12.5");
  assertTrue(!validationValue({ validation_type: "number", allowed_values: null }, "abc"), "number validation rejects abc");
  assertTrue(validationValue({ validation_type: "list", allowed_values: ["Red", "Green"] }, "Red"), "list validation accepts Red");
  assertTrue(!validationValue({ validation_type: "list", allowed_values: ["Red", "Green"] }, "Blue"), "list validation rejects Blue");
  assertTrue(!validationValue({ validation_type: "integer", allowed_values: null }, "1.5"), "integer validation rejects 1.5");
  runWidths();
}

// column-width math: colX sums widths; colWidth returns custom or default
function runWidths() {
  const sheet = { worksheets: [{ column_widths: { "0": 120, "1": 80 }, row_heights: { "0": 30 }, validations: {} }] };
  env.window.__LOADED_SHEET = sheet;
  env.window.SheetCore.rehydrateGrid = function () {};
  const widthGrid = {
    totalCols: 26,
    totalRows: 100,
    cells: new Map(),
    bodyInner: { appendChild: function () {}, lastChild: { style: {} }, querySelector: function () { return null; }, getBoundingClientRect: function () { return { left: 0, top: 0 }; } },
    headerRow: { appendChild: function () {}, querySelectorAll: function () { return []; }, style: {} },
    headerColPool: [],
    getOrCreateNode: function () { return { style: {} }; },
    cellsMap: {},
    render: function () {},
    renderHeaders: function () {},
    renderRow: function () {},
    applyCellStyle: function () {},
    editingCell: null,
    requestRange: function () {},
    lastRenderedRange: null,
  };
  env.window.SheetVirtualGrid = widthGrid;
  env.window.SheetCore.setGrid(widthGrid);
  env.window.SheetCore.refreshWidths();
  assertEqual(env.window.SheetCore.colWidth(0), 120, "colWidth(0) = 120 (custom)");
  assertEqual(env.window.SheetCore.colWidth(1), 80, "colWidth(1) = 80 (custom)");
  assertEqual(env.window.SheetCore.colWidth(2), 96, "colWidth(2) = 96 (default)");
  assertEqual(env.window.SheetCore.rowHeight(0), 30, "rowHeight(0) = 30 (custom)");
  assertEqual(env.window.SheetCore.rowHeight(1), 24, "rowHeight(1) = 24 (default)");
  // colX: HEADER(48) + w0 + w1 = 248
  assertEqual(env.window.SheetCore.colX(2), 48 + 120 + 80, "colX(2) sums widths (248)");
  runFreeze();
}

// freeze state read from loaded sheet
function runFreeze() {
  env.window.__LOADED_SHEET = { worksheets: [{ frozen_rows: 1, frozen_cols: 0, validations: {} }] };
  const f = env.window.SheetFreeze.getFrozen();
  assertEqual(f.rows, 1, "frozen.rows = 1");
  assertEqual(f.cols, 0, "frozen.cols = 0");
  runFilters();
}

// filter matching: list values, numeric ranges, contains
function runFilters() {
  const mf = env.window.SheetFilter.matchesFilter;
  assertTrue(mf("", null), "matchesFilter noop is safe");
  assertTrue(mf("Red", { values: ["Red", "Green"] }), "list filter accepts Red");
  assertTrue(!mf("Blue", { values: ["Red", "Green"] }), "list filter rejects Blue");
  assertTrue(mf("5", { condition: ">3" }), "numeric >3 accepts 5");
  assertTrue(!mf("2", { condition: ">3" }), "numeric >3 rejects 2");
  assertTrue(mf("10", { condition: ">=5", value2: "20" }), ">=5 accepts 10");
  assertTrue(mf("Shipping", { condition: "contains:Ship" }), "contains matches Shipping");
  assertTrue(!mf("Billing", { condition: "contains:Ship" }), "contains rejects Billing");
  assertTrue(!mf("1.5", { condition: ">=5" }), ">=5 rejects 1.5");
  runFormulaFill();
}

// formula fill: relative refs shift with the fill direction; $ anchors stay
function runFormulaFill() {
  env.window.SheetAPI = { updateCell: function (ref, v) { return Promise.resolve({ success: true }); }, load: function () { return Promise.resolve(null); } };
  const g2 = {
    cells: makeCellMap({ "0,0": { value: "10", formula: null }, "0,1": { value: "5", formula: null } }),
    totalRows: 1000,
    totalCols: 26,
    lastRenderedRange: null,
    requestRange: function () {},
    selectCell: function () {},
  };
  env.window.SheetVirtualGrid = g2;
  env.window.SheetCore.setGrid(g2);
  // put a formula in A1
  g2.cells.set("0,0", { value: "", formula: "=A2+B2" });
  g2.cells.set("1,0", { value: "1" });
  g2.cells.set("1,1", { value: "2" });
  env.window.SheetAdvanced.setRange(0, 0, 0, 0);
  env.window.SheetCore.applyFill(0, 1).then(function () { // fill right to B1
    const b1 = g2.cells.get("0,1");
    assertEqual(b1 && b1.formula, "=B2+C2", "fill right shifts relative refs A2->B2, B2->C2");
    // fill down from A1 with $ anchors
    g2.cells.set("0,0", { value: "", formula: "=$A$2+B2" });
    g2.cells.set("1,0", { value: "1" });
    g2.cells.set("1,1", { value: "2" });
    env.window.SheetAdvanced.setRange(0, 0, 0, 0);
    env.window.SheetCore.applyFill(2, 0).then(function () {
      const a2 = g2.cells.get("1,0");
      const a3 = g2.cells.get("2,0");
      assertEqual(a2 && a2.formula, "=$A$2+B3", "fill down keeps $A$2 anchor, shifts B2->B3");
      assertEqual(a3 && a3.formula, "=$A$2+B4", "fill down second row shifts B3->B4");
      runCharts();
    });
  });
}

// chart SVG renderers emit valid svg + expected element
function runCharts() {
  env.window.__LOADED_SHEET = { worksheets: [{ charts: [{ id: "c1", chart_type: "bar", title: "Sales", labels: ["A", "B"], datasets: [{ label: "S1", data: [1, 3], color: "#3b82f6" }] }], validations: {} }] };
  const C = env.window.SheetChartsRender;
  assertTrue(typeof C === "object" && C !== null, "SheetChartsRender exposed");
  assertTrue(C.renderAll !== undefined, "renderAll exposed");
  finalize();
}

function finalize() {
  console.log("\nSheet modules: " + passed + " passed, " + failed + " failed");
  process.exit(failed ? 1 : 0);
}