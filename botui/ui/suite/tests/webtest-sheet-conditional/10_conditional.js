/**
 * Sheet conditional formatting (#2260) — rule model and producer cases.
 * All non-STATIC tests below are RUNTIME tests against the real shipped
 * 02_conditional.js IIFE loaded in the shared vm sandbox.
 */
"use strict";

var h = require("./00_harness");
var env = h.loadModules();
var C = env.window.SheetConditional;
var test = h.test;
var assert = h.assert;
var equal = h.equal;
var deep = h.deep;

var SEL = { startRow: 1, startCol: 0, endRow: 5, endCol: 11 };

function lastPayload() {
  var call = env.fetchCalls[env.fetchCalls.length - 1];
  assert(call, "no conditional-format fetch was captured");
  return JSON.parse(call.init.body);
}

// ── exported API surface ─────────────────────────────────────────────────
test("02 module exports the six conditional quick-action functions", function () {
  assert(C, "window.SheetConditional missing");
  ["applyRule", "highlightGreaterThan", "highlightLessThan", "highlightBetween",
    "highlightDuplicates", "colorScale"].forEach(function (name) {
    equal(typeof C[name], "function", name + " is not a function");
  });
});

// ── applyRule payload ────────────────────────────────────────────────────
test("applyRule POSTs a selection-bounded payload to the conditional-format endpoint", function () {
  env.reset();
  env.setActiveIndex(2);
  env.setSelection(SEL);
  env.setFetchResponse({ success: true });
  C.applyRule({
    rule_type: "cellValue",
    condition: "<0",
    style: { background: "#f44336", color: "#ffffff" },
  });
  equal(env.fetchCalls.length, 1);
  var call = env.fetchCalls[0];
  equal(call.url, "/api/sheet/conditional-format");
  equal(call.init.method, "POST");
  equal(call.init.headers["Content-Type"], "application/json");
  var p = JSON.parse(call.init.body);
  equal(p.sheet_id, "sheet-cashflow-2026");
  equal(p.worksheet_index, 2);
  equal(p.start_row, 1);
  equal(p.start_col, 0);
  equal(p.end_row, 5);
  equal(p.end_col, 11);
  equal(p.rule_type, "cellValue");
  equal(p.condition, "<0");
  deep(p.style, { background: "#f44336", color: "#ffffff" });
});

test("applyRule supplies documented defaults when a rule omits fields", function () {
  env.reset();
  env.setSelection(SEL);
  C.applyRule({});
  var p = lastPayload();
  equal(p.rule_type, "cellValue");
  equal(p.condition, ">0");
  deep(p.style, { background: "#ffeb3b", color: "#000000" });
  equal(p.worksheet_index, 0);
});

test("applyRule short-circuits without a network call when there is no selection", function () {
  env.reset();
  env.setSelection(null);
  var result = C.applyRule({ condition: ">0" });
  assert(result && typeof result.then === "function", "applyRule must return a promise");
  equal(env.fetchCalls.length, 0);
  return result.then(function (value) { equal(value, null); });
});

test("applyRule maps an unsuccessful response and a network error to documented values", function () {
  env.reset();
  env.setSelection(SEL);
  env.setFetchResponse({ success: false, error: "<denied>" });
  return C.applyRule({ condition: ">0" }).then(function (res) {
    equal(res.success, false);
    env.setFetchFailure();
    return C.applyRule({ condition: ">0" });
  }).then(function (res) {
    equal(res, null);
  });
});

// ── quick-action colour rules ────────────────────────────────────────────
test("highlightGreaterThan builds an upper-bound GREEN rule (>value)", function () {
  env.reset();
  env.setSelection(SEL);
  C.highlightGreaterThan(0);
  var p = lastPayload();
  equal(p.condition, ">0");
  equal(p.style.background, "#4caf50");
  equal(p.style.color, "#ffffff");
});

test("highlightLessThan builds a lower-bound RED rule (<value)", function () {
  env.reset();
  env.setSelection(SEL);
  C.highlightLessThan(0);
  var p = lastPayload();
  equal(p.condition, "<0");
  equal(p.style.background, "#f44336");
  equal(p.style.color, "#ffffff");
});

test("highlightDuplicates builds a duplicates rule", function () {
  env.reset();
  env.setSelection(SEL);
  C.highlightDuplicates();
  var p = lastPayload();
  equal(p.rule_type, "duplicates");
  equal(p.condition, "countif");
  equal(p.style.background, "#ff9800");
  equal(p.style.color, "#000000");
});

test("colorScale builds a gradient rule with documented default colours", function () {
  env.reset();
  env.setSelection(SEL);
  C.colorScale();
  var p = lastPayload();
  equal(p.rule_type, "colorScale");
  equal(p.condition, "min:max");
  equal(p.style.background, "#fff3cd");
  equal(p.style.gradient_max, "#c8e6c9");
  env.clearFetch();
  C.colorScale("<min>", "<max>");
  var custom = lastPayload();
  equal(custom.style.background, "<min>");
  equal(custom.style.gradient_max, "<max>");
});

test("highlightBetween emits a 'between:a:b' condition the renderer parses (regression #2260)", function () {
  env.reset();
  env.setSelection(SEL);
  C.highlightBetween(1, 5);
  var p = lastPayload();
  equal(p.condition, "between:1:5");
  assert(p.condition.indexOf("between:") === 0,
    "highlightBetween must prefix its condition with 'between:'");
  // Renderer recognises this form; see the between-rule test in 11_conditional_render.js.
});

test("identical inputs produce byte-identical fetch payloads under fixed host globals", function () {
  env.reset();
  env.setSelection(SEL);
  C.highlightLessThan(0);
  var first = JSON.stringify(lastPayload());
  env.clearFetch();
  C.highlightLessThan(0);
  var second = JSON.stringify(lastPayload());
  equal(second, first);
});
