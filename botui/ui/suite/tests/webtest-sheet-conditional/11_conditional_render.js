/**
 * Sheet conditional formatting (#2260) — renderer runtime cases.
 * All tests below execute the real shipped 07_conditional_render.js IIFE; the
 * producer payloads in the "full chain" test come from the real 02 module.
 */
"use strict";

var h = require("./00_harness");
var env = h.loadModules();
var C = env.window.SheetConditional;
var R = env.window.SheetConditionalRender;
var test = h.test;
var assert = h.assert;
var equal = h.equal;

var RED = { background: "#f44336", color: "#ffffff" };
var GREEN = { background: "#4caf50", color: "#ffffff" };

function rule(overrides) {
  var base = {
    start_row: 0, end_row: 0, start_col: 0, end_col: 11,
    rule_type: "cellValue", condition: ">0", style: GREEN,
  };
  Object.keys(overrides || {}).forEach(function (k) { base[k] = overrides[k]; });
  return base;
}

function sheetWith(rules, validations) {
  return {
    worksheets: [{
      conditional_formats: rules || [],
      validations: validations || {},
    }],
  };
}

function bg(node) {
  return node ? node.style.backgroundColor : undefined;
}

// ── exported surface ─────────────────────────────────────────────────────
test("07 module exports apply/wire and attaches hooks to SheetCore", function () {
  equal(typeof R.apply, "function");
  equal(typeof R.wire, "function");
  equal(typeof env.window.SheetCore.conditionalRender, "function");
  equal(typeof env.window.SheetCore.validateEdit, "function");
  equal(typeof env.window.SheetCore.validationForCell, "function");
  assert(env.timers.some(function (t) { return t.delay === 0; }),
    "load-time wire() must be scheduled on a zero-delay timer");
});

// ── negative red / positive green ────────────────────────────────────────
test("a <0 rule paints only negative cells red", function () {
  var g = env.makeGrid(1, 4, 1);
  [-100, 0, 250, -7].forEach(function (v, c) { g.set(0, c, v); });
  env.setGrid(g);
  env.setSheet(sheetWith([rule({ condition: "<0", style: RED })]));
  R.apply();
  equal(bg(g.node(0, 0)), RED.background);
  equal(g.node(0, 0).style.color, RED.color);
  equal(bg(g.node(0, 1)), undefined);
  equal(bg(g.node(0, 2)), undefined);
  equal(bg(g.node(0, 3)), RED.background);
});

test("a >0 rule paints only positive cells green", function () {
  var g = env.makeGrid(1, 4, 1);
  [-100, 0, 250, 7].forEach(function (v, c) { g.set(0, c, v); });
  env.setGrid(g);
  env.setSheet(sheetWith([rule({ condition: ">0", style: GREEN })]));
  R.apply();
  equal(bg(g.node(0, 0)), undefined);
  equal(bg(g.node(0, 1)), undefined);
  equal(bg(g.node(0, 2)), GREEN.background);
  equal(bg(g.node(0, 3)), GREEN.background);
});

test("[UAT:SHEET-06] opposing <0 red and >0 green rules compose on one 12-column balance row", function () {
  var g = env.makeGrid(1, 12, 1);
  var values = [-500, 200, 0, -30, 900, 0, 45, -12, 300, 0, -75, 600];
  values.forEach(function (v, c) { g.set(0, c, v); });
  env.setGrid(g);
  env.setSheet(sheetWith([
    rule({ condition: "<0", style: RED }),
    rule({ condition: ">0", style: GREEN }),
  ]));
  R.apply();
  values.forEach(function (v, c) {
    if (v < 0) equal(bg(g.node(0, c)), RED.background, "col " + c + " must be red");
    else if (v > 0) equal(bg(g.node(0, c)), GREEN.background, "col " + c + " must be green");
    else equal(bg(g.node(0, c)), undefined, "col " + c + " (zero) must stay unstyled");
  });
});

