"use strict";
/* Sheet advanced module: 26a_audit_engine — evaluator half of the bounded local
   formula stepper (#2491). Uses the parser from 26_audit_eval.js. */
(function () {
  var I = window.__SheetAuditEvalInternals;
  var text = I.text;
  var isErr = I.isErr;
  var err = I.err;
  var display = I.display;
  var numeric = I.numeric;
  var truthy = I.truthy;
  var parse = I.parse;
  var MAX_NODES = I.MAX_NODES;
  var MAX_RANGE = I.MAX_RANGE;

  function pushStep(steps, node, value, depth, source) {
    if (steps.length >= MAX_NODES) return;
    steps.push({
      start: node.start, end: node.end,
      expr: text(source).slice(node.start, node.end),
      value: display(value), depth: depth,
    });
  }

  function rangeValues(node, ctx, state) {
    var out = [];
    var a = node.from;
    var minRow = Math.min(a.row, node.row);
    var maxRow = Math.max(a.row, node.row);
    var minCol = Math.min(a.col, node.col);
    var maxCol = Math.max(a.col, node.col);
    for (var r = minRow; r <= maxRow; r++) {
      for (var c = minCol; c <= maxCol; c++) {
        if (out.length >= MAX_RANGE) { state.truncated = true; return out; }
        var cell = ctx.resolve(node.sheetName || "", r, c);
        out.push(cell && cell.value);
      }
    }
    return out;
  }

  function evalNode(node, ctx, steps, visiting, depth, state) {
    var source = state.source;
    var value;
    if (node.type === "literal") {
      value = node.value;
    } else if (node.type === "name") {
      value = err("#NAME?");
    } else if (node.type === "group") {
      value = evalNode(node.value, ctx, steps, visiting, depth + 1, state);
    } else if (node.type === "unary") {
      var operand = evalNode(node.value, ctx, steps, visiting, depth + 1, state);
      var n = numeric(operand);
      value = isErr(operand) ? operand : (node.op === "-" ? -n : n);
    } else if (node.type === "ref") {
      var key = (node.sheetName || "") + "!" + node.row + "," + node.col;
      var cell = ctx.resolve(node.sheetName || "", node.row, node.col);
      value = cell ? cell.value : "";
      if ((value === "" || value == null) && cell && cell.formula && !visiting[key]) {
        var nested = parse(cell.formula);
        if (!nested.error) {
          visiting[key] = true;
          value = evalNode(nested.ast, ctx, steps, visiting, depth + 1, { source: cell.formula, truncated: false });
          delete visiting[key];
        }
      }
      if (value === "" || value == null) value = 0;
    } else if (node.type === "range") {
      value = rangeValues(node, ctx, state);
    } else if (node.type === "binary") {
      var left = evalNode(node.left, ctx, steps, visiting, depth + 1, state);
      var right = evalNode(node.right, ctx, steps, visiting, depth + 1, state);
      value = binary(node.op, left, right);
    } else if (node.type === "call") {
      value = call(node, ctx, steps, visiting, depth, state);
    } else {
      value = err("#VALUE!");
    }
    pushStep(steps, node, value, depth, source);
    return value;
  }

  function binary(op, left, right) {
    if (isErr(left)) return left;
    if (isErr(right)) return right;
    if (op === "&") return text(left) + text(right);
    var a = numeric(left);
    var b = numeric(right);
    if (op === "^") return isNaN(Math.pow(a, b)) ? err("#NUM!") : Math.pow(a, b);
    if (op === "/") return b === 0 || b === null ? err("#DIV/0!") : a / b;
    if (op === "*") return a * b;
    if (op === "+" || op === "-") return (a === null ? 0 : a) + (op === "+" ? 1 : -1) * (b === null ? 0 : b);
    if (a === null || b === null) {
      var x = text(left);
      var y = text(right);
      a = x; b = y;
    }
    switch (op) {
      case "=": return a === b;
      case "<>": case "!=": return a !== b;
      case "<": return a < b;
      case ">": return a > b;
      case "<=": return a <= b;
      case ">=": return a >= b;
      default: return err("#NAME?");
    }
  }

  function flatten(values) {
    var out = [];
    values.forEach(function (v) {
      if (Array.isArray(v)) out = out.concat(flatten(v));
      else out.push(v);
    });
    return out;
  }
  function criterionMatch(value, criterion) {
    var m = /^(<=|>=|<>|>|<|=)?(.*)$/.exec(text(criterion).trim());
    var op = m[1] || "=";
    var expected = m[2];
    var a = numeric(value);
    var b = numeric(expected);
    if (a !== null && b !== null) {
      switch (op) {
        case "=": return a === b;
        case "<>": return a !== b;
        case ">": return a > b;
        case "<": return a < b;
        case ">=": return a >= b;
        case "<=": return a <= b;
      }
    }
    var x = text(value).toLowerCase();
    var y = expected.toLowerCase();
    if (op === "<>") return x !== y;
    if (y.indexOf("*") >= 0) {
      var pattern = "^" + y.replace(/[.+?^${}()|[\]\\]/g, "\\$&").replace(/\*/g, ".*") + "$";
      return new RegExp(pattern, "i").test(x);
    }
    return x === y;
  }

  function call(node, ctx, steps, visiting, depth, state) {
    var name = node.name;
    var args = node.args;
    if (name === "IF" || name === "IFERROR") {
      var cond = evalNode(args[0], ctx, steps, visiting, depth + 1, state);
      var chosen;
      if (name === "IF") chosen = truthy(cond) ? args[1] : args[2];
      else chosen = isErr(cond) ? args[1] : { type: "literal", value: cond, start: args[0].start, end: args[0].end };
      if (!chosen) return false;
      return evalNode(chosen, ctx, steps, visiting, depth + 1, state);
    }
    var values = args.map(function (arg) { return evalNode(arg, ctx, steps, visiting, depth + 1, state); });
    var first = values[0];
    for (var i = 0; i < values.length; i++) if (isErr(values[i])) return values[i];
    var flat = flatten(values);
    var nums = flat.map(numeric).filter(function (n) { return n !== null; });
    switch (name) {
      case "SUM": return nums.reduce(function (a, b) { return a + b; }, 0);
      case "AVERAGE": return nums.length ? nums.reduce(function (a, b) { return a + b; }, 0) / nums.length : err("#DIV/0!");
      case "MIN": return nums.length ? Math.min.apply(null, nums) : 0;
      case "MAX": return nums.length ? Math.max.apply(null, nums) : 0;
      case "COUNT": return nums.length;
      case "COUNTA": return flat.filter(function (v) { return text(v).trim() !== ""; }).length;
      case "ABS": return Math.abs(numeric(first) || 0);
      case "ROUND": return Math.round((numeric(first) || 0) * Math.pow(10, numeric(values[1]) || 0)) / Math.pow(10, numeric(values[1]) || 0);
      case "LEN": return text(first).length;
      case "UPPER": return text(first).toUpperCase();
      case "LOWER": return text(first).toLowerCase();
      case "CONCAT": case "CONCATENATE": return flat.map(text).join("");
      case "AND": return flat.every(truthy);
      case "OR": return flat.some(truthy);
      case "NOT": return !truthy(first);
      case "ISERROR": return isErr(first);
      case "SUMIF": case "COUNTIF": {
        var range = Array.isArray(first) ? first : [first];
        var criterion = values[1];
        var sumRange = name === "SUMIF" && values[2] !== undefined ? flatten([values[2]]) : range;
        var total = 0;
        var count = 0;
        for (var k = 0; k < range.length; k++) {
          if (criterionMatch(range[k], criterion)) {
            count++;
            if (sumRange) total += numeric(sumRange[Math.min(k, sumRange.length - 1)]) || 0;
          }
        }
        return name === "SUMIF" ? total : count;
      }
      default: return err("#NAME?");
    }
  }

  function evaluate(formula, ctx) {
    var source = text(formula).replace(/^=/, "");
    var parsed = parse(source);
    if (parsed.error) return { steps: [], value: "", error: "#NAME?", parseError: parsed.error, truncated: false };
    var steps = [];
    var state = { source: source, truncated: false };
    var value = evalNode(parsed.ast, ctx || { resolve: function () { return { value: "" }; } }, steps, {}, 0, state);
    return { steps: steps, value: display(value), error: isErr(value) ? value.err : "", truncated: state.truncated || steps.length >= MAX_NODES };
  }


  window.SheetAuditEval.evaluate = evaluate;
})();
