"use strict";
/* MathLite — minimal local LaTeX → MathML/HTML converter
 * Supports: \frac, \sqrt, ^{}, _{}, \sum, \int, \prod, Greek letters, operators, parentheses
 * Output: inline HTML with semantic <span> tags (no MathML dependency)
 * For rendering to canvas, use MathLite.toText() or MathLite.toHTML().
 */
(function (window) {
  const GREEK = {
    ALPHA: "α", BETA: "β", GAMMA: "γ", DELTA: "δ", EPSILON: "ε", ZETA: "ζ",
    ETA: "η", THETA: "θ", IOTA: "ι", KAPPA: "κ", LAMBDA: "λ", MU: "μ",
    NU: "ν", XI: "ξ", OMICRON: "ο", PI: "π", RHO: "ρ", SIGMA: "σ",
    TAU: "τ", UPSILON: "υ", PHI: "φ", CHI: "χ", PSI: "ψ", OMEGA: "ω",
    ALPHA_: "Α", BETA_: "Β", GAMMA_: "Γ", DELTA_: "Δ", EPSILON_: "Ε", ZETA_: "Ζ",
    ETA_: "Η", THETA_: "Θ", IOTA_: "Ι", KAPPA_: "Κ", LAMBDA_: "Λ", MU_: "Μ",
    NU_: "Ν", XI_: "Ξ", OMICRON_: "Ο", PI_: "Π", RHO_: "Ρ", SIGMA_: "Σ",
    TAU_: "Τ", UPSILON_: "Υ", PHI_: "Φ", CHI_: "Χ", PSI_: "Ψ", OMEGA_: "Ω"
  };
  const SYMBOLS = {
    INFTY: "∞", PARTIAL: "∂", NABLA: "∇", FORALL: "∀", EXISTS: "∃",
    IN: "∈", NOTIN: "∉", SUBSET: "⊂", SUPSET: "⊃", UNION: "∪", INTERSECT: "∩",
    EMPTY: "∅", REALS: "ℝ", INTEGERS: "ℤ", NATURALS: "ℕ", RATIONALS: "ℚ", COMPLEX: "ℂ",
    LEQ: "≤", GEQ: "≥", NEQ: "≠", APPROX: "≈", EQUIV: "≡", PROPTO: "∝",
    TIMES: "×", DIVIDE: "÷", PM: "±", MP: "∓", CDOT: "·", AST: "∗",
    TO: "→", LEFTARROW: "←", RIGHTARROW: "→", UPARROW: "↑", DOWNARROW: "↓",
    LEFTRIGHTARROW: "↔", MAPSTO: "↦", SUM: "∑", PROD: "∏", INT: "∫", IINT: "∬", IIINT: "∭",
    oint: "∮", BIGO: "○", LOG: "log", LN: "ln", SIN: "sin", COS: "cos", TAN: "tan"
  };

  function escape(s) { return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;"); }

  function lookupGreek(name) {
    if (GREEK[name] !== undefined) return GREEK[name];
    if (GREEK[name + "_"] !== undefined) return GREEK[name + "_"];
    return null;
  }

  function lookupSymbol(name) {
    if (SYMBOLS[name] !== undefined) return SYMBOLS[name];
    if (SYMBOLS[name.toUpperCase()] !== undefined) return SYMBOLS[name.toUpperCase()];
    return null;
  }

  function findMatchingBrace(s, openIdx) {
    let depth = 0;
    for (let i = openIdx; i < s.length; i++) {
      if (s[i] === "{") depth++;
      else if (s[i] === "}") { depth--; if (depth === 0) return i; }
    }
    return -1;
  }

  function findArg(s, i) {
    while (i < s.length && /\s/.test(s[i])) i++;
    if (i >= s.length) return ["", i];
    if (s[i] === "{") {
      const end = findMatchingBrace(s, i);
      if (end < 0) return [s.substring(i), s.length];
      return [s.substring(i + 1, end), end + 1];
    }
    if (s[i] === "\\") {
      const m = s.substring(i).match(/^\\([a-zA-Z]+)/);
      if (m) return [m[1], i + m[0].length];
      return [s.substring(i, i + 1), i + 1];
    }
    return [s[i], i + 1];
  }

  function processSegment(s) {
    let out = "";
    let i = 0;
    while (i < s.length) {
      if (s[i] === "\\") {
        const m = s.substring(i).match(/^\\([a-zA-Z]+|.)/);
        if (m) {
          const cmd = m[1];
          const newI = i + m[0].length;
          if (cmd === "frac") {
            const [num, after1] = findArg(s, newI);
            const [den, after2] = findArg(s, after1);
            out += '<span class="math-frac"><span class="math-num">' + toHTML(num) + '</span><span class="math-den">' + toHTML(den) + '</span></span>';
            i = after2; continue;
          }
          if (cmd === "sqrt") {
            const [body, after] = findArg(s, newI);
            out += '<span class="math-sqrt">√' + toHTML(body) + '</span>';
            i = after; continue;
          }
          if (cmd === "overline" || cmd === "bar") {
            const [body, after] = findArg(s, newI);
            out += '<span class="math-bar">' + toHTML(body) + '</span>';
            i = after; continue;
          }
          if (cmd === "hat") {
            const [body, after] = findArg(s, newI);
            out += '<span class="math-hat">' + toHTML(body) + '</span>';
            i = after; continue;
          }
          if (cmd === "vec") {
            const [body, after] = findArg(s, newI);
            out += '<span style="text-decoration:overline;font-weight:600">' + toHTML(body) + '</span>';
            i = after; continue;
          }
          if (cmd === "mathbb" || cmd === "mathbf" || cmd === "mathit" || cmd === "mathrm") {
            const style = { mathbb: "double-struck", mathbf: "bold", mathit: "italic", mathrm: "roman" }[cmd];
            const [body, after] = findArg(s, newI);
            out += '<span style="font-' + style + '">' + toHTML(body) + '</span>';
            i = after; continue;
          }
          if (cmd === "begin") {
            const end = s.indexOf("\\end", newI);
            if (end > 0) {
              const block = s.substring(newI, end);
              const envMatch = block.match(/^\{([^}]+)\}([\s\S]*)/);
              if (envMatch) {
                const env = envMatch[1];
                const content = envMatch[2];
                if (env === "matrix" || env === "pmatrix" || env === "bmatrix") {
                  const open = env === "pmatrix" ? "(" : env === "bmatrix" ? "[" : "";
                  const close = env === "pmatrix" ? ")" : env === "bmatrix" ? "]" : "";
                  const rows = content.split("\\\\");
                  let matrix = '<span class="math-matrix">' + open + '<table>';
                  rows.forEach(function (r) {
                    matrix += "<tr>" + r.split("&").map(function (c) { return "<td>" + toHTML(c) + "</td>"; }).join("") + "</tr>";
                  });
                  matrix += "</table>" + close + "</span>";
                  out += matrix;
                  i = end + 4 + env.length + 1;
                  continue;
                }
              }
            }
            i = newI; continue;
          }
          const greek = lookupGreek(cmd);
          if (greek) { out += greek; i = newI; continue; }
          const sym = lookupSymbol(cmd);
          if (sym) { out += sym; i = newI; continue; }
          if (cmd === "left" || cmd === "right") {
            const [ch, after] = findArg(s, newI);
            out += ch === "." ? "" : ch;
            i = after; continue;
          }
          if (cmd === ",") { out += " "; i = newI; continue; }
          if (cmd === ";") { out += " "; i = newI; continue; }
          if (cmd === "!") { i = newI; continue; }
          if (cmd === "quad") { out += "&nbsp;&nbsp;&nbsp;&nbsp;"; i = newI; continue; }
          if (cmd === "qquad") { out += "&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;"; i = newI; continue; }
          out += " " + cmd + " ";
          i = newI; continue;
        }
        i++; continue;
      }
      if (s[i] === "^") {
        i++;
        if (i < s.length && s[i] === "{") {
          const end = findMatchingBrace(s, i);
          out += '<sup>' + toHTML(s.substring(i + 1, end)) + '</sup>';
          i = end + 1; continue;
        }
        if (i < s.length) {
          out += '<sup>' + toHTML(s[i]) + '</sup>';
          i++; continue;
        }
        continue;
      }
      if (s[i] === "_") {
        i++;
        if (i < s.length && s[i] === "{") {
          const end = findMatchingBrace(s, i);
          out += '<sub>' + toHTML(s.substring(i + 1, end)) + '</sub>';
          i = end + 1; continue;
        }
        if (i < s.length) {
          out += '<sub>' + toHTML(s[i]) + '</sub>';
          i++; continue;
        }
        continue;
      }
      if (s[i] === "{") {
        const end = findMatchingBrace(s, i);
        if (end > 0) {
          out += toHTML(s.substring(i + 1, end));
          i = end + 1; continue;
        }
      }
      if (s[i] === "<") { out += "&lt;"; i++; continue; }
      if (s[i] === ">") { out += "&gt;"; i++; continue; }
      if (s[i] === "&") { out += "&amp;"; i++; continue; }
      out += s[i];
      i++;
    }
    return out;
  }

  function toHTML(latex) {
    if (!latex) return "";
    return processSegment(latex);
  }

  function toText(latex) {
    if (!latex) return "";
    const div = document.createElement("div");
    div.innerHTML = toHTML(latex);
    return div.textContent || div.innerText || "";
  }

  function render(latex, target, displayMode) {
    if (typeof target === "string") target = document.getElementById(target);
    if (!target) return "";
    const html = toHTML(latex);
    const cls = displayMode ? "math-display" : "math-inline";
    target.innerHTML = '<span class="' + cls + '">' + html + '</span>';
    return html;
  }

  function injectStyles() {
    if (document.getElementById("mathlite-styles")) return;
    const style = document.createElement("style");
    style.id = "mathlite-styles";
    style.textContent = ".math-display{display:block;text-align:center;margin:12px 0;font-size:18px;}.math-inline{display:inline-block;}.math-frac{display:inline-block;vertical-align:middle;text-align:center;font-size:0.9em;padding:0 4px;}.math-frac .math-num{display:block;border-bottom:1px solid currentColor;padding:0 4px;}.math-frac .math-den{display:block;padding:0 4px;}.math-sqrt{border-top:1px solid currentColor;padding:2px 4px;display:inline-block;}.math-matrix{display:inline-block;vertical-align:middle;}.math-matrix table{border-collapse:collapse;}.math-matrix td{padding:2px 8px;text-align:center;}";
    document.head.appendChild(style);
  }

  function examples() {
    return [
      { label: "Equação quadrática", latex: "x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}" },
      { label: "Pitágoras", latex: "a^2 + b^2 = c^2" },
      { label: "Integral", latex: "\\int_{0}^{\\infty} e^{-x^2} dx = \\frac{\\sqrt{\\pi}}{2}" },
      { label: "Somatório", latex: "\\sum_{i=1}^{n} i = \\frac{n(n+1)}{2}" },
      { label: "Matriz", latex: "\\begin{pmatrix} a & b \\\\ c & d \\end{pmatrix}" },
      { label: "Euler", latex: "e^{i\\pi} + 1 = 0" }
    ];
  }

  window.MathLite = { toHTML: toHTML, toText: toText, render: render, injectStyles: injectStyles, examples: examples, lookupGreek: lookupGreek, lookupSymbol: lookupSymbol };
})(window);
