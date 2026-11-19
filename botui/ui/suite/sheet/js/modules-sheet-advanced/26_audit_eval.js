"use strict";
/* Sheet advanced module: 26_audit_eval — bounded, local-only formula stepper (#2491).
   Evaluates the common operators/functions over in-memory cell values. It records
   one step per AST node with source offsets, so the UI can highlight the active
   sub-expression. It deliberately does NOT mirror the server calc engine. */

(function () {
  var MAX_TOKENS = 1600;
  var MAX_NODES = 1200;
  var MAX_DEPTH = 32;
  var MAX_RANGE = 2000;
  var OPS = { "=": 2, "<>": 2, "!=": 2, "<": 2, ">": 2, "<=": 2, ">=": 2, "&": 3, "+": 3, "-": 3, "*": 4, "/": 4, "^": 5 };

  function text(value) {
    return value == null ? "" : String(value);
  }
  function isErr(value) {
    return !!(value && typeof value === "object" && value.err);
  }
  function err(code) {
    return { err: code };
  }
  function display(value) {
    if (isErr(value)) return value.err;
    if (typeof value === "boolean") return value ? "TRUE" : "FALSE";
    if (typeof value === "number") {
      if (!isFinite(value)) return "#NUM!";
      return String(Math.round(value * 1e6) / 1e6);
    }
    if (Array.isArray(value)) return value.map(display).join(", ");
    return text(value);
  }
  function numeric(value) {
    if (isErr(value)) return null;
    if (typeof value === "number") return value;
    if (typeof value === "boolean") return value ? 1 : 0;
    var trimmed = text(value).trim();
    if (trimmed === "") return null;
    var n = Number(trimmed.replace(/,/g, ""));
    return isNaN(n) ? null : n;
  }
  function truthy(value) {
    if (isErr(value)) return false;
    var n = numeric(value);
    if (n !== null) return n !== 0;
    var s = text(value).trim().toUpperCase();
    return s === "TRUE" || s === "YES";
  }

  function tokenize(source) {
    var src = text(source);
    var tokens = [];
    var i = 0;
    while (i < src.length && tokens.length < MAX_TOKENS) {
      var ch = src.charAt(i);
      if (/\s/.test(ch)) { i++; continue; }
      var rest = src.slice(i);
      var match;
      if ((match = /^(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?/.exec(rest))) {
        tokens.push({ type: "number", value: Number(match[0]), start: i, end: i + match[0].length });
        i += match[0].length;
        continue;
      }
      if (ch === '"') {
        var j = i + 1;
        var value = "";
        var closed = false;
        while (j < src.length) {
          if (src.charAt(j) === '"') {
            if (src.charAt(j + 1) === '"') { value += '"'; j += 2; continue; }
            closed = true; j++; break;
          }
          value += src.charAt(j++);
        }
        tokens.push({ type: "string", value: value, start: i, end: j, closed: closed });
        i = j;
        continue;
      }
      if (ch === "'") {
        var k = src.indexOf("'", i + 1);
        if (k < 0) return { error: "quote", tokens: tokens };
        var name = src.slice(i + 1, k);
        i = k + 1;
        if (src.charAt(i) !== "!") return { error: "sheet", tokens: tokens };
        i++;
        tokens.push({ type: "sheet", value: name, start: i - name.length - 2, end: i });
        continue;
      }
      if (/[A-Za-z_$]/.test(ch)) {
        match = /^[A-Za-z_$][A-Za-z0-9_.$]*/.exec(rest);
        var word = match[0];
        i += word.length;
        if (src.charAt(i) === "!") {
          i++;
          tokens.push({ type: "sheet", value: word, start: i - word.length - 1, end: i });
        } else {
          tokens.push({ type: "word", value: word, start: i - word.length, end: i });
        }
        continue;
      }
      var two = src.slice(i, i + 2);
      if (OPS[two]) {
        tokens.push({ type: "op", value: two, start: i, end: i + 2 });
        i += 2;
        continue;
      }
      if (OPS[ch] || "(),:+-*/^".indexOf(ch) >= 0) {
        tokens.push({ type: "op", value: ch, start: i, end: i + 1 });
        i++;
        continue;
      }
      return { error: "token", tokens: tokens };
    }
    return { tokens: tokens, error: tokens.length >= MAX_TOKENS ? "limit" : null };
  }

  function refParts(word) {
    var clean = text(word).replace(/\$/g, "");
    var match = /^([A-Za-z]{1,3})([1-9][0-9]{0,6})$/.exec(clean);
    if (!match) return null;
    var col = 0;
    var letters = match[1].toUpperCase();
    for (var i = 0; i < letters.length; i++) col = col * 26 + (letters.charCodeAt(i) - 64);
    return { row: parseInt(match[2], 10) - 1, col: col - 1 };
  }

  function parse(source) {
    var tk = tokenize(text(source).replace(/^=/, ""));
    if (tk.error) return { error: tk.error };
    var tokens = tk.tokens;
    var state = { tokens: tokens, i: 0, depth: 0, nodes: 0, error: null };
    if (!tokens.length) return { error: "empty" };
    function peek() { return tokens[state.i]; }
    function take() { return tokens[state.i++]; }
    function node(make) {
      state.nodes++;
      if (state.nodes > MAX_NODES || state.depth > MAX_DEPTH) { state.error = "limit"; return null; }
      return make();
    }
    function primary() {
      var token = peek();
      if (!token) { state.error = "end"; return null; }
      if (token.type === "number" || token.type === "string") { take(); return node(function () { return { type: "literal", value: token.value, start: token.start, end: token.end }; }); }
      if (token.type === "op" && (token.value === "(")) {
        take();
        var inner = expression(0);
        var close = peek();
        if (!inner || !close || close.value !== ")") { state.error = "paren"; return null; }
        take();
        return node(function () { return { type: "group", value: inner, start: token.start, end: close.end }; });
      }
      if (token.type === "op" && (token.value === "-" || token.value === "+")) {
        take();
        var arg = expression(5);
        if (!arg) return null;
        return node(function () { return { type: "unary", op: token.value, value: arg, start: token.start, end: arg.end }; });
      }
      var sheetName = "";
      if (token.type === "sheet") { sheetName = take().value; token = peek(); if (!token || token.type !== "word") { state.error = "sheet"; return null; } }
      if (token.type === "word") {
        var word = take();
        var upper = word.value.toUpperCase();
        if (peek() && peek().value === "(") {
          take();
          var args = [];
          if (peek() && peek().value !== ")") {
            while (state.i < tokens.length) {
              var argNode = expression(0);
              if (!argNode) return null;
              args.push(argNode);
              if (peek() && peek().value === ",") { take(); continue; }
              break;
            }
          }
          var closing = peek();
          if (!closing || closing.value !== ")") { state.error = "args"; return null; }
          take();
          return node(function () { return { type: "call", name: upper, args: args, start: word.start, end: closing.end }; });
        }
        var parts = refParts(word.value);
        if (parts) {
          var refNode = node(function () { return { type: "ref", sheetName: sheetName, row: parts.row, col: parts.col, start: sheetName ? word.start - sheetName.length - 2 : word.start, end: word.end }; });
          if (!refNode) return null;
          if (peek() && peek().value === ":") {
            take();
            var sheet2 = "";
            var next = peek();
            if (next && next.type === "sheet") { sheet2 = take().value; next = peek(); }
            if (!next || next.type !== "word") { state.error = "range"; return null; }
            var endPart = refParts(take().value);
            if (!endPart) { state.error = "range"; return null; }
            return node(function () { return { type: "range", sheetName: sheet2 || sheetName, from: refNode, row: endPart.row, col: endPart.col, start: refNode.start, end: tokens[state.i - 1].end }; });
          }
          return refNode;
        }
        if (upper === "TRUE" || upper === "FALSE") return node(function () { return { type: "literal", value: upper === "TRUE", start: word.start, end: word.end }; });
        if (/^#[A-Z/0-9?]+!$/.test(upper) || upper === "#N/A") return node(function () { return { type: "literal", value: err(upper), start: word.start, end: word.end }; });
        return node(function () { return { type: "name", value: word.value, start: word.start, end: word.end }; });
      }
      state.error = "primary";
      return null;
    }
    function expression(minPrec) {
      state.depth++;
      var left = primary();
      if (!left) { state.depth--; return null; }
      while (state.i < tokens.length && !state.error) {
        var op = peek();
        if (!op || op.type !== "op") break;
        var prec = OPS[op.value];
        if (prec === undefined || prec < minPrec) break;
        take();
        var right = expression(op.value === "^" ? prec : prec + 1);
        if (!right) break;
        var captured = op;
        left = node(function () { return { type: "binary", op: captured.value, left: left, right: right, start: left.start, end: right.end }; });
        if (!left) break;
      }
      state.depth--;
      return left;
    }
    var ast = expression(0);
    if (state.error || !ast) return { error: state.error || "syntax" };
    if (state.i < tokens.length) return { error: "trailing" };
    return { ast: ast };
  }


  window.__SheetAuditEvalInternals = {
    text: text, isErr: isErr, err: err, display: display, numeric: numeric,
    truthy: truthy, tokenize: tokenize, parse: parse, OPS: OPS,
    MAX_NODES: MAX_NODES, MAX_RANGE: MAX_RANGE,
  };

  window.SheetAuditEval = {
    tokenize: tokenize,
    parse: parse,
    display: display,
    numeric: numeric,
  };
})();
