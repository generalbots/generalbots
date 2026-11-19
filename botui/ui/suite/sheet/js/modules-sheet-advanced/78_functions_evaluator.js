"use strict";
/* Issue #2434 - local-only evaluator. Composes the pure function modules with
 * the parser, resolves locale aliases and implements iterative recalculation
 * for circular references. No network call and no workbook write. */
(function () {
  var E = window.SheetFormulaErrors;
  var P = window.SheetFunctionsParser;
  var C = window.SheetFunctionsCatalog;
  var B = window.SheetFunctionsBase;
  var F = window.SheetFunctionsFinancial;
  var D = window.SheetFunctionsDate;
  var S = window.SheetFunctionsStatText;

  var IMPL = {
    SUM: B.SUM, AVERAGE: B.AVERAGE, COUNT: B.COUNT, COUNTIF: B.COUNTIF,
    IF: B.IF, MAX: B.MAX, MIN: B.MIN, ROUND: B.ROUND,
    IFERROR: function (a) { return E.IFERROR(a[0], a[1]); },
    IFNA: function (a) { return E.IFNA(a[0], a[1]); },
    ISERROR: function (a) { return E.ISERROR(a[0]); },
    ISNA: function (a) { return E.ISNA(a[0]); },
    PMT: F.PMT, IPMT: F.IPMT, PPMT: F.PPMT, FV: F.FV, PV: F.PV,
    RATE: F.RATE, NPER: F.NPER, NPV: F.NPV, IRR: F.IRR, MIRR: F.MIRR,
    XIRR: F.XIRR, XNPV: F.XNPV, SLN: F.SLN, SYD: F.SYD, DB: F.DB,
    DDB: F.DDB, EFFECT: F.EFFECT, NOMINAL: F.NOMINAL,
    NETWORKDAYS: D.NETWORKDAYS, "NETWORKDAYS.INTL": D.NETWORKDAYS_INTL,
    WORKDAY: D.WORKDAY, "WORKDAY.INTL": D.WORKDAY_INTL,
    EOMONTH: D.EOMONTH, EDATE: D.EDATE, YEARFRAC: D.YEARFRAC,
    WEEKNUM: D.WEEKNUM, ISOWEEKNUM: D.ISOWEEKNUM, DATEDIF: D.DATEDIF,
    CORREL: S.CORREL, SLOPE: S.SLOPE, INTERCEPT: S.INTERCEPT,
    "FORECAST.LINEAR": S.FORECAST, "STDEV.S": S.STDEV_S, "STDEV.P": S.STDEV_P,
    "VAR.S": S.VAR_S, "VAR.P": S.VAR_P, "MODE.SNGL": S.MODE_SNGL,
    "NORM.DIST": S.NORM_DIST, "NORM.INV": S.NORM_INV,
    TEXTJOIN: S.TEXTJOIN, CONCAT: S.CONCAT, NUMBERVALUE: S.NUMBERVALUE,
  };

  /* These functions decide themselves which argument errors are consumed, so
   * the evaluator must not short-circuit them. IF also needs its unused branch
   * to stay unevaluated. */
  var LAZY = { IF: true, IFERROR: true, IFNA: true, ISERROR: true, ISNA: true };

  function settingsOf(options) {
    return (options && options.settings) ||
      (window.SheetFunctionsSettings ? window.SheetFunctionsSettings.state() : null);
  }

  function columnOf(ref) {
    var letters = String(ref).toUpperCase().replace(/\$/, "").match(/^[A-Z]+/);
    if (!letters) return -1;
    var value = 0;
    letters[0].split("").forEach(function (letter) {
      value = value * 26 + (letter.charCodeAt(0) - 64);
    });
    return value - 1;
  }

  function rowOf(ref) {
    var match = String(ref).match(/([1-9][0-9]*)$/);
    return match ? Number(match[1]) : -1;
  }

  function cellKey(col, row) {
    var letters = "";
    var value = col + 1;
    while (value > 0) {
      var remainder = (value - 1) % 26;
      letters = String.fromCharCode(65 + remainder) + letters;
      value = Math.floor((value - 1) / 26);
    }
    return letters + row;
  }

  function resolveCell(ref, context) {
    if (Object.prototype.hasOwnProperty.call(context.cache, ref)) return context.cache[ref];
    var cells = context.cells || {};
    var raw = cells[ref];
    if (raw === undefined || raw === null) return 0;
    if (typeof raw !== "string" || raw.charAt(0) !== "=") return raw;
    if (context.visiting[ref]) {
      var settings = settingsOf(context);
      return settings && settings.iterative && settings.iterative.enabled
        ? (Number.isFinite(context.cache[ref]) ? context.cache[ref] : 0)
        : E.err("#REF!");
    }
    context.visiting[ref] = true;
    var value = evaluate(raw, {
      locale: context.locale, cells: cells, cache: context.cache,
      visiting: context.visiting, settings: context.settings,
    });
    delete context.visiting[ref];
    return value;
  }

  function rangeValues(from, to, context) {
    var startCol = columnOf(from), endCol = columnOf(to);
    var startRow = rowOf(from), endRow = rowOf(to);
    if (startCol < 0 || endCol < 0 || startRow < 0 || endRow < 0) return E.err("#REF!");
    var result = [];
    for (var row = Math.min(startRow, endRow); row <= Math.max(startRow, endRow); row += 1) {
      for (var col = Math.min(startCol, endCol); col <= Math.max(startCol, endCol); col += 1) {
        var value = resolveCell(cellKey(col, row), context);
        result.push(value);
        if (E.isError(value)) return value;
      }
    }
    return result;
  }

  function compare(kind, left, right) {
    var failed = E.firstError([left, right]);
    if (failed) return failed;
    if (typeof left === "string" || typeof right === "string") {
      var a = E.format(left), b = E.format(right);
      switch (kind) {
        case "=": return a === b;
        case "<>": return a !== b;
        case "<": return a < b;
        case ">": return a > b;
        case "<=": return a <= b;
        default: return a >= b;
      }
    }
    var x = E.num(left), y = E.num(right);
    if (E.isError(x)) return x;
    if (E.isError(y)) return y;
    switch (kind) {
      case "=": return x === y;
      case "<>": return x !== y;
      case "<": return x < y;
      case ">": return x > y;
      case "<=": return x <= y;
      default: return x >= y;
    }
  }

  function evalNode(ast, context) {
    if (!ast) return E.err("#VALUE!");
    switch (ast.t) {
      case "num": return ast.v;
      case "str": return ast.v;
      case "err": return E.err(ast.v);
      case "bool": return ast.v;
      case "cell": return resolveCell(ast.v, context);
      case "range": return rangeValues(ast.from, ast.to, context);
      case "ident": return E.err("#NAME?");
      case "array":
        return ast.items.map(function (item) { return evalNode(item, context); });
      case "un": {
        var value = E.num(evalNode(ast.operand, context));
        return E.isError(value) ? value : E.finite(ast.v === "-" ? -value : value);
      }
      case "post": {
        var percent = E.num(evalNode(ast.operand, context));
        return E.isError(percent) ? percent : E.finite(percent / 100);
      }
      case "bin": {
        var left = evalNode(ast.l, context);
        var right = evalNode(ast.r, context);
        var failed = E.firstError([left, right]);
        if (failed) return failed;
        if (["=", "<>", "<", ">", "<=", ">="].indexOf(ast.v) >= 0) {
          return compare(ast.v, left, right);
        }
        if (ast.v === "&") {
          if (Array.isArray(left) || Array.isArray(right)) return E.err("#VALUE!");
          if (typeof left === "string" || typeof right === "string") return E.format(left) + E.format(right);
          return E.format(E.num(left)) + E.format(E.num(right));
        }
        return E.op(ast.v, left, right);
      }
      case "call": {
        var fn = C.resolve(ast.v, context.locale);
        if (!fn) return E.err("#NAME?");
        var args = ast.args.map(function (arg) { return evalNode(arg, context); });
        if (!LAZY[fn.name]) {
          var error = E.firstError(args);
          if (error) return error;
        }
        var implementation = IMPL[fn.name];
        if (!implementation) return E.err("#NAME?");
        return implementation(args);
      }
      default: return E.err("#VALUE!");
    }
  }

  function evaluate(formula, options) {
    var context = {
      locale: (options && options.locale) || "en",
      cells: (options && options.cells) || {},
      cache: (options && options.cache) || {},
      visiting: (options && options.visiting) || {},
      settings: settingsOf(options),
    };
    var ast = P.parse(formula);
    if (!ast) return E.err("#VALUE!");
    var value = evalNode(ast, context);
    context.cache.__last = value;
    return value;
  }

  function references(formula) {
    var tokens = P.tokenize(formula) || [];
    return tokens.filter(function (token) {
      return token.type === "ident" && /^[A-Za-z]{1,3}[1-9][0-9]*$/.test(token.value);
    }).map(function (token) { return token.value.toUpperCase(); });
  }

  function hasCycle(cells) {
    var state = {};
    function visit(ref) {
      if (state[ref] === 1) return true;
      if (state[ref] === 2) return false;
      state[ref] = 1;
      var raw = cells[ref];
      if (typeof raw === "string" && raw.charAt(0) === "=") {
        var deps = references(raw);
        for (var i = 0; i < deps.length; i += 1) {
          if (visit(deps[i])) return true;
        }
      }
      state[ref] = 2;
      return false;
    }
    return Object.keys(cells).some(visit);
  }

  function evaluateCircular(cells, options) {
    var settings = settingsOf(options) || { iterative: { enabled: false, maxIterations: 100, tolerance: 1e-7 } };
    var iterative = Boolean(settings.iterative && settings.iterative.enabled);
    if (hasCycle(cells) && !iterative) {
      return { error: E.err("#REF!"), iterations: 0, converged: false };
    }
    var refs = Object.keys(cells);
    var cache = {};
    refs.forEach(function (ref) { cache[ref] = Number(cells[ref]) || 0; });
    var limit = Math.min(1000, Math.max(1, Math.floor(settings.iterative && settings.iterative.maxIterations || 100)));
    var tolerance = settings.iterative && Number.isFinite(Number(settings.iterative.tolerance))
      ? Math.abs(Number(settings.iterative.tolerance)) : 1e-7;
    var iterations = 0;
    var converged = false;
    for (var pass = 0; pass < limit; pass += 1) {
      iterations = pass + 1;
      var next = {};
      var largest = 0;
      for (var i = 0; i < refs.length; i += 1) {
        var ref = refs[i];
        var value = evaluate(String(cells[ref]), {
          locale: options && options.locale, cells: cells, cache: cache,
          visiting: {}, settings: settings,
        });
        next[ref] = value;
        if (E.isError(value)) return { values: next, error: value, iterations: iterations, converged: false };
        var previous = Number(cache[ref]) || 0;
        largest = Math.max(largest, Math.abs(Number(value) - previous));
      }
      cache = next;
      if (largest <= tolerance) { converged = true; break; }
    }
    return { values: cache, iterations: iterations, converged: converged };
  }

  function format(value, options) {
    if (E.isError(value)) return E.format(value);
    if (typeof value === "boolean") return value ? "TRUE" : "FALSE";
    if (Array.isArray(value)) return "[" + value.map(function (item) { return format(item, options); }).join(", ") + "]";
    if (typeof value !== "number") return String(value);
    var settings = settingsOf(options);
    var decimals = settings && settings.precisionAsDisplayed ? 2 : null;
    var text = decimals === null ? String(Number(value.toPrecision(12))) : value.toFixed(decimals);
    if (settings && settings.separators) {
      var group = settings.separators.group;
      var decimal = settings.separators.decimal;
      if (group && decimal && group !== decimal) {
        var pieces = text.split(".");
        pieces[0] = pieces[0].replace(/\B(?=(\d{3})+(?!\d))/g, group);
        text = pieces.join(decimal);
      }
    }
    return text;
  }

  function evaluateExample(name, options) {
    var fn = C.get(name);
    if (!fn) return E.err("#NAME?");
    return evaluate(fn.example, options);
  }

  window.SheetFunctionsEvaluator = {
    evaluate: evaluate, evaluateCircular: evaluateCircular,
    evaluateExample: evaluateExample, format: format,
    IMPL: IMPL, hasCycle: hasCycle,
  };
})();
