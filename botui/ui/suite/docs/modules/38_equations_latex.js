"use strict";
/* Docs equation editor (#2483) — bounded, local LaTeX → MathML converter.
 *
 * No CDN, no external library, no network fetch. The parser is deliberately
 * small: it covers the constructs offered by the symbol palette and templates
 * plus common authoring shorthand. Every recursive step is bounded by depth
 * and node budgets, all text is escaped, and failures degrade to <mtext>
 * instead of throwing. */
(function (window) {
  var NS = (window.DocsEquations = window.DocsEquations || {});
  var LIMITS = { input: 2000, depth: 12, nodes: 400 };
  NS.LIMITS = LIMITS;

  function tr(key, fallback) {
    var nav = window.DocsNav || {};
    if (typeof nav.tr === "function") return nav.tr(key, fallback);
    if (typeof docsT === "function") return docsT(key, fallback);
    if (window.gbT) return window.gbT(key, fallback);
    return fallback || key;
  }
  function esc(value) {
    var nav = window.DocsNav || {};
    if (typeof nav.escapeHtml === "function") return nav.escapeHtml(value);
    return String(value == null ? "" : value)
      .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;").replace(/'/g, "&#39;");
  }
  NS.tr = tr;
  NS.esc = esc;

  var SYMBOLS = {
    times: "×", pm: "±", mp: "∓", le: "≤", leq: "≤", ge: "≥", geq: "≥",
    neq: "≠", ne: "≠", approx: "≈", infty: "∞", cdot: "·", prod: "∏",
    partial: "∂", nabla: "∇", propto: "∝", to: "→", rightarrow: "→",
    leftarrow: "←", leftrightarrow: "↔", forall: "∀", exists: "∃",
    in: "∈", notin: "∉", subset: "⊂", subseteq: "⊆", cup: "∪", cap: "∩",
    emptyset: "∅", degree: "°", div: "÷", alpha: "α", beta: "β",
    gamma: "γ", delta: "δ", Delta: "Δ", epsilon: "ε", varepsilon: "ε",
    theta: "θ", Theta: "Θ", lambda: "λ", Lambda: "Λ", mu: "μ", pi: "π",
    Pi: "Π", sigma: "σ", Sigma: "Σ", phi: "φ", varphi: "φ", Phi: "Φ",
    psi: "ψ", omega: "ω", Omega: "Ω"
  };
  var GREEK = /^(alpha|beta|gamma|delta|epsilon|varepsilon|theta|lambda|mu|pi|sigma|phi|varphi|psi|omega|Delta|Theta|Lambda|Pi|Sigma|Phi|Omega)$/;
  var LARGE = { sum: "∑", prod: "∏", int: "∫", iint: "∬", oint: "∮", lim: "lim", limsup: "lim sup", liminf: "lim inf" };

  var PALETTE = [
    ["\\times", "×"], ["\\pm", "±"], ["\\mp", "∓"], ["\\le", "≤"],
    ["\\ge", "≥"], ["\\ne", "≠"], ["\\approx", "≈"], ["\\infty", "∞"],
    ["\\cdot", "·"], ["\\sum", "∑"], ["\\prod", "∏"], ["\\int", "∫"],
    ["\\partial", "∂"], ["\\nabla", "∇"], ["\\to", "→"], ["\\forall", "∀"],
    ["\\in", "∈"], ["\\emptyset", "∅"], ["\\alpha", "α"], ["\\beta", "β"],
    ["\\gamma", "γ"], ["\\delta", "δ"], ["\\pi", "π"], ["\\sigma", "σ"],
    ["\\Omega", "Ω"], ["\\degree", "°"]
  ];
  var TEMPLATES = [
    { id: "fraction", key: "docs-eq-tpl-fraction", fallback: "Fraction", tex: "\\frac{a}{b}" },
    { id: "root", key: "docs-eq-tpl-root", fallback: "Root", tex: "\\sqrt{x}" },
    { id: "integral", key: "docs-eq-tpl-integral", fallback: "Integral", tex: "\\int_{a}^{b} f(x)\\,dx" },
    { id: "sum", key: "docs-eq-tpl-sum", fallback: "Sum", tex: "\\sum_{i=1}^{n} i^2" },
    { id: "matrix", key: "docs-eq-tpl-matrix", fallback: "Matrix", tex: "\\begin{matrix}a & b \\\\ c & d\\end{matrix}" },
    { id: "cases", key: "docs-eq-tpl-cases", fallback: "Cases", tex: "\\begin{cases}x & x\\ge 0 \\\\ -x & x<0\\end{cases}" }
  ];
  NS.PALETTE = PALETTE;
  NS.TEMPLATES = TEMPLATES;

  function budget(state) {
    state.count += 1;
    return state.count <= LIMITS.nodes;
  }
  function mathWrap(content, display, label) {
    var attr = ' xmlns="http://www.w3.org/1998/Math/MathML" display="' + (display ? "block" : "inline") + '"';
    if (label) attr += ' aria-label="' + esc(label) + '"';
    return "<math" + attr + ">" + content + "</math>";
  }
  NS.mathWrap = mathWrap;

  function readGroup(tex, i, depth, state) {
    if (tex[i] !== "{") return { xml: "", next: i };
    var inner = parseSequence(tex, i + 1, depth + 1, state, "}");
    return { xml: "<mrow>" + inner.xml + "</mrow>", next: inner.next };
  }

  function readArg(tex, i, depth, state) {
    if (i >= tex.length) return { xml: "", next: i };
    if (tex[i] === "{") return readGroup(tex, i, depth, state);
    var atom = parseAtom(tex, i, depth, state);
    return { xml: atom.xml, next: atom.next };
  }

  function parseMatrix(tex, start, env, depth, state) {
    var body = "";
    var j = start;
    while (j < tex.length && tex.slice(j, j + 5) !== "\\end{") {
      body += tex[j];
      j += 1;
      if (body.length > LIMITS.input) break;
    }
    var endMatch = /^\\end\{([a-z]*)\}/.exec(tex.slice(j));
    var next = endMatch ? j + endMatch[0].length : j;
    var rows = body.split(/\\\\/).slice(0, 20);
    var out = "";
    if (env === "cases") out += '<mo fence="true" stretchy="true">{</mo>';
    out += "<mtable>";
    for (var r = 0; r < rows.length; r += 1) {
      var cells = rows[r].split("&").slice(0, 10);
      out += "<mtr>";
      for (var c = 0; c < cells.length; c += 1) {
        var parsed = parseSequence(cells[c], 0, depth + 1, state, null);
        out += "<mtd>" + (parsed.xml || "<mtext></mtext>") + "</mtd>";
      }
      out += "</mtr>";
    }
    out += "</mtable>";
    if (env === "cases") out += '<mo fence="true" stretchy="true">}</mo>';
    return { xml: "<mrow>" + out + "</mrow>", next: next };
  }

  function parseCommand(tex, i, depth, state) {
    if (i + 1 >= tex.length) return { xml: "", next: tex.length, large: false };
    var rest = tex.slice(i);
    var simple = /^\\([^a-zA-Z])/.exec(rest);
    if (simple) {
      var ch = simple[1];
      if (ch === "," || ch === ";") return { xml: '<mspace width="0.3em"/>', next: i + 2 };
      if (ch === "!") return { xml: '<mspace width="-0.2em"/>', next: i + 2 };
      if (ch === "\\") return { xml: "<mo>&#8203;</mo>", next: i + 2 };
      return { xml: "<mo>" + esc(ch) + "</mo>", next: i + 2 };
    }
    var match = /^\\([a-zA-Z]+)/.exec(rest);
    if (!match) return { xml: "<mo>\\</mo>", next: i + 1 };
    var name = match[1];
    var after = i + match[0].length;
    if (name === "frac" || name === "dfrac" || name === "tfrac" || name === "binom") {
      var a = readArg(tex, after, depth, state);
      var b = readArg(tex, a.next, depth, state);
      if (name === "binom") {
        return { xml: "<mo>(</mo><mrow><mfrac linethickness=\"0\">" + a.xml + b.xml + "</mfrac></mrow><mo>)</mo>", next: b.next };
      }
      return { xml: "<mfrac>" + a.xml + b.xml + "</mfrac>", next: b.next };
    }
    if (name === "sqrt") {
      var idx = after;
      var rootIndex = "";
      if (tex[idx] === "[") {
        var close = tex.indexOf("]", idx + 1);
        if (close > idx && close - idx < 32) {
          rootIndex = "<mrow>" + parseSequence(tex.slice(idx + 1, close), 0, depth + 1, state, null).xml + "</mrow>";
          idx = close + 1;
        }
      }
      var rad = readArg(tex, idx, depth, state);
      return {
        xml: rootIndex ? "<mroot>" + rad.xml + rootIndex + "</mroot>" : "<msqrt>" + rad.xml + "</msqrt>",
        next: rad.next
      };
    }
    if (name === "text" || name === "mathrm" || name === "mathbf") {
      var txt = readArg(tex, after, depth, state);
      var variant = name === "mathbf" ? ' mathvariant="bold"' : "";
      var tag = name === "text" ? "mtext" : "mi";
      return { xml: "<" + tag + variant + ">" + txt.xml.replace(/<[^>]+>/g, "") + "</" + tag + ">", next: txt.next };
    }
    if (name === "overline" || name === "bar" || name === "hat" || name === "vec" || name === "tilde") {
      var base = readArg(tex, after, depth, state);
      var accent = name === "overline" ? "‾" : name === "bar" ? "¯" : name === "hat" ? "^" : name === "vec" ? "→" : "~";
      return { xml: "<mover>" + base.xml + "<mo>" + esc(accent) + "</mo></mover>", next: base.next };
    }
    if (name === "left" || name === "right") {
      var fence = tex[after] || "";
      return { xml: fence ? '<mo stretchy="true">' + esc(fence) + "</mo>" : "", next: Math.min(tex.length, after + 1) };
    }
    if (name === "begin") {
      var envMatch = /^\{([a-z]*)\}/.exec(tex.slice(after));
      if (envMatch) return parseMatrix(tex, after + envMatch[0].length, envMatch[1], depth, state);
      return { xml: "", next: after };
    }
    if (Object.prototype.hasOwnProperty.call(LARGE, name)) {
      return { xml: '<mo largeop="true">' + esc(LARGE[name]) + "</mo>", next: after, large: true };
    }
    if (Object.prototype.hasOwnProperty.call(SYMBOLS, name)) {
      var glyph = SYMBOLS[name];
      return { xml: (GREEK.test(name) ? "<mi>" : "<mo>") + esc(glyph) + (GREEK.test(name) ? "</mi>" : "</mo>"), next: after };
    }
    return { xml: "<mi>" + esc(name) + "</mi>", next: after };
  }

  function parseAtom(tex, i, depth, state) {
    if (!budget(state) || depth > LIMITS.depth) return { xml: "", next: Math.min(tex.length, i + 1) };
    var ch = tex[i];
    if (ch === "{") return readGroup(tex, i, depth, state);
    if (ch === "\\") return parseCommand(tex, i, depth, state);
    if (/[0-9]/.test(ch)) {
      var num = "";
      while (i < tex.length && /[0-9.]/.test(tex[i]) && num.length < 32) { num += tex[i]; i += 1; }
      return { xml: "<mn>" + esc(num) + "</mn>", next: i };
    }
    if (/[a-zA-Z]/.test(ch)) return { xml: "<mi>" + esc(ch) + "</mi>", next: i + 1 };
    return { xml: "<mo>" + esc(ch) + "</mo>", next: i + 1 };
  }

  function parseSequence(tex, i, depth, state, stopAt) {
    var out = "";
    while (i < tex.length && state.count <= LIMITS.nodes) {
      var ch = tex[i];
      if (stopAt && ch === stopAt) { i += 1; break; }
      if (/\s/.test(ch)) { i += 1; continue; }
      var atom = parseAtom(tex, i, depth, state);
      if (atom.next <= i) { i += 1; continue; }
      i = atom.next;
      var sup = "";
      var sub = "";
      while (i < tex.length && (tex[i] === "^" || tex[i] === "_")) {
        var isSup = tex[i] === "^";
        var script = readArg(tex, i + 1, depth, state);
        if (isSup) sup += script.xml; else sub += script.xml;
        i = script.next;
      }
      if (sup && sub) {
        out += atom.large
          ? "<munderover>" + atom.xml + sub + sup + "</munderover>"
          : "<msubsup>" + atom.xml + sub + sup + "</msubsup>";
      } else if (sup) {
        out += atom.large
          ? "<mover>" + atom.xml + sup + "</mover>"
          : "<msup>" + atom.xml + sup + "</msup>";
      } else if (sub) {
        out += atom.large
          ? "<munder>" + atom.xml + sub + "</munder>"
          : "<msub>" + atom.xml + sub + "</msub>";
      } else {
        out += atom.xml;
      }
    }
    return { xml: out, next: i };
  }

  function latexToMathml(latex, display, label) {
    var src = String(latex == null ? "" : latex).slice(0, LIMITS.input);
    var state = { count: 0 };
    try {
      var parsed = parseSequence(src, 0, 0, state, null);
      var content = "<mrow>" + parsed.xml + "</mrow>";
      if (!parsed.xml) content = "<mtext>" + esc(src) + "</mtext>";
      return mathWrap(content, !!display, label);
    } catch (error) {
      return mathWrap("<mtext>" + esc(src) + "</mtext>", !!display, label);
    }
  }
  NS.latexToMathml = latexToMathml;

  function isBalanced(latex) {
    var src = String(latex == null ? "" : latex);
    var depth = 0;
    for (var i = 0; i < src.length; i += 1) {
      if (src[i] === "{") depth += 1;
      if (src[i] === "}") { depth -= 1; if (depth < 0) return false; }
    }
    if (depth !== 0) return false;
    var begins = (src.match(/\\begin\{/g) || []).length;
    var ends = (src.match(/\\end\{/g) || []).length;
    return begins === ends;
  }
  NS.isBalanced = isBalanced;

  function describe(latex) {
    var term = tr("docs-eq-term", "Equation");
    var src = String(latex == null ? "" : latex).trim().slice(0, 240);
    return src ? term + ": " + src : term;
  }
  NS.describe = describe;
})(window);
