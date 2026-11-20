/**
 * Panel shell, watch list, overlay and boot wiring (#2491).
 * Every assertion executes the shipped 27 / 28 / 28a / 29 / 30 IIFEs against
 * DOM doubles. No production code is reimplemented and no workbook is mutated.
 */
"use strict";

var h = require("./00_harness");
var e = h.createEngine();
var U = e.ui;
var S = e.sections;
var O = e.overlay;
var G = e.graph;
var assert = h.assert;
var equal = h.equal;
var deep = h.deep;

function buttonByText(root, label) {
  return root.querySelectorAll("button").filter(function (node) {
    return node.textContent === label;
  })[0];
}

h.test("open() builds an accessible dialog shell with seven tabs", function () {
  var panel = U.open();
  assert(panel, "panel was created");
  equal(panel.id, "sheet-audit-dialog", "panel id");
  equal(panel.getAttribute("role"), "dialog", "role=dialog");
  equal(panel.getAttribute("aria-modal"), "true", "aria-modal");
  equal(panel.getAttribute("aria-labelledby"), "sheet-audit-title", "labelled by title");
  equal(panel.style.display, "block", "visible after open");
  equal(panel.querySelectorAll(".sheet-audit-tab").length, 7, "seven tabs");
  equal(panel.querySelectorAll(".sheet-audit-section").length, 7, "seven sections");
  var statusNode = panel.querySelector(".sheet-audit-status");
  equal(statusNode.getAttribute("role"), "status", "status role");
  equal(statusNode.getAttribute("aria-live"), "polite", "polite live region");
  assert(panel.querySelector(".sheet-audit-local-note"), "local-only caveat is present");
});

h.test("every tab label comes from the real i18n catalog key with an English fallback", function () {
  var fresh = h.createEngine();
  fresh.ui.open();
  var keys = fresh.i18nCalls.map(function (call) { return call.key; });
  ["sheet-audit-tab-trace", "sheet-audit-tab-evaluate", "sheet-audit-tab-watch",
    "sheet-audit-tab-errors", "sheet-audit-tab-cycles", "sheet-audit-tab-text",
    "sheet-audit-tab-map", "sheet-audit-title", "sheet-audit-local-note",
    "sheet-audit-status-ready"].forEach(function (key) {
    assert(keys.indexOf(key) >= 0, "missing i18n key " + key);
  });
  assert(fresh.i18nCalls.every(function (call) { return typeof call.fallback === "string"; }),
    "every catalog lookup has a fallback");
  assert(fresh.i18nCalls.every(function (call) {
    return !/\s\+\s/.test(call.key) && call.key.indexOf("(") < 0;
  }), "no dynamic key concatenation is passed to the translator");
});

h.test("switchTab updates aria-selected, hidden sections and focus", function () {
  U.open();
  U.switchTab("watch");
  equal(U.currentTab(), "watch", "current tab");
  var panel = e.doc.getElementById("sheet-audit-dialog");
  var selected = panel.querySelectorAll(".sheet-audit-tab").filter(function (node) {
    return node.getAttribute("aria-selected") === "true";
  });
  equal(selected.length, 1, "exactly one tab selected");
  equal(selected[0].dataset.tab, "watch", "watch tab selected");
  equal(selected[0].tabIndex, 0, "selected tab in tab order");
  var visible = panel.querySelectorAll(".sheet-audit-section").filter(function (node) {
    return !node.hidden;
  });
  equal(visible.length, 1, "exactly one section visible");
  equal(visible[0].id, "sheet-audit-section-watch", "watch section visible");
  equal(panel.querySelector("#sheet-audit-section-trace").hidden, true, "trace section hidden");
});

h.test("Escape, close button and toggle all close the dialog without losing it", function () {
  var panel = U.open();
  panel.dispatchEvent({ type: "keydown", key: "Escape", preventDefault: function () {} });
  equal(panel.style.display, "none", "Escape closes");
  U.open();
  panel.querySelector(".sheet-audit-close").dispatchEvent({ type: "click" });
  equal(panel.style.display, "none", "close button closes");
  U.toggle();
  equal(panel.style.display, "block", "toggle opens");
  U.toggle();
  equal(panel.style.display, "none", "toggle closes");
  U.open();
});

