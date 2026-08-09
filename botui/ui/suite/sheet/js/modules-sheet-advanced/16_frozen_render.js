"use strict";
/* Sheet advanced module: 16_frozen_render — sticky frozen top rows and left columns */

(function () {
  const HEADER_W = 48;
  const DEFAULT_COL = 96;
  const DEFAULT_ROW = 24;
  let topLayer = null;
  let leftLayer = null;
  let wired = false;

  function grid() {
    if (window.SheetCore && window.SheetCore.getGrid) return window.SheetCore.getGrid();
    return window.SheetVirtualGrid || null;
  }

  function cw(idx) {
    if (window.SheetCore && window.SheetCore.colWidth) return window.SheetCore.colWidth(idx);
    return DEFAULT_COL;
  }

  function cx(idx) {
    if (window.SheetCore && window.SheetCore.colX) return window.SheetCore.colX(idx);
    return HEADER_W + idx * DEFAULT_COL;
  }

  function rh(idx) {
    if (window.SheetCore && window.SheetCore.rowHeight) return window.SheetCore.rowHeight(idx);
    return DEFAULT_ROW;
  }

  function frozen() {
    const sheet = window.__LOADED_SHEET;
    const g = grid();
    const idx = window.SheetCore && window.SheetCore.wsIndex ? window.SheetCore.wsIndex() : 0;
    if (!sheet || !sheet.worksheets || !sheet.worksheets[idx]) return { rows: 0, cols: 0 };
    const ws = sheet.worksheets[idx];
    let rows = ws.frozen_rows || 0;
    let cols = ws.frozen_cols || 0;
    if (g) {
      rows = Math.min(rows, g.totalRows);
      cols = Math.min(cols, g.totalCols);
    }
    return { rows: rows, cols: cols };
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

  function ensureLayers() {
    const g = grid();
    if (!g || !g.midRow) return;
    g.midRow.style.position = "relative";
    if (!g.midRow.__frozenZ) {
      g.midRow.__frozenZ = true;
      g.scrollArea.style.zIndex = "1";
    }
    if (!topLayer || !topLayer.isConnected) {
      topLayer = document.createElement("div");
      topLayer.className = "vg-frozen-top";
      topLayer.style.cssText = "position:absolute;left:0;right:0;top:0;height:0;overflow:hidden;background:#0f172a;z-index:9;pointer-events:none;";
      g.midRow.appendChild(topLayer);
    }
    if (!leftLayer || !leftLayer.isConnected) {
      leftLayer = document.createElement("div");
      leftLayer.className = "vg-frozen-left";
      leftLayer.style.cssText = "position:absolute;left:0;top:0;width:0;overflow:hidden;background:#0f172a;z-index:9;pointer-events:none;";
      g.midRow.appendChild(leftLayer);
    }
  }

  function renderFrozenRows() {
    const g = grid();
    const f = frozen();
    ensureLayers();
    if (!g || !topLayer) return;
    topLayer.innerHTML = "";
    if (!f.rows) {
      topLayer.style.height = "0px";
      return;
    }
    let h = 0;
    for (let r = 0; r < f.rows; r++) h += rh(r);
    topLayer.style.height = h + "px";
    for (let r = 0; r < f.rows; r++) {
      for (let c = 0; c < g.totalCols; c++) {
        const cell = document.createElement("div");
        const data = g.cells.get(r + "," + c);
        cell.style.cssText =
          "position:absolute;left:" + cx(c) + "px;top:" + rowTopAt(g, r) + "px;width:" + cw(c) + "px;height:" + rh(r) + "px;" +
          "background:#0f172a;color:#f8fafc;border-right:1px solid #334155;border-bottom:1px solid #334155;" +
          "padding:2px 4px;font-size:12px;overflow:hidden;box-sizing:border-box;";
        cell.textContent = data ? (data.value != null ? data.value : data.formula || "") : "";
        if (data && data.style) {
          const s = data.style;
          if (s.font_weight) cell.style.fontWeight = s.font_weight;
          if (s.font_style) cell.style.fontStyle = s.font_style;
          if (s.color) cell.style.color = s.color;
          if (s.background) cell.style.backgroundColor = s.background;
        }
        topLayer.appendChild(cell);
      }
    }
  }

  function rowTopAt(g, r) {
    let y = 0;
    for (let i = 0; i < r; i++) y += rh(i);
    return y;
  }

  function renderFrozenCols() {
    const g = grid();
    const f = frozen();
    ensureLayers();
    if (!g || !leftLayer) return;
    leftLayer.innerHTML = "";
    if (!f.cols) {
      leftLayer.style.width = "0px";
      return;
    }
    let w = 0;
    for (let c = 0; c < f.cols; c++) w += cw(c);
    leftLayer.style.width = w + "px";
    const visible = g.visibleRowRange ? g.visibleRowRange() : { start: 0, end: 60 };
    const padding = 6;
    const rFrom = Math.max(0, visible.start - padding);
    const rTo = Math.min(g.totalRows, visible.end + padding);
    for (let r = rFrom; r < rTo; r++) {
      for (let c = 0; c < f.cols; c++) {
        const cell = document.createElement("div");
        const data = g.cells.get(r + "," + c);
        let colX = 0;
        for (let cc = 0; cc < c; cc++) colX += cw(cc);
        cell.style.cssText =
          "position:absolute;left:" + colX + "px;top:" + (r * rh(r)) + "px;width:" + cw(c) + "px;height:" + rh(r) + "px;" +
          "background:#0f172a;color:#f8fafc;border-right:1px solid #334155;border-bottom:1px solid #334155;" +
          "padding:2px 4px;font-size:12px;overflow:hidden;box-sizing:border-box;";
        cell.textContent = data ? (data.value != null ? data.value : data.formula || "") : "";
        if (data && data.style) {
          const s = data.style;
          if (s.font_weight) cell.style.fontWeight = s.font_weight;
          if (s.font_style) cell.style.fontStyle = s.font_style;
          if (s.color) cell.style.color = s.color;
          if (s.background) cell.style.backgroundColor = s.background;
        }
        leftLayer.appendChild(cell);
      }
    }
  }

  function syncScroll() {
    const g = grid();
    if (!g) return;
    if (topLayer) topLayer.style.transform = "translateX(" + (-g.scrollLeft) + "px)";
    if (leftLayer) leftLayer.style.transform = "translateY(" + (-g.scrollTop) + "px)";
    if (leftLayer && g.__saLastTop !== g.scrollTop) {
      g.__saLastTop = g.scrollTop;
      renderFrozenCols();
    }
  }

  function patchScroll() {
    const g = grid();
    if (!g || !g.onScroll) return;
    const orig = g.onScroll.bind(g);
    g.onScroll = function () {
      orig();
      syncScroll();
    };
  }

  function wire() {
    const g = grid();
    if (!g) {
      setTimeout(wire, 100);
      return;
    }
    if (!wired) {
      wired = true;
      patchScroll();
    }
    ensureLayers();
    renderFrozenRows();
    renderFrozenCols();
    syncScroll();
  }

  if (window.SheetCore) {
    window.SheetCore.refreshFrozen = wire;
  }

  document.addEventListener("gb-sheet-tab", function () {
    setTimeout(wire, 50);
  });
  document.addEventListener("gb-sheet-selection", function () {
    setTimeout(wire, 50);
  });

  setTimeout(wire, 0);
})();