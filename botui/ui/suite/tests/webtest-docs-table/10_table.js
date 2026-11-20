/**
 * #2482 offline verification of the DOM-free docs table helpers that ship in
 * docs/modules/33_tables_model.js and 33b_tables_commit.js.
 *
 * Every RUNTIME case loads the REAL modules into a vm sandbox with fake
 * table/cell objects (rows, spans, attributes, outerHTML). STATIC cases are
 * used only where a source-order contract cannot be observed from outside.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var readFile = h.readFile;

var SOURCE_MODEL = "docs/modules/33_tables_model.js";
var SOURCE_COMMIT = "docs/modules/33b_tables_commit.js";

/* ---- tiny fake-table builders ---------------------------------------- */

function cell(text, opts) {
  var o = opts || {};
  var c = h.element(o.tag || "td", o.attrs || []);
  if (o.scope) c.setAttribute("scope", o.scope);
  if (o.rowspan) c.setAttribute("rowspan", String(o.rowspan));
  if (o.colspan) c.setAttribute("colspan", String(o.colspan));
  if (o.anchor) {
    var span = h.element("span", [["class", "gb-t"]]);
    span.innerHTML = text;
    c.appendChild(span);
    c._anchor = span;
  } else {
    c.innerHTML = text;
  }
  return c;
}

function row(cells, attrs) {
  var tr = h.element("tr", attrs || []);
  cells.forEach(function (c) { tr.appendChild(c); });
  return tr;
}

function section(tag, rows, attrs) {
  var sec = h.element(tag, attrs || []);
  rows.forEach(function (r) { sec.appendChild(r); });
  return sec;
}

function table(sections, attrs) {
  var t = h.element("table", attrs || []);
  sections.forEach(function (s) { t.appendChild(s); });
  return t;
}

function grid(rows, cols) {
  var body = [];
  for (var r = 0; r < rows; r++) {
    var cells = [];
    for (var c = 0; c < cols; c++) cells.push(cell("r" + r + "c" + c));
    body.push(row(cells));
  }
  return table([section("tbody", body)]);
}

function headerTable() {
  return table([
    section("thead", [row([
      cell("H1", { tag: "th", scope: "col" }),
      cell("H2", { tag: "th", scope: "col" }),
    ])]),
    section("tbody", [row([cell("A"), cell("B")])]),
  ]);
}

function mergedTable() {
  return table([
    section("thead", [row([cell("H", { tag: "th", scope: "col", colspan: 2, anchor: true })])]),
    section("tbody", [
      row([cell("M", { rowspan: 2 }), cell("B")]),
      row([cell("D")]),
    ]),
  ]);
}

/* ---- load / export surface ------------------------------------------- */

test("RUNTIME #2482: both real table modules load with stubbed DOM and export their API", function () {
  var env = h.tableEnvironment();
  ["activeTable", "cellAt", "cellFromSelection", "selectedCells", "selectionRect",
    "focusCell", "replaceTable", "buildModel", "layout", "modelToTable",
    "ownCells", "ownCell", "closestEl", "cellText"].forEach(function (name) {
    assert(typeof env.NS[name] === "function", name + " must be exported onto DocsTables");
  });
});

/* ---- grid model + cellAt ---------------------------------------------- */

test("RUNTIME #2482: buildModel/layout produce a rectangular placement grid", function () {
  var env = h.tableEnvironment();
  var t = headerTable();
  env.attach(t);
  var model = env.NS.buildModel(t);
  assert(model.rows === 2 && model.cols === 2, "expected a 2x2 model");
  assert(model.rowSections[0] === "thead" && model.rowSections[1] === "tbody", "section tags must be kept in order");
  var occ = env.NS.layout(model);
  assert(occ[0][0].ref.html === "H1" && occ[1][1].ref.html === "B", "layout must map refs to coordinates");
  assert(occ[0][0].start === true && occ[0][1].ref.html === "H2",
    "the two header cells occupy (0,0) and (0,1)");
});

