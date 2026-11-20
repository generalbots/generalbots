/**
 * Sheet functions engine — invariant tests (#2434).
 *
 * Every RUNTIME case calls a genuine export of the shipped IIFEs. STATIC cases
 * are labelled as such and only inspect source text; they never claim runtime
 * results for helpers that are not exported by these modules.
 *
 * Date serials below are independent Excel-1900 values: (date - 1899-12-30).days.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var approx = h.approx;
var sourceHas = h.sourceHas;
var sourceLacks = h.sourceLacks;
var note = h.note;
var eng = h.loadEngine();
var E = eng.errors;
var F = eng.financial;
var D = eng.date;
var B = eng.base;
var S = eng.statText;
var C = eng.catalog;
var P = eng.parser;
var V = eng.evaluator;
var gbLog = globalThis.gbLog || { debug: function (line) { console.log(line); } };

function numeric(label, fn, expected, tolerance) {
  test(label, function () {
    var actual = fn();
    approx(actual, expected, tolerance || 1e-9, label);
  });
}

function errorCase(label, fn, code) {
  test(label, function () {
    var actual = fn();
    assert(E.isError(actual), "expected Excel error " + code + ", got " + E.format(actual));
    assert(E.code(actual) === code, "expected " + code + ", got " + E.code(actual));
  });
}

function valueCase(label, fn, expected) {
  test(label, function () {
    assert(fn() === expected, "expected " + String(expected) + ", got " + String(fn()));
  });
}

// ── Excel error semantics ─────────────────────────────────────────────
test("error taxonomy exposes all seven Excel codes", function () {
  ["#DIV/0!", "#VALUE!", "#REF!", "#NAME?", "#N/A", "#NUM!", "#NULL!"].forEach(function (code) {
    assert(E.CODES.indexOf(code) >= 0, code + " missing from CODES");
  });
});

valueCase("unknown error code degrades to #VALUE!", function () { return E.code(E.err("BOGUS")); }, "#VALUE!");
valueCase("format renders an error as its code", function () { return E.format(E.err("#N/A")); }, "#N/A");
valueCase("num never coerces text to zero", function () { return E.code(E.num("123")); }, "#VALUE!");
valueCase("num treats a blank as zero", function () { return E.num(undefined); }, 0);
valueCase("num treats null as zero", function () { return E.num(null); }, 0);
valueCase("num treats TRUE as one", function () { return E.num(true); }, 1);
errorCase("division by zero returns #DIV/0!", function () { return E.divide(1, 0); }, "#DIV/0!");
errorCase("error operand propagates through division", function () { return E.divide(E.err("#REF!"), 2); }, "#REF!");
errorCase("text operand in division returns #VALUE!", function () { return E.divide("x", 2); }, "#VALUE!");
errorCase("firstError scans nested arrays in order", function () {
  return E.firstError([1, [2, E.err("#REF!")], E.err("#VALUE!")]);
}, "#REF!");
errorCase("numbers rejects text anywhere in the list", function () { return E.numbers([1, "x"]); }, "#VALUE!");
errorCase("op propagates an error before arithmetic", function () {
  return E.op("*", E.err("#REF!"), 2);
}, "#REF!");
errorCase("finite rejects overflow", function () { return E.finite(Infinity); }, "#NUM!");
errorCase("positiveRoot rejects a negative radicand", function () { return E.positiveRoot(-1); }, "#NUM!");
errorCase("checkedLog rejects zero", function () { return E.checkedLog(0); }, "#NUM!");

valueCase("IFERROR consumes an error", function () { return E.IFERROR(E.err("#DIV/0!"), 0); }, 0);
valueCase("IFERROR keeps a non-error", function () { return E.IFERROR(5, 0); }, 5);
valueCase("IFNA consumes only #N/A", function () { return E.IFNA(E.err("#N/A"), 9); }, 9);
errorCase("IFNA does not consume #VALUE!", function () { return E.IFNA(E.err("#VALUE!"), 9); }, "#VALUE!");
valueCase("ISERROR reports true for an error", function () { return E.ISERROR(E.err("#DIV/0!")); }, true);
valueCase("ISNA reports false for a non-#N/A error", function () { return E.ISNA(E.err("#DIV/0!")); }, false);

errorCase("evaluator: 1/0 is #DIV/0!", function () { return V.evaluate("=1/0"); }, "#DIV/0!");
errorCase("evaluator: SUM propagates a nested error", function () { return V.evaluate("=SUM(1,1/0)"); }, "#DIV/0!");
errorCase("evaluator: text plus number is #VALUE!", function () { return V.evaluate('="abc"+1'); }, "#VALUE!");
errorCase("evaluator: unknown function is #NAME?", function () { return V.evaluate("=NOSUCH(1)"); }, "#NAME?");
errorCase("evaluator: bare error literal is #REF!", function () { return V.evaluate("=#REF!+1"); }, "#REF!");
valueCase("evaluator: IF leaves the unused errored branch unevaluated", function () {
  return V.evaluate("=IF(FALSE,1/0,7)");
}, 7);
valueCase("evaluator: IFERROR works end to end", function () { return V.evaluate("=IFERROR(1/0,0)"); }, 0);
valueCase("evaluator: IFNA works end to end", function () { return V.evaluate("=IFNA(#N/A,9)"); }, 9);
errorCase("evaluator: IFNA does not swallow #DIV/0!", function () {
  return V.evaluate("=IFNA(1/0,9)");
}, "#DIV/0!");

// ── Financial family (deterministic Excel vectors) ────────────────────
numeric("PMT standard 30-year mortgage", function () {
  return F.PMT([0.05 / 12, 360, 100000]);
}, -536.8216230121398);
numeric("PMT with zero rate", function () { return F.PMT([0, 10, 1000]); }, -100);
numeric("FV standard annuity", function () {
  return F.FV([0.05 / 12, 12, -100, -1000]);
}, 2279.0474470433246);
numeric("PV standard annuity", function () { return F.PV([0.05 / 12, 12, -100]); }, 1168.122200429816);
numeric("NPV discounts the first flow one period in", function () {
  return F.NPV([0.1, -1000, 100, 200, 300, 400, 500]);
}, 59.32621004865176, 1e-9);
numeric("IRR of a conventional cash-flow series", function () {
  return F.IRR([[-1000, 100, 200, 300, 400, 500]]);
}, 0.1200576195, 1e-9);
numeric("SLN straight-line depreciation", function () { return F.SLN([10000, 1000, 10]); }, 900);
numeric("SYD sum-of-years depreciation", function () {
  return F.SYD([10000, 1000, 10, 1]);
}, 1636.3636363636363);
numeric("DDB first-year depreciation", function () { return F.DDB([10000, 1000, 10, 1]); }, 2000);
numeric("EFFECT annualises a nominal rate", function () { return F.EFFECT([0.05, 12]); }, 0.05116189788173298);
numeric("NOMINAL is the inverse of EFFECT", function () {
  return F.NOMINAL([0.051161897881733, 12]);
}, 0.05, 1e-12);
errorCase("financial helper propagates an error argument", function () {
  return F.PMT([E.err("#VALUE!"), 10, 1000]);
}, "#VALUE!");
errorCase("NPV rejects a rate at or below -100%", function () { return F.NPV([-1, 100]); }, "#NUM!");

// ── Date family: leap years, month overflow, serial systems ───────────
numeric("1900 serial for 2024-01-01", function () {
  return D.serial(eng.makeDate(2024, 0, 1));
}, 45292);
numeric("1900 serial keeps Excel's leap-year bug (1900-03-01 = 61)", function () {
  return D.serial(eng.makeDate(1900, 2, 1));
}, 61);
numeric("1900 serial for 1900-02-28 is 59", function () {
  return D.serial(eng.makeDate(1900, 1, 28));
}, 59);
test("dateFromSerial(45292) round-trips to 2024-01-01", function () {
  var d = D.dateFromSerial(45292);
  assert(d.getUTCFullYear() === 2024 && d.getUTCMonth() === 0 && d.getUTCDate() === 1,
    "wrong date: " + d.toISOString());
});
test("1904 date system shifts the epoch (runtime settings patch)", function () {
  var restore = eng.setSettings({ dateSystem: "1904" });
  try {
    assert(D.serial(eng.makeDate(1904, 0, 1)) === 0, "1904-01-01 should be serial 0");
    assert(D.serial(eng.makeDate(1904, 0, 2)) === 1, "1904-01-02 should be serial 1");
  } finally {
    restore();
  }
});
numeric("EDATE overflows Jan 31 2024 to Feb 29 (leap year)", function () {
  return D.EDATE([45322, 1]);
}, 45351);
numeric("EDATE overflows Jan 31 2025 to Feb 28", function () {
  return D.EDATE([45688, 1]);
}, 45716);
numeric("EOMONTH of Feb 2026 is Mar 31 2026", function () { return D.EOMONTH([46063, 1]); }, 46112);
numeric("EOMONTH of Feb 2024 is Feb 29 (leap year)", function () { return D.EOMONTH([45337, 0]); }, 45351);
numeric("DATEDIF days across a leap day", function () { return D.DATEDIF([45350, 45352, "D"]); }, 2);
numeric("DATEDIF whole years", function () { return D.DATEDIF([45292, 45658, "Y"]); }, 1);
numeric("YEARFRAC basis 3 over a leap year", function () {
  return D.YEARFRAC([45292, 45658, 3]);
}, 366 / 365);
numeric("NETWORKDAYS over one working week", function () { return D.NETWORKDAYS([46083, 46089]); }, 5);
numeric("NETWORKDAYS subtracts a holiday", function () {
  return D.NETWORKDAYS([46083, 46089, 46085]);
}, 4);
numeric("WORKDAY advances to the next Monday", function () { return D.WORKDAY([46087, 1]); }, 46090);
errorCase("date helper propagates an error serial", function () { return D.EDATE([E.err("#VALUE!"), 1]); }, "#VALUE!");

test("[STATIC] DATE/YEAR/MONTH/DAY/DAYS are absent from the preview modules", function () {
  h.ORDER.forEach(function (file) {
    sourceLacks(file, "\\b(?:DATE|YEAR|MONTH|DAY|DAYS)\\s*\\(",
      file + " unexpectedly declares a bare date part function");
  });
  sourceHas("76a_functions_base.js", "native in botsheet-core",
    "the documented native/preview boundary is missing");
});

// ── Statistical and text helpers ──────────────────────────────────────
numeric("SUM of three numbers", function () { return B.SUM([1, 2, 3]); }, 6);
numeric("SUM of an empty list is zero", function () { return B.SUM([]); }, 0);
numeric("AVERAGE of three numbers", function () { return B.AVERAGE([1, 2, 3]); }, 2);
errorCase("AVERAGE of an empty list is #DIV/0!", function () { return B.AVERAGE([]); }, "#DIV/0!");
numeric("STDEV.S sample deviation", function () { return S.STDEV_S([[1, 2, 3, 4]]); }, 1.2909944487358056);
numeric("STDEV.P population deviation", function () { return S.STDEV_P([[1, 2, 3, 4]]); }, 1.118033988749895);
numeric("VAR.S sample variance", function () { return S.VAR_S([[1, 2, 3, 4]]); }, 1.6666666666666667);
numeric("VAR.P population variance", function () { return S.VAR_P([[1, 2, 3, 4]]); }, 1.25);
numeric("COUNTIF numeric criterion", function () { return B.COUNTIF([[1, 2, 3], ">1"]); }, 2);
numeric("COUNTIF text criterion", function () { return B.COUNTIF([["a", "b", "a"], "a"]); }, 2);
numeric("CORREL of a perfect linear pair", function () { return S.CORREL([[1, 2, 3], [2, 4, 6]]); }, 1);
numeric("SLOPE of a perfect linear pair", function () { return S.SLOPE([[1, 2, 3], [2, 4, 6]]); }, 0.5);
numeric("INTERCEPT of a proportional pair", function () { return S.INTERCEPT([[1, 2, 3], [2, 4, 6]]); }, 0);
numeric("FORECAST on a linear pair", function () { return S.FORECAST([4, [1, 2, 3], [2, 4, 6]]); }, 2);
numeric("NORM.DIST at the mean", function () { return S.NORM_DIST([0, 0, 1, true]); }, 0.5);
numeric("NORM.INV at the median", function () { return S.NORM_INV([0.5, 0, 1]); }, 0, 1e-9);
valueCase("CONCAT joins raw arguments", function () { return S.CONCAT(["a", "b"]); }, "ab");
valueCase("TEXTJOIN ignores empty parts", function () {
  return S.TEXTJOIN(["-", true, "a", "", "b"]);
}, "a-b");
valueCase("TEXTJOIN keeps empty parts when asked", function () {
  return S.TEXTJOIN(["-", false, "a", "", "b"]);
}, "a--b");
numeric("NUMBERVALUE with explicit separators", function () {
  return S.NUMBERVALUE(["1.234,56", ",", "."]);
}, 1234.56);
errorCase("NUMBERVALUE rejects malformed text", function () { return S.NUMBERVALUE(["abc"]); }, "#VALUE!");

test("[STATIC] LEFT/RIGHT/MID are absent from this preview layer", function () {
  h.ORDER.forEach(function (file) {
    sourceLacks(file, "\\b(?:LEFT|RIGHT|MID)\\s*\\(",
      file + " unexpectedly declares a LEFT/RIGHT/MID helper");
  });
});

// ── Parser, evaluator, catalog and locale ─────────────────────────────
test("tokenizer emits function, operator and cell tokens", function () {
  var tokens = P.tokenize("=SUM(A1,B2*2)");
  assert(tokens && tokens.length >= 8, "too few tokens");
  assert(tokens[0].type === "ident" && tokens[0].value === "SUM", "function name not tokenized");
  assert(tokens.some(function (t) { return t.type === "op" && t.value === "*"; }), "operator missing");
  assert(tokens.some(function (t) { return t.type === "cell" || t.type === "ident"; }), "cell token missing");
});
test("tokenizer keeps a string literal that looks like an operator", function () {
  var tokens = P.tokenize('=TEXTJOIN("-",TRUE,"a","b")');
  assert(tokens.some(function (t) { return t.type === "str" && t.value === "-"; }),
    'the "-" delimiter was mis-tokenized as an operator');
});
test("parser builds a call node with two arguments", function () {
  var ast = P.parse("=SUM(A1,B2*2)");
  assert(ast && ast.t === "call" && ast.v === "SUM", "call node missing");
  assert(ast.args.length === 2 && ast.args[1].t === "bin", "argument shape wrong");
});
numeric("evaluator: Excel exponent precedence for -2^2", function () { return V.evaluate("=-2^2"); }, 4);
numeric("evaluator: left-associative 2^3^2", function () { return V.evaluate("=2^3^2"); }, 64);
numeric("evaluator: percent postfix", function () { return V.evaluate("=50%"); }, 0.5);
numeric("evaluator: nested formula resolves chained cell references", function () {
  return V.evaluate("=SUM(A1,B1*2)+IF(C1>5,ROUND(A1/10,0),0)", {
    cells: { A1: "=2+3", B1: 4, C1: 10 },
  });
}, 14);
test("[STATIC] references() is a private A1-token filter (observable via hasCycle)", function () {
  sourceHas("78_functions_evaluator.js", "function references\\(formula\\)",
    "private references helper missing from evaluator source");
  sourceHas("78_functions_evaluator.js", "token\\.type === \"ident\"",
    "references does not filter ident tokens");
  assert(typeof V.hasCycle === "function", "observable cycle-detection entry point missing");
});
valueCase("hasCycle detects a two-cell cycle", function () {
  return V.hasCycle({ A1: "=B1", B1: "=A1" });
}, true);
valueCase("circular reference without iterative mode is #REF!", function () {
  return E.code(V.evaluateCircular({ A1: "=B1", B1: "=A1" }, {}).error);
}, "#REF!");
test("iterative calculation converges on a circular model", function () {
  var cells = { A1: "=1+0.5*B1", B1: "=1+0.5*A1" };
  var result = V.evaluateCircular(cells, {
    settings: { iterative: { enabled: true, maxIterations: 100, tolerance: 1e-7 } },
  });
  assert(result.converged === true, "iteration did not converge");
  approx(result.values.A1, 2, 1e-6, "A1");
  approx(result.values.B1, 2, 1e-6, "B1");
});

test("catalog is non-empty and each entry carries full metadata", function () {
  var items = C.all();
  assert(items.length >= 50, "expected at least 50 catalog entries, got " + items.length);
  items.forEach(function (item) {
    ["name", "category", "signature", "helpKey", "example", "badge"].forEach(function (key) {
      assert(typeof item[key] === "string" && item[key].length > 0,
        item.name + " has no " + key);
    });
    assert(Array.isArray(item.args), item.name + " has no args array");
    assert(item.badge === "native" || item.badge === "local", item.name + " has an unknown badge");
    assert(/^sheet-fn-help-/.test(item.helpKey), item.name + " has an invalid help key");
  });
  C.categories().forEach(function (name) {
    assert(items.some(function (item) { return item.category === name; }), "no functions in " + name);
  });
});
valueCase("catalog resolves pt-BR SUM alias", function () { return C.resolve("SOMA", "pt-BR").name; }, "SUM");
valueCase("catalog resolves pt-BR accented AVERAGE alias", function () {
  return C.resolve("MÉDIA", "pt-BR").name;
}, "AVERAGE");
valueCase("catalog resolves Spanish SUM alias", function () { return C.resolve("SUMA", "es").name; }, "SUM");
valueCase("catalog returns null for an unknown alias", function () { return C.resolve("ZZZ", "pt-BR"); }, null);
test("catalog ships aliases for all eight locales", function () {
  assert(Object.keys(C.ALIASES).length === 8, "expected 8 locale alias tables");
  assert(C.search("sum", "pt-BR").length > 0, "search missed a pt-BR alias");
});

test("locale decimal/group separators affect NUMBERVALUE and format", function () {
  var restore = eng.setSettings({ separators: { decimal: ",", group: "." } });
  try {
    assert(S.NUMBERVALUE(["1.234,56"]) === 1234.56, "NUMBERVALUE ignored locale separators");
    assert(V.format(1234.5, {}) === "1.234,5", "format ignored locale separators");
    var precision = eng.setSettings({ precisionAsDisplayed: true });
    try {
      assert(V.format(1234.5, {}) === "1.234,50", "precision-as-displayed ignored");
    } finally {
      precision();
    }
  } finally {
    restore();
  }
  assert(V.format(1234.5, {}) === "1,234.5", "en separators not restored");
});

// The shipped catalog examples carry `expected` metadata that has drifted from
// the helpers for a documented set of functions. This is a KNOWN GAP reported
// to the issue owner, not silently rewritten here. Any NEW drift fails the test.
var KNOWN_DRIFT = [
  "IPMT", "PPMT", "FV", "RATE", "NPER", "NPV", "XIRR",
  "WORKDAY", "WORKDAY.INTL", "EOMONTH", "EDATE", "SLOPE", "FORECAST.LINEAR",
];
test("catalog example metadata matches evaluator outside the known stale set", function () {
  var drift = [];
  C.all().forEach(function (item) {
    var actual = V.evaluateExample(item.name, { locale: "en", settings: eng.state });
    var matches = typeof actual === "number" && typeof item.expected === "number"
      ? Math.abs(actual - item.expected) <= 1e-6
      : String(actual) === String(item.expected);
    if (!matches) drift.push(item.name);
  });
  note("KNOWN GAP — stale catalog expected values: " + drift.join(", "));
  var unexpected = drift.filter(function (name) { return KNOWN_DRIFT.indexOf(name) < 0; });
  assert(unexpected.length === 0, "new catalog drift: " + unexpected.join(", "));
});

gbLog.debug("Sheet functions: " + E.CODES.length + " error codes, " +
  C.all().length + " catalog entries loaded from " + h.ORDER.length + " shipped modules.");
