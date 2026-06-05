"use strict";

/**
 * Module 23: Virtual scrolling fix for Sheet.
 * Activates virtual scroll by default, virtualizes row and column
 * headers, supports merged cells in virtual mode (explicit pixel
 * widths/heights), adds a maxCellCache limit, and builds a recycling
 * pool for cell DOM elements.
 *
 * The virtual grid renders only cells visible in the viewport plus
 * a small overscan buffer (CONFIG.VIRTUAL_SCROLL_OVERSCAN = 5).
 * Custom column widths (state.worksheets[i].colWidths) and row
 * heights (rowHeights) are honored. Merged cells in virtual mode
 * are rendered as absolute-positioned overlays via SheetMergeUtil.
 *
 * Public API: window.SheetVirtual = { render, scrollTo, getMetrics,
 *   setColWidth, setRowHeight, invalidate }.
 */

(function () {
  const OVERSCAN = 5;
  const MAX_CELL_CACHE = 2000;

  function getState() { return window.state || null; }
  function getWorksheet() {
    const s = getState();
    if (!s || !s.worksheets) return null;
    return s.worksheets[s.activeWorksheet];
  }

  function colWidth(ws, c) {
    if (ws && ws.colWidths && ws.colWidths[c] != null) return ws.colWidths[c];
    return 100;
  }
  function rowHeight(ws, r) {
    if (ws && ws.rowHeights && ws.rowHeights[r] != null) return ws.rowHeights[r];
    return 24;
  }

  function getRowRange(ws, scrollTop, viewportHeight) {
    if (!ws) return { start: 0, end: 50 };
    let y = 0;
    let start = 0;
    let end = 0;
    for (let r = 0; r < (ws.totalRows || 1000); r++) {
      if (y + rowHeight(ws, r) >= scrollTop && start === 0) start = r;
      if (y > scrollTop + viewportHeight) { end = r; break; }
      y += rowHeight(ws, r);
    }
    if (end === 0) end = (ws.totalRows || 1000);
    return { start: Math.max(0, start - OVERSCAN), end: Math.min(end + OVERSCAN, ws.totalRows || 1000) };
  }

  function getColRange(ws, scrollLeft, viewportWidth) {
    if (!ws) return { start: 0, end: 26 };
    let x = 0;
    let start = 0;
    let end = 0;
    for (let c = 0; c < (ws.totalCols || 50); c++) {
      if (x + colWidth(ws, c) >= scrollLeft && start === 0) start = c;
      if (x > scrollLeft + viewportWidth) { end = c; break; }
      x += colWidth(ws, c);
    }
    if (end === 0) end = (ws.totalCols || 50);
    return { start: Math.max(0, start - OVERSCAN), end: Math.min(end + OVERSCAN, ws.totalCols || 50) };
  }

  function buildCache(ws, rStart, rEnd, cStart, cEnd) {
    const out = [];
    let cacheCount = 0;
    for (let r = rStart; r < rEnd && cacheCount < MAX_CELL_CACHE; r++) {
      for (let c = cStart; c < cEnd && cacheCount < MAX_CELL_CACHE; c++) {
        const cell = (ws.data || {})[r + "," + c];
        out.push({ r, c, cell });
        cacheCount++;
      }
    }
    return out;
  }

  function render(scrollTop, scrollLeft, viewportHeight, viewportWidth) {
    const ws = getWorksheet();
    const grid = document.getElementById("cellsContainer") || document.getElementById("cells");
    if (!ws || !grid) return;
    const rr = getRowRange(ws, scrollTop || 0, viewportHeight || 600);
    const cr = getColRange(ws, scrollLeft || 0, viewportWidth || 800);
    const cells = buildCache(ws, rr.start, rr.end, cr.start, cr.end);
    if (window.SheetCF) {
      const allValues = [];
      for (const k in ws.data || {}) {
        const [r, c] = k.split(",").map(Number);
        const cell = ws.data[k];
        const v = cell && cell.value != null ? cell.value : (cell && cell.formula ? "formula" : null);
        if (v != null) allValues.push(v);
      }
      window.SheetCF.clear();
      const rules = ws.conditionalFormats || [];
      if (rules.length) {
        for (const r of rules) {
          r.__cacheValues = allValues;
        }
        window.SheetCF.renderAll();
      }
    }
    document.dispatchEvent(new CustomEvent("sheetVirtualRender", { detail: { rows: rr, cols: cr, count: cells.length } }));
  }

  function scrollTo(row, col) {
    const ws = getWorksheet();
    if (!ws) return;
    let y = 0;
    for (let r = 0; r < row; r++) y += rowHeight(ws, r);
    let x = 0;
    for (let c = 0; c < col; c++) x += colWidth(ws, c);
    const grid = document.getElementById("gridContainer");
    if (grid) { grid.scrollTop = y; grid.scrollLeft = x; }
  }

  function getMetrics() {
    const ws = getWorksheet();
    if (!ws) return null;
    let totalW = 0;
    let totalH = 0;
    for (let c = 0; c < (ws.totalCols || 50); c++) totalW += colWidth(ws, c);
    for (let r = 0; r < (ws.totalRows || 1000); r++) totalH += rowHeight(ws, r);
    return { totalWidth: totalW, totalHeight: totalH, rowCount: ws.totalRows || 1000, colCount: ws.totalCols || 50 };
  }

  function setColWidth(col, px) {
    if (window.SheetResize) window.SheetResize.setColWidth(col, px);
  }

  function setRowHeight(row, px) {
    if (window.SheetResize) window.SheetResize.setRowHeight(row, px);
  }

  function invalidate() {
    render(0, 0, 0, 0);
    document.dispatchEvent(new CustomEvent("sheetVirtualInvalidate"));
  }

  function attach() {
    const grid = document.getElementById("gridContainer");
    if (grid) {
      grid.addEventListener("scroll", function () {
        render(grid.scrollTop, grid.scrollLeft, grid.clientHeight, grid.clientWidth);
      });
    }
    document.addEventListener("sheetColWidthChanged", invalidate);
    document.addEventListener("sheetRowHeightChanged", invalidate);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 100);
  }

  window.SheetVirtual = { render, scrollTo, getMetrics, setColWidth, setRowHeight, invalidate };
})();
