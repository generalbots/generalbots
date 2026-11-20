/**
 * Ingest review, batch, retention policy and boot (#2494), plus labelled
 * [STATIC] assertions for deliberately backend-deferred behaviour.
 */
"use strict";

var h = require("./00_harness");
var g = h.loadModules();
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var deep = h.deep;
var source = h.readFile;
var C = g.core;
var ctx = g.boot.ctx;

test("reviewableCells lists low-confidence and unreadable cells only", function () {
  g.reset();
  g.seed();
  var cells = g.review.reviewableCells(C.state().parses.stmt, "stmt");
  equal(cells.length, 3);
  assert(cells.some(function (item) { return item.row === 2 && item.col === 2; }));
  assert(cells.some(function (item) { return item.row === 4 && item.col === 2; }));
  assert(!cells.some(function (item) { return item.row === 0 && item.col === 0; }));
  deep(g.review.reviewableCells(null, "stmt"), []);
});

test("a correction records original, correction, author and fixed timestamp", function () {
  g.reset();
  g.seed();
  var body = g.dom.makeNode("div");
  g.review.render(body, ctx);
  var form = body.children[0].children[2];
  var picker = form.children.filter(function (node) { return node.className === "ss-ingest-select"; })[0];
  var input = form.children.filter(function (node) { return node.className === "ss-ingest-input"; })[0];
  var save = form.children.filter(function (node) { return node.className === "ss-ingest-btn"; })[0];
  equal(picker.children.length, 3);
  picker.value = "4:2";
  input.value = "  77.70  ";
  save.dispatch("click");
  var overrides = C.state().overrides;
  equal(overrides.length, 1);
  deep({
    key: overrides[0].key, cell: overrides[0].cell, original: overrides[0].original,
    corrected: overrides[0].corrected, author: overrides[0].author,
    timestamp: overrides[0].timestamp,
  }, {
    key: "stmt:4:2", cell: "C5", original: "", corrected: "77.70",
    author: "Local reviewer", timestamp: g.fixedIso,
  });
  equal(C.effectiveValue(C.state(), "stmt", 4, 2), "77.70");
  equal(C.isResolved(C.state(), "stmt", 4, 2), true);
  equal(C.formulaBlocked(C.state(), "stmt", 4, 2), false);
});

test("the provenance log renders a recorded override without changing source", function () {
  g.reset();
  g.seed();
  C.mutate(function (target) {
    target.overrides.push({ key: "stmt:2:2", fileId: "stmt", row: 2, col: 2,
      cell: "C3", original: "120.50", corrected: "125.00",
      author: "<a>", timestamp: g.fixedIso });
  });
  var body = g.dom.makeNode("div");
  g.review.render(body, ctx);
  var log = body.children[0].children[3];
  equal(log.className, "ss-ingest-log");
  var lines = log.children.filter(function (node) { return node.className === "ss-ingest-detail"; });
  equal(lines.length, 1);
  assert(lines[0]._text.indexOf("C3") === 0);
  assert(lines[0]._text.indexOf("125.00") > 0);
  equal(C.cellAt(C.state().parses.stmt, 2, 2).value, "120.50");
});

test("a blank correction is rejected and never writes an override", function () {
  g.reset();
  g.seed();
  var body = g.dom.makeNode("div");
  g.review.render(body, ctx);
  var form = body.children[0].children[2];
  var input = form.children.filter(function (node) { return node.className === "ss-ingest-input"; })[0];
  var save = form.children.filter(function (node) { return node.className === "ss-ingest-btn"; })[0];
  input.value = "   ";
  save.dispatch("click");
  deep(C.state().overrides, []);
});

test("creating a preview persists formatted local rows with balance formulas", function () {
  g.reset();
  g.seed();
  var body = g.dom.makeNode("div");
  g.review.render(body, ctx);
  var create = body.children[0].children.filter(function (node) {
    return node.className.indexOf("ss-ingest-btn-primary") >= 0;
  })[0];
  create.dispatch("click");
  var preview = C.state().preview;
  equal(preview.rows.length, 7);
  equal(preview.createdAt, g.fixedIso);
  var first = preview.rows[0].cells;
  equal(first[0].format, "date");
  equal(first[4].label, "E2");
  equal(first[4].formula, "=E2");
  equal(preview.rows[1].cells[4].formula, "=E2+D3-C3");
  assert(preview.rows.every(function (row) { return row.cells.length === 5; }));
});