test("full chain: 02 producer payloads render as red/green through 07", function () {
  env.reset();
  env.setSelection({ startRow: 0, startCol: 0, endRow: 0, endCol: 11 });
  env.setFetchResponse({ success: true });
  C.highlightLessThan(0);
  C.highlightGreaterThan(0);
  equal(env.fetchCalls.length, 2);
  var rules = env.fetchCalls.map(function (call) {
    var p = JSON.parse(call.init.body);
    return {
      start_row: p.start_row, start_col: p.start_col,
      end_row: p.end_row, end_col: p.end_col,
      rule_type: p.rule_type, condition: p.condition, style: p.style,
    };
  });
  var values = [-120, 80, -1, 0, 340, -55, 12, 0, 99, -7, 410, 3];
  var g = env.makeGrid(1, 12, 1);
  values.forEach(function (v, c) { g.set(0, c, v); });
  env.setGrid(g);
  env.setSheet(sheetWith(rules));
  R.apply();
  values.forEach(function (v, c) {
    if (v < 0) equal(bg(g.node(0, c)), RED.background, "col " + c + " red");
    else if (v > 0) equal(bg(g.node(0, c)), GREEN.background, "col " + c + " green");
    else equal(bg(g.node(0, c)), undefined, "col " + c + " zero unstyled");
  });
});

// ── range and visibility ─────────────────────────────────────────────────
test("rules respect declared row and column bounds", function () {
  var g = env.makeGrid(6, 8, 6);
  for (var r = 0; r < 6; r++) for (var c = 0; c < 8; c++) g.set(r, c, 1);
  env.setGrid(g);
  env.setSheet(sheetWith([rule({
    start_row: 1, end_row: 4, start_col: 2, end_col: 4,
    condition: ">0", style: RED,
  })]));
  R.apply();
  for (var rr = 0; rr < 6; rr++) {
    for (var cc = 0; cc < 8; cc++) {
      var expected = rr >= 1 && rr <= 4 && cc >= 2 && cc <= 4 ? RED.background : undefined;
      equal(bg(g.node(rr, cc)), expected, "cell " + rr + "," + cc);
    }
  }
});

test("rules are clamped to the visible row window (visible.end is exclusive)", function () {
  var g = env.makeGrid(6, 1, 3); // only rows 0..2 are visible
  for (var r = 0; r < 6; r++) g.set(r, 0, 1);
  env.setGrid(g);
  env.setSheet(sheetWith([rule({
    start_row: 0, end_row: 99, start_col: 0, end_col: 0,
    condition: ">0", style: GREEN,
  })]));
  R.apply();
  equal(bg(g.node(0, 0)), GREEN.background);
  equal(bg(g.node(2, 0)), GREEN.background);
  equal(bg(g.node(3, 0)), undefined);
  equal(bg(g.node(5, 0)), undefined);
});

// ── condition evaluation ─────────────────────────────────────────────────
test("numeric comparison operators evaluate correctly", function () {
  var g = env.makeGrid(1, 6, 1);
  var cases = [
    { v: 5, cond: ">=5", hit: true },
    { v: -10, cond: "<=-10", hit: true },
    { v: 4, cond: "<>5", hit: true },
    { v: 7, cond: "=7", hit: true },
    { v: 9, cond: ">3", hit: true },
    { v: 2, cond: ">3", hit: false },
  ];
  var rules = cases.map(function (x, c) {
    g.set(0, c, x.v);
    return rule({ start_col: c, end_col: c, condition: x.cond, style: RED });
  });
  env.setGrid(g);
  env.setSheet(sheetWith(rules));
  R.apply();
  cases.forEach(function (x, c) {
    equal(bg(g.node(0, c)), x.hit ? RED.background : undefined,
      "operator " + x.cond + " on " + x.v);
  });
});

test("text operators match case-insensitively", function () {
  var g = env.makeGrid(1, 4, 1);
  ["Hello World", "Worldly", "Say world", "Worldly"].forEach(function (v, c) {
    g.set(0, c, v);
  });
  env.setGrid(g);
  env.setSheet(sheetWith([
    rule({ start_col: 0, end_col: 0, condition: "text:contains:WORLD", style: RED }),
    rule({ start_col: 1, end_col: 1, condition: "text:startswith:wor", style: RED }),
    rule({ start_col: 2, end_col: 2, condition: "text:endswith:WORLD", style: RED }),
    rule({ start_col: 3, end_col: 3, condition: "text:contains:xyz", style: RED }),
  ]));
  R.apply();
  equal(bg(g.node(0, 0)), RED.background);
  equal(bg(g.node(0, 1)), RED.background);
  equal(bg(g.node(0, 2)), RED.background);
  equal(bg(g.node(0, 3)), undefined);
});

