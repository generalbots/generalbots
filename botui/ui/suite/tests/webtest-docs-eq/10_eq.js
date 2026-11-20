/**
 * #2483 offline verification of docs/modules/38_equations_latex.js.
 *
 * The shipped module is an IIFE bound to window.DocsEquations. Nothing touches
 * document at load time, so the REAL file is executed in a vm sandbox with a
 * stub window; exported pure helpers (esc, latexToMathml, isBalanced, describe,
 * mathWrap, LIMITS, PALETTE, TEMPLATES) are exercised as genuine runtime calls.
 * The three internal helpers (parseCommand, parseMatrix, budget) are not
 * exported, so their guards are covered transitively plus clearly-labelled
 * STATIC source assertions. No production file is modified here.
 */
"use strict";

var h = require("./00_harness");
var test = h.test;
var assert = h.assert;
var readFile = h.readFile;
var loadScript = h.loadScript;

var SOURCE = "docs/modules/38_equations_latex.js";

/** Load a fresh instance of the real shipped module. */
function fixture(options) {
  var opts = options || {};
  var context = loadScript(SOURCE, {
    window: opts.window || {},
    console: console,
  });
  return { context: context, NS: context.window.DocsEquations };
}

function count(haystack, needle) {
  return haystack.split(needle).length - 1;
}

function sourceLine(src, needle) {
  var lines = src.split("\n");
  for (var i = 0; i < lines.length; i += 1) {
    if (lines[i].indexOf(needle) >= 0) return { number: i + 1, text: lines[i] };
  }
  return { number: -1, text: "" };
}

// ── Load / export surface ─────────────────────────────────────────────

test("RUNTIME #2483: real module loads in a DOM-free sandbox and exports its pure API", function () {
  var f = fixture();
  ["esc", "latexToMathml", "isBalanced", "describe", "mathWrap"].forEach(function (name) {
    assert(typeof f.NS[name] === "function", name + " must be exported on DocsEquations");
  });
  assert(f.NS.LIMITS && f.NS.PALETTE && f.NS.TEMPLATES, "constants must be exported");
  assert(f.NS.PALETTE.length > 0 && f.NS.TEMPLATES.length > 0, "palette/templates must be non-empty");
});

// ── esc (XSS-safe escaping) ───────────────────────────────────────────

test("RUNTIME #2483: esc escapes every HTML-significant character", function () {
  var f = fixture();
  assert(f.NS.esc("&<>\"'") === "&amp;&lt;&gt;&quot;&#39;", "esc must escape & < > \" and '");
});

test("RUNTIME #2483: esc coerces null/undefined to an empty string", function () {
  var f = fixture();
  assert(f.NS.esc(null) === "" && f.NS.esc(undefined) === "", "null/undefined must become empty text");
});

// ── Converter structure ───────────────────────────────────────────────

test("RUNTIME #2483: x^2 produces an msup structure", function () {
  var out = fixture().NS.latexToMathml("x^2");
  assert(out.indexOf("<msup>") >= 0, "superscript must be <msup>");
  assert(out.indexOf("<mi>x</mi>") >= 0 && out.indexOf("<mn>2</mn>") >= 0, "base/exponent must be mi/mn");
});

test("RUNTIME #2483: \\frac{a}{b} produces an mfrac structure", function () {
  var out = fixture().NS.latexToMathml("\\frac{a}{b}");
  assert(out.indexOf("<mfrac>") >= 0, "fraction must be <mfrac>");
  assert(count(out, "<mrow>") === 3, "numerator and denominator must each be an mrow inside the root mrow");
});

test("RUNTIME #2483: \\sqrt{x} produces an msqrt structure", function () {
  var out = fixture().NS.latexToMathml("\\sqrt{x}");
  assert(out.indexOf("<msqrt>") >= 0, "square root must be <msqrt>");
  assert(out.indexOf("<mi>x</mi>") >= 0, "radicand must be preserved");
});

test("RUNTIME #2483: matrix environment produces mtable rows and cells", function () {
  var tex = "\\begin{matrix}a & b \\\\ c & d\\end{matrix}";
  var out = fixture().NS.latexToMathml(tex);
  assert(count(out, "<mtable>") === 1, "matrix must emit exactly one mtable");
  assert(count(out, "<mtr>") === 2, "matrix must emit two rows");
  assert(count(out, "<mtd>") === 4, "matrix must emit four cells");
  assert(out.indexOf("<mi>a</mi>") >= 0 && out.indexOf("<mi>d</mi>") >= 0, "cell content must be preserved");
});

