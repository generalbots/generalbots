"use strict";
/* Issue #2434 - base functions needed for the locale aliases and the Excel
 * error semantics. These are native in botsheet-core; the JS copies here are
 * only used by the local preview. */
(function () {
  var E = window.SheetFormulaErrors;

  function SUM(values) {
    var numbers = E.numbers(values);
    if (E.isError(numbers)) return numbers;
    return E.finite(numbers.reduce(function (a, b) { return a + b; }, 0));
  }

  function AVERAGE(values) {
    var numbers = E.numbers(values);
    if (E.isError(numbers)) return numbers;
    if (!numbers.length) return E.err("#DIV/0!");
    return E.finite(numbers.reduce(function (a, b) { return a + b; }, 0) / numbers.length);
  }

  function COUNT(values) {
    var flat = E.flatten(values);
    return flat.filter(function (value) {
      return typeof value === "number" && Number.isFinite(value);
    }).length;
  }

  function COUNTIF(values) {
    var range = E.flatten([values[0]]);
    var criterion = values[1];
    if (E.isError(criterion)) return criterion;
    var text = String(criterion);
    var match = text.match(/^(<=|>=|<>|<|>|=)?(-?\d+(?:\.\d+)?)$/);
    return range.filter(function (value) {
      if (E.isError(value)) return false;
      if (match) {
        var expected = Number(match[2]);
        var operator = match[1] || "=";
        if (typeof value !== "number") return false;
        switch (operator) {
          case "=": return value === expected;
          case "<>": return value !== expected;
          case "<": return value < expected;
          case ">": return value > expected;
          case "<=": return value <= expected;
          default: return value >= expected;
        }
      }
      return String(value) === text;
    }).length;
  }

  function MAX(values) {
    var numbers = E.numbers(values);
    if (E.isError(numbers)) return numbers;
    return numbers.length ? E.finite(Math.max.apply(null, numbers)) : 0;
  }

  function MIN(values) {
    var numbers = E.numbers(values);
    if (E.isError(numbers)) return numbers;
    return numbers.length ? E.finite(Math.min.apply(null, numbers)) : 0;
  }

  function IF(values) {
    var condition = values[0];
    if (E.isError(condition)) return condition;
    if (typeof condition === "string") return E.err("#VALUE!");
    return condition ? values[1] : (values.length > 2 ? values[2] : false);
  }

  function ROUND(values) {
    var value = E.num(values[0]);
    if (E.isError(value)) return value;
    var digits = values.length > 1 ? E.num(values[1]) : 0;
    if (E.isError(digits)) return digits;
    var factor = Math.pow(10, Math.max(-100, Math.min(100, Math.floor(digits))));
    return E.finite(Math.round((value + Number.EPSILON) * factor) / factor);
  }

  window.SheetFunctionsBase = {
    SUM: SUM, AVERAGE: AVERAGE, COUNT: COUNT, COUNTIF: COUNTIF,
    IF: IF, IFERROR: E.IFERROR, IFNA: E.IFNA, ISERROR: E.ISERROR,
    ISNA: E.ISNA, MAX: MAX, MIN: MIN, ROUND: ROUND,
  };
})();
