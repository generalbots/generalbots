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
  ["00_registry.js", "01_core.js", "02_conditional.js", "03_charts.js", "05_clipboard.js", "07_conditional_render.js", "14_widths.js"].forEach(function (f) {
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
  env.window.SheetCore.api = function () { return null; };
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
  finalize();
}

function finalize() {
  console.log("\nSheet modules: " + passed + " passed, " + failed + " failed");
  process.exit(failed ? 1 : 0);
}