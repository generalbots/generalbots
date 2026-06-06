"use strict";

/**
 * Module 24: Formula engine for Spreadsheet (P0 critical).
 * Replaces the insecure `new Function("return " + expr)` parser with a
 * proper recursive-descent parser and 50+ safe built-in functions.
 * Supports cell references (A1, B2:D10, sheet refs, named ranges),
 * operators (+, -, *, /, ^, &, =, <>, <, >, <=, >=), string literals,
 * number literals, boolean, error propagation, and array references.
 *
 * ARCHITECTURAL NOTE (2026-06):
 *   This module is a CLIENT-SIDE CACHE for instant UI feedback only.
 *   The source of truth for formula evaluation is
 *   `botserver/crates/botsheet-core/src/formulas/` (40+ functions in
 *   Rust), exposed via `POST /api/sheet/formula`.
 *
 *   The pattern (Lotus 1-2-3 Network model, 1989):
 *     1. User types `=A1+1` -> client.evaluate() runs immediately
 *        (zero latency, no network).
 *     2. Concurrently, the same formula is sent to the server via
 *        client.evaluateViaServer() for authoritative evaluation.
 *     3. If the server returns a different value, a "stale" indicator
 *        appears. On save, the server value wins.
 *
 *   In offline mode (no network), client.evaluate() is the only path
 *   and the result is committed to the cache. On reconnect, the
 *   server re-evaluates everything and reconciles.
 *
 *   Custom functions (F.registerFunction) are client-only; they are
 *   not callable from .bas scripts on the backend.
 *
 * Public API: window.SheetFormulaEngine = {
 *   parse, evaluate, evaluateViaServer, evaluateDual,
 *   registerFunction, getFunctions, isCellRef, isRange,
 *   parseCellRef, parseRange, normalize, FUNCTION_LIST,
 *   serverAuthoritative, setSheetId
 * }.
 */