test("RUNTIME #2482: cellAt returns the exact cell for every in-range coordinate", function () {
  var env = h.tableEnvironment();
  var t = grid(2, 3);
  env.attach(t);
  var expected = env.NS.ownCells(t);
  for (var r = 0; r < 2; r++) {
    for (var c = 0; c < 3; c++) {
      assert(env.NS.cellAt(t, r, c) === expected[r * 3 + c], "cellAt(" + r + "," + c + ") must be the matching cell");
    }
  }
});

test("RUNTIME #2482: cellAt is safe for null tables and out-of-range coordinates", function () {
  var env = h.tableEnvironment();
  var t = grid(2, 2);
  env.attach(t);
  assert(env.NS.cellAt(null, 0, 0) === null, "null table must yield null");
  assert(env.NS.cellAt(t, -1, 0) === null, "negative row must yield null");
  assert(env.NS.cellAt(t, 0, -1) === null, "negative column must yield null");
  assert(env.NS.cellAt(t, 2, 0) === null, "row past the end must yield null");
  assert(env.NS.cellAt(t, 0, 2) === null, "column past the end must yield null");
});

/* ---- merges ------------------------------------------------------------ */

test("RUNTIME #2482: rowSpan/colSpan cells are returned for every covered coordinate", function () {
  var env = h.tableEnvironment();
  var t = mergedTable();
  env.attach(t);
  var head = env.NS.ownCells(t)[0];
  var merged = env.NS.ownCells(t)[1];
  var model = env.NS.buildModel(t);
  assert(model.rows === 3 && model.cols === 2, "merges must not change the grid dimensions");
  assert(model.placements.length === 4, "four real cells must back six placements");
  assert(env.NS.cellAt(t, 0, 1) === head, "colspan-covered (0,1) must resolve to the header cell");
  assert(env.NS.cellAt(t, 2, 0) === merged, "rowspan-covered (2,0) must resolve to the merged cell");
  assert(env.NS.cellAt(t, 1, 1) === env.NS.ownCells(t)[2], "unmerged neighbour must stay addressable");
  assert(env.NS.cellAt(t, 2, 1) === env.NS.ownCells(t)[3], "last row neighbour must stay addressable");
});

/* ---- selection scoping ------------------------------------------------- */

test("RUNTIME #2482: activeTable resolves the innermost table from an element anchor", function () {
  var env = h.tableEnvironment();
  var innerCell = cell("inner", { anchor: true });
  var inner = table([section("tbody", [row([innerCell])])]);
  var host = cell("");
  host.appendChild(inner);
  var outer = table([section("tbody", [row([host])])]);
  env.attach(outer);
  env.selection.anchorNode = innerCell._anchor;
  assert(env.NS.activeTable() === inner, "nested selection must scope to the inner table");
  assert(env.NS.ownCells(outer).length === 1 && env.NS.ownCells(outer)[0] === host,
    "outer ownCells must exclude nested-table cells");
  assert(env.NS.ownCells(inner).length === 1 && env.NS.ownCells(inner)[0] === innerCell,
    "inner ownCells must contain only its own cell");
});

test("RUNTIME #2482: activeTable returns null for an anchor outside the article", function () {
  var env = h.tableEnvironment();
  var t = grid(1, 1);
  env.attach(t);
  env.selection.anchorNode = h.element("div");
  assert(env.NS.activeTable() === null, "non-cell anchor must not select a table");
  env.selection.anchorNode = env.article;
  assert(env.NS.activeTable() === null, "article itself must not resolve to a table");
});

test("RUNTIME #2482: cellFromSelection maps an element anchor to its placement origin", function () {
  var env = h.tableEnvironment();
  var t = mergedTable();
  env.attach(t);
  var head = env.NS.ownCells(t)[0];
  env.selection.anchorNode = head._anchor;
  var found = env.NS.cellFromSelection();
  assert(found && found.table === t && found.cell === head, "selection must report its table and cell");
  assert(found.row === 0 && found.col === 0, "colspan origin must be (0,0), not a covered coordinate");
  assert(found.ref && found.ref.colSpan === 2, "placement ref must keep the colSpan");
});

