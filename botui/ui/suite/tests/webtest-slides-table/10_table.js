/* Slides table & media — pure-helper invariant tests (#2487).
 * Every "runtime" check calls a genuine shipped function inside a vm sandbox.
 * IIFE-private helpers are reached through ONE appended export line, proved
 * source-identical below. Fake hosts are stubs only; no production logic is copied. */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;

var modelLoad = h.loadTableModel();
var STM = modelLoad.stm;
var editor = h.loadEditor(STM);
var internals = editor.internals;
var state = STM.tableState;

function mk(rows, cols, seed) {
  var grid = [];
  var widths = [];
  for (var c = 0; c < cols; c++) widths.push((c + 1) * 10);
  for (var r = 0; r < rows; r++) {
    var row = [];
    for (var i = 0; i < cols; i++) row.push({ text: seed ? seed(r, i) : "", colspan: 1, rowspan: 1 });
    grid.push(row);
  }
  return { cols: cols, grid: grid, options: { header: true, banded: false, borders: true,
    padding: 4, align: "left", style: "plain", widths: widths } };
}

function indicesOfRail(host) {
  return host.rail.children.map(function (t) { return t.getAttribute("data-index"); }).join(",");
}

// ── API surface + instrumentation honesty ────────────────────────────
test("shipped model module exports the pure helper surface", function () {
  ["cloneTable", "tableExpandAll", "tableSetCell", "tableInsertRow", "tableDeleteRow",
    "tableInsertCol", "tableDeleteCol", "tableMerge", "tableSplit", "tableDefaults",
    "tableWidths"].forEach(function (name) {
    assert(typeof STM[name] === "function", name + " missing");
  });
});

test("shipped editor and bulk internals are reachable without a browser", function () {
  ["finishEdit", "cellCoords", "currentCellModel", "selectCell", "applyMutation"].forEach(function (name) {
    assert(typeof internals[name] === "function", "editor " + name + " missing");
  });
  ["selected", "thumbFor", "label", "snapshot", "restoreOrder", "move", "duplicate",
    "remove"].forEach(function (name) {
    assert(typeof h.loadBulkOps(h.makeOutlineHost([0]).O).internals[name] === "function",
      "bulk " + name + " missing");
  });
});

test("instrumentation differs from both shipped sources by one export line only", function () {
  var bulk = h.loadBulkOps(h.makeOutlineHost([0]).O);
  assert(h.stripInstrumentation(editor.instrumented, h.EDITOR_MARKER, h.EDITOR_EXPORT) === editor.source,
    "editor source altered beyond the appended export");
  assert(h.stripInstrumentation(bulk.instrumented, h.BULK_MARKER, h.BULK_EXPORT) === bulk.source,
    "bulk source altered beyond the appended export");
});

// ── Table model: addressing, clone, defaults ─────────────────────────
test("cloneTable returns a deep-independent copy", function () {
  var original = mk(2, 2);
  var copy = STM.cloneTable(original);
  copy.grid[0][0].text = "mutated";
  copy.options.widths[0] = 999;
  assert(original.grid[0][0].text === "", "grid aliased through clone");
  assert(original.options.widths[0] === 10, "options aliased through clone");
});

test("tableDefaults is deterministic and independent per call", function () {
  var a = STM.tableDefaults();
  var b = STM.tableDefaults();
  assert(JSON.stringify(a) === JSON.stringify(b), "defaults differ between calls");
  a.widths.push(123);
  assert(b.widths.length === 0, "defaults aliased between calls");
});

test("A1 maps to row 0 / col 0 consistently and stays row-major", function () {
  var base = mk(2, 3);
  var next = STM.tableSetCell(base, 0, 0, "A1");
  assert(next.grid[0][0].text === "A1", "A1 did not land at row 0 / col 0");
  assert(next.grid[0][1].text === "" && next.grid[1][0].text === "",
    "A1 write leaked to a neighbouring coordinate");
  var coords = internals.cellCoords({
    getAttribute: function (key) { return key === "data-stm-r" ? "0" : "0"; }
  });
  assert(coords.row === 0 && coords.col === 0, "coordinate reader disagrees with A1");
});

test("tableSetCell is immutable and ignores out-of-range addresses", function () {
  var base = STM.tableSetCell(mk(2, 2), 1, 1, "keep");
  var outside = STM.tableSetCell(base, 9, 9, "lost");
  assert(base.grid[1][1].text === "keep", "source model mutated");
  assert(JSON.stringify(outside) === JSON.stringify(base), "out-of-range write changed the model");
});