test("RUNTIME #2483: trailing \\end{env} does NOT leak into the last matrix cell", function () {
  var tex = "\\begin{matrix}a & b \\\\ c & d\\end{matrix}";
  var out = fixture().NS.latexToMathml(tex);
  var lastCell = out.slice(out.lastIndexOf("<mtd>"));
  assert(lastCell.indexOf("<mi>end</mi>") < 0 && lastCell.indexOf("matrix") < 0,
    "parseMatrix stops at the 5-char \\end{ marker; the end marker must not be parsed as cell content");
});

test("RUNTIME #2483: cases environment adds stretchy fences around its mtable", function () {
  var tex = "\\begin{cases}x & x\\ge 0 \\\\ -x & x<0\\end{cases}";
  var out = fixture().NS.latexToMathml(tex);
  assert(count(out, 'fence="true"') === 2, "cases must emit an opening and a closing fence");
  assert(out.indexOf("<mtable>") >= 0 && out.indexOf("<mo>&lt;</mo>") >= 0, "cases table and escaped '<' must be present");
});

test("RUNTIME #2483: large operators with both scripts use munderover", function () {
  var out = fixture().NS.latexToMathml("\\sum_{i=1}^{n} i^2");
  assert(out.indexOf("<munderover>") >= 0, "large op with sub+sup must be munderover");
  assert(out.indexOf('<mo largeop="true">∑</mo>') >= 0, "sum must be a large operator glyph");
  assert(out.indexOf("<msup>") >= 0, "trailing i^2 must remain an msup");
});

test("RUNTIME #2483: palette symbols and greek letters map to MathML tokens", function () {
  var out = fixture().NS.latexToMathml("\\times \\alpha");
  assert(out.indexOf("<mo>×</mo>") >= 0, "\\times must be a mo glyph");
  assert(out.indexOf("<mi>α</mi>") >= 0, "\\alpha must be an mi glyph");
});

test("RUNTIME #2483: \\text{...} strips inner tags and keeps escaped text", function () {
  var out = fixture().NS.latexToMathml("\\text{a<b}");
  assert(out.indexOf("<mtext>a&lt;b</mtext>") >= 0, "text content must be escaped, with no nested markup");
  assert(out.indexOf("<mi>") < 0, "no element markup may leak into mtext");
});

test("RUNTIME #2483: unknown commands degrade to mi instead of throwing", function () {
  var out = fixture().NS.latexToMathml("\\foobar");
  assert(out.indexOf("<mi>foobar</mi>") >= 0, "unknown command must degrade to an mi node");
});

// ── isBalanced ────────────────────────────────────────────────────────

test("RUNTIME #2483: isBalanced accepts matched braces and environments", function () {
  var NS = fixture().NS;
  assert(NS.isBalanced("{a}{b}") === true, "matched braces must be balanced");
  assert(NS.isBalanced("\\begin{matrix}x\\end{matrix}") === true, "matched env must be balanced");
});

test("RUNTIME #2483: isBalanced rejects unmatched braces and env mismatch", function () {
  var NS = fixture().NS;
  assert(NS.isBalanced("{a") === false, "unclosed brace must be unbalanced");
  assert(NS.isBalanced("a}") === false, "extra closing brace must be unbalanced");
  assert(NS.isBalanced("\\begin{matrix}x") === false, "missing end env must be unbalanced");
});

// ── describe ──────────────────────────────────────────────────────────

test("RUNTIME #2483: describe is deterministic, non-empty and formula-specific", function () {
  var NS = fixture().NS;
  var a = NS.describe("x^2");
  var b = NS.describe("x^2");
  assert(typeof a === "string" && a.length > 0, "describe must return a non-empty label");
  assert(a === b, "same input must produce the same label");
  assert(a.indexOf("x^2") >= 0, "label must include the formula source");
  assert(NS.describe("y^3") !== a, "different formulas must produce different labels");
});

// ── mathWrap / display vs inline ──────────────────────────────────────

