/**
 * Parse -> evaluate -> format round-trip invariants (#2572).
 * Every assertion below executes the shipped IIFEs loaded by 00_harness.js.
 */
"use strict";

var h = require("./00_harness");
var e = h.engine;
var P = e.parser;
var E = e.evaluator;
var C = e.catalog;
var S = e.settings;
var D = e.date;
var assert = h.assert;
var approx = h.approx;
var errCode = h.errCode;

h.test("nine shipped IIFEs are loaded in one shared sandbox", function () {
  assert(h.ORDER.length === 9, "expected nine shipped modules, got " + h.ORDER.length);
  [
    "SheetFormulaErrors", "SheetFunctionsFinancial", "SheetFunctionsDate",
    "SheetFunctionsBase", "SheetFunctionsStatText", "SheetFunctionsCatalog",
    "SheetFunctionsParser", "SheetFunctionsEvaluator", "SheetFunctionsSettings",
  ].forEach(function (name) {
    assert(e.window[name] && typeof e.window[name] === "object", name + " was not exported");
  });
  assert(e.settings.storageKey() === "gb-sheet-functions:" + e.fixedSheetId,
    "settings did not load from the real 79 module (fixed uid stub)");
});

[0, 1, 42.75, 0.3, 0.0001].forEach(function (value) {
  h.test("numeric literal round-trip: " + value, function () {
    var ast = P.parse(String(value));
    assert(ast && ast.t === "num", "tokenizer lost the numeric literal " + value);
    approx(ast.v, value, 0, "parsed AST value");
    approx(E.evaluate(String(value)), value, 0, "evaluated literal");
    var text = E.format(value);
    approx(E.evaluate(text), value, 1e-9, "format -> parse -> evaluate");
  });
});

e.errors.CODES.forEach(function (code) {
  h.test("error literal round-trip: " + code, function () {
    var ast = P.parse(code);
    assert(ast && ast.t === "err" && ast.v === code, "parser lost " + code);
    var value = E.evaluate(code);
    assert(errCode(value) === code, "evaluator lost " + code);
    var text = E.format(value);
    assert(text === code, "formatter rewrote " + code + " to " + text);
    var back = P.parse(text);
    assert(back && back.t === "err" && back.v === code, "re-parse lost " + code);
  });
});

Object.keys(C.ALIASES).forEach(function (locale) {
  var sumAlias = Object.keys(C.ALIASES[locale]).filter(function (key) {
    return C.ALIASES[locale][key] === "SUM";
  })[0];
  h.test("locale function names resolve to canonical SUM: " + locale, function () {
    if (!sumAlias) {
      assert(locale === "en", "unexpected locale without a SUM alias: " + locale);
      assert(C.resolve("SUM", locale).name === "SUM", "en canonical name changed");
    } else {
      var resolved = C.resolve(sumAlias, locale);
      assert(resolved && resolved.name === "SUM", "alias " + sumAlias + " did not resolve");
      assert(E.evaluate("=" + sumAlias + "(2,3)", { locale: locale }) === 5,
        "alias formula did not evaluate for " + locale);
      assert(C.resolve(sumAlias, locale).name === C.resolve("SUM", locale).name,
        "alias and canonical names diverged for " + locale);
    }
    assert(E.evaluate("=SUM(2,3)", { locale: locale }) === 5,
      "canonical name is not locale-stable for " + locale);
  });

  h.test("format -> NUMBERVALUE separator inverse: " + locale, function () {
    var sep = S.SEPARATORS[locale];
    var text = E.format(1234.5, { settings: { separators: sep, precisionAsDisplayed: false } });
    assert(text.indexOf(sep.group) >= 0, "group mark missing from " + text);
    assert(text.indexOf(sep.decimal) >= 0, "decimal mark missing from " + text);
    var back = e.statText.NUMBERVALUE([text, sep.decimal, sep.group]);
    approx(back, 1234.5, 1e-9, "NUMBERVALUE did not invert format for " + locale);
  });
});

h.test("locale aliases are scoped: en rejects a pt-BR function name", function () {
  assert(C.resolve("SOMA", "en") === null, "en unexpectedly resolved SOMA");
  assert(errCode(E.evaluate("=SOMA(1,2)", { locale: "en" })) === "#NAME?",
    "en should report #NAME? for SOMA");
});

h.test("constant aliases and array literals survive evaluation", function () {
  assert(E.evaluate("=TRUE") === true && E.evaluate("=FALSE") === false,
    "boolean literals changed");
  assert(E.evaluate("=SUM([1,2,3])") === 6, "array literal changed");
  assert(E.evaluate("=SUM(A1:B1)", { cells: { A1: "=B1+1", B1: 2 } }) === 5,
    "range reference changed");
});

h.test("1900 serial system round-trips dates and keeps the leap-year quirk", function () {
  e.withPersistedSettings({ dateSystem: "1900" }, function () {
    assert(D.serial(e.makeDate(2026, 0, 1)) === 46023, "1900 epoch offset drifted");
    approx(D.serial(D.dateFromSerial(46023)), 46023, 0, "serial -> date -> serial");
    assert(D.serial("1900-02-28") === 59, "serial 59 changed");
    assert(D.serial("1900-03-01") === 61, "1900 leap-day skip (serial 60) changed");
    assert(errCode(D.serial("1900-02-29")) === "#VALUE!", "invalid leap day was accepted");
  });
});

h.test("1904 serial system round-trips dates with a stable 1462-day offset", function () {
  var serial1900 = e.withPersistedSettings({ dateSystem: "1900" }, function () {
    return D.serial(e.makeDate(2026, 0, 1));
  });
  e.withPersistedSettings({ dateSystem: "1904" }, function () {
    assert(D.serial(e.makeDate(2026, 0, 1)) === 44561, "1904 epoch offset drifted");
    assert(serial1900 - D.serial(e.makeDate(2026, 0, 1)) === 1462, "epoch offset changed");
    [0, 1, 59, 60, 46100].forEach(function (serial) {
      approx(D.serial(D.dateFromSerial(serial)), serial, 0, "1904 round-trip " + serial);
    });
  });
});

h.test("date system setting is honored by shipped date functions", function () {
  var a = e.withPersistedSettings({ dateSystem: "1900" }, function () {
    return D.EOMONTH(["2026-02-10", 1]);
  });
  var b = e.withPersistedSettings({ dateSystem: "1904" }, function () {
    return D.EOMONTH(["2026-02-10", 1]);
  });
  assert(a - b === 1462, "EOMONTH ignored the persisted date system");
});

h.test("precision-as-displayed changes only the formatted representation", function () {
  var raw = E.evaluate("=1.2345");
  assert(E.format(raw, { settings: { precisionAsDisplayed: false, separators: { decimal: ".", group: "," } } }) === "1.2345",
    "raw precision changed");
  assert(E.format(raw, { settings: { precisionAsDisplayed: true, separators: { decimal: ".", group: "," } } }) === "1.23",
    "precision-as-displayed was ignored");
});

h.test("circular references need iterative settings and converge when enabled", function () {
  var cells = { A1: "=1+0.5*B1", B1: "=1+0.5*A1" };
  var blocked = E.evaluateCircular(cells, {});
  assert(errCode(blocked.error) === "#REF!" && blocked.converged === false,
    "a cycle was allowed without iterative mode");
  var converged = E.evaluateCircular(cells, {
    settings: { iterative: { enabled: true, maxIterations: 100, tolerance: 1e-7 } },
  });
  assert(converged.converged === true, "iterative model did not converge");
  approx(converged.values.A1, 2, 1e-6, "A1");
  approx(converged.values.B1, 2, 1e-6, "B1");
});