test("duplicates rules highlight only repeated, non-empty values", function () {
  var g = env.makeGrid(1, 5, 1);
  ["A", "B", "A", "C", ""].forEach(function (v, c) { g.set(0, c, v); });
  env.setGrid(g);
  env.setSheet(sheetWith([rule({
    start_col: 0, end_col: 4, rule_type: "duplicates",
    condition: "countif", style: GREEN,
  })]));
  R.apply();
  equal(bg(g.node(0, 0)), GREEN.background);
  equal(bg(g.node(0, 1)), undefined);
  equal(bg(g.node(0, 2)), GREEN.background);
  equal(bg(g.node(0, 3)), undefined);
  equal(bg(g.node(0, 4)), undefined);
});

test("colorScale rules apply the minimum colour to in-range cells", function () {
  var g = env.makeGrid(1, 2, 1);
  g.set(0, 0, 10);
  g.set(0, 1, -5);
  env.setGrid(g);
  env.setSheet(sheetWith([rule({
    start_col: 0, end_col: 1, rule_type: "colorScale", condition: "min:max",
    style: { background: "#fff3cd", color: "#000000", gradient_max: "#c8e6c9" },
  })]));
  R.apply();
  equal(bg(g.node(0, 0)), "#fff3cd");
  equal(bg(g.node(0, 1)), "#fff3cd");
});

test("[KNOWN BUG] a 'a:b' condition (highlightBetween output) matches no cell", function () {
  var g = env.makeGrid(1, 2, 1);
  g.set(0, 0, 3);
  g.set(0, 1, 3);
  env.setGrid(g);
  env.setSheet(sheetWith([
    rule({ start_col: 0, end_col: 0, condition: "1:5", style: RED }),
    rule({ start_col: 1, end_col: 1, condition: "between:1:5", style: GREEN }),
  ]));
  R.apply();
  equal(bg(g.node(0, 0)), undefined, "'1:5' is silently ignored by evalRule");
  equal(bg(g.node(0, 1)), GREEN.background, "'between:1:5' is the recognised form");
});

// ── validation & markers ─────────────────────────────────────────────────
test("validateEdit enforces all validation types and uses the localized fallback", function () {
  env.setSheet(sheetWith([], {
    "0,0": { validation_type: "number", error_message: "<numErr>" },
    "0,1": { validation_type: "integer" },
    "0,2": { validation_type: "list", allowed_values: ["A", "B"] },
    "0,3": { validation_type: "date" },
    "0,4": { validation_type: "text_length", value1: 2, value2: 4 },
    "0,5": { validation_type: "custom" },
  }));
  var V = env.window.SheetCore.validateEdit;
  equal(V(0, 0, "abc").valid, false);
  equal(V(0, 0, "abc").message, "<numErr>");
  equal(V(0, 0, "12.5").valid, true);
  equal(V(0, 1, "3.2").valid, false);
  equal(V(0, 2, "C").valid, false);
  equal(V(0, 2, "A").valid, true);
  equal(V(0, 3, "2026-01-15").valid, true);
  equal(V(0, 3, "15/01/2026").valid, false);
  equal(V(0, 4, "abc").valid, true);
  equal(V(0, 4, "a").valid, false);
  equal(V(0, 5, "anything").valid, true);
  var free = V(2, 2, "x");
  equal(free.valid, true);
  equal(free.message, null);
  env.i18nCalls.length = 0;
  var defaulted = V(0, 1, "3.2");
  equal(defaulted.valid, false);
  assert(env.i18nCalls.length >= 1, "t() must be consulted for the default message");
  equal(env.i18nCalls[env.i18nCalls.length - 1].key, "sheet-invalid-value");
  equal(defaulted.message, env.i18nCalls[env.i18nCalls.length - 1].fallback);
});

test("[REGRESSION] validation/comment markers render with no conditional rules (#2260)", function () {
  var g = env.makeGrid(1, 3, 1);
  g.set(0, 0, "1");
  g.set(0, 1, "2", { has_comment: true, note: "<note>" });
  g.set(0, 2, "3");
  env.setGrid(g);
  env.setSheet(sheetWith([], { "0,0": { validation_type: "number" } }));
  R.apply();
  // The marker pass runs even when no conditional rules are defined.
  assert(g.node(0, 0).querySelector(".ss-validation-dot"), "validation dot missing");
  assert(g.node(0, 1).querySelector(".ss-comment-marker"), "comment marker missing");
});