// ── Table model: deterministic row/column mutations ──────────────────
test("insert row above/below updates row count without changing columns", function () {
  var base = STM.tableSetCell(mk(3, 4), 1, 0, "keep");
  var above = STM.tableInsertRow(base, 1, false);
  var below = STM.tableInsertRow(base, 1, true);
  assert(above.grid.length === 4 && above.cols === 4, "row-above dimensions wrong");
  assert(below.grid.length === 4 && below.cols === 4, "row-below dimensions wrong");
  assert(above.grid[2][0].text === "keep", "row-above did not shift the existing row down");
  assert(below.grid[1][0].text === "keep", "row-below should leave the existing row in place");
  assert(base.grid.length === 3, "source model mutated by insert");
});

test("delete row updates row count and refuses to empty the table", function () {
  var base = STM.tableSetCell(mk(3, 4), 2, 0, "keep");
  var next = STM.tableDeleteRow(base, 0);
  assert(next.grid.length === 2 && next.cols === 4, "row-delete dimensions wrong");
  assert(next.grid[1][0].text === "keep", "content lost on row delete");
  var last = STM.tableDeleteRow(mk(1, 3), 0);
  assert(last.grid.length === 1, "last-row guard failed");
});

test("insert column updates dimensions, widths and shifts content", function () {
  var base = STM.tableSetCell(mk(3, 4), 0, 1, "keep");
  var next = STM.tableInsertCol(base, 1, false);
  assert(next.cols === 5 && next.grid.every(function (row) { return row.length === 5; }),
    "column-insert dimensions wrong");
  assert(next.grid[0][2].text === "keep", "content not shifted to the new column");
  assert(next.options.widths.length === 5, "widths not extended with the column");
});

test("delete column updates dimensions, widths and guards the last column", function () {
  var base = STM.tableSetCell(mk(3, 4), 0, 3, "keep");
  var next = STM.tableDeleteCol(base, 1);
  assert(next.cols === 3 && next.grid.every(function (row) { return row.length === 3; }),
    "column-delete dimensions wrong");
  assert(next.grid[0][2].text === "keep", "content lost on column delete");
  assert(next.options.widths.length === 3, "widths not reduced with the column");
  var last = STM.tableDeleteCol(mk(2, 1), 0);
  assert(last.cols === 1, "last-column guard failed");
});

test("resize maps old cells to new coordinates without losing content", function () {
  var base = STM.tableSetCell(mk(3, 3), 1, 1, "anchor");
  var afterInsert = STM.tableInsertCol(STM.tableInsertRow(base, 1, false), 1, false);
  assert(afterInsert.grid[2][2].text === "anchor", "content lost across row+column resize");
  var afterDelete = STM.tableDeleteCol(STM.tableDeleteRow(afterInsert, 0), 0);
  assert(afterDelete.grid[1][1].text === "anchor", "content lost on inverse resize");
});

// ── Merge/split + export-facing serialisation helpers ────────────────
test("merge writes spans and null placeholders, split resets them", function () {
  var merged = STM.tableMerge(STM.tableSetCell(mk(3, 3), 0, 0, "head"), 0, 0, 1, 1);
  assert(merged.grid[0][0].colspan === 2 && merged.grid[0][0].rowspan === 2, "span not recorded");
  assert(merged.grid[0][1] === null && merged.grid[1][0] === null, "placeholder missing");
  var split = STM.tableSplit(merged, 0, 0);
  assert(split.grid[0][0].colspan === 1 && split.grid[0][0].rowspan === 1, "split did not reset span");
  assert(split.grid.every(function (row) { return row.every(Boolean); }), "split left a null placeholder");
});

test("tableExpandAll emits a valid, fully explicit table grid (export-facing)", function () {
  var merged = STM.tableMerge(STM.tableSetCell(mk(3, 3), 0, 0, "head"), 0, 0, 1, 1);
  var expanded = STM.tableExpandAll(merged);
  assert(expanded.grid.length === 3 && expanded.cols === 3, "expanded dimensions changed");
  assert(expanded.grid.every(function (row) { return row.every(Boolean); }), "expanded grid still has nulls");
  assert(expanded.grid.every(function (row) {
    return row.every(function (cell) { return cell.colspan === 1 && cell.rowspan === 1; });
  }), "expanded grid still has spans");
  assert(expanded.grid[0][0].text === "head", "anchor text lost during expansion");
  assert(JSON.stringify(STM.tableExpandAll(merged)) === JSON.stringify(expanded),
    "expansion is not deterministic");
});

