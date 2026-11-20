/**
 * Sheet reporting — preview, dashboard, schedule, boot and local-only
 * source assertions (#2438). All non-STATIC tests below are RUNTIME tests
 * against the real shipped IIFEs in the shared vm sandbox.
 */
"use strict";

var h = require("./00_harness");
var g = h.loadModules();
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var deep = h.deep;
var source = h.readFile;

/* [KNOWN BUG] characterization: these two tests assert the behaviour the
 * shipped code actually has, not the intended behaviour. Flip them when the
 * production bugs described in the final agent report are fixed. */

// ── preview variants ─────────────────────────────────────────────────────
test("variantFor implements all/odd/even/first page visibility", function () {
  equal(g.preview.variantFor(0, "all"), true);
  equal(g.preview.variantFor(1, "all"), true);
  equal(g.preview.variantFor(0, "first"), true);
  equal(g.preview.variantFor(1, "first"), false);
  equal(g.preview.variantFor(0, "even"), false);
  equal(g.preview.variantFor(1, "even"), true);
  equal(g.preview.variantFor(2, "even"), false);
  equal(g.preview.variantFor(0, "odd"), true);
  equal(g.preview.variantFor(1, "odd"), false);
});

test("repeatRows parses single/inclusive ranges and rejects bad input", function () {
  deep(g.preview.repeatRows(""), []);
  deep(g.preview.repeatRows("1"), [0]);
  deep(g.preview.repeatRows("1:3"), [0, 1, 2]);
  deep(g.preview.repeatRows("3:2"), []);
  deep(g.preview.repeatRows("0:2"), []);
  deep(g.preview.repeatRows("abc"), []);
  equal(g.preview.repeatRows("1:99").length, 21);
});

test("fieldText interpolates page, sheet, file and date fields", function () {
  var plan = { pages: 3 };
  var ctx = { sheetName: "Budget", fileName: "<book>", date: "2026-01-01" };
  equal(g.preview.fieldText("page", ctx, 1, plan), "Page 2 of 3");
  equal(g.preview.fieldText("sheet", ctx, 0, plan), "Sheet: Budget");
  equal(g.preview.fieldText("file", ctx, 0, plan), "File: <book>");
  equal(g.preview.fieldText("date", ctx, 0, plan), "Date: 2026-01-01");
  equal(g.preview.fieldText("<custom>", ctx, 0, plan), "<custom>");
  equal(g.preview.fieldText("", ctx, 0, plan), "");
  equal(g.preview.fieldText(null, ctx, 0, plan), "");
});

test("lineFor honours header/footer variants", function () {
  var setup = g.core.defaultSetup();
  setup.header.left = "page";
  setup.header.center = "sheet";
  setup.footer.right = "page";
  setup.footerVariant = "first";
  var ctx = { setup: setup, sheetName: "Budget" };
  var plan = { pages: 2 };
  var header = g.preview.lineFor("header", 0, ctx, plan);
  equal(header.left, "Page 1 of 2");
  equal(header.center, "Sheet: Budget");
  equal(header.right, "");
  var firstFooter = g.preview.lineFor("footer", 0, ctx, plan);
  equal(firstFooter.right, "Page 1 of 2");
  var laterFooter = g.preview.lineFor("footer", 1, ctx, plan);
  deep(laterFooter, { left: "", center: "", right: "" });
});

// ── page cell layout ─────────────────────────────────────────────────────
test("pageCells walks a down-then-across page order", function () {
  var plan = g.print.paginate("A1:H60", g.core.defaultSetup());
  var first = g.preview.pageCells(0, plan, "");
  equal(first.length, 36);
  equal(first[0].ref, "A1");
  equal(first[5].ref, "F1");
  equal(first[first.length - 1].ref, "F6");
  assert(first.every(function (cell) { return cell.repeat === false; }));
  var second = g.preview.pageCells(1, plan, "");
  equal(second[0].ref, "A44");
  equal(second[second.length - 1].ref, "F49");
});

