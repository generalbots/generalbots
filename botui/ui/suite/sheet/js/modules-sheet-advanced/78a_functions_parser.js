"use strict";
/* Issue #2434 - tokenizer and recursive-descent parser for local formulas.
 * The parser never throws: malformed input becomes a #VALUE! node so a broken
 * formula cannot take down the preview. */
(function () {
  var ERROR_RE = /^#(DIV\/0!|VALUE!|REF!|NAME\?|N\/A|NUM!|NULL!)/;
  var CELL_RE = /^\$?[A-Za-z]{1,3}\$?[1-9][0-9]*$/;
  var IDENT_RE = /^[A-Za-zÀ-ÿ\u4e00-\u9fff가-힣_$][A-Za-zÀ-ÿ0-9\u4e00-\u9fff가-힣_.$]*/;

  function tokenize(text) {
    var source = String(text || "").trim();
    if (source.charAt(0) === "=") source = source.slice(1);
    var tokens = [];
    var i = 0;
    while (i < source.length) {
      var rest = source.slice(i);
      var ch = source.charAt(i);
      if (/\s/.test(ch)) { i += 1; continue; }
      var error = rest.match(ERROR_RE);
      if (error) {
        tokens.push({ type: "err", value: "#" + error[1] });
        i += error[0].length;
        continue;
      }
      if (/[0-9.]/.test(ch)) {
        var number = rest.match(/^(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?/);
        if (!number) return null;
        tokens.push({ type: "num", value: Number(number[0]) });
        i += number[0].length;
        continue;
      }
      if (ch === "\"") {
        var end = i + 1;
        var raw = "";
        while (end < source.length) {
          if (source.charAt(end) === "\"" && source.charAt(end + 1) === "\"") {
            raw += "\""; end += 2; continue;
          }
          if (source.charAt(end) === "\"") break;
          raw += source.charAt(end);
          end += 1;
        }
        if (end >= source.length) return null;
        tokens.push({ type: "str", value: raw });
        i = end + 1;
        continue;
      }
      var ident = rest.match(IDENT_RE);
      if (ident) {
        tokens.push({ type: "ident", value: ident[0] });
        i += ident[0].length;
        continue;
      }
      var pair = rest.slice(0, 2);
      if (pair === "<=" || pair === ">=" || pair === "<>") {
        tokens.push({ type: "op", value: pair });
        i += 2;
        continue;
      }
      if ("+-*/^&=%:,;()[]<>".indexOf(ch) >= 0) {
        tokens.push({ type: ch === "(" || ch === ")" || ch === "," || ch === ";" || ch === "[" || ch === "]" ? ch : "op", value: ch });
        i += 1;
        continue;
      }
      return null;
    }
    return tokens;
  }

  function node(type, value, extra) {
    var result = { t: type, v: value };
    if (extra) Object.keys(extra).forEach(function (key) { result[key] = extra[key]; });
    return result;
  }

  function Parser(tokens) {
    this.tokens = tokens;
    this.index = 0;
  }

  Parser.prototype.peek = function (value) {
    var token = this.tokens[this.index];
    if (!token) return false;
    if (value === undefined) return true;
    return token.value === value;
  };

  Parser.prototype.peekOp = function (value) {
    var token = this.tokens[this.index];
    if (!token || token.type !== "op") return false;
    if (value === undefined) return true;
    return token.value === value;
  };

  Parser.prototype.peekAnyOp = function (values) {
    var token = this.tokens[this.index];
    if (!token || token.type !== "op") return false;
    return values.indexOf(token.value) >= 0;
  };

  Parser.prototype.take = function () {
    return this.tokens[this.index++];
  };

  Parser.prototype.expect = function (value) {
    if (!this.peek(value)) return false;
    this.take();
    return true;
  };

  Parser.prototype.parse = function () {
    var result = this.comparison();
    return this.index === this.tokens.length ? result : null;
  };

  Parser.prototype.comparison = function () {
    var left = this.concat();
    while (this.peekAnyOp(["=", "<>", "<", ">", "<=", ">="])) {
      var operator = this.take().value;
      left = node("bin", operator, { l: left, r: this.concat() });
    }
    return left;
  };

  Parser.prototype.concat = function () {
    var left = this.additive();
    while (this.peekOp("&")) {
      this.take();
      left = node("bin", "&", { l: left, r: this.additive() });
    }
    return left;
  };

  Parser.prototype.additive = function () {
    var left = this.multiplicative();
    while (this.peekAnyOp(["+", "-"])) {
      var operator = this.take().value;
      left = node("bin", operator, { l: left, r: this.multiplicative() });
    }
    return left;
  };

  Parser.prototype.multiplicative = function () {
    var left = this.power();
    while (this.peekAnyOp(["*", "/"])) {
      var operator = this.take().value;
      left = node("bin", operator, { l: left, r: this.power() });
    }
    return left;
  };

  /* Excel precedence: a sign binds to the value before ^ and ^ is evaluated
   * left to right, so -2^2 = 4 and 2^3^2 = 64. */
  Parser.prototype.power = function () {
    var left = this.signed();
    while (this.peekOp("^")) {
      this.take();
      left = node("bin", "^", { l: left, r: this.signed() });
    }
    return left;
  };

  Parser.prototype.signed = function () {
    if (this.peekOp("-") || this.peekOp("+")) {
      var operator = this.take().value;
      return node("un", operator, { operand: this.signed() });
    }
    return this.postfix();
  };

  Parser.prototype.postfix = function () {
    var result = this.primary();
    while (this.peekOp("%")) {
      this.take();
      result = node("post", "%", { operand: result });
    }
    return result;
  };

  Parser.prototype.primary = function () {
    var token = this.take();
    if (!token) return null;
    if (token.type === "num" || token.type === "str" || token.type === "err") {
      return node(token.type, token.value);
    }
    if (token.value === "(") {
      var inner = this.comparison();
      return this.expect(")") ? inner : null;
    }
    if (token.value === "[") {
      var items = [];
      if (!this.peek("]")) {
        do { items.push(this.comparison()); } while (this.expect(",") || this.expect(";"));
      }
      return this.expect("]") ? node("array", null, { items: items }) : null;
    }
    if (token.type !== "ident") return null;
    if (this.peek("(")) {
      this.take();
      var args = [];
      if (!this.peek(")")) {
        do { args.push(this.comparison()); } while (this.expect(",") || this.expect(";"));
      }
      return this.expect(")") ? node("call", token.value.toUpperCase(), { args: args }) : null;
    }
    var upper = token.value.toUpperCase();
    if (upper === "TRUE") return node("bool", true);
    if (upper === "FALSE") return node("bool", false);
    var ref = token.value.replace(/\$/g, "");
    if (CELL_RE.test(ref)) {
      var normal = ref.toUpperCase();
      if (this.peek(":") && this.tokens[this.index + 1] && CELL_RE.test(String(this.tokens[this.index + 1].value).replace(/\$/g, ""))) {
        this.take();
        return node("range", null, { from: normal, to: this.take().value.toUpperCase().replace(/\$/g, "") });
      }
      return node("cell", normal);
    }
    return node("ident", upper);
  };

  function parse(text) {
    var tokens = tokenize(text);
    if (!tokens || !tokens.length) return null;
    return new Parser(tokens).parse();
  }

  window.SheetFunctionsParser = { tokenize: tokenize, parse: parse };
})();