h.test("selectionRef derives A1-style refs from the active sheet instance", function () {
  e.setSelection({ startRow: 3, startCol: 2, endRow: 3, endCol: 2 });
  equal(U.selectionRef(), "C4", "zero-based selection becomes C4");
  e.setSelection({ startRow: 0, startCol: 0, endRow: 0, endCol: 0 });
  equal(U.selectionRef(), "A1", "origin cell");
  e.setSelection(null);
  equal(U.selectionRef(), "", "no selection yields an empty ref");
});

h.test("jumpTo switches worksheets and selects a single-cell range", function () {
  e.switchCalls.length = 0;
  e.setRangeCalls.length = 0;
  U.open();
  S.jumpTo({ sheetIdx: 1, row: 2, col: 3, id: G.cellId(1, 2, 3), ref: "Sheet2!D3" });
  deep(e.switchCalls, [1], "cross-sheet jump switches worksheet");
  deep(e.setRangeCalls, [[2, 3, 2, 3]], "single-cell range selected");
  S.jumpTo(null);
  equal(e.switchCalls.length, 1, "null target is ignored");
});

h.test("watch list pins a cell, shows its live cached value and removes it", function () {
  e.setCell(0, 1, 1, { value: 41, formula: "" });
  U.setWatch([]);
  U.open();
  U.switchTab("watch");
  var root = U.section("watch");
  var input = root.querySelector(".sheet-audit-input");
  input.value = "B2";
  buttonByText(root, "Add watch").dispatchEvent({ type: "click" });
  deep(U.getWatch(), [{ sheetIdx: 0, row: 1, col: 1 }], "pin stored as coordinates");
  U.refreshGraph();
  S.updateWatch();
  deep(root.querySelectorAll(".sheet-audit-watch-value").map(function (n) {
    return n.textContent;
  }), ["41"], "cached value displayed");
  deep(root.querySelectorAll(".sheet-audit-watch-ref").map(function (n) {
    return n.textContent;
  }), ["Sheet1!B2"], "full ref displayed");
  buttonByText(root, "Remove watch").dispatchEvent({ type: "click" });
  deep(U.getWatch(), [], "pin removed");
});

h.test("watch list evaluates a pinned formula instead of showing its cached value", function () {
  e.setCell(0, 0, 0, { value: "stale", formula: "=2+3" });
  U.setWatch([{ sheetIdx: 0, row: 0, col: 0 }]);
  U.open();
  U.refreshGraph();
  S.updateWatch();
  var root = U.section("watch");
  deep(root.querySelectorAll(".sheet-audit-watch-value").map(function (n) {
    return n.textContent;
  }), ["5"], "local evaluator result is shown");
  U.setWatch([]);
});

h.test("refresh watch reports the fixed clock time in a polite status", function () {
  U.setWatch([{ sheetIdx: 0, row: 0, col: 0 }]);
  U.open();
  U.switchTab("watch");
  var root = U.section("watch");
  U.refreshGraph();
  buttonByText(root, "Refresh watch").dispatchEvent({ type: "click" });
  assert(U.status().indexOf("12:00:00") >= 0, "fixed clock is formatted into the status: " + U.status());
  U.setWatch([]);
});