test("RUNTIME #2483: mathWrap switches the display attribute and escapes labels", function () {
  var NS = fixture().NS;
  var inline = NS.mathWrap("X", false);
  var block = NS.mathWrap("X", true);
  var labelled = NS.mathWrap("X", false, 'a"b');
  assert(inline.indexOf('display="inline"') >= 0, "inline wrap must mark display=inline");
  assert(block.indexOf('display="block"') >= 0, "display wrap must mark display=block");
  assert(labelled.indexOf('aria-label="a&quot;b"') >= 0, "label must be escaped into aria-label");
});

test("RUNTIME #2483: latexToMathml display flag changes the wrapper only", function () {
  var NS = fixture().NS;
  var inline = NS.latexToMathml("x", false);
  var block = NS.latexToMathml("x", true);
  assert(inline.indexOf('display="inline"') >= 0 && block.indexOf('display="block"') >= 0,
    "display flag must control the math wrapper");
  function withoutDisplay(value) {
    return value.replace(/display="(inline|block)"/, "");
  }
  assert(withoutDisplay(inline) === withoutDisplay(block),
    "only the display attribute may differ between the two modes");
});

// ── budget / bounded input ────────────────────────────────────────────

test("RUNTIME #2483: declared budgets are the documented caps", function () {
  var L = fixture().NS.LIMITS;
  assert(L.input === 2000 && L.depth === 12 && L.nodes === 400, "caps must be 2000 chars / depth 12 / 400 nodes");
});

test("RUNTIME #2483: deeply nested input terminates within budget without overflow", function () {
  var deep = "{".repeat(5000) + "x" + "}".repeat(5000);
  var start = Date.now();
  var out = fixture().NS.latexToMathml(deep, false, null);
  var elapsed = Date.now() - start;
  assert(out.indexOf("<math") === 0, "deep input must still produce a math element");
  assert(out.length < 1000, "depth cap must keep output bounded, got " + out.length + " chars");
  assert(elapsed < 5000, "parser must terminate promptly, took " + elapsed + "ms");
});

test("RUNTIME #2483: input and node caps bound oversized formulas", function () {
  var huge = "x".repeat(5000) + "SENTINEL";
  var out = fixture().NS.latexToMathml(huge, false, null);
  assert(out.indexOf("SENTINEL") < 0, "input beyond the char cap must be sliced away");
  assert(count(out, "<mi>") <= 400, "node budget must cap emitted nodes at 400");
});

// ── STATIC coverage for the three non-exported helpers ────────────────

test("STATIC #2483: parseCommand, parseMatrix and budget are internal helpers", function () {
  var src = readFile(SOURCE);
  ["function budget(state)", "function parseCommand(tex, i, depth, state)",
    "function parseMatrix(tex, start, env, depth, state)"].forEach(function (needle) {
    assert(src.indexOf(needle) >= 0, "source must define " + needle);
  });
  ["NS.parseCommand", "NS.parseMatrix", "NS.budget"].forEach(function (needle) {
    assert(src.indexOf(needle) < 0, needle + " must stay private to the IIFE");
  });
});

test("STATIC #2483: budget enforces node/depth/input caps at the recursive steps", function () {
  var src = readFile(SOURCE);
  assert(src.indexOf("state.count <= LIMITS.nodes") >= 0, "node cap must gate parseSequence");
  assert(src.indexOf("depth > LIMITS.depth") >= 0, "depth cap must gate parseAtom");
  assert(src.indexOf(".slice(0, LIMITS.input)") >= 0, "input cap must gate latexToMathml");
  assert(src.indexOf("if (!budget(state)") >= 0, "parseAtom must call budget before recursing");
});

test("STATIC #2483: parseMatrix bounds rows/cells, scans the 5-char end marker and emits mtable", function () {
  var src = readFile(SOURCE);
  assert(src.indexOf('out += "<mtable>"') >= 0, "parseMatrix must emit mtable");
  assert(src.indexOf("body.split") >= 0 && src.indexOf(".slice(0, 20)") >= 0, "rows must be capped at 20");
  assert(src.indexOf('.split("&").slice(0, 10)') >= 0, "cells must be capped at 10");
  var line = sourceLine(src, "j + 5");
  assert(line.number > 0 && line.text.indexOf("end{") >= 0,
    "parseMatrix scans the 5-char \\end{ marker so it is not parsed as cell content");
});