test("RUNTIME #2482: selectedCells dedupes cells seen by both range and anchor/focus", function () {
  var env = h.tableEnvironment();
  var t = grid(2, 2);
  env.attach(t);
  var first = env.NS.ownCells(t)[0];
  var anchor = h.element("span");
  first.appendChild(anchor);
  env.selection.setRanges([h.makeRange([first])]);
  env.selection.anchorNode = anchor;
  env.selection.focusNode = first;
  var found = env.NS.selectedCells(t);
  assert(found.length === 1 && found[0] === first, "three signals for one cell must yield one cell");
});

test("RUNTIME #2482: empty selection yields no cells and a null rectangle", function () {
  var env = h.tableEnvironment();
  var t = grid(2, 2);
  env.attach(t);
  env.selection.setRanges([]);
  env.selection.anchorNode = null;
  env.selection.focusNode = null;
  assert(env.NS.selectedCells(t).length === 0, "empty selection must return an empty array");
  assert(env.NS.selectionRect(t) === null, "empty selection must return a null rectangle");
  assert(env.NS.cellFromSelection() === null, "empty selection must have no placement");
});

/* ---- selectionRect ----------------------------------------------------- */

test("RUNTIME #2482: selectionRect computes min/max over a disjoint selection", function () {
  var env = h.tableEnvironment();
  var t = grid(3, 4);
  env.attach(t);
  var cells = env.NS.ownCells(t);
  env.selection.setRanges([h.makeRange([cells[1], cells[11]])]);
  var rect = env.NS.selectionRect(t);
  assert(rect, "disjoint selection must produce a rectangle");
  assert(rect.minR === 0 && rect.minC === 1 && rect.maxR === 2 && rect.maxC === 3,
    "rectangle must bound (0,1) and (2,3) exactly");
});

test("RUNTIME #2482: selectionRect expands to merged cells' covered positions", function () {
  var env = h.tableEnvironment();
  var t = table([
    section("thead", [row([
      cell("H1", { tag: "th", scope: "col" }),
      cell("H2", { tag: "th", scope: "col" }),
      cell("H3", { tag: "th", scope: "col" }),
    ])]),
    section("tbody", [
      row([cell("M", { colspan: 3 })]),
      row([cell("A"), cell("B"), cell("C")]),
    ]),
  ]);
  env.attach(t);
  var cells = env.NS.ownCells(t);
  env.selection.setRanges([h.makeRange([cells[0], cells[3]])]);
  var rect = env.NS.selectionRect(t);
  assert(rect && rect.maxC === 2,
    "a colspan=3 cell at column 0 must extend the rectangle to column 2");
  assert(rect.minR === 0 && rect.maxR === 1, "row bounds must cover header and merged row");
});

/* ---- focus ------------------------------------------------------------- */

test("RUNTIME #2482: focusCell places a collapsed caret in the requested cell", function () {
  var env = h.tableEnvironment();
  var t = grid(2, 2);
  env.attach(t);
  var before = h.makeRange([env.NS.ownCells(t)[0]]);
  env.selection.setRanges([before]);
  env.NS.focusCell(t, 1, 1);
  assert(env.selection.rangeCount === 1, "focusCell must leave exactly one range");
  assert(env.selection._ranges[0]._contents === env.NS.cellAt(t, 1, 1), "caret must be in (1,1)");
  assert(env.selection._ranges[0]._collapsed === true, "caret range must be collapsed");
});

