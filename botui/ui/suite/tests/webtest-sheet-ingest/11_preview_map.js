/**
 * Ingest preview and column mapping (#2494). DOM rendering, selection,
 * confirmation gating and mapping persistence are executed against the real
 * shipped modules using the harness DOM double.
 */
"use strict";

var h = require("./00_harness");
var g = h.loadModules();
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var deep = h.deep;
var C = g.core;
var ctx = g.boot.ctx;

function gridBody() {
  var body = g.dom.makeNode("div");
  g.preview.render(body, ctx);
  return body;
}

function buttonAt(table, row, col) {
  return table.children[1].children[row].children[col + 1].children[0];
}

test("preview render builds a grid, structure list and provenance panel", function () {
  g.reset();
  g.seed();
  var body = gridBody();
  equal(body.children.length, 2);
  var section = body.children[0];
  equal(section.className, "ss-ingest-section");
  equal(section.children.length, 4);
  var table = section.children[2].children[0];
  equal(table.children[0].children[0].children.length, 6);
  equal(table.children[1].children.length, 10);
  equal(section.children[3].children.length, 4);
  var aside = body.children[1].children[0].children[0];
  equal(aside.className, "ss-ingest-provenance");
  equal(aside.getAttribute("aria-label"), "Cell provenance");
});

test("cell nodes carry addresses, provenance labels and review classes", function () {
  g.reset();
  g.seed();
  var table = gridBody().children[0].children[2].children[0];
  var low = buttonAt(table, 2, 2);
  equal(low.tagName, "BUTTON");
  equal(low.dataset.row, "2");
  equal(low.dataset.col, "2");
  equal(low._text, "120.50");
  assert(low.getAttribute("aria-label").indexOf("C3 - Page 1 - Low confidence") === 0);
  equal(low.classList.contains("ss-ingest-cell-low"), true);
  var blank = buttonAt(table, 4, 2);
  equal(blank._text, "");
  assert(blank.getAttribute("aria-label").indexOf("Unreadable - left empty") > 0);
  equal(blank.classList.contains("ss-ingest-cell-empty"), true);
  equal(buttonAt(table, 1, 0).classList.contains("ss-ingest-cell-low"), false);
});

test("structure rows and merged spans are tagged in the rendered grid", function () {
  g.reset();
  g.seed();
  var rows = gridBody().children[0].children[2].children[0].children[1].children;
  equal(rows[1].classList.contains("ss-ingest-row-data"), true);
  equal(rows[6].classList.contains("ss-ingest-row-subtotal"), true);
  equal(rows[7].classList.contains("ss-ingest-row-repeated-header"), true);
  equal(rows[6].children[2].children[0].classList.contains("ss-ingest-cell-structured"), true);
});

test("selecting a cell exposes page, region, confidence and original value", function () {
  g.reset();
  g.seed();
  var holder = g.dom.makeNode("div");
  g.preview.select("stmt", 2, 2);
  deep(g.preview.selected(), { fileId: "stmt", row: 2, col: 2 });
  g.preview.renderProvenance(holder, ctx);
  var panel = holder.children[0];
  var lines = panel.children.filter(function (node) { return node.className === "ss-ingest-detail"; });
  equal(lines.length, 5);
  equal(lines[0].children[1]._text, "C3");
  equal(lines[1].children[1]._text, "1");
  equal(lines[2].children[1]._text, "x 410, y 110, 90 x 18");
  equal(lines[3].children[1]._text, "0.62");
  equal(lines[4].children[1]._text, "120.50");
  assert(panel.children.some(function (node) { return node.className === "ss-ingest-warning"; }));
});