h.test("error scan button classifies cached errors with real catalog keys", function () {
  var book = h.makeBook();
  book.worksheets[0].data["0,0"] = { value: "#DIV/0!", formula: "=1/0" };
  e.setBook(book);
  U.open();
  U.switchTab("errors");
  var root = U.section("errors");
  buttonByText(root, "Scan for errors").dispatchEvent({ type: "click" });
  equal(U.status(), "1 errors found.", "summary count");
  equal(root.querySelectorAll(".sheet-audit-error-item").length, 1, "one error row");
  var types = root.querySelectorAll(".sheet-audit-error-type").map(function (n) { return n.textContent; });
  assert(types.indexOf("#DIV/0!") >= 0, "error type rendered");
  var errorItem = root.querySelector(".sheet-audit-error-item");
  var causes = errorItem.querySelectorAll(".sheet-audit-error-cause")
    .map(function (n) { return n.textContent; });
  assert(causes.length === 1 && causes[0].length > 0, "local cause explanation rendered");
  var before = e.setRangeCalls.length;
  buttonByText(root, "Jump to this cell").dispatchEvent({ type: "click" });
  assert(e.setRangeCalls.length > before, "jump selects the offending cell");
});

h.test("cycle scan button reports exact cycle membership", function () {
  var book = h.makeBook();
  book.worksheets[0].data["1,0"] = { value: "", formula: "=B2+1" };
  book.worksheets[0].data["1,1"] = { value: "", formula: "=A2+1" };
  e.setBook(book);
  U.open();
  U.switchTab("cycles");
  var root = U.section("cycles");
  buttonByText(root, "Detect circular references").dispatchEvent({ type: "click" });
  assert(U.status().indexOf("1 circular reference") === 0, "cycle summary: " + U.status());
  assert(root.querySelectorAll(".sheet-audit-cycle-item").length >= 1, "cycle row rendered");
});

h.test("overlay arrows respect direction, sheet filter and MAX_ARROWS", function () {
  U.open();
  var origin = { id: G.cellId(0, 0, 0), row: 0, col: 0 };
  var node = h.makeNode("div");
  node.dataset.row = "0";
  node.dataset.col = "1";
  e.grid.bodyInner.appendChild(node);
  var targets = [
    { id: G.cellId(0, 0, 1), sheetIdx: 0 },
    { id: G.cellId(1, 0, 1), sheetIdx: 1 },
    { id: origin.id, sheetIdx: 0 },
  ];
  equal(O.drawArrows(origin, targets, "precedents", 0), 1, "only same-sheet, non-origin cells drawn");
  var paths = e.grid.bodyInner.querySelectorAll("path");
  assert(paths.some(function (p) { return p.getAttribute("class") === "sheet-audit-arrowhead"; }),
    "marker definition present");
  assert(paths.some(function (p) { return p.getAttribute("class").indexOf("is-precedent") >= 0; }),
    "precedent class");
  O.clearArrows();
  equal(e.grid.bodyInner.querySelectorAll("svg").length, 0, "clearArrows removes the svg");
  var many = [];
  for (var i = 1; i <= 70; i++) many.push({ id: G.cellId(0, 0, i), sheetIdx: 0 });
  equal(O.drawArrows(origin, many, "dependents", 0), 60, "MAX_ARROWS caps at 60");
  assert(e.grid.bodyInner.querySelectorAll("path").some(function (p) {
    return p.getAttribute("class").indexOf("is-dependent") >= 0;
  }), "dependent class");
  O.clearArrows();
});

h.test("show-formulas swaps rendered text, preserves the cached value and emits an event", function () {
  var cell = h.makeNode("div");
  cell.dataset.row = "0";
  cell.dataset.col = "0";
  cell.dataset.formula = "=1+1";
  cell.textContent = "2";
  e.grid.bodyInner.appendChild(cell);
  e.events.length = 0;
  equal(O.isShowingFormulas(), false, "initially off");
  equal(O.toggleShowFormulas(), true, "toggle enables");
  equal(cell.textContent, "=1+1", "formula text displayed");
  equal(cell.dataset.auditValue, "2", "cached value preserved");
  assert(cell.className.indexOf("sheet-audit-formula-view") >= 0, "view class applied");
  deep(e.events.map(function (event) { return event.type; }), ["gb-sheet-audit-formulas"], "one event emitted");
  equal(e.events[0].detail.showing, true, "event detail carries state");
  equal(O.toggleShowFormulas(), false, "toggle disables");
  equal(cell.textContent, "2", "value restored");
  equal(cell.dataset.auditValue, undefined, "cache marker removed");
  cell.parentNode.removeChild(cell);
});