test("validation and comment markers are added once and carry cell metadata", function () {
  var g = env.makeGrid(1, 3, 1);
  g.set(0, 0, "1");
  g.set(0, 1, "2", { has_comment: true, note: "<note>" });
  g.set(0, 2, "3");
  env.setGrid(g);
  // A harmless always-false rule gets past the early return so the marker
  // pass actually executes.
  env.setSheet(sheetWith(
    [rule({ condition: "text:contains:<never>", style: RED })],
    { "0,0": { validation_type: "number" } }
  ));
  R.apply();
  var n0 = g.node(0, 0);
  assert(n0.querySelector(".ss-validation-dot"), "validation dot missing");
  assert(!n0.querySelector(".ss-comment-marker"));
  var n1 = g.node(0, 1);
  var mark = n1.querySelector(".ss-comment-marker");
  assert(mark, "comment marker missing");
  equal(mark.dataset.row, "0");
  equal(mark.dataset.col, "1");
  equal(mark.title, "<note>");
  assert(!n1.querySelector(".ss-validation-dot"));
  R.apply();
  equal(n0.querySelectorAll(".ss-validation-dot").length, 1);
  equal(n1.querySelectorAll(".ss-comment-marker").length, 1);
});

// ── determinism & wiring ─────────────────────────────────────────────────
test("render output is deterministic across repeated applications", function () {
  var g = env.makeGrid(2, 3, 2);
  for (var r = 0; r < 2; r++) {
    for (var c = 0; c < 3; c++) g.set(r, c, c === 1 ? 5 : -1);
  }
  env.setGrid(g);
  env.setSheet(sheetWith([rule({
    start_row: 0, end_row: 1, start_col: 0, end_col: 2,
    condition: "<0", style: RED,
  })]));
  function snapshot() {
    var out = {};
    for (var rr = 0; rr < 2; rr++) {
      for (var cc = 0; cc < 3; cc++) out[rr + "," + cc] = JSON.stringify(g.node(rr, cc).style);
    }
    return JSON.stringify(out);
  }
  R.apply();
  var first = snapshot();
  for (var r2 = 0; r2 < 2; r2++) for (var c2 = 0; c2 < 3; c2++) g.node(r2, c2).style = {};
  R.apply();
  equal(snapshot(), first);
});

test("wire wraps grid.render once and re-applies rules after each render", function () {
  var g = env.makeGrid(1, 2, 1);
  g.set(0, 0, -5);
  g.set(0, 1, 5);
  env.setGrid(g);
  env.setSheet(sheetWith([
    rule({ condition: "<0", style: RED }),
    rule({ condition: ">0", style: GREEN }),
  ]));
  R.wire();
  equal(g.renderCount, 0);
  equal(bg(g.node(0, 0)), RED.background);
  equal(bg(g.node(0, 1)), GREEN.background);
  g.node(0, 0).style = {};
  g.node(0, 1).style = {};
  g.render();
  equal(g.renderCount, 1);
  equal(bg(g.node(0, 0)), RED.background);
  R.wire();
  g.node(0, 0).style = {};
  g.render();
  equal(g.renderCount, 2, "render must not be wrapped twice");
  equal(bg(g.node(0, 0)), RED.background);
});

test("renderer re-evaluates an edited or removed live rule list", function () {
  var g = env.makeGrid(1, 1, 1);
  g.set(0, 0, -5);
  env.setGrid(g);
  var sheet = sheetWith([rule({ condition: "<0", style: RED })]);
  env.setSheet(sheet);
  R.apply();
  equal(bg(g.node(0, 0)), RED.background);
  sheet.worksheets[0].conditional_formats[0].condition = ">0";
  sheet.worksheets[0].conditional_formats[0].style = GREEN;
  g.node(0, 0).style = {};
  R.apply();
  equal(bg(g.node(0, 0)), undefined, "edited rule no longer matches -5");
  g.set(0, 0, 5);
  R.apply();
  equal(bg(g.node(0, 0)), GREEN.background);
  sheet.worksheets[0].conditional_formats.length = 0;
  g.node(0, 0).style = {};
  R.apply();
  equal(bg(g.node(0, 0)), undefined);
});

test("apply is a no-op when the sheet or rules are absent", function () {
  var g = env.makeGrid(1, 1, 1);
  g.set(0, 0, 5);
  env.setGrid(g);
  env.setSheet(null);
  R.apply();
  equal(bg(g.node(0, 0)), undefined);
  env.setSheet({ worksheets: [{}] });
  R.apply();
  equal(bg(g.node(0, 0)), undefined);
  env.setSheet({});
  R.apply();
  equal(bg(g.node(0, 0)), undefined);
});