test("a low-confidence value stays blocked until its confirm button is used", function () {
  g.reset();
  g.seed();
  g.boot.open(null);
  equal(C.formulaBlocked(C.state(), "stmt", 2, 2), true);
  var holder = g.dom.makeNode("div");
  g.preview.select("stmt", 2, 2);
  g.preview.renderProvenance(holder, ctx);
  var buttons = holder.children[0].children.filter(function (node) { return node.className === "ss-ingest-btn"; });
  equal(buttons.length, 1);
  buttons[0].dispatch("click");
  equal(C.isConfirmed(C.state(), "stmt", 2, 2), true);
  equal(C.formulaBlocked(C.state(), "stmt", 2, 2), false);
  g.preview.renderProvenance(holder, ctx);
  var confirmed = holder.children[0].children.filter(function (node) { return node.className === "ss-ingest-badge-ok"; });
  equal(confirmed.length, 1);
  equal(g.dom.document.querySelector(".ss-ingest-status") !== null, true);
});

test("an unreadable cell renders empty with no invented value", function () {
  g.reset();
  g.seed();
  var table = gridBody().children[0].children[2].children[0];
  equal(buttonAt(table, 4, 2)._text, "");
  equal(C.effectiveValue(C.state(), "stmt", 4, 2), "");
  var holder = g.dom.makeNode("div");
  g.preview.select("stmt", 4, 2);
  g.preview.renderProvenance(holder, ctx);
  var lines = holder.children[0].children.filter(function (node) { return node.className === "ss-ingest-detail"; });
  assert(lines[4].children[1]._text.indexOf("No value read") === 0);
});

test("clearing the selection restores the provenance hint", function () {
  g.reset();
  g.seed();
  var holder = g.dom.makeNode("div");
  g.preview.select("stmt", 2, 2);
  g.preview.renderProvenance(holder, ctx);
  equal(holder.children[0].children.length > 5, true);
  g.preview.clear();
  equal(g.preview.selected(), null);
  g.preview.renderProvenance(holder, ctx);
  equal(holder.children[0].children.length, 2);
});

test("map render offers every target type with the detected default selected", function () {
  g.reset();
  g.seed();
  var body = g.dom.makeNode("div");
  g.map.render(body, ctx);
  var selects = body.querySelectorAll("select");
  equal(selects.length, 5);
  deep(selects.map(function (node) { return node.children.length; }), [4, 4, 4, 4, 4]);
  var chosen = selects.map(function (node) {
    return node.children.filter(function (option) { return option.selected; }).map(function (option) { return option.value; })[0];
  });
  deep(chosen, ["date", "text", "currency", "currency", "currency"]);
});

test("saving a mapping persists the user choices and updates status", function () {
  g.reset();
  g.seed();
  var body = g.dom.makeNode("div");
  g.map.render(body, ctx);
  var selects = body.querySelectorAll("select");
  ["text", "date", "number", "currency", "text"].forEach(function (value, index) {
    selects[index].value = value;
  });
  var save = body.querySelectorAll("button")[0];
  save.dispatch("click");
  deep(C.state().mappings.stmt, ["text", "date", "number", "currency", "text"]);
  deep(C.columnTypes(C.state(), "stmt"), ["text", "date", "number", "currency", "text"]);
});

test("profile and balance sections report detection and blocked rows", function () {
  g.reset();
  g.seed();
  var body = g.dom.makeNode("div");
  g.map.render(body, ctx);
  var sections = body.children[0].children;
  equal(sections.length, 3);
  assert(sections[1].children[1]._text.indexOf("Bank statement detected") === 0);
  equal(sections[1].children[2]._text, "Date A, description B, debit C, credit D, balance E");
  assert(sections[2].children[1]._text.indexOf("3 row(s) blocked") === 0);
  equal(sections[2].children[2]._text.indexOf("Row 3: blocked") === 0, true);
  equal(sections[2].children[3]._text.indexOf("Row 5: blocked") === 0, true);
  assert(sections[2].children.some(function (node) { return node.className === "ss-ingest-mismatch"; }) === false);
});

test("typeLabel maps all four shipped column types", function () {
  equal(g.map.typeLabel("text"), "Text");
  equal(g.map.typeLabel("number"), "Number");
  equal(g.map.typeLabel("date"), "Date");
  equal(g.map.typeLabel("currency"), "Currency");
  equal(g.map.typeLabel("unknown"), "Text");
});
