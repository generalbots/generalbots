"use strict";

/**
 * Module 28: Inline text runs runtime for Slides (P0 critical).
 * Refactors text elements to use an array of runs (text, bold, italic,
 * underline, color, size, font) instead of a single string. Renders as
 * multiple <span> elements. The toolbar Bold/Italic/Underline/Color
 * buttons now apply only to the current selection (or future typing
 * if collapsed) by splitting runs at the caret and inserting new runs.
 *
 * Public API: window.SlidesTextRunsRuntime = {
 *   ensureRuns, getSelectionOffset, applyFormat, renderRuns,
 *   splitAt, normalize, getCaretRun, hasFormat, isElementText
 * }.
 */

(function () {
  function getState() { return window.state || null; }
  function getCanvas() { return document.querySelector(".slide-canvas, .slides-canvas, #slideCanvas"); }

  function getActiveElement() {
    return document.querySelector(".slide-element.editing") || document.querySelector(".slide-element.selected") || (document.querySelector(".slide-element[data-active='1']"));
  }

  function isElementText(el) {
    if (!el) return false;
    const slide = findSlideForElement(el);
    if (!slide) return false;
    return (slide.elements || []).some(function (e) { return e.domRef === el; });
  }

  function findSlideForElement(el) {
    if (!el) return null;
    const id = el.dataset.elementId || el.id;
    const s = getState();
    if (!s) return null;
    for (const slide of s.slides || []) {
      for (const e of slide.elements || []) {
        if (e.domRef === el || e.id === id) return slide;
      }
    }
    return null;
  }

  function getElementData(el) {
    const slide = findSlideForElement(el);
    if (!slide) return null;
    const id = el.dataset.elementId || el.id;
    return (slide.elements || []).find(function (e) { return e.id === id || e.domRef === el; }) || null;
  }

  function ensureRuns(data) {
    if (!data) return null;
    if (!data.runs && data.text != null) {
      data.runs = [{ text: String(data.text), bold: !!data.bold, italic: !!data.italic, underline: !!data.underline, color: data.color || null, size: data.size || null, font: data.font || null }];
      delete data.text;
    }
    if (!data.runs) data.runs = [];
    if (data.runs.length === 0) data.runs.push({ text: "" });
    return data;
  }

  function renderRuns(el, data) {
    ensureRuns(data);
    el.innerHTML = "";
    for (const r of data.runs) {
      if (!r.text) continue;
      const span = document.createElement("span");
      span.className = "run";
      let style = "";
      if (r.bold) style += "font-weight:bold;";
      if (r.italic) style += "font-style:italic;";
      if (r.underline) style += "text-decoration:underline;";
      if (r.color) style += "color:" + r.color + ";";
      if (r.size) style += "font-size:" + r.size + "px;";
      if (r.font) style += "font-family:" + r.font + ";";
      if (r.bg) style += "background-color:" + r.bg + ";";
      if (style) span.style.cssText = style;
      span.textContent = r.text;
      el.appendChild(span);
    }
  }

  function getCaretRun(el) {
    if (!el) return null;
    const sel = window.getSelection ? window.getSelection() : null;
    if (!sel || sel.rangeCount === 0) return null;
    const range = sel.getRangeAt(0);
    let node = range.startContainer;
    if (node.nodeType === 3) node = node.parentNode;
    if (!node || !node.classList || !node.classList.contains("run")) return null;
    const data = getElementData(el);
    if (!data) return null;
    ensureRuns(data);
    const idx = Array.from(el.querySelectorAll(".run")).indexOf(node);
    if (idx < 0) return null;
    const offset = sel.anchorOffset;
    return { run: data.runs[idx], runIndex: idx, offset: offset, data: data };
  }

  function applyFormat(el, format, value) {
    if (!el) return false;
    const data = getElementData(el);
    if (!data) return false;
    ensureRuns(data);
    const sel = window.getSelection ? window.getSelection() : null;
    if (!sel || sel.rangeCount === 0) {
      for (const r of data.runs) { if (format !== "color" && format !== "size" && format !== "font" && format !== "bg") r[format] = value; }
      renderRuns(el, data);
      return true;
    }
    if (sel.isCollapsed) {
      const cr = getCaretRun(el);
      if (!cr) return false;
      const offset = cr.offset;
      if (offset === 0) {
        const newRun = Object.assign({}, cr.run);
        newRun[format] = value;
        data.runs.splice(cr.runIndex, 0, newRun);
      } else if (offset === cr.run.text.length) {
        const newRun = Object.assign({}, cr.run);
        newRun[format] = value;
        data.runs.splice(cr.runIndex + 1, 0, newRun);
      } else {
        const left = { text: cr.run.text.slice(0, offset) };
        const right = { text: cr.run.text.slice(offset) };
        const middle = Object.assign({}, cr.run, { text: "" });
        const newRun = Object.assign({}, cr.run, { [format]: value, text: "" });
        data.runs.splice(cr.runIndex, 1, left, newRun, middle, right);
      }
      renderRuns(el, data);
      return true;
    }
    const range = sel.getRangeAt(0);
    if (!el.contains(range.startContainer) || !el.contains(range.endContainer)) return false;
    const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT, { acceptNode: function (n) { return range.intersectsNode(n) ? NodeFilter.FILTER_ACCEPT : NodeFilter.FILTER_REJECT; } });
    const ranges = [];
    let n = walker.nextNode();
    while (n) {
      const r = document.createRange();
      if (n === range.startContainer) r.setStart(n, range.startOffset);
      else r.setStart(n, 0);
      if (n === range.endContainer) r.setEnd(n, range.endOffset);
      else r.setEnd(n, n.length);
      ranges.push({ node: n, range: r });
      n = walker.nextNode();
    }
    for (const item of ranges) {
      const node = item.node;
      const runSpan = node.parentNode;
      const runIndex = Array.from(el.querySelectorAll(".run")).indexOf(runSpan);
      if (runIndex < 0) continue;
      const run = data.runs[runIndex];
      const startOff = item.range.startOffset;
      const endOff = item.range.endOffset;
      const left = { text: run.text.slice(0, startOff) };
      const middle = { text: run.text.slice(startOff, endOff) };
      middle[format] = value;
      const right = { text: run.text.slice(endOff) };
      const replace = [];
      if (left.text) replace.push(Object.assign({}, run, left));
      if (middle.text) replace.push(Object.assign({}, run, middle));
      if (right.text) replace.push(Object.assign({}, run, right));
      data.runs.splice(runIndex, 1, ...replace);
    }
    renderRuns(el, data);
    return true;
  }

  function splitAt(run, offset) {
    if (offset <= 0 || offset >= run.text.length) return [run, null];
    return [
      Object.assign({}, run, { text: run.text.slice(0, offset) }),
      Object.assign({}, run, { text: run.text.slice(offset) }),
    ];
  }

  function normalize(data) {
    ensureRuns(data);
    const out = [];
    for (const r of data.runs) {
      if (!r.text) continue;
      const last = out[out.length - 1];
      if (last && last.bold === r.bold && last.italic === r.italic && last.underline === r.underline && last.color === r.color && last.size === r.size && last.font === r.font && last.bg === r.bg) {
        last.text += r.text;
      } else {
        out.push(Object.assign({}, r));
      }
    }
    data.runs = out;
    return data;
  }

  function hasFormat(el, format) {
    const cr = getCaretRun(el);
    if (!cr) return false;
    return !!cr.run[format];
  }

  function attach() {
    const buttons = document.querySelectorAll("[data-format-cmd]");
    buttons.forEach(function (b) {
      b.addEventListener("mousedown", function (e) { e.preventDefault(); });
      b.addEventListener("click", function (e) {
        e.preventDefault();
        const el = getActiveElement();
        if (!el) return;
        const cmd = b.getAttribute("data-format-cmd");
        const val = b.getAttribute("data-format-value") || true;
        applyFormat(el, cmd, val);
      });
    });
    const ffamily = document.getElementById("fontFamily");
    if (ffamily) ffamily.addEventListener("change", function (e) { const el = getActiveElement(); if (el) applyFormat(el, "font", e.target.value); });
    const fsize = document.getElementById("fontSize");
    if (fsize) fsize.addEventListener("change", function (e) { const el = getActiveElement(); if (el) applyFormat(el, "size", parseInt(e.target.value, 10)); });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 50);
  }

  window.SlidesTextRunsRuntime = { ensureRuns, renderRuns, getCaretRun, applyFormat, splitAt, normalize, hasFormat, getElementData, isElementText, getActiveElement };
})();
