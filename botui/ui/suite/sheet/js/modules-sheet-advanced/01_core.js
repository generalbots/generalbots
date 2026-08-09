"use strict";
/* Sheet advanced module: 01_core — selection core, shared state facade, wiring */

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
    window.dispatchEvent(new CustomEvent("gb-sheet-selection", { detail: sel }));
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

  function cw(idx) {
    if (window.SheetCore && window.SheetCore.colWidth) return window.SheetCore.colWidth(idx);
    return COL_WIDTH;
  }

  function cx(idx) {
    if (window.SheetCore && window.SheetCore.colX) return window.SheetCore.colX(idx);
    return HEADER_WIDTH + idx * COL_WIDTH;
  }

  function cellFromEvent(e) {
    const rect = grid.bodyInner.getBoundingClientRect();
    const x = e.clientX - rect.left - HEADER_WIDTH;
    const y = e.clientY - rect.top;
    let col = 0;
    let acc = 0;
    for (let c = 0; c < grid.totalCols; c++) {
      if (x < acc + cw(c)) { col = c; break; }
      acc += cw(c);
      col = c;
    }
    const row = Math.floor(y / ROW_HEIGHT);
    if (row < 0 || col < 0) return null;
    return { row: Math.min(row, grid.totalRows - 1), col: Math.min(col, grid.totalCols - 1) };
  }

  function positionRangeBox() {
    if (!rangeBox || !sel) return;
    rangeBox.style.display = "block";
    rangeBox.style.left = (cx(sel.startCol) - 1) + "px";
    rangeBox.style.top = (sel.startRow * ROW_HEIGHT - 1) + "px";
    rangeBox.style.width = (cx(sel.endCol) + cw(sel.endCol) - cx(sel.startCol) + 2) + "px";
    rangeBox.style.height = ((sel.endRow - sel.startRow + 1) * ROW_HEIGHT + 2) + "px";
  }

  function positionFillHandle() {
    if (!fillHandle || !sel) return;
    fillHandle.style.display = "block";
    fillHandle.style.left = (cx(sel.endCol) + cw(sel.endCol) - 5) + "px";
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
      preview.style.left = (cx(sel.endCol) - 1) + "px";
      preview.style.top = (sel.endRow * ROW_HEIGHT - 1) + "px";
      preview.style.width = (cx(c) + cw(c) - cx(sel.endCol) + 2) + "px";
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
    if (window.SheetCore && window.SheetCore.applyFill) window.SheetCore.applyFill(pt.row, pt.col);
  }

  function ensureFillPreview() {
    if (document.getElementById("ss-fill-preview")) return;
    const p = document.createElement("div");
    p.id = "ss-fill-preview";
    p.style.cssText = "position:absolute;border:2px dashed #3b82f6;pointer-events:none;z-index:13;display:none;";
    if (grid && grid.bodyInner) grid.bodyInner.appendChild(p);
  }

  function bindGridEvents() {
    if (!grid || !grid.bodyInner) return;
    if (grid.bodyInner.__saBound) return;
    grid.bodyInner.__saBound = true;
    grid.bodyInner.addEventListener("mousedown", onCellMDown, true);
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
    if (hostEl) {
      if (!tabBar || !tabBar.isConnected) {
        tabBar = document.createElement("div");
        tabBar.className = "ss-tab-bar";
        tabBar.style.cssText = "display:flex;gap:4px;align-items:center;padding:6px 12px;background:#0f172a;border-top:1px solid #334155;flex-shrink:0;overflow-x:auto;";
        hostEl.appendChild(tabBar);
      }
    }
    if (window.SheetCore) {
      window.SheetCore.setTabBar(tabBar);
      if (window.SheetCore.renderTabBar) window.SheetCore.renderTabBar();
    }
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
      sync: sync,
    };
  }

  if (window.SheetCore) {
    window.SheetCore.setSelection = setSelection;
    window.SheetCore.getGrid = function () { return grid; };
    window.SheetCore.setGrid = function (g) { grid = g; };
    window.SheetCore.getHost = function () { return hostEl; };
    window.SheetCore.setHost = function (h) { hostEl = h; };
    window.SheetCore.getTabBar = function () { return tabBar; };
    window.SheetCore.setTabBar = function (t) { tabBar = t; };
    window.SheetCore.wsIndex = wsIndex;
    window.SheetCore.api = api;
    window.SheetCore.currentSheetId = currentSheetId;
    window.SheetCore.colName = colName;
    window.SheetCore.cellValue = cellValue;
    window.SheetCore.setRange = setSelection;
    window.SheetCore.refreshGrid = function () {
      if (grid) {
        grid.lastRenderedRange = null;
        grid.requestRange();
      }
    };
  }

  window.SheetAdvanced = {
    init: init,
    sync: sync,
    getSelection: function () { return sel; },
    setRange: function (r1, c1, r2, c2) {
      setSelection(r1, c1, r2, c2);
      if (grid) grid.selectCell(Math.max(r1, r2), Math.max(c1, c2));
    },
    clearSelection: function () {
      clearSelection();
    },
  };
})();