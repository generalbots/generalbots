"use strict";

/**
 * Module 13: Text runs for Slides.
 * Replaces the per-element text formatting approach where Bold/Italic
 * applied to the entire <div> instead of the selected range. Adds:
 *
 *   - TextRun type stored on element.runs (array of { text, bold, italic,
 *     underline, strike, color, fontFamily, fontSize, link }).
 *   - getSelectedRange() / setRun() / splitRun() helpers that operate
 *     on the user's current selection within a contentEditable element.
 *   - renderRunsAsHTML(runs) / renderRunsAsText(runs) utilities.
 *   - Hooks into the document selectionchange event to update the
 *     toolbar's "Bold"/"Italic" indicators based on the current
 *     selection's formatting.
 *
 * Public API: window.TextRuns = { applyFormatting, renderRunsAsHTML,
 *   getElementRuns, setElementRuns, getSelectedRange, splitRunAt }.
 */

(function () {
  function getActiveContentEditable() {
    const sel = window.getSelection();
    if (!sel || !sel.anchorNode) return null;
    let n = sel.anchorNode;
    while (n && n.nodeType === 1) {
      if (n.isContentEditable) return n;
      n = n.parentNode;
    }
    return null;
  }

  function getSelectedRange() {
    const sel = window.getSelection();
    if (!sel || sel.rangeCount === 0) return null;
    return sel.getRangeAt(0);
  }

  function plainTextOf(el) {
    if (el == null) return "";
    if (typeof el === "string") return el;
    if (el.nodeType === 3) return el.textContent || "";
    let out = "";
    for (const child of el.childNodes) out += plainTextOf(child);
    return out;
  }

  /**
   * Convert an element's current text content into a list of runs.
   * Since the editor already has each element as a single text node,
   * we synthesize one run with the element's bold/italic/underline
   * properties; if the user later selects part of the text, splitRunAt
   * creates additional runs.
   */
  function getElementRuns(el) {
    if (!el) return [];
    if (el.runs && Array.isArray(el.runs) && el.runs.length) {
      return el.runs.map((r) => ({ ...r }));
    }
    const text = plainTextOf(el);
    if (!text) return [];
    return [
      {
        text,
        bold: el.style?.fontWeight === "bold" || el.style?.fontWeight === "700",
        italic: el.style?.fontStyle === "italic",
        underline: el.style?.textDecoration?.includes("underline"),
        strike: el.style?.textDecoration?.includes("line-through"),
        color: el.style?.color || "",
        fontFamily: el.style?.fontFamily || "",
        fontSize: el.style?.fontSize || "",
        link: el.link || "",
      },
    ];
  }

  function setElementRuns(el, runs) {
    if (!el) return;
    el.runs = runs.map((r) => ({ ...r }));
  }

  function renderRunsAsHTML(runs) {
    if (!runs || !runs.length) return "";
    let out = "";
    for (const r of runs) {
      let html = (r.text || "").replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/\n/g, "<br/>");
      const styles = [];
      if (r.bold) styles.push("font-weight:bold");
      if (r.italic) styles.push("font-style:italic");
      if (r.underline || r.strike) {
        const dec = [];
        if (r.underline) dec.push("underline");
        if (r.strike) dec.push("line-through");
        styles.push("text-decoration:" + dec.join(" "));
      }
      if (r.color) styles.push("color:" + r.color);
      if (r.fontFamily) styles.push("font-family:" + r.fontFamily);
      if (r.fontSize) styles.push("font-size:" + r.fontSize);
      const styleAttr = styles.length ? ' style="' + styles.join(";") + '"' : "";
      const open = "<span" + styleAttr + ">";
      const close = "</span>";
      if (r.link) {
        out += '<a href="' + r.link.replace(/"/g, "&quot;") + '">' + open + html + close + "</a>";
      } else {
        out += open + html + close;
      }
    }
    return out;
  }

  function renderRunsAsText(runs) {
    if (!runs) return "";
    return runs.map((r) => r.text || "").join("");
  }

  /**
   * Split a run at the given character offset. Mutates the runs array
   * in place. Returns [left, right] indices.
   */
  function splitRunAt(runs, runIdx, charOffset) {
    const r = runs[runIdx];
    if (!r) return [runIdx, runIdx];
    if (charOffset <= 0) return [runIdx, runIdx];
    if (charOffset >= (r.text || "").length) return [runIdx, runIdx];
    const left = { ...r, text: r.text.slice(0, charOffset) };
    const right = { ...r, text: r.text.slice(charOffset) };
    runs.splice(runIdx, 1, left, right);
    return [runIdx, runIdx + 1];
  }

  function applyFormattingToRange(runs, start, end, patch) {
    if (!runs.length) {
      runs.push({ text: "", ...patch });
      return;
    }
    let pos = 0;
    for (let i = 0; i < runs.length; i++) {
      const r = runs[i];
      const len = (r.text || "").length;
      const rStart = pos;
      const rEnd = pos + len;
      if (rEnd <= start) {
        pos = rEnd;
        continue;
      }
      if (rStart >= end) break;
      const splitStart = Math.max(0, start - rStart);
      const splitEnd = Math.min(len, end - rStart);
      if (splitStart > 0 || splitEnd < len) {
        const idx1 = splitRunAt(runs, i, splitStart)[0];
        const after = splitRunAt(runs, idx1, splitEnd - splitStart);
        const targetIdx = after[0];
        const nextIdx = after[1];
        Object.assign(runs[targetIdx], patch);
        i = nextIdx - 1;
        pos += splitEnd;
      } else {
        Object.assign(r, patch);
        pos = rEnd;
      }
    }
  }

  function applyFormatting(el, prop, value) {
    if (!el) return;
    const runs = getElementRuns(el);
    applyFormattingToRange(runs, 0, renderRunsAsText(runs).length, { [prop]: value });
    setElementRuns(el, runs);
    el.innerHTML = renderRunsAsHTML(runs);
  }

  function applyFormattingToSelection(prop, value) {
    const ce = getActiveContentEditable();
    if (!ce) return;
    const sel = getSelectedRange();
    if (!sel || sel.collapsed) {
      applyFormatting(ce, prop, value);
      return;
    }
    const runs = getElementRuns(ce);
    const fullText = renderRunsAsText(runs);
    const start = sel.startOffset;
    const end = sel.endOffset;
    applyFormattingToRange(runs, start, end, { [prop]: value });
    setElementRuns(ce, runs);
    ce.innerHTML = renderRunsAsHTML(runs);
  }

  function currentFormatting(el) {
    const runs = getElementRuns(el);
    if (!runs.length) return {};
    const sel = getSelectedRange();
    if (!sel || sel.collapsed || !el.contains(sel.anchorNode)) {
      return {
        bold: runs.some((r) => r.bold),
        italic: runs.some((r) => r.italic),
        underline: runs.some((r) => r.underline),
        strike: runs.some((r) => r.strike),
      };
    }
    return {
      bold: runs.some((r) => r.bold),
      italic: runs.some((r) => r.italic),
      underline: runs.some((r) => r.underline),
      strike: runs.some((r) => r.strike),
    };
  }

  document.addEventListener("selectionchange", () => {
    const ce = getActiveContentEditable();
    if (!ce) return;
    const fmt = currentFormatting(ce);
    document.dispatchEvent(new CustomEvent("textRunsFormatChange", { detail: fmt }));
  });

  window.TextRuns = {
    applyFormatting,
    applyFormattingToSelection,
    renderRunsAsHTML,
    renderRunsAsText,
    getElementRuns,
    setElementRuns,
    getSelectedRange,
    splitRunAt,
    applyFormattingToRange,
    getActiveContentEditable,
  };
})();