test("pageCells repeats title rows and marks them distinctly", function () {
  var plan = g.print.paginate("A1:H60", g.core.defaultSetup());
  var cells = g.preview.pageCells(0, plan, "1:2");
  equal(cells.length, 36);
  equal(cells.filter(function (cell) { return cell.repeat; }).length, 12);
  deep(cells.slice(0, 2), [
    { ref: "A1", repeat: true }, { ref: "B1", repeat: true },
  ]);
  deep(cells.slice(12, 14), [
    { ref: "A3", repeat: false }, { ref: "B3", repeat: false },
  ]);
  equal(g.preview.pageCells(1, plan, "1:2").filter(function (c) { return c.repeat; }).length, 12);
});

test("pageCells caps at 80 rendered cells and wraps page indices", function () {
  var plan = g.print.paginate("A1:H60", g.core.defaultSetup());
  equal(g.preview.pageCells(0, plan, "1:20").length, 80);
  // Indices beyond the plan wrap by design (colPages/rowPages modulo).
  deep(g.preview.pageCells(99, plan, ""), g.preview.pageCells(1, plan, ""));
});

test("preview render builds one article per page and a single page break", function () {
  var plan = g.print.paginate("A1:H60", g.core.defaultSetup());
  var container = g.dom.makeNode("div");
  var ctx = {
    setup: g.core.defaultSetup(), area: { range: "A1:H60" },
    sheetName: "Budget", fileName: "<book>", date: "2026-01-01",
  };
  var returned = g.preview.render(container, ctx);
  equal(returned.pages, plan.pages);
  equal(container.children.length, 5);
  equal(container.children[2].tagName, "ARTICLE");
  equal(container.children[3].className, "ss-rpt-page-break");
  equal(container.children[4].tagName, "ARTICLE");
  equal(container.children[2].children.length, 3);
});

// ── dashboard layout ─────────────────────────────────────────────────────
test("dashboard exposes the four shipped widget types", function () {
  deep(g.dashboard.TYPES, ["chart", "table", "kpi", "sparkline"]);
});

test("validateWidget requires a title, source and known type", function () {
  equal(g.dashboard.validateWidget({ title: "<t>", source: "<s>", type: "chart" }).ok, true);
  equal(g.dashboard.validateWidget({ title: "", source: "<s>", type: "chart" }).key,
    "sheet-rpt-invalid-widget");
  equal(g.dashboard.validateWidget({ title: "<t>", source: "", type: "chart" }).ok, false);
  equal(g.dashboard.validateWidget({ title: "<t>", source: "<s>", type: "pie" }).ok, false);
  var long = g.dashboard.validateWidget({ title: new Array(90).join("t"), source: new Array(130).join("s"), type: "kpi" });
  equal(long.title.length, 80);
  equal(long.source.length, 120);
});

test("createDashboard trims titles and rejects blank ones", function () {
  g.resetState();
  equal(g.dashboard.createDashboard("   ").ok, false);
  var created = g.dashboard.createDashboard("  Overview  ");
  equal(created.ok, true);
  equal(created.dashboard.title, "Overview");
  deep(created.dashboard.widgets, []);
  equal(g.core.state().dashboards.length, 1);
  g.resetState();
});

test("addWidget auto-lays out widgets on a 12-column grid", function () {
  g.resetState();
  var id = g.dashboard.createDashboard("<d>").dashboard.id;
  var expected = [
    { col: 1, row: 1 }, { col: 4, row: 1 }, { col: 7, row: 1 },
    { col: 10, row: 1 }, { col: 1, row: 3 },
  ];
  expected.forEach(function (position) {
    var result = g.dashboard.addWidget(id, {
      title: "<w>", source: "<s>", type: "chart", col: 99, row: 99, w: 99, h: 99,
    });
    equal(result.ok, true);
  });
  var widgets = g.core.state().dashboards[0].widgets;
  deep(widgets.map(function (w) { return { col: w.col, row: w.row }; }), expected);
  assert(widgets.every(function (w) { return w.w === 12 && w.h === 6; }));
  g.resetState();
});