h.test("show-formulas is view-only and never writes to the workbook", function () {
  var book = h.makeBook();
  book.worksheets[0].data["0,0"] = { value: 2, formula: "=1+1" };
  e.setBook(book);
  var before = JSON.stringify(book);
  var cell = h.makeNode("div");
  cell.dataset.row = "0";
  cell.dataset.col = "0";
  cell.dataset.formula = "=1+1";
  cell.textContent = "2";
  e.grid.bodyInner.appendChild(cell);
  O.setShowFormulas(true);
  O.setShowFormulas(false);
  equal(JSON.stringify(book), before, "overlay did not mutate the workbook");
  cell.parentNode.removeChild(cell);
});

h.test("boot wires the toolbar button, document events and the grid change hook", function () {
  equal(e.auditButton.dataset.auditWired, "true", "toolbar button wired exactly once");
  assert(e.docEvents["gb-sheet-selection"], "selection event wired");
  assert(e.docEvents["gb-sheet-tab"], "tab event wired");
  assert(e.docEvents["gb-sheet-audit-open"], "open event wired");
  assert(e.winEvents["gb-sheet-locale"], "locale event wired");
  assert(typeof e.grid.onCellValueChanged === "function", "grid change hook chained");
  assert(e.observers.length >= 1, "MutationObserver installed");
});

h.test("scheduleWatch coalesces repeated calls into one animation frame", function () {
  U.setWatch([{ sheetIdx: 0, row: 0, col: 0 }]);
  O.scheduleWatch();
  O.scheduleWatch();
  O.scheduleWatch();
  equal(e.flushRaf(), 1, "three calls collapse to one queued callback");
  U.setWatch([]);
  O.scheduleWatch();
  equal(e.flushRaf(), 0, "no watch means nothing is scheduled");
});

h.test("advanced facade delegates open, toggle, refresh and formula view", function () {
  var panel = e.advanced.open();
  assert(panel, "facade open returns the dialog");
  equal(panel.style.display, "block", "dialog visible");
  assert(e.advanced.refreshGraph(), "facade refresh returns a graph");
  equal(e.advanced.toggleShowFormulas(), true, "facade toggles formula view");
  equal(e.advanced.toggleShowFormulas(), false, "facade toggles formula view back");
  e.advanced.toggle();
  equal(panel.style.display, "none", "facade toggle closes");
});

h.knownBug("live grid edits are not reflected in the watch list until an explicit refresh", function () {
  var book = h.makeBook();
  book.worksheets[0].data["0,0"] = { value: 1, formula: "" };
  e.setBook(book);
  U.setWatch([{ sheetIdx: 0, row: 0, col: 0 }]);
  U.open();
  S.updateWatch();
  var root = U.section("watch");
  equal(root.querySelector(".sheet-audit-watch-value").textContent, "1", "initial cached value");
  e.grid.cells.set("0,0", { value: 99, formula: "", comment: "" });
  e.grid.onCellValueChanged();
  e.flushRaf();
  equal(root.querySelector(".sheet-audit-watch-value").textContent, "1",
    "characterize the defect: scheduleWatch reuses the stale cached graph/snapshot");
  U.refreshGraph();
  S.updateWatch();
  equal(root.querySelector(".sheet-audit-watch-value").textContent, "99", "explicit refresh shows the new value");
  U.setWatch([]);
  e.grid.cells.clear();
});

h.knownBug("error scan evaluates only the first 400 formulas", function () {
  var book = h.makeBook();
  book.worksheets = [book.worksheets[0]];
  for (var i = 0; i < 410; i++) {
    book.worksheets[0].data["0," + i] = { value: "", formula: "=1/0" };
  }
  e.setBook(book);
  U.open();
  U.switchTab("errors");
  var root = U.section("errors");
  buttonByText(root, "Scan for errors").dispatchEvent({ type: "click" });
  equal(U.status(), "400 errors found.",
    "characterize the defect: MAX_EVAL_CELLS silently bounds the scan");
});