test("RUNTIME #2482: focusCell is a no-op for out-of-range coordinates", function () {
  var env = h.tableEnvironment();
  var t = grid(1, 1);
  env.attach(t);
  var original = h.makeRange([env.NS.ownCells(t)[0]]);
  env.selection.setRanges([original]);
  env.NS.focusCell(t, 5, 5);
  assert(env.selection.rangeCount === 1 && env.selection._ranges[0] === original,
    "an invalid focus must leave the selection untouched");
});

/* ---- serialization and commit ------------------------------------------ */

test("RUNTIME #2482: modelToTable preserves head semantics, attributes and spans", function () {
  var env = h.tableEnvironment();
  var t = table([
    section("thead", [row([cell("H", { tag: "th", scope: "col", colspan: 2 })])], [["data-docs-thead", "1"]]),
    section("tbody", [row([cell("A"), cell("B")])]),
  ], [["data-docs-tbl-repeat", "1"], ["class", "docs-tbl-style-banded"]]);
  env.attach(t);
  var fresh = env.NS.modelToTable(env.NS.buildModel(t));
  assert(fresh.getAttribute("data-docs-tbl-repeat") === "1", "table repeat attribute must survive serialization");
  assert(fresh.childNodes[0].tagName === "THEAD", "thead must be serialized first");
  assert(fresh.childNodes[0].getAttribute("data-docs-thead") === "1", "thead attributes must survive");
  var headCell = fresh.childNodes[0].childNodes[0].childNodes[0];
  assert(headCell.tagName === "TH" && headCell.getAttribute("scope") === "col", "header scope must survive");
  assert(headCell.getAttribute("colspan") === "2", "colspan must survive");
});

test("RUNTIME #2482: outerHTML fallback swaps the table in the DOM exactly once", function () {
  var env = h.tableEnvironment({ execCommand: false });
  var t = grid(2, 2);
  t.setAttribute("data-docs-tables-id", "tbl-2482");
  env.attach(t);
  var fresh = env.NS.modelToTable(env.NS.buildModel(t));
  var result = env.NS.replaceTable(t, fresh);
  assert(env.events.writes.length === 1, "exactly one outerHTML write must occur");
  assert(env.events.formats.length === 0, "fallback path must not call insertHTML");
  assert(env.article.childNodes.length === 1 && env.article.childNodes[0] === result,
    "the article must contain exactly the replacement table");
  assert(result !== t, "the old table object must be gone");
  assert(result.getAttribute("data-docs-tables-id") === "tbl-2482", "table id must be carried over");
  assert(env.events.saves === 1, "one commit must trigger one save");
  assert(env.events.refreshes.length === 1 && env.events.refreshes[0] === result,
    "table UI must refresh once with the replacement");
});

test("RUNTIME #2482: header-repeat attribute and thead survive a replace", function () {
  var env = h.tableEnvironment({ execCommand: false });
  var t = table([
    section("thead", [row([cell("H", { tag: "th", scope: "col" })])], [["data-docs-thead", "1"]]),
    section("tbody", [row([cell("A")])]),
  ], [["data-docs-tbl-repeat", "1"]]);
  t.setAttribute("data-docs-tables-id", "tbl-repeat");
  env.attach(t);
  var result = env.NS.replaceTable(t, env.NS.modelToTable(env.NS.buildModel(t)));
  var html = result.outerHTML;
  assert(html.indexOf('data-docs-tbl-repeat="1"') >= 0, "repeat attribute must be in the committed markup");
  assert(html.indexOf("<thead") >= 0 && html.indexOf('scope="col"') >= 0, "repeatable header markup must be intact");
  var reparsed = env.parse(html);
  var model = env.NS.buildModel(reparsed);
  assert(model.sectionAttrs.thead.some(function (pair) { return pair[0] === "data-docs-thead" && pair[1] === "1"; }),
    "reparsed thead attribute must still be present");
  assert(model.rowSections[0] === "thead", "reparsed first row must still be a header");
});

