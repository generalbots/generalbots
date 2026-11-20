/**
 * Bounded local formula evaluator and step trace (#2491).
 * Acceptance target: intermediate values for a nested IF(SUMIF(...)) formula.
 * Every assertion executes the shipped 26_audit_eval.js / 26a_audit_engine.js.
 */
"use strict";

var h = require("./00_harness");
var e = h.createEngine();
var E = e.eval;
var I = e.evalInternals;
var G = e.graph;
var S = e.sections;
var assert = h.assert;
var equal = h.equal;
var deep = h.deep;

h.test("tokenizer emits typed tokens with stable source offsets", function () {
  var tokens = E.tokenize("SUM(A1,2)+\"x\"").tokens;
  deep(tokens.map(function (token) { return token.type; }),
    ["word", "op", "word", "op", "number", "op", "op", "string"], "token types");
  var ref = tokens.filter(function (token) { return token.value === "A1"; })[0];
  deep({ start: ref.start, end: ref.end }, { start: 4, end: 6 }, "reference offsets");
  equal(tokens[tokens.length - 1].closed, true, "string literal is closed");
});

h.test("tokenizer rejects an unterminated quoted sheet name", function () {
  assert(E.tokenize("'Sheet1!A1").error === "quote", "missing quote must be an error");
});

h.test("parser honours operator precedence and groups", function () {
  var ast = I.parse("1+2*3").ast;
  equal(ast.type, "binary", "root is binary");
  equal(ast.op, "+", "lowest precedence at the root");
  equal(ast.right.op, "*", "multiplication binds tighter");
  equal(I.parse("(1+2)*3").ast.left.type, "group", "parentheses produce a group node");
  equal(I.parse("-A1").ast.type, "unary", "leading minus is unary");
});

h.test("parser models calls, refs, ranges and boolean literals", function () {
  var call = I.parse("SUM(A1:B2,TRUE)").ast;
  equal(call.type, "call", "call node");
  equal(call.name, "SUM", "uppercased function name");
  equal(call.args[0].type, "range", "range node");
  equal(call.args[1].value, true, "boolean literal");
});

h.test("parser reports empty, end, paren and trailing syntax errors", function () {
  ["", "1+", "(1+2", "1 2"].forEach(function (source) {
    assert(I.parse(source).error, "expected a parse error for " + source);
  });
  assert(!I.parse("SUM(1,2)").error, "well-formed call must parse");
});

h.test("display and numeric helpers are deterministic", function () {
  equal(E.display(1.23456789), "1.234568", "six-decimal rounding");
  equal(E.display(true), "TRUE", "boolean display");
  equal(E.display([1, 2]), "1, 2", "array display");
  equal(E.display({ err: "#DIV/0!" }), "#DIV/0!", "error display");
  equal(E.numeric("1,234"), 1234, "group separators accepted");
  equal(E.numeric(true), 1, "boolean coercion");
  equal(E.numeric("x"), null, "non-numeric text is null");
  equal(I.truthy("YES"), true, "YES is truthy locally");
  equal(I.truthy(""), false, "blank is not truthy");
});

h.test("arithmetic and comparison operators evaluate over in-memory values", function () {
  equal(E.evaluate("=1+2*3").value, "7", "precedence");
  equal(E.evaluate("=(1+2)*3").value, "9", "grouping");
  equal(E.evaluate("=7/2").value, "3.5", "division");
  equal(E.evaluate("=2^3").value, "8", "power");
  equal(E.evaluate("=\"a\"&\"b\"").value, "ab", "concatenation");
  equal(E.evaluate("=3>2").value, "TRUE", "comparison");
  equal(E.evaluate("=1/0").error, "#DIV/0!", "divide by zero");
});

h.test("supported function catalog evaluates with local semantics", function () {
  equal(E.evaluate("=SUM(1,2,3)").value, "6", "SUM");
  equal(E.evaluate("=AVERAGE(2,4)").value, "3", "AVERAGE");
  equal(E.evaluate("=MAX(1,9,3)").value, "9", "MAX");
  equal(E.evaluate("=MIN(1,9,3)").value, "1", "MIN");
  equal(E.evaluate("=COUNT(1,\"x\",3)").value, "2", "COUNT");
  equal(E.evaluate("=UPPER(\"ok\")").value, "OK", "UPPER");
  equal(E.evaluate("=ROUND(1.234,2)").value, "1.23", "ROUND");
  equal(E.evaluate("=AND(TRUE,FALSE)").value, "FALSE", "AND");
  equal(E.evaluate("=IFERROR(1/0,9)").value, "9", "IFERROR");
  equal(E.evaluate("=NOPE(1)").error, "#NAME?", "unknown function");
});

function evalContextForGrid() {
  var book = h.makeBook();
  [1, 2, 3].forEach(function (value, row) {
    book.worksheets[0].data[row + ",0"] = { value: value, formula: "" };
  });
  [2, 3, 4].forEach(function (value, row) {
    book.worksheets[0].data[row + ",1"] = { value: value, formula: "" };
  });
  return S.makeEvalContext(G.normalizeSnapshot(book, 0), 0);
}