test("blocked cells carry no derived formula (regression #2494)", function () {
  g.reset();
  g.seed();
  // A blocked cell has a blank value because its source is unconfirmed; a
  // formula computed from that unconfirmed data must not be emitted either.
  var preview = g.review.buildPreview(C.state(), "stmt");
  var blockedBalance = preview.rows[5].cells[4];
  equal(blockedBalance.label, "E9");
  equal(blockedBalance.status, "blocked");
  equal(blockedBalance.value, "");
  equal(blockedBalance.formula, "");
});

test("batch start isolates the corrupt file and advances every other file", function () {
  g.reset();
  deep(g.batch.summary(), { queued: 4, parsing: 0, review: 0, ready: 0, failed: 0, total: 4 });
  g.batch.startBatch(ctx);
  deep(g.batch.summary(), { queued: 0, parsing: 3, review: 0, ready: 0, failed: 1, total: 4 });
  var files = C.state().files;
  equal(files[3].status, "failed");
  assert(files[3].error.indexOf("corrupt-scan.pdf") === 0);
  assert(files.slice(0, 3).every(function (file) { return file.status === "parsing"; }));
});

test("batch completion creates parses and selects the first reviewable file", function () {
  g.reset();
  g.batch.startBatch(ctx);
  g.batch.completeBatch(ctx);
  deep(C.state().files.map(function (file) { return file.status; }),
    ["review", "review", "review", "failed"]);
  equal(C.state().activeFileId, "stmt");
  deep(Object.keys(C.state().parses).sort(), ["inv", "photo", "stmt"]);
  assert(!C.state().parses.bad);
});

test("markReady and activate are targeted and clear a stale selection", function () {
  g.reset();
  g.batch.startBatch(ctx);
  g.batch.completeBatch(ctx);
  g.batch.markReady("stmt", ctx);
  deep(C.state().files.map(function (file) { return file.status; }),
    ["ready", "review", "review", "failed"]);
  g.preview.select("stmt", 2, 2);
  g.batch.activate("inv", ctx);
  equal(C.state().activeFileId, "inv");
  equal(g.preview.selected(), null);
  g.batch.activate("bad", ctx);
  equal(C.state().activeFileId, "inv");
});

test("batch render shows per-file status badges and a summary", function () {
  g.reset();
  g.batch.startBatch(ctx);
  var body = g.dom.makeNode("div");
  g.batch.render(body, ctx);
  var section = body.children[0];
  var rows = section.children.filter(function (node) { return node.className === "ss-ingest-batch-row"; });
  equal(rows.length, 4);
  deep(rows.map(function (row) { return row.dataset.status; }),
    ["parsing", "parsing", "parsing", "failed"]);
  assert(rows[3].children.some(function (node) { return node.className === "ss-ingest-error"; }));
  var summary = section.children.filter(function (node) { return node.className === "ss-ingest-summary"; })[0];
  assert(summary._text.indexOf("failed 1") > 0);
  equal(g.batch.statusLabel("review"), "Review");
  equal(g.batch.statusLabel("bogus"), "Queued");
});

test("retention policy validates the range and floors the day count", function () {
  g.reset();
  equal(g.policy.savePolicy(0, true, ctx), false);
  equal(g.policy.savePolicy("abc", true, ctx), false);
  equal(g.policy.savePolicy(3651, true, ctx), false);
  equal(g.policy.savePolicy("45.9", false, ctx), true);
  equal(C.state().policy.retainDays, 45);
  equal(C.state().policy.ocrStored, false);
});

test("removing OCR text clears local text and stamps the removal", function () {
  g.reset();
  assert(C.state().policy.ocrText.length > 0);
  g.policy.removeOcrText(ctx);
  equal(C.state().policy.ocrText, "");
  equal(C.state().policy.ocrStored, false);
  equal(C.state().policy.lastRemoved, g.fixedIso);
});

