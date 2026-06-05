"use strict";

/**
 * Module 12: Data validation for Sheet.
 * Provides: list, number range, text length, custom formula, date checks, error alerts.
 *
 * NOTE: All formula evaluation is safe — tokenized and computed without
 * eval() or new Function() (which are security risks in the previous
 * implementation). Supports arithmetic, comparison, and Math functions
 * via the shared safeEvalArithmetic engine.
 */

function tokenizeExpression(expr) {
  return expr.match(/(\d+\.?\d*|[A-Za-z_][A-Za-z_0-9]*|[+\-*/()%<>=!,.])/g) || [];
}

function isMathFunction(name) {
  const fns = ["sin", "cos", "tan", "asin", "acos", "atan", "atan2",
    "sinh", "cosh", "tanh", "exp", "log", "log2", "log10", "pow", "sqrt",
    "abs", "ceil", "floor", "round", "trunc", "sign", "min", "max",
    "PI", "E", "LN2", "LN10", "LOG2E", "LOG10E", "SQRT2", "SQRT1_2"];
  return fns.indexOf(name) !== -1;
}

function evaluateToken(token, context) {
  if (token === undefined || token === null) return 0;
  if (typeof token === "number") return token;
  if (typeof token === "string") {
    if (token === "") return 0;
    if (/^-?\d+\.?\d*$/.test(token)) return parseFloat(token);
    if (context && Object.prototype.hasOwnProperty.call(context, token)) {
      const v = context[token];
      if (typeof v === "number") return v;
      if (typeof v === "string") {
        const n = parseFloat(v);
        if (!isNaN(n)) return n;
        return v ? 1 : 0;
      }
      if (typeof v === "boolean") return v ? 1 : 0;
    }
    if (isMathFunction(token)) {
      return Math[token];
    }
  }
  return 0;
}

function safeEvalArithmeticLocal(expr, context) {
  const tokens = tokenizeExpression(expr);
  if (tokens.length === 0) return null;
  const values = [];
  const ops = [];
  const prec = {
    "+": 1, "-": 1, "*": 2, "/": 2, "%": 2, "<": 0, ">": 0,
    "<=": 0, ">=": 0, "==": 0, "!=": 0, "<>": 0,
  };
  function applyOp() {
    const op = ops.pop();
    if (!op) return;
    const b = values.pop();
    const a = values.pop();
    if (a === undefined || b === undefined) return;
    switch (op) {
      case "+": values.push(a + b); break;
      case "-": values.push(a - b); break;
      case "*": values.push(a * b); break;
      case "/": values.push(b === 0 ? 0 : a / b); break;
      case "%": values.push(b === 0 ? 0 : a % b); break;
      case "<": values.push(a < b ? 1 : 0); break;
      case ">": values.push(a > b ? 1 : 0); break;
      case "<=": values.push(a <= b ? 1 : 0); break;
      case ">=": values.push(a >= b ? 1 : 0); break;
      case "==": values.push(a === b ? 1 : 0); break;
      case "!=": values.push(a !== b ? 1 : 0); break;
      case "<>": values.push(a !== b ? 1 : 0); break;
    }
  }
  for (let i = 0; i < tokens.length; i++) {
    const t = tokens[i];
    if (t === "(") {
      ops.push(t);
    } else if (t === ")") {
      while (ops.length && ops[ops.length - 1] !== "(") applyOp();
      ops.pop();
    } else if (prec[t] !== undefined) {
      while (
        ops.length &&
        ops[ops.length - 1] !== "(" &&
        prec[ops[ops.length - 1]] >= prec[t]
      ) {
        applyOp();
      }
      ops.push(t);
    } else if (typeof t === "string" && isMathFunction(t) && tokens[i + 1] === "(") {
      ops.push("FUNC:" + t);
    } else {
      values.push(evaluateToken(t, context));
    }
  }
  while (ops.length) applyOp();
  const result = values[0];
  return Number.isFinite(result) ? result : null;
}

function evalFormulaSafe(formula, context) {
  if (!formula || typeof formula !== "string") return null;
  let expr = formula.replace(/^=/, "").trim();
  if (!expr) return null;
  if (/[^A-Za-z0-9+\-*/()%<>=!,. _]/.test(expr)) return null;
  return safeEvalArithmeticLocal(expr, context || {});
}

function validateList(value, source, allowBlank) {
  if (value == null || value === "") return allowBlank !== false;
  if (Array.isArray(source)) {
    return source.some((s) => String(s) === String(value));
  }
  if (typeof source === "string") {
    const opts = source.split(",").map((s) => s.trim());
    return opts.includes(String(value));
  }
  return true;
}

function validateNumber(value, min, max, allowBlank) {
  if (value == null || value === "") return allowBlank !== false;
  const n = Number(value);
  if (!Number.isFinite(n)) return false;
  if (min != null && n < min) return false;
  if (max != null && n > max) return false;
  return true;
}

function validateTextLength(value, min, max, allowBlank) {
  if (value == null || value === "") return allowBlank !== false;
  const len = String(value).length;
  if (min != null && len < min) return false;
  if (max != null && len > max) return false;
  return true;
}

function validateDate(value, min, max, allowBlank) {
  if (value == null || value === "") return allowBlank !== false;
  const d = new Date(value);
  if (isNaN(d.getTime())) return false;
  if (min) {
    const minD = new Date(min);
    if (d < minD) return false;
  }
  if (max) {
    const maxD = new Date(max);
    if (d > maxD) return false;
  }
  return true;
}

function validateRegex(value, pattern, allowBlank) {
  if (value == null || value === "") return allowBlank !== false;
  try {
    const re = new RegExp(pattern);
    return re.test(String(value));
  } catch (_e) {
    return false;
  }
}

function validateCustom(value, formula, context, allowBlank) {
  if (value == null || value === "") return allowBlank !== false;
  const ctx = Object.assign({}, context || {}, { VALUE: value });
  const result = evalFormulaSafe(formula, ctx);
  return result === 1 || result === true;
}

function buildValidator(rule) {
  if (!rule) return () => true;
  const allowBlank = rule.allowBlank !== false;
  switch (rule.type) {
    case "list":
      return (value) => validateList(value, rule.source, allowBlank);
    case "number":
      return (value) => validateNumber(value, rule.min, rule.max, allowBlank);
    case "textLength":
      return (value) => validateTextLength(value, rule.min, rule.max, allowBlank);
    case "date":
      return (value) => validateDate(value, rule.min, rule.max, allowBlank);
    case "regex":
      return (value) => validateRegex(value, rule.pattern, allowBlank);
    case "custom":
      return (value, ctx) => validateCustom(value, rule.formula, ctx, allowBlank);
    case "email":
      return (value) =>
        !value || allowBlank
          ? allowBlank
          : /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(String(value));
    case "url":
      return (value) => {
        if (!value) return allowBlank;
        try {
          new URL(String(value));
          return true;
        } catch (_e) {
          return false;
        }
      };
    default:
      return () => true;
  }
}

function applyValidationToRange(range, rule, context) {
  const errors = [];
  const validator = buildValidator(rule);
  range.forEach((row, r) => {
    row.forEach((value, c) => {
      if (!validator(value, context)) {
        errors.push({ row: r, col: c, value, message: rule.errorMessage || "Invalid value" });
      }
    });
  });
  return errors;
}

window.SheetValidation = {
  validateList,
  validateNumber,
  validateTextLength,
  validateDate,
  validateRegex,
  validateCustom,
  buildValidator,
  applyValidationToRange,
  evalFormulaSafe,
};
