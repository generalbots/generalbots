"use strict";

/**
 * Module 24b: Tokenizer + recursive-descent Parser for SheetFormulaEngine.
 * Split out from 24_formula_engine.js to keep the core engine file under
 * 450 lines. Loaded after 24, before 24a. Attaches `tokenize` and
 * `Parser` to window.__SHEET_FORMULA_PARSE so 24's `parse` (which
 * already lives in 24's IIFE) can pull them in lazily on first use.
 *
 * 24's `parse` resolves the namespace on every call:
 *   const ns = window.__SHEET_FORMULA_PARSE;
 *   const tokens = ns.tokenize(src);
 *   const p = new ns.Parser(tokens);
 */

(function () {
  const ns = window.__SHEET_FORMULA_PARSE = window.__SHEET_FORMULA_PARSE || {};

  function tokenize(src) {
    const tokens = [];
    let i = 0;
    while (i < src.length) {
      const c = src[i];
      if (c === " " || c === "\t" || c === "\n") { i++; continue; }
      if (c === "\"" ) {
        let j = i + 1;
        let s = "";
        while (j < src.length && src[j] !== "\"") {
          if (src[j] === "\\" && j + 1 < src.length) { s += src[j + 1]; j += 2; continue; }
          s += src[j]; j++;
        }
        if (j >= src.length) throw new Error("Unterminated string");
        tokens.push({ type: "string", value: s });
        i = j + 1;
        continue;
      }
      if (/[0-9.]/.test(c)) {
        let j = i;
        while (j < src.length && /[0-9.]/.test(src[j])) j++;
        const num = parseFloat(src.slice(i, j));
        if (Number.isNaN(num)) throw new Error("Bad number");
        tokens.push({ type: "number", value: num });
        i = j;
        continue;
      }
      if (/[A-Za-z_]/.test(c)) {
        let j = i;
        while (j < src.length && /[A-Za-z0-9_]/.test(src[j])) j++;
        const id = src.slice(i, j);
        const up = id.toUpperCase();
        if (up === "TRUE") tokens.push({ type: "bool", value: true });
        else if (up === "FALSE") tokens.push({ type: "bool", value: false });
        else if (j < src.length && src[j] === "(") tokens.push({ type: "func", name: up });
        else if (/^\$?[A-Z]+\$?\d+$/.test(id)) tokens.push({ type: "ref", value: id.replace(/\$/g, "").toUpperCase() });
        else if (/^[A-Za-z]+\d+$/.test(id)) tokens.push({ type: "ref", value: id.toUpperCase() });
        else tokens.push({ type: "name", value: id });
        i = j;
        continue;
      }
      if (c === "'") {
        let j = i + 1;
        let s = "";
        while (j < src.length && src[j] !== "'") { s += src[j]; j++; }
        if (j >= src.length) throw new Error("Unterminated sheet ref");
        tokens.push({ type: "sheetname", value: s });
        i = j + 1;
        continue;
      }
      if (c === "(" ) { tokens.push({ type: "lparen" }); i++; continue; }
      if (c === ")" ) { tokens.push({ type: "rparen" }); i++; continue; }
      if (c === "," ) { tokens.push({ type: "comma" }); i++; continue; }
      if (c === ":") {
        if (tokens.length > 0 && tokens[tokens.length - 1].type === "ref") {
          tokens[tokens.length - 1] = { type: "rangestart", value: tokens[tokens.length - 1].value };
          tokens.push({ type: "colon" });
        } else {
          tokens.push({ type: "colon" });
        }
        i++;
        continue;
      }
      if (c === "!" ) { tokens.push({ type: "bang" }); i++; continue; }
      if (c === "+" || c === "-" || c === "*" || c === "/" || c === "^" || c === "&") {
        tokens.push({ type: "op", op: c }); i++; continue;
      }
      if (c === "=") { tokens.push({ type: "op", op: "=" }); i++; continue; }
      if (c === "<") {
        if (src[i + 1] === ">") { tokens.push({ type: "op", op: "<>" }); i += 2; continue; }
        if (src[i + 1] === "=") { tokens.push({ type: "op", op: "<=" }); i += 2; continue; }
        tokens.push({ type: "op", op: "<" }); i++; continue;
      }
      if (c === ">") {
        if (src[i + 1] === "=") { tokens.push({ type: "op", op: ">=" }); i += 2; continue; }
        tokens.push({ type: "op", op: ">" }); i++; continue;
      }
      throw new Error("Unexpected char: " + c);
    }
    tokens.push({ type: "eof" });
    return tokens;
  }

  class Parser {
    constructor(tokens) { this.tokens = tokens; this.pos = 0; }
    peek() { return this.tokens[this.pos]; }
    consume() { return this.tokens[this.pos++]; }
    expect(type) {
      const t = this.consume();
      if (t.type !== type) throw new Error("Expected " + type + " got " + t.type);
      return t;
    }
    match(type) {
      if (this.peek().type === type) { this.pos++; return true; }
      return false;
    }
    parseExpression() {
      let left = this.parseCompare();
      while (this.peek().type === "op" && this.peek().op === "&") {
        this.consume();
        const right = this.parseCompare();
        left = { type: "concat", left, right };
      }
      return left;
    }
    parseCompare() {
      let left = this.parseAdd();
      while (this.peek().type === "op" && ["=", "<>", "<", ">", "<=", ">="].indexOf(this.peek().op) >= 0) {
        const op = this.consume().op;
        const right = this.parseAdd();
        left = { type: "binop", op, left, right };
      }
      return left;
    }
    parseAdd() {
      let left = this.parseMul();
      while (this.peek().type === "op" && (this.peek().op === "+" || this.peek().op === "-")) {
        const op = this.consume().op;
        const right = this.parseMul();
        left = { type: "binop", op, left, right };
      }
      return left;
    }
    parseMul() {
      let left = this.parsePow();
      while (this.peek().type === "op" && (this.peek().op === "*" || this.peek().op === "/")) {
        const op = this.consume().op;
        const right = this.parsePow();
        left = { type: "binop", op, left, right };
      }
      return left;
    }
    parsePow() {
      const left = this.parseUnary();
      if (this.peek().type === "op" && this.peek().op === "^") {
        this.consume();
        const right = this.parsePow();
        return { type: "binop", op: "^", left, right };
      }
      return left;
    }
    parseUnary() {
      if (this.peek().type === "op" && (this.peek().op === "-" || this.peek().op === "+")) {
        const op = this.consume().op;
        const operand = this.parseUnary();
        return { type: "unary", op, operand };
      }
      return this.parsePrimary();
    }
    parsePrimary() {
      const t = this.peek();
      if (t.type === "number") { this.consume(); return { type: "num", value: t.value }; }
      if (t.type === "string") { this.consume(); return { type: "str", value: t.value }; }
      if (t.type === "bool") { this.consume(); return { type: "bool", value: t.value }; }
      if (t.type === "lparen") {
        this.consume();
        const inner = this.parseExpression();
        this.expect("rparen");
        return inner;
      }
      if (t.type === "func") {
        const name = this.consume().name;
        this.expect("lparen");
        const args = [];
        if (this.peek().type !== "rparen") {
          args.push(this.parseExpression());
          while (this.match("comma")) args.push(this.parseExpression());
        }
        this.expect("rparen");
        return { type: "call", name, args };
      }
      if (t.type === "rangestart") {
        const start = t.value;
        this.consume();
        this.expect("colon");
        if (this.peek().type !== "ref") throw new Error("Expected ref after :");
        const end = this.consume().value;
        return { type: "range", start: { col: 0, row: 0, raw: start }, end: { col: 0, row: 0, raw: end } };
      }
      if (t.type === "ref") {
        this.consume();
        if (this.peek().type === "colon") {
          this.consume();
          if (this.peek().type !== "ref") throw new Error("Expected ref after :");
          const end = this.consume().value;
          return { type: "range", start: { col: 0, row: 0, raw: t.value }, end: { col: 0, row: 0, raw: end } };
        }
        return { type: "cell", ref: t.value };
      }
      if (t.type === "name") {
        this.consume();
        return { type: "name", value: t.value };
      }
      if (t.type === "sheetname") {
        this.consume();
        if (this.peek().type !== "bang") throw new Error("Expected ! after sheetname");
        this.consume();
        const inner = this.parsePrimary();
        inner.sheet = t.value;
        return inner;
      }
      throw new Error("Unexpected token at position " + this.pos);
    }
  }

  ns.tokenize = tokenize;
  ns.Parser = Parser;
})();