test("[KNOWN BUG] addWidget reports success at MAX_ITEMS without adding", function () {
  g.resetState();
  var id = g.dashboard.createDashboard("<d>").dashboard.id;
  for (var i = 0; i < g.core.MAX_ITEMS; i++) {
    g.dashboard.addWidget(id, { title: "<w>", source: "<s>", type: "chart" });
  }
  var before = g.core.state().dashboards[0].widgets.length;
  // 56_reporting_dashboard.js:57,74 skips the push but still returns ok.
  var result = g.dashboard.addWidget(id, { title: "<over>", source: "<s>", type: "chart" });
  assert(result.ok === true);
  equal(g.core.state().dashboards[0].widgets.length, before);
  g.resetState();
});

test("addWidget fails cleanly for a missing dashboard; removeWidget is targeted", function () {
  g.resetState();
  equal(g.dashboard.addWidget("missing", { title: "<w>", source: "<s>", type: "chart" }).key,
    "sheet-rpt-dash-missing");
  var first = g.dashboard.createDashboard("<a>").dashboard.id;
  var second = g.dashboard.createDashboard("<b>").dashboard.id;
  g.dashboard.addWidget(first, { title: "<one>", source: "<s>", type: "chart" });
  g.dashboard.addWidget(second, { title: "<two>", source: "<s>", type: "chart" });
  g.dashboard.removeWidget(first, g.core.state().dashboards[0].widgets[0].id);
  equal(g.core.state().dashboards[0].widgets.length, 0);
  equal(g.core.state().dashboards[1].widgets.length, 1);
  g.resetState();
});

test("sparklinePoints is deterministic, bounded and has eight x positions", function () {
  var points = g.dashboard.sparklinePoints("Growth");
  equal(points, g.dashboard.sparklinePoints("Growth"));
  var pairs = points.split(" ").map(function (pair) { return pair.split(",").map(Number); });
  equal(pairs.length, 8);
  deep(pairs.map(function (pair) { return pair[0]; }), [0, 10, 20, 30, 40, 50, 60, 70]);
  assert(pairs.every(function (pair) { return pair[1] >= 2 && pair[1] <= 28; }));
  equal(g.dashboard.sparklinePoints("").split(" ").length, 8);
});

test("widgetNode emits grid placement and type-specific content", function () {
  var chart = g.dashboard.widgetNode({
    type: "chart", title: "<w>", source: "<s>", col: 2, row: 3, w: 4, h: 5,
  }, null);
  equal(chart.style.gridColumn, "2 / span 4");
  equal(chart.style.gridRow, "3 / span 5");
  equal(chart.children.length, 2);
  var kpi = g.dashboard.widgetNode({ type: "kpi", title: "<w>", source: "<s>", col: 1, row: 1, w: 3, h: 2 }, null);
  equal(kpi.children.length, 3);
  var table = g.dashboard.widgetNode({ type: "table", title: "<w>", source: "<s>", col: 1, row: 1, w: 3, h: 2 }, null);
  equal(table.children.length, 3);
  var spark = g.dashboard.widgetNode({ type: "sparkline", title: "<w>", source: "<s>", col: 1, row: 1, w: 3, h: 2 }, null);
  equal(spark.children[2].tagName, "SVG");
  equal(spark.children[2].getAttribute("viewBox"), "0 0 70 30");
  equal(spark.children[2].children[0].tagName, "POLYLINE");
  var removable = g.dashboard.widgetNode({ type: "chart", title: "<w>", source: "<s>", col: 1, row: 1, w: 3, h: 2 },
    { onRemove: function () {} });
  equal(removable.children.length, 3);
});