test("tableWidths clamps invalid values and matches the column count", function () {
  var options = STM.tableWidths(mk(2, 3), STM.tableDefaults(), ["120", "bad", "-4"]);
  assert(options.widths.length === 3, "width array length wrong");
  assert(options.widths[0] === 120 && options.widths[1] === 0 && options.widths[2] === 0,
    "width clamping wrong: " + JSON.stringify(options.widths));
});

// ── Editor: commit, undo/redo, cell editing ──────────────────────────
var rendered = [];
var persisted = [];
var statuses = [];
var domModel = null;

STM.elementIdOf = function () { return "e1"; };
STM.tableById = function () { return { id: "e1" }; };
STM.tableRender = function (table, model, options) {
  rendered.push({ model: STM.cloneTable(model), options: STM.cloneTable(options) });
};
STM.persistTable = function (table, model, options) {
  persisted.push({ model: STM.cloneTable(model), options: STM.cloneTable(options) });
};
STM.status = function (key, fallback) { statuses.push(key); return fallback; };
STM.tableModelFromDom = function () { return STM.cloneTable(domModel); };
STM.syncTablePanel = function () {};

function resetEditor(model) {
  domModel = STM.cloneTable(model);
  state.model = STM.cloneTable(model);
  state.options = STM.cloneTable(model.options);
  state.activeId = "e1";
  state.cell = null;
  state.undo.length = 0;
  state.redo.length = 0;
  state.editing = null;
  rendered.length = 0;
  persisted.length = 0;
  statuses.length = 0;
}

function commitCell(model, text, key) {
  var before = { model: STM.cloneTable(model), options: STM.cloneTable(model.options) };
  STM.tableCommit({ id: "e1" }, before,
    { model: STM.tableSetCell(model, 0, 0, text), options: before.options },
    key || "slides-stm-table-cell-saved", "Cell updated");
}

function fakeCell(text, original) {
  return {
    textContent: text,
    isContentEditable: true,
    contentEditable: "true",
    removeEventListener: function () {},
    closest: function () { return null; }
  };
}

function editSession(cell, original, row, col) {
  state.editing = { cell: cell, original: original, row: row, col: col,
    onKey: function () {}, onBlur: function () {} };
}

test("tableCommit records before/after, renders, persists and clears redo", function () {
  var base = mk(2, 3);
  resetEditor(base);
  state.redo.push({ marker: "stale" });
  commitCell(base, "changed");
  assert(state.undo.length === 1 && state.redo.length === 0, "history not reset on new commit");
  assert(state.undo[0].id === "e1", "commit not scoped to the element id");
  assert(state.undo[0].before.grid[0][0].text === "", "before snapshot wrong");
  assert(state.undo[0].after.grid[0][0].text === "changed", "after snapshot wrong");
  assert(rendered.length === 1 && rendered[0].model.grid[0][0].text === "changed", "render not called");
  assert(persisted.length === 1, "persist not called");
  assert(statuses[0] === "slides-stm-table-cell-saved", "status key not reported");
});

test("undo/redo round-trip a committed edit", function () {
  var base = mk(2, 3);
  resetEditor(base);
  commitCell(base, "changed");
  STM.tableUndoLast();
  assert(state.undo.length === 0 && state.redo.length === 1, "undo did not move the action");
  assert(rendered[1].model.grid[0][0].text === "", "undo rendered the wrong model");
  assert(statuses[statuses.length - 1] === "slides-stm-undo-done", "undo status missing");
  STM.tableRedoLast();
  assert(state.undo.length === 1 && state.redo.length === 0, "redo did not restore history");
  assert(rendered[2].model.grid[0][0].text === "changed", "redo rendered the wrong model");
  assert(statuses[statuses.length - 1] === "slides-stm-redo-done", "redo status missing");
});

test("empty undo/redo report their guarded status without rendering", function () {
  resetEditor(mk(2, 3));
  STM.tableUndoLast();
  STM.tableRedoLast();
  assert(rendered.length === 0, "empty history triggered a render");
  assert(statuses.join(",") === "slides-stm-undo-empty,slides-stm-redo-empty",
    "empty-history status wrong: " + statuses.join(","));
});

test("undo history is bounded to 100 steps", function () {
  resetEditor(mk(2, 3));
  for (var i = 0; i < 101; i++) commitCell(mk(2, 3), "v" + i, "k");
  assert(state.undo.length === 100, "undo cap wrong: " + state.undo.length);
});