h.test("nested IF(SUMIF(...)) exposes every intermediate value with offsets", function () {
  var formula = "=IF(SUMIF(A1:A3,\">2\",B1:B3)>5,\"HIGH\",\"LOW\")";
  var stripped = formula.slice(1);
  var result = E.evaluate(formula, evalContextForGrid());
  equal(result.error, "", "no evaluation error");
  equal(result.value, "LOW", "SUMIF(>2)=4, which is not greater than 5");
  assert(result.steps.length >= 8, "expected at least eight sub-expression steps, got " + result.steps.length);
  var values = result.steps.map(function (step) { return step.value; });
  assert(values.indexOf("1, 2, 3") >= 0, "A1:A3 intermediate range is recorded");
  assert(values.indexOf("4") >= 0, "SUMIF intermediate value 4 is recorded");
  assert(values.indexOf("FALSE") >= 0, "comparison intermediate is recorded");
  result.steps.forEach(function (step) {
    equal(stripped.slice(step.start, step.end), step.expr,
      "step offsets must address the highlighted sub-expression: " + step.expr);
    assert(typeof step.depth === "number", "each step has a nesting depth");
  });
  equal(result.steps[result.steps.length - 1].expr, stripped, "last step is the whole expression");
});

h.test("IF branch selection changes with the same formula shape", function () {
  var formula = "=IF(SUMIF(A1:A3,\">=2\",B1:B3)>5,\"HIGH\",\"LOW\")";
  var result = E.evaluate(formula, evalContextForGrid());
  equal(result.value, "HIGH", "SUMIF(>=2)=7, which is greater than 5");
  assert(result.steps.some(function (step) { return step.expr.indexOf("HIGH") >= 0; }),
    "the selected branch appears in the step trace");
});

h.test("parse failures produce a structured evaluation result", function () {
  var result = E.evaluate("=1+");
  equal(result.value, "", "no value on parse failure");
  equal(result.error, "#NAME?", "parse failures surface as an evaluation error");
  assert(result.parseError, "parseError is populated");
  deep(result.steps, [], "no steps on parse failure");
});

h.test("bounded range evaluation flags truncation", function () {
  var result = E.evaluate("=SUM(A1:HA100)", S.makeEvalContext(G.normalizeSnapshot(h.makeBook(), 0), 0));
  equal(result.truncated, true, "range exceeds MAX_RANGE and must be bounded");
});

h.test("makeEvalContext resolves local and cross-sheet cells without mutation", function () {
  var book = h.makeBook();
  book.worksheets[0].data["0,0"] = { value: 3, formula: "" };
  book.worksheets[1].data["1,2"] = { value: 8, formula: "" };
  var snapshot = G.normalizeSnapshot(book, 0);
  var before = JSON.stringify(snapshot);
  var ctx = S.makeEvalContext(snapshot, 0);
  deep(ctx.resolve("", 0, 0), { value: 3, formula: "" }, "local cell");
  deep(ctx.resolve("Sheet2", 1, 2), { value: 8, formula: "" }, "cross-sheet cell");
  equal(ctx.resolve("Missing", 0, 0), null, "unknown sheet resolves to null");
  equal(JSON.stringify(snapshot), before, "context resolution is read-only");
});

h.test("formula cells with no cached value are recursively evaluated", function () {
  var book = h.makeBook();
  book.worksheets[0].data["0,0"] = { value: "", formula: "=2+3" };
  var ctx = S.makeEvalContext(G.normalizeSnapshot(book, 0), 0);
  equal(E.evaluate("=A1", ctx).value, "5", "nested formula result");
});

h.test("a circular reference returns a value instead of hanging", function () {
  var book = h.makeBook();
  book.worksheets[0].data["0,0"] = { value: "", formula: "=B1" };
  book.worksheets[0].data["0,1"] = { value: "", formula: "=A1" };
  var ctx = S.makeEvalContext(G.normalizeSnapshot(book, 0), 0);
  var result = E.evaluate("=A1", ctx);
  equal(result.error, "", "the visiting guard prevents recursion failure");
  assert(typeof result.value === "string", "a value is returned promptly");
});

h.knownBug("error literals are rejected by the tokenizer despite parser support", function () {
  equal(I.parse("#DIV/0!").error, "token", "tokenizer rejects the # prefix");
  equal(E.evaluate("#DIV/0!").parseError, "token", "evaluate reports a parse failure");
  h.sourceHas("26_audit_eval.js", "upper === \"#N/A\"", "dead parser branch is still shipped");
});

h.knownBug("nested formula step offsets are sliced from the unstripped source", function () {
  var book = h.makeBook();
  book.worksheets[0].data["0,0"] = { value: "", formula: "=2+3" };
  var ctx = S.makeEvalContext(G.normalizeSnapshot(book, 0), 0);
  var result = E.evaluate("=A1", ctx);
  var nested = result.steps.filter(function (step) { return step.depth > 1; });
  assert(nested.length > 0, "nested steps exist");
  var misaligned = nested.filter(function (step) {
    return "A1".slice(step.start, step.end) !== step.expr;
  });
  assert(misaligned.length > 0,
    "characterize the defect: nested step offsets do not address the top-level source");
});