test("policy render exposes retention fields and the OCR preview", function () {
  g.reset();
  var body = g.dom.makeNode("div");
  g.policy.render(body, ctx);
  var section = body.children[0];
  var inputs = section.querySelectorAll("input");
  var days = inputs.filter(function (node) { return node.id === "ss-ingest-retain-days"; })[0];
  var stored = inputs.filter(function (node) { return node.id === "ss-ingest-ocr-stored"; })[0];
  equal(days.value, "30");
  equal(days.min, "1");
  equal(days.max, "3650");
  equal(stored.checked, true);
  assert(section.children.some(function (node) { return node.classList.contains("ss-ingest-btn-danger"); }));
});

test("boot open builds an accessible five-tab dialog with a live status", function () {
  g.reset();
  var root = g.boot.open(g.trigger);
  equal(root.getAttribute("role"), "dialog");
  equal(root.getAttribute("aria-modal"), "true");
  equal(root.getAttribute("aria-labelledby"), "ss-ingest-title");
  var tabs = root.children.filter(function (node) { return node.className === "ss-ingest-tabs"; })[0];
  deep(tabs.children.map(function (button) { return button.dataset.tab; }),
    ["preview", "map", "review", "batch", "policy"]);
  assert(tabs.children.every(function (button) { return button.getAttribute("role") === "tab"; }));
  var status = root.querySelector(".ss-ingest-status");
  equal(status.getAttribute("role"), "status");
  equal(status.getAttribute("aria-live"), "polite");
  g.boot.close();
  equal(root.isConnected, false);
});

test("switchTab updates selection, focus and the rendered module", function () {
  g.reset();
  var root = g.boot.open(g.trigger);
  g.boot.switchTab("policy");
  equal(g.boot.activeTab(), "policy");
  var tabs = root.children.filter(function (node) { return node.className === "ss-ingest-tabs"; })[0];
  deep(tabs.children.map(function (button) { return button.getAttribute("aria-selected"); }),
    ["false", "false", "false", "false", "true"]);
  var active = tabs.children.filter(function (button) { return button.dataset.tab === "policy"; })[0];
  assert(active.focusCount > 0);
  assert(root.children.filter(function (node) { return node.className === "ss-ingest-body"; })[0].children.length > 0);
  g.boot.close();
});

test("toggle, relocalize and the toolbar button work without a server", function () {
  g.reset();
  g.trigger.dispatch("click");
  equal(g.dom.document.querySelectorAll(".ss-ingest-dialog").length, 1);
  g.dom.windowEvents["gb-sheet-locale"].forEach(function (handler) { handler({}); });
  g.boot.toggle(null);
  g.boot.toggle(null);
  g.boot.rerender();
  g.boot.close();
});

// ── STATIC source assertions: backend-deferred, never runtime PASS ─────────
test("[STATIC] the nine modules contain no network, parser or OCR engine", function () {
  h.SOURCES.forEach(function (rel) {
    var src = source(rel);
    assert(!/\bfetch\s*\(/.test(src), rel + " calls fetch");
    assert(!/XMLHttpRequest/.test(src), rel + " uses XMLHttpRequest");
    assert(!/new\s+WebSocket/.test(src), rel + " opens a WebSocket");
    assert(!/FileReader/.test(src), rel + " reads file bytes");
    assert(!/Tesseract|pdfjs|PDFJS/.test(src), rel + " bundles an OCR/PDF parser");
  });
});

test("[STATIC] local-only and deferred boundaries are declared in shipped sources", function () {
  assert(/SIMULATED in-browser/.test(source(h.SOURCES[0])));
  assert(/No document bytes are read/.test(source(h.SOURCES[1])));
  assert(/Pure logic: no DOM, no[\s\S]*persistence side effects/.test(source(h.SOURCES[2])));
  assert(/Local-only rendering of a simulated parse/.test(source(h.SOURCES[3])));
  assert(/All outputs are local preview observations/.test(source(h.SOURCES[4])));
  assert(/Nothing here writes to the workbook or to a server/.test(source(h.SOURCES[5])));
  assert(/All transitions are simulated/.test(source(h.SOURCES[6])));
  assert(/never uploads, processes in a tenant[\s\S]*real original/.test(source(h.SOURCES[7])));
  assert(/boot never contacts the server or claims a real parse/.test(source(h.SOURCES[8])));
});
