"use strict";

// GB Calculator Engine: safe arithmetic expression evaluator shared by the
// calculator widget and the Calculator app. Tokenizer + shunting-yard +
// RPN evaluation; no eval/Function construction. Supports + - * / ^, unary
// minus, parentheses, postfix percent and factorial, constants (pi, e),
// functions (sin cos tan asin acos atan sqrt ln log abs exp floor ceil round)
// and implicit multiplication for forms like 2(3+1) or 2pi. Percent is a
// strict divide-by-100 postfix; non-finite results (e.g. 1/0) raise errors.
//
// Angle unit: trig functions accept {angle:"deg"} in options (default rad).

(function () {
  if (window.GBCalcEngine) return;

  var FUNCS = {
    sin: 1, cos: 1, tan: 1, asin: 1, acos: 1, atan: 1,
    sqrt: 1, ln: 1, log: 1, abs: 1, exp: 1,
    floor: 1, ceil: 1, round: 1,
  };

  var PRECEDENCE = { "+": 1, "-": 1, "*": 2, "/": 2, "u-": 3, "^": 4, "!": 5, "%": 5 };
  var RIGHT_ASSOC = { "^": true, "u-": true };

  function isDigit(ch) {
    return ch >= "0" && ch <= "9";
  }

  function isLetter(ch) {
    return (ch >= "a" && ch <= "z") || (ch >= "A" && ch <= "Z");
  }

  function tokenize(input) {
    var tokens = [];
    var src = String(input || "").replace(/\s+/g, "");
    var i = 0;
    while (i < src.length) {
      var ch = src[i];
      if (isDigit(ch) || ch === ".") {
        var start = i;
        while (i < src.length && (isDigit(src[i]) || src[i] === ".")) i++;
        // Scientific notation: 1.2e3 (only when e is followed by digits/sign).
        if (
          src[i] === "e" &&
          (isDigit(src[i + 1]) ||
            ((src[i + 1] === "+" || src[i + 1] === "-") && isDigit(src[i + 2])))
        ) {
          i++;
          if (src[i] === "+" || src[i] === "-") i++;
          while (i < src.length && isDigit(src[i])) i++;
        }
        var text = src.slice(start, i);
        var value = Number(text);
        if (!isFinite(value) && text !== Infinity) throw new Error("Bad number: " + text);
        tokens.push({ type: "num", value: value });
        continue;
      }
      if (ch === "π") {
        tokens.push({ type: "num", value: Math.PI });
        i++;
        continue;
      }
      if (isLetter(ch)) {
        var name = "";
        while (i < src.length && isLetter(src[i])) {
          name += src[i];
          i++;
        }
        var lower = name.toLowerCase();
        if (lower === "e") {
          tokens.push({ type: "num", value: Math.E });
          continue;
        }
        if (lower === "pi") {
          tokens.push({ type: "num", value: Math.PI });
          continue;
        }
        if (!FUNCS[lower]) throw new Error("Unknown name: " + name);
        tokens.push({ type: "func", name: lower });
        continue;
      }
      if ("+-*/^()!%".indexOf(ch) !== -1) {
        tokens.push({ type: ch });
        i++;
        continue;
      }
      if (ch === "×" || ch === "·") { tokens.push({ type: "*" }); i++; continue; }
      if (ch === "÷" || ch === ":") { tokens.push({ type: "/" }); i++; continue; }
      if (ch === "−") { tokens.push({ type: "-" }); i++; continue; }
      if (ch === ",") { i++; continue; }
      throw new Error("Unexpected character: " + ch);
    }
    return tokens;
  }

  // Insert implicit multiplication between number/')'/postfix and a value
  // starter ('(', number, constant or function).
  function withImplicitMultiplication(tokens) {
    var out = [];
    for (var i = 0; i < tokens.length; i++) {
      var prev = out[out.length - 1];
      var cur = tokens[i];
      if (
        prev &&
        (prev.type === "num" || prev.type === ")" || prev.type === "!" || prev.type === "%") &&
        (cur.type === "num" || cur.type === "(" || cur.type === "func")
      ) {
        out.push({ type: "*" });
      }
      out.push(cur);
    }
    return out;
  }

  function markUnary(tokens) {
    var out = [];
    for (var i = 0; i < tokens.length; i++) {
      var t = tokens[i];
      var prev = out[out.length - 1];
      var startsValue =
        !prev || prev.type === "(" || prev.type === "," ||
        (prev.type === "op" && prev.name !== "!" && prev.name !== "%");
      if (t.type === "-" && startsValue) {
        out.push({ type: "op", name: "u-" });
        continue;
      }
      if (t.type === "+" && startsValue) continue;
      if ("+-*/^".indexOf(t.type) !== -1) {
        out.push({ type: "op", name: t.type });
        continue;
      }
      if (t.type === "!" || t.type === "%") {
        out.push({ type: "op", name: t.type });
        continue;
      }
      out.push(t);
    }
    return out;
  }

  function toRpn(tokens) {
    var output = [];
    var stack = [];
    for (var i = 0; i < tokens.length; i++) {
      var t = tokens[i];
      if (t.type === "num") {
        output.push(t);
      } else if (t.type === "func") {
        stack.push(t);
      } else if (t.type === "(") {
        stack.push(t);
      } else if (t.type === ")") {
        while (stack.length && stack[stack.length - 1].type !== "(") {
          output.push(stack.pop());
        }
        if (!stack.length) throw new Error("Unbalanced parentheses");
        stack.pop();
        if (stack.length && stack[stack.length - 1].type === "func") {
          output.push(stack.pop());
        }
      } else if (t.type === "op") {
        while (stack.length) {
          var top = stack[stack.length - 1];
          if (top.type !== "op") break;
          var topPrec = PRECEDENCE[top.name];
          var curPrec = PRECEDENCE[t.name];
          var stop = topPrec < curPrec || (topPrec === curPrec && RIGHT_ASSOC[t.name]);
          if (stop) break;
          output.push(stack.pop());
        }
        stack.push(t);
      } else {
        throw new Error("Unexpected token: " + t.type);
      }
    }
    while (stack.length) {
      var rest = stack.pop();
      if (rest.type === "(") throw new Error("Unbalanced parentheses");
      output.push(rest);
    }
    return output;
  }

  function factorial(n) {
    if (n < 0 || n % 1 !== 0 || n > 170) return NaN;
    var acc = 1;
    for (var k = 2; k <= n; k++) acc *= k;
    return acc;
  }

  function toRad(x, opts) {
    return opts.angle === "deg" ? (x * Math.PI) / 180 : x;
  }

  function fromRad(x, opts) {
    return opts.angle === "deg" ? (x * 180) / Math.PI : x;
  }

  function applyOp(name, stack, opts) {
    function pop() {
      if (!stack.length) throw new Error("Malformed expression");
      return stack.pop();
    }
    if (name === "u-") { var v = pop(); stack.push(-v); return; }
    if (name === "!") { var f = pop(); stack.push(factorial(f)); return; }
    // Percent is a plain divide-by-100 postfix (strict semantics).
    if (name === "%") { var p = pop(); stack.push(p / 100); return; }
    var b = pop();
    var a = pop();
    switch (name) {
      case "+": stack.push(a + b); break;
      case "-": stack.push(a - b); break;
      case "*": stack.push(a * b); break;
      case "/": stack.push(a / b); break;
      case "^": stack.push(Math.pow(a, b)); break;
      default: throw new Error("Unknown operator: " + name);
    }
  }

  function applyFunc(fname, stack, opts) {
    if (!stack.length) throw new Error("Malformed expression");
    var x = stack.pop();
    switch (fname) {
      case "sin": stack.push(Math.sin(toRad(x, opts))); break;
      case "cos": stack.push(Math.cos(toRad(x, opts))); break;
      case "tan": stack.push(Math.tan(toRad(x, opts))); break;
      case "asin": stack.push(fromRad(Math.asin(x), opts)); break;
      case "acos": stack.push(fromRad(Math.acos(x), opts)); break;
      case "atan": stack.push(fromRad(Math.atan(x), opts)); break;
      case "sqrt": stack.push(Math.sqrt(x)); break;
      case "ln": stack.push(Math.log(x)); break;
      case "log": stack.push(Math.log10 ? Math.log10(x) : Math.log(x) / Math.LN10); break;
      case "abs": stack.push(Math.abs(x)); break;
      case "exp": stack.push(Math.exp(x)); break;
      case "floor": stack.push(Math.floor(x)); break;
      case "ceil": stack.push(Math.ceil(x)); break;
      case "round": stack.push(Math.round(x)); break;
      default: throw new Error("Unknown function: " + fname);
    }
  }

  function evaluateRpn(rpn, opts) {
    var values = [];
    for (var i = 0; i < rpn.length; i++) {
      var t = rpn[i];
      if (t.type === "num") values.push(t.value);
      else if (t.type === "op") applyOp(t.name, values, opts);
      else if (t.type === "func") applyFunc(t.name, values, opts);
      else throw new Error("Malformed expression");
    }
    if (values.length !== 1) throw new Error("Malformed expression");
    var result = values[0];
    if (typeof result !== "number" || isNaN(result)) throw new Error("Undefined result");
    if (!isFinite(result)) throw new Error("Result is not finite");
    return result;
  }

  function evaluate(expression, options) {
    var opts = options || {};
    var tokens = markUnary(withImplicitMultiplication(tokenize(expression)));
    return evaluateRpn(toRpn(tokens), opts);
  }

  // Human-friendly formatting: up to 12 significant digits, trailing zeros
  // trimmed, exponential notation for extreme magnitudes.
  function format(value) {
    if (typeof value !== "number" || !isFinite(value)) return String(value);
    var abs = Math.abs(value);
    if (abs !== 0 && (abs >= 1e12 || abs < 1e-9)) {
      return value.toExponential(6).replace(/(\.\d*?)0+e/, "$1e").replace(/\.e/, "e");
    }
    var text = value.toPrecision(12);
    if (text.indexOf(".") !== -1) {
      text = text.replace(/0+$/, "").replace(/\.$/, "");
    }
    return text;
  }

  window.GBCalcEngine = {
    evaluate: evaluate,
    format: format,
  };
})();