test("cellCoords reads row-major data attributes", function () {
  var coords = internals.cellCoords({
    getAttribute: function (key) { return key === "data-stm-r" ? "4" : "2"; }
  });
  assert(coords.row === 4 && coords.col === 2, "cellCoords returned " + JSON.stringify(coords));
});

test("finishEdit commits typed text through the same table commit path", function () {
  var base = STM.tableSetCell(mk(2, 3), 0, 0, "old");
  resetEditor(base);
  var cell = fakeCell("typed", "old");
  editSession(cell, "old", 0, 0);
  internals.finishEdit(true);
  assert(state.editing === null, "edit state not cleared");
  assert(cell.contentEditable === "false", "contentEditable not disabled");
  assert(cell.textContent === "typed", "cell text not committed to the DOM");
  assert(rendered.length === 1, "cell edit did not commit");
  assert(statuses[0] === "slides-stm-table-cell-saved", "wrong cell-edit status key");
});

test("finishEdit restores the original on Escape and skips no-op commits", function () {
  var base = STM.tableSetCell(mk(2, 3), 0, 0, "old");
  resetEditor(base);
  var cancelled = fakeCell("discarded", "old");
  editSession(cancelled, "old", 0, 0);
  internals.finishEdit(false);
  assert(cancelled.textContent === "old", "Escape did not restore the original text");
  assert(rendered.length === 0, "Escape produced an undo step");

  resetEditor(mk(2, 3));
  var unchanged = fakeCell("", "");
  editSession(unchanged, "", 0, 0);
  internals.finishEdit(true);
  assert(rendered.length === 0, "unchanged cell produced an undo step");
});

// ── Bulk operations: pure-ish helpers with a fake outline host ───────
test("selected reads checked indices from the fake host", function () {
  var fixture = h.makeOutlineHost([0, 1, 2]);
  var bulk = h.loadBulkOps(fixture.O);
  fixture.host.check([0, 2]);
  assert(JSON.stringify(bulk.internals.selected(fixture.host)) === "[0,2]", "selected indices wrong");
});

test("thumbFor resolves a slide thumb and returns null for an unknown index", function () {
  var fixture = h.makeOutlineHost([0, 1, 2]);
  var bulk = h.loadBulkOps(fixture.O);
  assert(bulk.internals.thumbFor(1) === fixture.thumbs[1], "wrong thumb resolved");
  assert(bulk.internals.thumbFor(99) === null, "unknown index should yield null");
});

test("label([0,1,2]) is deterministic and label([]) is safe", function () {
  var fixture = h.makeOutlineHost([0, 1, 2]);
  var bulk = h.loadBulkOps(fixture.O);
  assert(JSON.stringify(bulk.internals.label([0, 1, 2])) === '["Slide 1","Slide 2","Slide 3"]',
    "label output not human-deterministic");
  assert(JSON.stringify(bulk.internals.label([])) === "[]", "empty label should be safe");
});

test("snapshot/restoreOrder round-trip an ordered list", function () {
  var fixture = h.makeOutlineHost([0, 1, 2]);
  var bulk = h.loadBulkOps(fixture.O);
  var original = bulk.internals.snapshot();
  assert(original.length === 3 && original[0] === fixture.thumbs[0], "snapshot wrong");
  bulk.internals.restoreOrder(original.slice().reverse());
  assert(indicesOfRail(fixture) === "2,1,0", "restoreOrder did not reorder the rail");
  assert(JSON.stringify(fixture.store["bulk-order"]) === '["2","1","0"]', "order not persisted");
  bulk.internals.restoreOrder([]);
  assert(indicesOfRail(fixture) === "2,1,0", "empty restoreOrder should be a no-op");
});

test("move shifts a selected index and clamps at the edges", function () {
  var fixture = h.makeOutlineHost([0, 1, 2]);
  var bulk = h.loadBulkOps(fixture.O);
  fixture.host.check([1]);
  bulk.internals.move(fixture.host, 1);
  assert(indicesOfRail(fixture) === "0,2,1", "move down did not shift the selection");
  assert(fixture.undos.length === 1, "move did not push undo");
  fixture.undos[0].undo();
  assert(indicesOfRail(fixture) === "0,1,2", "move undo did not restore order");

  var edge = h.makeOutlineHost([0, 1, 2]);
  var edgeBulk = h.loadBulkOps(edge.O);
  edge.host.check([0]);
  edgeBulk.internals.move(edge.host, -1);
  assert(indicesOfRail(edge) === "0,1,2", "first slide moved past the top edge");
  edge.host.check([2]);
  edgeBulk.internals.move(edge.host, 1);
  assert(indicesOfRail(edge) === "0,1,2", "last slide moved past the bottom edge");
});