// ── schedule policy ──────────────────────────────────────────────────────
test("schedule exposes the shipped formats and frequencies", function () {
  deep(g.schedule.FORMATS, ["PDF", "XLSX", "CSV"]);
  deep(g.schedule.FREQUENCIES, ["daily", "weekly", "monthly"]);
});

test("validatePolicy accepts a complete local policy", function () {
  var result = g.schedule.validatePolicy({
    name: "<report>", sourceRange: "A1:H60", format: "PDF", frequency: "daily",
    recipientList: "a@example.com, b@example.com",
  });
  equal(result.ok, true);
  deep(result.errors, []);
});

test("validatePolicy reports every invalid field", function () {
  var result = g.schedule.validatePolicy({});
  equal(result.ok, false);
  deep(result.errors.map(function (error) { return error.key; }), [
    "sheet-rpt-policy-name-missing", "sheet-rpt-invalid-range",
    "sheet-rpt-policy-format-invalid", "sheet-rpt-policy-frequency-invalid",
    "sheet-rpt-policy-recipients-invalid",
  ]);
  var badMail = g.schedule.validatePolicy({
    name: "<r>", sourceRange: "A1:B2", format: "CSV", frequency: "weekly",
    recipientList: "not-an-address",
  });
  equal(badMail.ok, false);
  equal(badMail.errors.length, 1);
});

test("addPolicy trims recipients and serialises a local policy", function () {
  g.resetState();
  var result = g.schedule.addPolicy({
    name: "  <r>  ", sourceRange: " a1:b2 ", format: "XLSX", frequency: "monthly",
    dayOfWeek: "3", dayOfMonth: "15", recipientList: " a@example.com ,, b@example.com ",
    enabled: true,
  });
  equal(result.ok, true);
  var policy = g.core.state().schedules[0];
  equal(policy.sourceRange, "A1:B2");
  equal(policy.recipientList, "a@example.com, b@example.com");
  equal(policy.dayOfWeek, 3);
  equal(policy.dayOfMonth, 15);
  equal(policy.enabled, true);
  assert(/^policy-[a-z0-9]+-[a-z0-9]+$/.test(policy.id));
  g.schedule.removePolicy(policy.id);
  deep(g.core.state().schedules, []);
  g.resetState();
});

test("nextRun computes the next 07:00 daily occurrence", function () {
  var later = g.schedule.nextRun({ frequency: "daily" }, g.makeDate(2026, 0, 2, 8, 0));
  equal(later.getDate(), 3);
  equal(later.getHours(), 7);
  equal(later.getMinutes(), 0);
  var earlier = g.schedule.nextRun({ frequency: "daily" }, g.makeDate(2026, 0, 2, 6, 0));
  equal(earlier.getDate(), 2);
  equal(earlier.getHours(), 7);
});

test("nextRun resolves weekly targets and same-day rollover", function () {
  var from = g.makeDate(2026, 0, 7, 8, 0);
  equal(from.getDay(), 3);
  var monday = g.schedule.nextRun({ frequency: "weekly", dayOfWeek: "1" }, from);
  equal(monday.getDay(), 1);
  equal(monday.getDate(), 12);
  var rollover = g.schedule.nextRun({ frequency: "weekly", dayOfWeek: "3" }, from);
  equal(rollover.getDate(), 14);
  var invalid = g.schedule.nextRun({ frequency: "weekly", dayOfWeek: "99" }, from);
  equal(invalid.getDay(), 1);
});

test("nextRun clamps monthly days and rolls into the next month", function () {
  var from = g.makeDate(2026, 0, 15, 8, 0);
  var rolled = g.schedule.nextRun({ frequency: "monthly", dayOfMonth: "1" }, from);
  equal(rolled.getMonth(), 1);
  equal(rolled.getDate(), 1);
  var upcoming = g.schedule.nextRun({ frequency: "monthly", dayOfMonth: "20" }, from);
  equal(upcoming.getMonth(), 0);
  equal(upcoming.getDate(), 20);
  var clamped = g.schedule.nextRun({ frequency: "monthly", dayOfMonth: "31" }, from);
  equal(clamped.getDate(), 1);
});

