"use strict";
/* Issue #2434 - Excel error taxonomy used by the local Sheets preview.
 * Errors are values, not exceptions: they travel through nested calls and are
 * only consumed by IFERROR / IFNA / ISERROR / ISNA. Text is never coerced to
 * zero by the numeric helpers. */
(function () {
  var CODES = ["#DIV/0!", "#VALUE!", "#REF!", "#NAME?", "#N/A", "#NUM!", "#NULL!"];

  function err(code) {
    return CODES.indexOf(code) >= 0 ? { __gbError: true, code: code } : err("#VALUE!");
  }

  function isError(value) {
    return Boolean(value) && typeof value === "object" && value.__gbError === true;
  }

  function code(value) {
    return isError(value) ? value.code : null;
  }

  function format(value) {
    return isError(value) ? value.code : String(value);
  }

  function firstError(values) {
    if (!Array.isArray(values)) return isError(values) ? values : null;
    for (var i = 0; i < values.length; i += 1) {
      if (isError(values[i])) return values[i];
      if (Array.isArray(values[i])) {
        var nested = firstError(values[i]);
        if (nested) return nested;
      }
    }
    return null;
  }

  /* Strict numeric conversion. Blank cells (undefined/null) are zero because
   * Excel treats an empty cell as zero; strings are NEVER parsed here. */
  function num(value) {
    if (isError(value)) return value;
    if (value === undefined || value === null) return 0;
    if (typeof value === "number") {
      return Number.isFinite(value) ? value : err("#NUM!");
    }
    if (typeof value === "boolean") return value ? 1 : 0;
    return err("#VALUE!");
  }

  function flatten(values) {
    var result = [];
    values.forEach(function (value) {
      if (Array.isArray(value)) result = result.concat(flatten(value));
      else result.push(value);
    });
    return result;
  }

  function numbers(values) {
    var failed = firstError(values);
    if (failed) return failed;
    var flat = flatten(values);
    var out = [];
    for (var i = 0; i < flat.length; i += 1) {
      var converted = num(flat[i]);
      if (isError(converted)) return converted;
      out.push(converted);
    }
    return out;
  }

  function divide(a, b) {
    var failed = firstError([a, b]);
    if (failed) return failed;
    var x = num(a);
    if (isError(x)) return x;
    var y = num(b);
    if (isError(y)) return y;
    if (y === 0) return err("#DIV/0!");
    return x / y;
  }

  function finite(value) {
    return Number.isFinite(value) ? value : err("#NUM!");
  }

  function positiveRoot(value) {
    if (isError(value)) return value;
    if (value < 0) return err("#NUM!");
    return Math.sqrt(value);
  }

  function checkedLog(value) {
    if (isError(value)) return value;
    if (value <= 0) return err("#NUM!");
    return Math.log(value);
  }

  function IFERROR(value, fallback) {
    return isError(value) ? fallback : value;
  }

  function IFNA(value, fallback) {
    return code(value) === "#N/A" ? fallback : value;
  }

  function ISERROR(value) {
    return isError(value);
  }

  function ISNA(value) {
    return code(value) === "#N/A";
  }

  function op(kind, left, right) {
    var failed = firstError([left, right]);
    if (failed) return failed;
    var a = num(left);
    if (isError(a)) return a;
    var b = num(right);
    if (isError(b)) return b;
    switch (kind) {
      case "+": return finite(a + b);
      case "-": return finite(a - b);
      case "*": return finite(a * b);
      case "/": return divide(a, b);
      case "^": return finite(Math.pow(a, b));
      default: return err("#NAME?");
    }
  }

  window.SheetFormulaErrors = {
    CODES: CODES,
    err: err,
    isError: isError,
    code: code,
    format: format,
    firstError: firstError,
    num: num,
    flatten: flatten,
    numbers: numbers,
    divide: divide,
    finite: finite,
    positiveRoot: positiveRoot,
    checkedLog: checkedLog,
    IFERROR: IFERROR,
    IFNA: IFNA,
    ISERROR: ISERROR,
    ISNA: ISNA,
    op: op,
  };
})();
