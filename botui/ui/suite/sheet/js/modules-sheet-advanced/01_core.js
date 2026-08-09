"use strict";
/* Sheet advanced module: 01_core — range selection, clipboard, autofill, worksheet tabs */

(function () {
  const COL_WIDTH = 96;
  const ROW_HEIGHT = 24;
  const HEADER_WIDTH = 48;

  let hostEl = null;
  let grid = null;
  let sel = null;
  let anchor = null;
  let dragging = false;
  let fillDragging = false;
  let rangeBox = null;
  let fillHandle = null;
  let tabBar = null;

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

  function currentSheetId() {
    return window.__SHEET_INITIAL_ID || "current";
  }

  function api() {
    return window.SheetAPI || null;
  }

  function wsIndex() {
    try {
      const v = parseInt(sessionStorage.getItem("sheet_active_index") || "0", 10);
      return isNaN(v) || v < 0 ? 0 : v;
    } catch (_) {
      return 0;
    }
  }

  function normalize(r1, c1, r2, c2) {
    return {
      startRow: Math.min(r1, r2),
      startCol: Math.min(c1, c2),
      endRow: Math.max(r1, r2),
      endCol: Math.max(c1, c2),
    };
  }

  function setSelection(r1, c1, r2, c2) {
    sel = normalize(r1, c1, r2, c2);
    positionRangeBox();
    positionFillHandle();
  }

  function clearSelection() {
    sel = null;
    if (rangeBox) rangeBox.style.display = "none";
    if (fillHandle) fillHandle.style.display = "none";
  }

  function cellValue(r, c) {
    const d = grid.cells.get(r + "," + c);
    if (!d) return "";
    return d.value != null ? String(d.value) : d.formula || "";
  }

  function cellFromEvent(e) {
    const rect = grid.bodyInner.getBoundingClientRect();
    const x = e.clientX - rect.left - HEADER_WIDTH;
    const y = e.clientY - rect.top;
    const col = Math.floor(x / COL_WIDTH);
    const row = Math.floor(y / ROW_HEIGHT);
    if (row < 0 || col < 0) return null;
    return { row: Math.min(row, grid.totalRows - 1), col: Math.min(col, grid.totalCols - 1) };
  }

  function positionRangeBox() {
    if (!rangeBox || !sel) return;
    rangeBox.style.display = "block";
    rangeBox.style.left = (HEADER_WIDTH + sel.startCol * COL_WIDTH - 1) + "px";
    rangeBox.style.top = (sel.startRow * ROW_HEIGHT - 1) + "px";
    rangeBox.style.width = ((sel.endCol - sel.startCol + 1) * COL_WIDTH + 2) + "px";
    rangeBox.style.height = ((sel.endRow - sel.startRow + 1) * ROW_HEIGHT + 2) + "px";
  }

  function positionFillHandle() {
    if (!fillHandle || !sel) return;
    fillHandle.style.display = "block";
    fillHandle.style.left = (HEADER_WIDTH + sel.endCol * COL_WIDTH + COL_WIDTH - 5) + "px";
    fillHandle.style.top = (sel.endRow * ROW_HEIGHT + ROW_HEIGHT - 5) + "px";
  }

  function ensureOverlays() {
    if (!grid || !grid.bodyInner) return;
    if (!rangeBox || !rangeBox.isConnected) {
      rangeBox = document.createElement("div");
      rangeBox.className = "ss-range-box";
      rangeBox.style.cssText = "position:absolute;border:2px solid #3b82f6;background:rgba(59,130,246,0.10);pointer-events:none;z-index:11;display:none;";
      grid.bodyInner.appendChild(rangeBox);
    }
    if (!fillHandle || !fillHandle.isConnected) {
      fillHandle = document.createElement("div");
      fillHandle.className = "ss-fill-handle";
      fillHandle.style.cssText = "position:absolute;width:9px;height:9px;background:#3b82f6;border:1px solid #fff;cursor:crosshair;z-index:14;display:none;";
      grid.bodyInner.appendChild(fillHandle);
      fillHandle.addEventListener("mousedown", onFillStart);
    }
  }

  function onCellMDown(e) {
    const t = e.target;
    if (!t || !t.classList || !t.classList.contains("vg-cell")) return;
    const r = parseInt(t.dataset.row, 10);
    const c = parseInt(t.dataset.col, 10);
    if (isNaN(r) || isNaN(c)) return;
    e.preventDefault();
    if (e.shiftKey && sel) {
      setSelection(sel.startRow, sel.startCol, r, c);
    } else {
      anchor = { row: r, col: c };
      setSelection(r, c, r, c);
    }
    dragging = true;
    document.addEventListener("mousemove", onDrag, true);
    document.addEventListener("mouseup", onDragEnd, true);
  }

  function onDrag(e) {
    if (!dragging || !anchor) return;
    const pt = cellFromEvent(e);
    if (pt) setSelection(anchor.row, anchor.col, pt.row, pt.col);
  }

  function onDragEnd() {
    dragging = false;
    document.removeEventListener("mousemove", onDrag, true);
    document.removeEventListener("mouseup", onDragEnd, true);
    if (grid && sel) grid.selectCell(sel.endRow, sel.endCol);
  }

  function onFillStart(e) {
    e.preventDefault();
    e.stopPropagation();
    fillDragging = true;
    document.addEventListener("mousemove", onFillDrag, true);
    document.addEventListener("mouseup", onFillEnd, true);
  }

  function onFillDrag(e) {
    if (!fillDragging || !sel) return;
    const pt = cellFromEvent(e);
    if (!pt) return;
    const r = Math.max(sel.endRow, pt.row);
    const c = Math.max(sel.endCol, pt.col);
    highlightFillPreview(r, c);
  }

  function highlightFillPreview(r, c) {
    const preview = document.getElementById("ss-fill-preview");
    if (preview) {
      preview.style.display = "block";
      preview.style.left = (HEADER_WIDTH + sel.endCol * COL_WIDTH - 1) + "px";
      preview.style.top = (sel.endRow * ROW_HEIGHT - 1) + "px";
      preview.style.width = ((c - sel.endCol + 1) * COL_WIDTH + 2) + "px";
      preview.style.height = ((r - sel.endRow + 1) * ROW_HEIGHT + 2) + "px";
    }
  }

  function onFillEnd(e) {
    fillDragging = false;
    document.removeEventListener("mousemove", onFillDrag, true);
    document.removeEventListener("mouseup", onFillEnd, true);
    const preview = document.getElementById("ss-fill-preview");
    if (preview) preview.style.display = "none";
    const pt = cellFromEvent(e);
    if (!pt || !sel) return;
    applyFill(pt.row, pt.col);
  }

  function adjustFormula(formula, dr, dc) {
    const re = /(\$?)([A-Z]+)(\$?)(\d+)/g;
    return formula.replace(re, function (m, c1, letters, c2, digits) {
      let col = colName(letters.length ? colIdx(letters) : 0);
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

  function colIdx(name) {
    let n = 0;
    for (let i = 0; i < name.length; i++) n = n * 26 + (name.charCodeAt(i) - 64);
    return n - 1;
  }

  function isNum(v) {
    if (v == null || v === "") return false;
    return !isNaN(Number(v));
  }

  function applyFill(targetRow, targetCol) {
    if (!sel || !grid) return;
    const srcH = sel.endRow - sel.startRow + 1;
    const srcW = sel.endCol - sel.startCol + 1;
    const updates = [];
    for (let r = sel.endRow + 1; r <= targetRow; r++) {
      for (let c = sel.endCol + 1; c <= targetCol; c++) {
        const fill = computeFillCell(r, c, srcH, srcW);
        if (!fill) continue;
        const key = r + "," + c;
        const ref = colName(c) + (r + 1);
        grid.cells.set(key, { value: fill.value, formula: fill.isFormula ? fill.value : undefined });
        updates.push(api().updateCell(ref, fill.value));
      }
    }
    Promise.all(updates).then(function () {
      grid.lastRenderedRange = null;
      grid.requestRange();
    });
  }

  function computeFillCell(targetRow, targetCol, srcH, srcW) {
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
    const cell = grid.cells.get(srcKey);
    if (!cell) return null;
    if (cell.formula) {
      const dr = targetRow - srcR;
      const dc = targetCol - srcC;
      return { value: adjustFormula(cell.formula, dr, dc), isFormula: true };
    }
    if (srcW === 1 && srcH >= 2 && targetRow > sel.endRow) {
      const a = grid.cells.get(sel.startRow + "," + sel.startCol);
      const b = grid.cells.get((sel.startRow + 1) + "," + sel.startCol);
      if (a && b && isNum(a.value) && isNum(b.value)) {
        const step = Number(b.value) - Number(a.value);
        const count = targetRow - sel.endRow;
        return { value: String(Number(b.value) + step * count), isFormula: false };
      }
    }
    if (srcH === 1 && srcW >= 2 && targetCol > sel.endCol) {
      const a = grid.cells.get(sel.startRow + "," + sel.startCol);
      const b = grid.cells.get(sel.startRow + "," + (sel.startCol + 1));
      if (a && b && isNum(a.value) && isNum(b.value)) {
        const step = Number(b.value) - Number(a.value);
        const count = targetCol - sel.endCol;
        return { value: String(Number(b.value) + step * count), isFormula: false };
      }
    }
    return { value: cell.value != null ? cell.value : "", isFormula: false };
  }

  function mod(n, m) {
    return ((n % m) + m) % m;
  }

  function buildClipboardText() {
    if (!sel) return "";
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
    if (!sel) return;
    const updates = [];
    for (let r = sel.startRow; r <= sel.endRow; r++) {
      for (let c = sel.startCol; c <= sel.endCol; c++) {
        const ref = colName(c) + (r + 1);
        grid.cells.set(r + "," + c, { value: "" });
        updates.push(api().updateCell(ref, ""));
      }
    }
    Promise.all(updates).then(function () {
      grid.lastRenderedRange = null;
      grid.requestRange();
    });
  }

  async function pasteToSelection() {
    if (!sel) return;
    let text = "";
    try {
      text = await navigator.clipboard.readText();
    } catch (_) {
      text = window.__gbClipboard || "";
    }
    if (!text) return;
    const lines = text.split(/\r?\n/);
    while (lines.length && lines[lines.length - 1] === "") lines.pop();
    const updates = [];
    for (let i = 0; i < lines.length; i++) {
      const cols = lines[i].split("\t");
      for (let j = 0; j < cols.length; j++) {
        const r = sel.startRow + i;
        const c = sel.startCol + j;
        if (r >= grid.totalRows || c >= grid.totalCols) continue;
        const val = cols[j];
        const ref = colName(c) + (r + 1);
        grid.cells.set(r + "," + c, val.startsWith("=") ? { value: "", formula: val } : { value: val });
        updates.push(api().updateCell(ref, val));
      }
    }
    Promise.all(updates).then(function () {
      grid.lastRenderedRange = null;
      grid.requestRange();
    });
  }

  function clearSelectionCells() {
    if (!sel) return;
    const updates = [];
    for (let r = sel.startRow; r <= sel.endRow; r++) {
      for (let c = sel.startCol; c <= sel.endCol; c++) {
        const ref = colName(c) + (r + 1);
        grid.cells.set(r + "," + c, { value: "" });
        updates.push(api().updateCell(ref, ""));
      }
    }
    Promise.all(updates).then(function () {
      grid.lastRenderedRange = null;
      grid.requestRange();
    });
  }

  function onKeyDown(e) {
    if (!grid) return;
    const editing = grid.editingCell != null;
    if (editing) return;
    const t = e.target;
    if (t && (t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.tagName === "SELECT" || t.isContentEditable)) return;
    const mod = e.ctrlKey || e.metaKey;
    if (mod && e.key.toLowerCase() === "c") {
      e.preventDefault();
      copySelection();
    } else if (mod && e.key.toLowerCase() === "x") {
      e.preventDefault();
      cutSelection();
    } else if (mod && e.key.toLowerCase() === "v") {
      e.preventDefault();
      pasteToSelection();
    } else if (e.key === "Delete" || e.key === "Backspace") {
      e.preventDefault();
      clearSelectionCells();
    } else if (e.key === "Escape") {
      clearSelection();
    }
  }

  function ensureFillPreview() {
    if (document.getElementById("ss-fill-preview")) return;
    const p = document.createElement("div");
    p.id = "ss-fill-preview";
    p.style.cssText = "position:absolute;border:2px dashed #3b82f6;pointer-events:none;z-index:13;display:none;";
    if (grid && grid.bodyInner) grid.bodyInner.appendChild(p);
  }

  function renderTabBar() {
    if (!hostEl || !tabBar) return;
    const sheet = window.__LOADED_SHEET;
    const idx = wsIndex();
    tabBar.innerHTML = "";
    const addBtn = document.createElement("button");
    addBtn.type = "button";
    addBtn.className = "ss-tab-add";
    addBtn.textContent = "+";
    addBtn.title = "Nova planilha";
    addBtn.addEventListener("click", addWorksheetClient);
    tabBar.appendChild(addBtn);
    if (!sheet || !sheet.worksheets || !sheet.worksheets.length) return;
    sheet.worksheets.forEach(function (ws, i) {
      const tab = document.createElement("div");
      tab.className = "ss-tab" + (i === idx ? " ss-tab-active" : "");
      tab.dataset.index = i;
      const label = document.createElement("span");
      label.textContent = ws.name;
      label.addEventListener("dblclick", function () { renameWorksheetClient(i); });
      tab.appendChild(label);
      const del = document.createElement("button");
      del.type = "button";
      del.className = "ss-tab-del";
      del.textContent = "×";
      del.title = "Excluir planilha";
      del.addEventListener("click", function (e) { e.stopPropagation(); deleteWorksheetClient(i); });
      tab.appendChild(del);
      tab.addEventListener("click", function () { switchWorksheetClient(i); });
      tabBar.appendChild(tab);
    });
  }

  function reloadSheetAfterMutation() {
    return api().load(currentSheetId()).then(function (sheet) {
      if (sheet) {
        window.__LOADED_SHEET = sheet;
        window.__SHEET_INITIAL_ID = sheet.id;
      }
      renderTabBar();
      rehydrateGrid();
      return sheet;
    });
  }

  function rehydrateGrid() {
    if (!grid) return;
    const sheet = window.__LOADED_SHEET;
    const idx = wsIndex();
    if (!sheet || !sheet.worksheets || !sheet.worksheets[idx]) return;
    const ws = sheet.worksheets[idx];
    grid.cells = new Map();
    if (ws.data) {
      for (const cellRef in ws.data) {
        grid.cells.set(cellRef, ws.data[cellRef]);
      }
    }
    grid.requestSeq++;
    grid.lastRenderedRange = null;
    grid.requestRange();
    clearSelection();
  }

  function switchWorksheetClient(i) {
    window.dispatchEvent(new CustomEvent("gb-sheet-tab", { detail: { index: i } }));
    renderTabBar();
    rehydrateGrid();
  }

  function addWorksheetClient() {
    api().addWorksheet().then(function () {
      reloadSheetAfterMutation().then(function () {
        const sheet = window.__LOADED_SHEET;
        if (sheet) switchWorksheetClient(sheet.worksheets.length - 1);
      });
    });
  }

  function deleteWorksheetClient(i) {
    const sheet = window.__LOADED_SHEET;
    if (!sheet || !sheet.worksheets || sheet.worksheets.length <= 1) return;
    if (!window.confirm("Excluir planilha " + sheet.worksheets[i].name + "?")) return;
    api().deleteWorksheet(i).then(function () {
      reloadSheetAfterMutation().then(function () {
        const idx = wsIndex();
        if (idx >= sheet.worksheets.length) switchWorksheetClient(sheet.worksheets.length - 1);
      });
    });
  }

  function renameWorksheetClient(i) {
    const sheet = window.__LOADED_SHEET;
    if (!sheet || !sheet.worksheets || !sheet.worksheets[i]) return;
    const name = window.prompt("Novo nome da planilha:", sheet.worksheets[i].name);
    if (!name || !name.trim()) return;
    api().renameWorksheet(i, name.trim()).then(function () {
      sheet.worksheets[i].name = name.trim();
      renderTabBar();
    });
  }

  function bindGridEvents() {
    if (!grid || !grid.bodyInner) return;
    if (grid.bodyInner.__saBound) return;
    grid.bodyInner.__saBound = true;
    grid.bodyInner.addEventListener("mousedown", onCellMDown, true);
  }

  function bindDocument() {
    if (window.__saDocBound) return;
    window.__saDocBound = true;
    document.addEventListener("keydown", onKeyDown, true);
  }

  function wire() {
    grid = window.SheetVirtualGrid;
    if (!grid || !grid.bodyInner) {
      setTimeout(wire, 100);
      return;
    }
    hostEl = hostEl || document.querySelector("#sheet-content");
    ensureOverlays();
    ensureFillPreview();
    bindGridEvents();
    bindDocument();
    if (hostEl) {
      if (!tabBar || !tabBar.isConnected) {
        tabBar = document.createElement("div");
        tabBar.className = "ss-tab-bar";
        tabBar.style.cssText = "display:flex;gap:4px;align-items:center;padding:6px 12px;background:#0f172a;border-top:1px solid #334155;flex-shrink:0;overflow-x:auto;";
        hostEl.appendChild(tabBar);
      }
    }
    renderTabBar();
  }

  function sync() {
    wire();
  }

  function init(host, opts) {
    hostEl = host;
    if (opts && opts.sheetId) window.__SHEET_INITIAL_ID = opts.sheetId;
    setTimeout(wire, 0);
    return {
      getSelection: function () { return sel; },
      copy: copySelection,
      cut: cutSelection,
      paste: pasteToSelection,
      fill: applyFill,
      sync: sync,
    };
  }

  window.SheetAdvanced = {
    init: init,
    sync: sync,
    getSelection: function () { return sel; },
  };
})();