/* Regression: bulk move redo re-applies the post-move order after undo.
 * 24_bulk_ops.js used to define redo as restoreOrder(snapshot()), which
 * re-read the already-restored DOM and made redo a no-op. The fix captures
 * `after = snapshot()` once after the move and reuses it here. */
test("bulk move redo re-applies the post-move order after undo", function () {
  var fixture = h.makeOutlineHost([0, 1, 2]);
  var bulk = h.loadBulkOps(fixture.O);
  fixture.host.check([1]);
  bulk.internals.move(fixture.host, 1);
  assert(indicesOfRail(fixture) === "0,2,1", "precondition: move must apply");
  fixture.undos[0].undo();
  assert(indicesOfRail(fixture) === "0,1,2", "precondition: undo must restore");
  fixture.undos[0].redo();
  assert(indicesOfRail(fixture) === "0,2,1",
    "redo must restore the captured post-move order, not re-snapshot the restored DOM");
});

test("duplicate inserts the clone immediately after the selection", function () {
  var fixture = h.makeOutlineHost([0, 1]);
  var bulk = h.loadBulkOps(fixture.O);
  fixture.host.check([0]);
  bulk.internals.duplicate(fixture.host);
  assert(fixture.rail.children.length === 3, "duplicate did not add one item");
  var clone = fixture.rail.children[1];
  assert(clone.getAttribute("data-local-duplicate") === "true", "clone not marked local");
  assert(clone.getAttribute("data-slide") === "0", "clone lost its source slide index");
  assert(/^local-/.test(clone.getAttribute("data-index")), "clone has no local index");
  assert(fixture.confirms.length === 1, "duplicate skipped the confirmation");
  fixture.undos[0].undo();
  assert(fixture.rail.children.length === 2, "duplicate undo did not remove the clone");
  fixture.undos[0].redo();
  assert(fixture.rail.children.length === 3, "duplicate redo did not restore the clone");
});

test("remove deletes the selection, supports undo and honours cancellation", function () {
  var fixture = h.makeOutlineHost([0, 1, 2]);
  var bulk = h.loadBulkOps(fixture.O);
  fixture.host.check([1]);
  bulk.internals.remove(fixture.host);
  assert(indicesOfRail(fixture) === "0,2", "remove did not delete the selection");
  assert(JSON.stringify(fixture.store["bulk-removed"]) === "[1]", "removal not persisted");
  fixture.undos[0].undo();
  assert(indicesOfRail(fixture) === "0,1,2", "remove undo did not restore the item");
  fixture.undos[0].redo();
  assert(indicesOfRail(fixture) === "0,2", "remove redo did not re-delete the item");

  var cancelled = h.makeOutlineHost([0, 1]);
  var cancelledBulk = h.loadBulkOps(cancelled.O);
  cancelled.O.setConfirmResult(false);
  cancelled.host.check([0]);
  cancelledBulk.internals.remove(cancelled.host);
  assert(indicesOfRail(cancelled) === "0,1", "cancelled removal still changed the rail");
});

test("remove of the last item shows the explicit empty-state guard", function () {
  var fixture = h.makeOutlineHost([0]);
  var bulk = h.loadBulkOps(fixture.O);
  fixture.host.check([0]);
  bulk.internals.remove(fixture.host);
  assert(fixture.rail.children.length === 0, "rail should be empty after removing the last slide");
  assert(fixture.host.children.length === 1, "empty list has no explicit guard node");
  assert(fixture.host.children[0].getAttribute("data-i18n") === "slides-outline-bulk-empty",
    "empty-state guard is missing its catalog key");
});

// ── STATIC source contract (not presented as runtime evidence) ───────
test("STATIC: destructive table deletes stay behind the accessible confirm dialog", function () {
  var source = h.readFile(h.EDITOR_REL);
  var dialogs = source.split("STM.confirmDialog(").length - 1;
  assert(dialogs >= 2, "row/column delete guards are missing from the shipped editor source");
  assert(source.indexOf("slides-stm-table-cell-saved") !== -1,
    "cell-edit commit key missing from the shipped editor source");
});