test("RUNTIME #2482: replaceTable restores focus in the new table when asked", function () {
  var env = h.tableEnvironment({ execCommand: false });
  var t = grid(2, 2);
  t.setAttribute("data-docs-tables-id", "tbl-focus");
  env.attach(t);
  var result = env.NS.replaceTable(t, env.NS.modelToTable(env.NS.buildModel(t)), { row: 1, col: 1 });
  assert(result, "replace must succeed");
  assert(env.selection.rangeCount === 1, "focus request must leave exactly one range");
  assert(env.selection._ranges[0]._contents === env.NS.cellAt(result, 1, 1),
    "restored caret must be in the new table at (1,1)");
});

test("RUNTIME #2482: replaceTable refuses a detached table without writing", function () {
  var env = h.tableEnvironment({ execCommand: false });
  var t = grid(1, 1);
  var fresh = env.NS.modelToTable(env.NS.buildModel(t));
  assert(env.NS.replaceTable(t, fresh) === null, "detached table must be rejected");
  assert(env.events.writes.length === 0 && env.events.saves === 0, "rejection must not mutate or save");
});

test("RUNTIME #2482: execCommand path commits one insertHTML and no fallback write", function () {
  var env = h.tableEnvironment();
  var t = grid(2, 2);
  t.setAttribute("data-docs-tables-id", "tbl-exec");
  env.attach(t);
  var result = env.NS.replaceTable(t, env.NS.modelToTable(env.NS.buildModel(t)));
  assert(env.events.formats.length === 1, "exactly one gbFormat call must occur");
  assert(env.events.formats[0].command === "insertHTML", "the native command must be insertHTML");
  assert(env.events.writes.length === 0, "the native path must not use the outerHTML fallback");
  assert(env.article.childNodes.length === 1 && env.article.childNodes[0] === result,
    "the replacement table must be installed in the article");
  assert(env.events.saves === 1, "the native path must still save once");
});

test("RUNTIME #2482: cellText normalizes non-breaking spaces", function () {
  var env = h.tableEnvironment();
  var c = cell("a\u00a0b");
  assert(env.NS.cellText(c) === "a b", "nbsp must become a regular space");
});

/* ---- source contracts -------------------------------------------------- */

test("STATIC #2482: replaceTable has one write boundary and one save", function () {
  var src = readFile(SOURCE_COMMIT);
  var body = src.slice(src.indexOf("function replaceTable"), src.indexOf("NS.replaceTable = replaceTable"));
  assert(body.indexOf('gbFormat("insertHTML", html)') >= 0, "native single-step write must be present");
  assert(body.indexOf("table.outerHTML = html") >= 0, "fallback single-step write must be present");
  assert((body.match(/NS\.save\(\)/g) || []).length === 1, "there must be exactly one save call");
  assert(body.indexOf("NS.save()") > body.indexOf("table.outerHTML = html"),
    "save must follow the commit");
});

test("STATIC #2482: selectionRect derives bounds from the occupied layout", function () {
  var src = readFile(SOURCE_COMMIT);
  var body = src.slice(src.indexOf("function selectionRect"), src.indexOf("NS.selectionRect = selectionRect"));
  assert(body.indexOf("NS.layout(model)") >= 0, "rectangle must be layout-driven");
  assert(body.indexOf("minR = Math.min") >= 0 && body.indexOf("maxC = Math.max") >= 0,
    "rectangle must track both row and column bounds");
});

test("STATIC #2482: modelToTable applies row and section attributes before append", function () {
  var src = readFile(SOURCE_MODEL);
  var body = src.slice(src.indexOf("function modelToTable"), src.indexOf("NS.modelToTable = modelToTable"));
  var applySection = body.indexOf("applyAttrs(section, model.sectionAttrs[tag])");
  var applyRow = body.indexOf("applyAttrs(tr, model.rowAttrs[r] || [])");
  var append = body.indexOf("table.appendChild(section)");
  assert(applySection >= 0 && applyRow >= 0 && append > applyRow,
    "attributes must be applied before the section is appended");
});