test("nextRun returns null for invalid frequency or invalid from-date", function () {
  assert(g.schedule.nextRun({}, g.makeDate(2026, 0, 1, 8, 0)) === null);
  assert(g.schedule.nextRun({ frequency: "daily" }, "not-a-date") === null);
});

// ── dialog shell ─────────────────────────────────────────────────────────
test("boot exposes the four local tabs with their modules", function () {
  deep(g.boot.tabs.map(function (tab) { return [tab.id, tab.module]; }), [
    ["print", "SheetReportingPrint"],
    ["preview", "SheetReportingPreview"],
    ["dash", "SheetReportingDashboard"],
    ["sched", "SheetReportingSchedule"],
  ]);
});

test("open/switchTab/close drive one focus-trapped dialog", function () {
  var root = g.boot.open(null);
  equal(root.getAttribute("role"), "dialog");
  equal(root.getAttribute("aria-modal"), "true");
  var tabs = root.children.filter(function (node) { return node.className === "ss-rpt-tabs"; })[0];
  assert(tabs && tabs.children.length === 4);
  equal(tabs.children[0].getAttribute("aria-selected"), "true");
  g.boot.switchTab("sched");
  var selected = tabs.children.filter(function (button) {
    return button.getAttribute("aria-selected") === "true";
  });
  equal(selected.length, 1);
  equal(selected[0].dataset.tab, "sched");
  assert(selected[0].focusCount > 0);
  g.boot.close();
  equal(root.isConnected, false);
});

test("toggle and relocalize keep the shell usable without a server", function () {
  g.resetState();
  g.boot.toggle(null);
  g.boot.relocalize();
  g.boot.rerender();
  g.boot.refreshPreview();
  g.boot.close();
  g.resetState();
});

// ── STATIC source assertions (private/DOM-only and backend-deferred) ──────
test("[STATIC] all six shipped modules avoid network and delivery APIs", function () {
  Object.keys(h.SOURCES).forEach(function (key) {
    var src = source(h.SOURCES[key]);
    assert(!/\bfetch\s*\(/.test(src), key + " calls fetch");
    assert(!/XMLHttpRequest/.test(src), key + " uses XMLHttpRequest");
    assert(!/new\s+WebSocket/.test(src), key + " opens a WebSocket");
  });
});

test("[STATIC] local-only claims are present in the shipped sources", function () {
  assert(/localOnly:\s*true/.test(source(h.SOURCES.core)));
  assert(/not a PDF engine and makes no parity guarantee/.test(source(h.SOURCES.print)));
  assert(/not a real PDF export; page breaks and layout are approximate/.test(source(h.SOURCES.preview)));
  assert(/nothing is executed\s+\*\s+against the workbook/.test(source(h.SOURCES.dashboard)));
  assert(/never registers a cron job, exports a file, sends a message/.test(source(h.SOURCES.schedule)));
  assert(/boot never contacts the\s+\*\s+server/.test(source(h.SOURCES.boot)));
});

test("[STATIC] private helpers remain behind the exported surfaces", function () {
  var printSrc = source(h.SOURCES.print);
  assert(/function autoBreaks\(start, end, perPage, manual, axis\)/.test(printSrc));
  assert(/pages:\s*colBreaks\.length \* rowBreaks\.length/.test(printSrc));
  var previewSrc = source(h.SOURCES.preview);
  assert(/function contextFromState\(\)/.test(previewSrc));
  assert(/document\.title/.test(previewSrc));
  var bootSrc = source(h.SOURCES.boot);
  assert(/function wireButton\(\)/.test(bootSrc));
  assert(/gb-sheet-locale/.test(bootSrc));
});
