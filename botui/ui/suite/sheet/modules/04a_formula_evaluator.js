"use strict";

  function evaluateMin(expr) {
    const match = expr.match(/MIN\(([^)]+)\)/i);
    if (!match) return "#ERROR";
    const values = parseRange(match[1]);
    return values.length ? Math.min(...values) : 0;
  }

  function safeEvalArithmetic(expr) {
    expr = expr.trim();
    if (/[^0-9+\-*/().%\s<>=!&|]/.test(expr)) return "#ERROR";
    const tokens = expr.match(/(\d+\.?\d*|[+\-*/().%<>=!&|]+)/g);
    if (!tokens) return "#ERROR";
    function evalTokens(tokens) {
      const values = [];
      const ops = [];
      const prec = { "+": 1, "-": 1, "*": 2, "/": 2, "%": 2, "<": 0, ">": 0, "<=": 0, ">=": 0, "==": 0, "!=": 0 };
      function applyOp() {
        const op = ops.pop();
        const b = values.pop();
        const a = values.pop();
        switch (op) {
          case "+": values.push(a + b); break;
          case "-": values.push(a - b); break;
          case "*": values.push(a * b); break;
          case "/": values.push(b === 0 ? "#DIV/0!" : a / b); break;
          case "%": values.push(b === 0 ? "#DIV/0!" : a % b); break;
          case "<": values.push(a < b ? 1 : 0); break;
          case ">": values.push(a > b ? 1 : 0); break;
          case "<=": values.push(a <= b ? 1 : 0); break;
          case ">=": values.push(a >= b ? 1 : 0); break;
          case "==": values.push(a === b ? 1 : 0); break;
          case "!=": values.push(a !== b ? 1 : 0); break;
        }
      }
      for (let i = 0; i < tokens.length; i++) {
        const t = tokens[i];
        if (t === "(") { ops.push(t); }
        else if (t === ")") { while (ops.length && ops[ops.length - 1] !== "(") applyOp(); ops.pop(); }
        else if (t in prec) {
          while (ops.length && ops[ops.length - 1] !== "(" && prec[ops[ops.length - 1]] >= prec[t]) applyOp();
          ops.push(t);
        } else { values.push(parseFloat(t) || 0); }
      }
      while (ops.length) applyOp();
      return values[0];
    }
    return evalTokens(tokens);
  }

  function safeEvalCondition(expr) {
    expr = expr.trim();
    const m = expr.match(/^(.+?)\s*(>=|<=|!=|>|<|==)\s*(.+)$/);
    if (m) {
      const a = safeEvalArithmetic(m[1]);
      const b = safeEvalArithmetic(m[3]);
      if (typeof a === "string" && a.startsWith("#")) return false;
      if (typeof b === "string" && b.startsWith("#")) return false;
      switch (m[2]) {
        case ">": return a > b;
        case "<": return a < b;
        case ">=": return a >= b;
        case "<=": return a <= b;
        case "==": return a === b;
        case "!=": return a !== b;
      }
    }
    return !!safeEvalArithmetic(expr);
  }

  function evaluateIf(expr) {
    const match = expr.match(/IF\(([^,]+),([^,]+),([^)]+)\)/i);
    if (!match) return "#ERROR";
    try {
      const condition = safeEvalCondition(match[1]);
      return condition
        ? safeEvalArithmetic(match[2])
        : safeEvalArithmetic(match[3]);
    } catch {
      return "#ERROR";
    }
  }

  function evaluateAnd(expr) {
    const match = expr.match(/AND\(([^)]+)\)/i);
    if (!match) return "#ERROR";
    const parts = match[1].split(",");
    for (let i = 0; i < parts.length; i++) {
      const val = safeEvalArithmetic(parts[i].trim());
      if (!val) return 0;
    }
    return 1;
  }

  function evaluateOr(expr) {
    const match = expr.match(/OR\(([^)]+)\)/i);
    if (!match) return "#ERROR";
    const parts = match[1].split(",");
    for (let i = 0; i < parts.length; i++) {
      const val = safeEvalArithmetic(parts[i].trim());
      if (val) return 1;
    }
    return 0;
  }

  function evaluateNot(expr) {
    const match = expr.match(/NOT\(([^)]+)\)/i);
    if (!match) return "#ERROR";
    const val = safeEvalArithmetic(match[1].trim());
    return val ? 0 : 1;
  }

  function parseRange(rangeStr) {
    const values = [];
    const parts = rangeStr.split(":");

    if (parts.length === 2) {
      const start = parseCellRef(parts[0].trim());
      const end = parseCellRef(parts[1].trim());
      if (start && end) {
        for (let r = start.row; r <= end.row; r++) {
          for (let c = start.col; c <= end.col; c++) {
            const val = parseFloat(getCellValue(r, c));
            if (!isNaN(val)) values.push(val);
          }
        }
      }
    } else {
      const ref = parseCellRef(parts[0].trim());
      if (ref) {
        const val = parseFloat(getCellValue(ref.row, ref.col));
        if (!isNaN(val)) values.push(val);
      }
    }

    return values;
  }
