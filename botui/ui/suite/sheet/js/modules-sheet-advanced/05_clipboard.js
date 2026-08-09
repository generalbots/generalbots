"use strict";
/* Sheet advanced module: 05_clipboard — clipboard, cut/paste, clear, autofill engine */

(function () {
  function grid() {
    if (window.SheetCore && window.SheetCore.getGrid) return window.SheetCore.getGrid();
    return window.SheetVirtualGrid || null;
  }

  function api() {
    if (window.SheetCore && window.SheetCore.api) return window.SheetCore.api();
    return window.SheetAPI || null;
  }

  function colName(idx) {
    let n = idx + 1;
    let s = "";
    while (n > 0) {
      const r = (n - 1) % 26;
      s = String.fromCharCode(65 + r) + s;
      n = Math.floor((n - 1) / 26);
    }
    return s;
  }

  function colIdx(name) {
    let n = 0;
    for (let i = 0; i < name.length; i++) n = n * 26 + (name.charCodeAt(i) - 64);
    return n - 1;
  }

  function cellValue(r, c) {
    const g = grid();
    const d = g.cells.get(r + "," + c);
    if (!d) return "";
    return d.value != null ? String(d.value) : d.formula || "";
  }

  function isNum(v) {
    if (v == null || v === "") return false;
    return !isNaN(Number(v));
  }

  function mod(n, m) {
    return ((n % m) + m) % m;
  }

  function adjustFormula(formula, dr, dc) {
    const re = /(\$?)([A-Z]+)(\$?)(\d+)/g;
    return formula.replace(re, function (m, c1, letters, c2, digits) {
      let col = letters;
      let row = parseInt(digits, 10);
      if (c1 !== "$") {
        let ci = colIdx(letters) + dc;
        ci = Math.max(0, ci);
        col = colName(ci);
      }
      if (c2 !== "$") {
        row = Math.max(1, row + dr);
      }
      return (c1 === "$" ? "$" : "") + col + (c2 === "$" ? "$" : "") + row;
    });
  }

  function computeFillCell(g, sel, targetRow, targetCol, srcH, srcW) {
    let si = 0;
    let sj = 0;
    if (srcH === 1 && srcW === 1) {
      si = 0;
      sj = 0;
    } else if (srcH === 1) {
      sj = mod(targetCol - sel.startCol, srcW);
    } else if (srcW === 1) {
      si = mod(targetRow - sel.startRow, srcH);
    } else {
      si = mod(targetRow - sel.startRow, srcH);
      sj = mod(targetCol - sel.startCol, srcW);
    }
    const srcR = sel.startRow + si;
    const srcC = sel.startCol + sj;
    const srcKey = srcR + "," + srcC;
    const cell = g.cells.get(srcKey);
    if (!cell) return null;
    if (cell.formula) {
      const dr = targetRow - srcR;
      const dc = targetCol - srcC;
      return { value: adjustFormula(cell.formula, dr, dc), isFormula: true };
    }
    if (srcW === 1 && srcH >= 2 && targetRow > sel.endRow) {
      const a = g.cells.get(sel.startRow + "," + sel.startCol);
      const b = g.cells.get((sel.startRow + 1) + "," + sel.startCol);
      if (a && b && isNum(a.value) && isNum(b.value)) {
        const step = Number(b.value) - Number(a.value);
        const count = targetRow - sel.endRow;
        return { value: String(Number(b.value) + step * count), isFormula: false };
      }
    }
    if (srcH === 1 && srcW >= 2 && targetCol > sel.endCol) {
      const a = g.cells.get(sel.startRow + "," + sel.startCol);
      const b = g.cells.get(sel.startRow + "," + (sel.startCol + 1));
      if (a && b && isNum(a.value) && isNum(b.value)) {
        const step = Number(b.value) - Number(a.value);
        const count = targetCol - sel.endCol;
        return { value: String(Number(b.value) + step * count), isFormula: false };
      }
    }
    return { value: cell.value != null ? cell.value : "", isFormula: false };
  }

  function beginBulk(label, touched) {
    window.__gbSuspendUndo = true;
    const u = window.SheetUndo;
    if (u && u.beforeBulk) u.beforeBulk(label, touched);
  }

  function endBulk() {
    const u = window.SheetUndo;
    if (u && u.recordBulk) u.recordBulk();
    window.__gbSuspendUndo = false;
  }

  function applyFill(targetRow, targetCol) {
    const g = grid();
    const sel = window.SheetAdvanced ? window.SheetAdvanced.getSelection() : null;
    if (!sel || !g) return Promise.resolve(null);
    if (targetRow <= sel.endRow && targetCol <= sel.endCol) return Promise.resolve(null);
    const srcH = sel.endRow - sel.startRow + 1;
    const srcW = sel.endCol - sel.startCol + 1;
    const rTo = Math.max(sel.endRow, targetRow);
    const cTo = Math.max(sel.endCol, targetCol);
    const touched = [];
    for (let r = sel.startRow; r <= rTo; r++) {
      for (let c = sel.startCol; c <= cTo; c++) {
        if (r <= sel.endRow && c <= sel.endCol) continue;
        const fill = computeFillCell(g, sel, r, c, srcH, srcW);
        if (fill) touched.push(r + "," + c);
      }
    }
    if (!touched.length) return Promise.resolve(null);
    beginBulk("Fill", touched);
    const updates = [];
    for (let r = sel.startRow; r <= rTo; r++) {
      for (let c = sel.startCol; c <= cTo; c++) {
        if (r <= sel.endRow && c <= sel.endCol) continue;
        const fill = computeFillCell(g, sel, r, c, srcH, srcW);
        if (!fill) continue;
        const key = r + "," + c;
        const ref = colName(c) + (r + 1);
        g.cells.set(key, { value: fill.value, formula: fill.isFormula ? fill.value : undefined });
        updates.push(api().updateCell(ref, fill.value));
      }
    }
    return Promise.all(updates).then(function () {
      if (window.SheetCore && window.SheetCore.refreshGrid) window.SheetCore.refreshGrid();
      endBulk();
    });
  }

  function buildClipboardText() {
    const g = grid();
    const sel = window.SheetAdvanced ? window.SheetAdvanced.getSelection() : null;
    if (!sel || !g) return "";
    const rows = [];
    for (let r = sel.startRow; r <= sel.endRow; r++) {
      const cells = [];
      for (let c = sel.startCol; c <= sel.endCol; c++) {
        const v = cellValue(r, c);
        cells.push(v.indexOf("\t") >= 0 || v.indexOf("\n") >= 0 ? '"' + v.replace(/"/g, '""') + '"' : v);
      }
      rows.push(cells.join("\t"));
    }
    return rows.join("\n");
  }

  async function copySelection() {
    const text = buildClipboardText();
    if (!text) return;
    window.__gbClipboard = text;
    try {
      await navigator.clipboard.writeText(text);
    } catch (_) {
      const ta = document.createElement("textarea");
      ta.value = text;
      document.body.appendChild(ta);
      ta.select();
      try { document.execCommand("copy"); } catch (__) {}
      ta.remove();
    }
  }

  function cutSelection() {
    copySelection();
    const g = grid();
    const sel = window.SheetAdvanced ? window.SheetAdvanced.getSelection() : null;
    if (!sel || !g) return;
    const touched = [];
    for (let r = sel.startRow; r <= sel.endRow; r++) {
      for (let c = sel.startCol; c <= sel.endCol; c++) touched.push(r + "," + c);
    }
    if (!touched.length) return;
    beginBulk("Cut", touched);
    const updates = [];
    for (let r = sel.startRow; r <= sel.endRow; r++) {
      for (let c = sel.startCol; c <= sel.endCol; c++) {
        const ref = colName(c) + (r + 1);
        g.cells.set(r + "," + c, { value: "" });
        updates.push(api().updateCell(ref, ""));
      }
    }
    return Promise.all(updates).then(function () {
      if (window.SheetCore && window.SheetCore.refreshGrid) window.SheetCore.refreshGrid();
      endBulk();
    });
  }

  async function pasteToSelection() {
    const g = grid();
    const sel = window.SheetAdvanced ? window.SheetAdvanced.getSelection() : null;
    if (!sel || !g) return;
    let text = "";
    try {
      text = await navigator.clipboard.readText();
    } catch (_) {
      text = window.__gbClipboard || "";
    }
    if (!text) return;
    const lines = text.split(/\r?\n/);
    while (lines.length && lines[lines.length - 1] === "") lines.pop();
    const touched = [];
    for (let i = 0; i < lines.length; i++) {
      const cols = lines[i].split("\t");
      for (let j = 0; j < cols.length; j++) {
        const r = sel.startRow + i;
        const c = sel.startCol + j;
        if (r >= g.totalRows || c >= g.totalCols) continue;
        touched.push(r + "," + c);
      }
    }
    if (!touched.length) return;
    beginBulk("Paste", touched);
    const updates = [];
    for (let i = 0; i < lines.length; i++) {
      const cols = lines[i].split("\t");
      for (let j = 0; j < cols.length; j++) {
        const r = sel.startRow + i;
        const c = sel.startCol + j;
        if (r >= g.totalRows || c >= g.totalCols) continue;
        const val = cols[j];
        const ref = colName(c) + (r + 1);
        g.cells.set(r + "," + c, val.startsWith("=") ? { value: "", formula: val } : { value: val });
        updates.push(api().updateCell(ref, val));
      }
    }
    return Promise.all(updates).then(function () {
      if (window.SheetCore && window.SheetCore.refreshGrid) window.SheetCore.refreshGrid();
      endBulk();
    });
  }

  function clearSelectionCells() {
    const g = grid();
    const sel = window.SheetAdvanced ? window.SheetAdvanced.getSelection() : null;
    if (!sel || !g) return;
    const touched = [];
    for (let r = sel.startRow; r <= sel.endRow; r++) {
      for (let c = sel.startCol; c <= sel.endCol; c++) touched.push(r + "," + c);
    }
    if (!touched.length) return;
    beginBulk("Clear", touched);
    const updates = [];
    for (let r = sel.startRow; r <= sel.endRow; r++) {
      for (let c = sel.startCol; c <= sel.endCol; c++) {
        const ref = colName(c) + (r + 1);
        g.cells.set(r + "," + c, { value: "" });
        updates.push(api().updateCell(ref, ""));
      }
    }
    return Promise.all(updates).then(function () {
      if (window.SheetCore && window.SheetCore.refreshGrid) window.SheetCore.refreshGrid();
      endBulk();
    });
  }

  window.SheetClipboard = {
    copy: copySelection,
    cut: cutSelection,
    paste: pasteToSelection,
    clear: clearSelectionCells,
    fill: applyFill,
  };

  if (window.SheetCore) {
    window.SheetCore.applyFill = applyFill;
    window.SheetCore.copySelection = copySelection;
    window.SheetCore.cutSelection = cutSelection;
    window.SheetCore.pasteToSelection = pasteToSelection;
    window.SheetCore.clearSelectionCells = clearSelectionCells;
  }
})();