(function () {
  const FUNCS = {};
  const SHEET_REF_CACHE = {};

  function colNameToIndex(name) {
    let n = 0;
    for (let i = 0; i < name.length; i++) {
      const c = name.charCodeAt(i) - 64;
      if (c < 1 || c > 26) return -1;
      n = n * 26 + c;
    }
    return n - 1;
  }

  function indexToColName(idx) {
    let n = idx + 1;
    let s = "";
    while (n > 0) {
      const r = (n - 1) % 26;
      s = String.fromCharCode(65 + r) + s;
      n = Math.floor((n - 1) / 26);
    }
    return s;
  }

  function parseCellRef(ref) {
    if (!ref) return null;
    const m = String(ref).match(/^\$?([A-Z]+)\$?(\d+)$/);
    if (!m) return null;
    return { col: colNameToIndex(m[1]), row: parseInt(m[2], 10) - 1 };
  }

  function parseRange(rng) {
    if (!rng || typeof rng !== "string") return null;
    const parts = rng.split(":");
    if (parts.length === 1) {
      const c = parseCellRef(parts[0]);
      return c ? { start: c, end: c } : null;
    }
    if (parts.length === 2) {
      const s = parseCellRef(parts[0]);
      const e = parseCellRef(parts[1]);
      if (!s || !e) return null;
      return { start: { col: Math.min(s.col, e.col), row: Math.min(s.row, e.row) }, end: { col: Math.max(s.col, e.col), row: Math.max(s.row, e.row) } };
    }
    return null;
  }

  function isCellRef(s) { return /^\$?[A-Z]+\$?\d+$/.test(String(s)); }
  function isRange(s) { return /^\$?[A-Z]+\$?\d+(:\$?[A-Z]+\$?\d+)?$/.test(String(s)); }

  function parse(expr) {
    if (!expr || typeof expr !== "string") return null;
    if (!expr.startsWith("=")) return null;
    const ns = window.__SHEET_FORMULA_PARSE;
    if (!ns) return null;
    const src = expr.slice(1);
    const tokens = ns.tokenize(src);
    const p = new ns.Parser(tokens);
    const tree = p.parseExpression();
    if (p.peek().type !== "eof") throw new Error("Unexpected token at end");
    return tree;
  }

  function lookupCell(ref, ctx) {
    const c = parseCellRef(ref);
    if (!c) return { type: "error", value: "#REF!" };
    const ws = (ctx && ctx.worksheet) || (ctx && ctx.sheet) || null;
    if (!ws || !ws.cells) return { type: "empty" };
    const v = ws.cells[c.row] && ws.cells[c.row][c.col];
    if (v == null || v === "") return { type: "empty" };
    if (typeof v === "number") return { type: "num", value: v };
    if (typeof v === "boolean") return { type: "bool", value: v };
    const s = String(v);
    if (s.startsWith("=")) {
      const cacheKey = (ctx.sheetName || "_") + ":" + ref;
      if (ctx.visited && ctx.visited.has(cacheKey)) return { type: "error", value: "#REF!" };
      if (!ctx.visited) ctx.visited = new Set();
      ctx.visited.add(cacheKey);
      try {
        const subtree = parse(s);
        if (subtree) {
          const r = evaluateTree(subtree, ctx);
          if (r && r.type === "str") return r;
          if (r && r.type === "num") return r;
          if (r && r.type === "bool") return r;
          if (r && r.type === "error") return r;
          if (r && r.type === "empty") return r;
          return r;
        }
      } catch (e) { return { type: "error", value: "#ERROR!" }; }
      return { type: "error", value: "#ERROR!" };
    }
    const n = parseFloat(s);
    if (!Number.isNaN(n) && /^-?\d+(\.\d+)?$/.test(s)) return { type: "num", value: n };
    return { type: "str", value: s };
  }

  function rangeValues(rangeNode, ctx) {
    const startRef = (rangeNode.start && rangeNode.start.raw) || rangeNode.a;
    const endRef = (rangeNode.end && rangeNode.end.raw) || rangeNode.b;
    if (!startRef || !endRef) return { type: "error", value: "#REF!" };
    const a = parseRange(startRef + ":" + endRef);
    if (!a) return { type: "error", value: "#REF!" };
    const out = [];
    for (let r = a.start.row; r <= a.end.row; r++) {
      for (let c = a.start.col; c <= a.end.col; c++) {
        const ref = indexToColName(c) + (r + 1);
        out.push(lookupCell(ref, ctx));
      }
    }
    return { type: "array", values: out, start: a.start, end: a.end, width: (a.end.col - a.start.col) + 1, height: (a.end.row - a.start.row) + 1 };
  }

  function numCoerce(v) {
    if (v == null || v === "") return 0;
    if (typeof v === "number") return v;
    if (typeof v === "boolean") return v ? 1 : 0;
    if (v && v.type === "num") return v.value;
    if (v && v.type === "str") {
      const n = parseFloat(v.value);
      return Number.isNaN(n) ? 0 : n;
    }
    if (v && v.type === "empty") return 0;
    if (v && v.type === "error") return NaN;
    if (v && v.type === "bool") return v.value ? 1 : 0;
    if (v && v.type === "array") return v.values.length;
    return 0;
  }

  function strCoerce(v) {
    if (v == null) return "";
    if (typeof v === "string") return v;
    if (typeof v === "number") return String(v);
    if (typeof v === "boolean") return v ? "TRUE" : "FALSE";
    if (v && v.type === "num") return String(v.value);
    if (v && v.type === "str") return v.value;
    if (v && v.type === "bool") return v.value ? "TRUE" : "FALSE";
    if (v && v.type === "error") return v.value;
    if (v && v.type === "empty") return "";
    return String(v);
  }

  function cmpValues(a, b) {
    if (a && a.type === "error") return a;
    if (b && b.type === "error") return b;
    if (a && a.type === "empty" && b && b.type === "empty") return { type: "bool", value: true };
    if (a && a.type === "empty") return { type: "bool", value: false };
    if (b && b.type === "empty") return { type: "bool", value: false };
    const an = a && a.type === "num" ? a.value : parseFloat(strCoerce(a));
    const bn = b && b.type === "num" ? b.value : parseFloat(strCoerce(b));
    if (!Number.isNaN(an) && !Number.isNaN(bn)) return { type: "num", value: an - bn };
    return { type: "str", value: strCoerce(a) < strCoerce(b) ? -1 : strCoerce(a) > strCoerce(b) ? 1 : 0 };
  }

  function cmpOp(op, a, b) {
    if (a && a.type === "error") return a;
    if (b && b.type === "error") return b;
    const c = cmpValues(a, b);
    if (c.type === "num") {
      switch (op) {
        case "=": return { type: "bool", value: c.value === 0 };
        case "<>": return { type: "bool", value: c.value !== 0 };
        case "<": return { type: "bool", value: c.value < 0 };
        case ">": return { type: "bool", value: c.value > 0 };
        case "<=": return { type: "bool", value: c.value <= 0 };
        case ">=": return { type: "bool", value: c.value >= 0 };
      }
    } else if (c.type === "str") {
      const s = c.value;
      switch (op) {
        case "=": return { type: "bool", value: s === 0 };
        case "<>": return { type: "bool", value: s !== 0 };
        case "<": return { type: "bool", value: s < 0 };
        case ">": return { type: "bool", value: s > 0 };
        case "<=": return { type: "bool", value: s <= 0 };
        case ">=": return { type: "bool", value: s >= 0 };
      }
    }
    return { type: "error", value: "#VALUE!" };
  }

  function flatten(v) {
    if (v && v.type === "array") return v.values;
    if (Array.isArray(v)) return v;
    return [v];
  }

  function isError(v) { return v && v.type === "error"; }
  function toBool(v) {
    if (v && v.type === "bool") return v.value;
    if (v && v.type === "num") return v.value !== 0;
    if (v && v.type === "empty") return false;
    if (v && v.type === "str") return v.value.length > 0 && v.value !== "0" && v.value.toUpperCase() !== "FALSE";
    return false;
  }

  function evaluateTree(tree, ctx) {
    if (!tree) return { type: "empty" };
    switch (tree.type) {
      case "num": return { type: "num", value: tree.value };
      case "str": return { type: "str", value: tree.value };
      case "bool": return { type: "bool", value: tree.value };
      case "cell": return lookupCell(tree.ref, ctx);
      case "range": return rangeValues(tree, ctx);
      case "unary": {
        const v = evaluateTree(tree.operand, ctx);
        if (tree.op === "-") return { type: "num", value: -numCoerce(v) };
        if (tree.op === "+") return { type: "num", value: numCoerce(v) };
        if (tree.op === "%") return { type: "num", value: numCoerce(v) / 100 };
        return v;
      }
      case "binop": {
        const a = evaluateTree(tree.left, ctx);
        const b = evaluateTree(tree.right, ctx);
        if (isError(a)) return a;
        if (isError(b)) return b;
        if (tree.op === "&") return { type: "str", value: strCoerce(a) + strCoerce(b) };
        if (tree.op === "+") return { type: "num", value: numCoerce(a) + numCoerce(b) };
        if (tree.op === "-") return { type: "num", value: numCoerce(a) - numCoerce(b) };
        if (tree.op === "*") return { type: "num", value: numCoerce(a) * numCoerce(b) };
        if (tree.op === "/") {
          const db = numCoerce(b);
          if (db === 0) return { type: "error", value: "#DIV/0!" };
          return { type: "num", value: numCoerce(a) / db };
        }
        if (tree.op === "^") return { type: "num", value: Math.pow(numCoerce(a), numCoerce(b)) };
        return cmpOp(tree.op, a, b);
      }
      case "call": {
        const fn = FUNCS[tree.name];
        if (!fn) return { type: "error", value: "#NAME?" };
        const argVals = tree.args.map(function (a) { return evaluateTree(a, ctx); });
        try {
          return fn(argVals, ctx) || { type: "empty" };
        } catch (e) {
          return { type: "error", value: "#ERROR!" };
        }
      }
    }
    return { type: "empty" };
  }

  function registerFunction(name, fn) { FUNCS[name.toUpperCase()] = fn; }
  function getFunctions() { return Object.keys(FUNCS).sort(); }

  function errorValue(v) {
    if (v && v.type === "error") return v.value;
    return null;
  }

  function evaluate(expr, ctx) {
    const tree = parse(expr);
    if (!tree) return { type: "empty" };
    return evaluateTree(tree, ctx || {});
  }

  const AGG = {
    _flattenNumeric(arr) {
      const out = [];
      for (const v of flatten(arr)) {
        if (isError(v)) continue;
        if (v && v.type === "empty") continue;
        if (v && v.type === "str") {
          const n = parseFloat(v.value);
          if (Number.isNaN(n)) continue;
          out.push(n);
          continue;
        }
        out.push(numCoerce(v));
      }
      return out;
    },
    _countAll(arr) {
      let n = 0;
      for (const v of flatten(arr)) {
        if (isError(v)) continue;
        if (v && v.type === "empty") continue;
        n++;
      }
      return n;
    },
  };

  function reg(name, arity, fn) {
    registerFunction(name, function (args, ctx) {
      if (arity >= 0 && args.length < arity) return { type: "error", value: "#VALUE!" };
      return fn(args, ctx);
    });
  }

  const FUNCTION_LIST = getFunctions();
  const _helpers = { flatten, numCoerce, strCoerce, cmpValues, toBool, isError, AGG: { _flattenNumeric: AGG._flattenNumeric, _countAll: AGG._countAll } };

  // ---- Server-authoritative hybrid (Phase 7 of remediation) ----
  let _sheetId = null;
  let _serverAuthoritative = true;
  let _pendingDual = 0;
  const _dualListeners = [];

  function setSheetId(id) {
    _sheetId = id;
  }

  function getServerAuthoritative() {
    return _serverAuthoritative;
  }

  function setServerAuthoritative(v) {
    _serverAuthoritative = v === true;
  }

  function _toComparable(v) {
    if (v === null || v === undefined) return null;
    if (typeof v === "object" && v.type) {
      if (v.type === "num") return v.value;
      if (v.type === "bool") return v.value ? 1 : 0;
      if (v.type === "str") return v.value;
      if (v.type === "empty") return null;
      if (v.type === "error") return "#" + v.value;
    }
    return v;
  }

  function _divergent(local, server) {
    const a = _toComparable(local);
    const b = _toComparable(server);
    if (a === null && b === null) return false;
    if (typeof a === "number" && typeof b === "number") {
      if (isNaN(a) && isNaN(b)) return false;
      return Math.abs(a - b) > 1e-9;
    }
    return a !== b;
  }

  function _fireDual(payload) {
    for (let i = 0; i < _dualListeners.length; i++) {
      try { _dualListeners[i](payload); } catch (e) { /* listener error ignored */ }
    }
  }

  function evaluateViaServer(formula, sheetId) {
    if (!window.SheetAPI) {
      return Promise.resolve({ ok: false, error: { message: "SheetAPI not loaded" } });
    }
    const id = sheetId || _sheetId;
    if (!id) {
      return Promise.resolve({ ok: false, error: { message: "No sheet_id" } });
    }
    return window.SheetAPI.evaluate(id, formula);
  }

  function evaluateDual(formula, state, sheetId) {
    const localResult = evaluate(formula, state);
    const id = sheetId || _sheetId;
    if (!window.SheetAPI || !id) {
      return Promise.resolve({ local: localResult, server: null, divergent: false, network: false });
    }
    _pendingDual++;
    return window.SheetAPI.evaluate(id, formula).then(function (r) {
      _pendingDual--;
      if (!r.ok) {
        const payload = { local: localResult, server: null, divergent: false, network: false, error: r.error };
        _fireDual(payload);
        return payload;
      }
      const serverResult = (r.data && r.data.result) || null;
      const divergent = _divergent(localResult, serverResult);
      const payload = {
        local: localResult,
        server: serverResult,
        divergent: divergent,
        network: true,
        formula: formula,
      };
      _fireDual(payload);
      return payload;
    }).catch(function (e) {
      _pendingDual--;
      const payload = { local: localResult, server: null, divergent: false, network: false, error: e };
      _fireDual(payload);
      return payload;
    });
  }

  function onDualResult(fn) {
    if (typeof fn !== "function") return function () {};
    _dualListeners.push(fn);
    return function off() {
      const idx = _dualListeners.indexOf(fn);
      if (idx >= 0) _dualListeners.splice(idx, 1);
    };
  }

  function pendingDualCount() { return _pendingDual; }

  window.SheetFormulaEngine = Object.assign({
    parse, evaluate, registerFunction, getFunctions,
    isCellRef, isRange, parseCellRef, parseRange, normalize: indexToColName,
    colNameToIndex, indexToColName,
    FUNCTION_LIST, FUNCS,
    evaluateViaServer, evaluateDual, onDualResult, pendingDualCount,
    setSheetId, getServerAuthoritative, setServerAuthoritative,
  }, _helpers);
})();
