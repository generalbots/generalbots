"use strict";
/* Sheet advanced module: 14_widths — render imported column widths / row heights */

(function () {
  const DEFAULT_COL = 96;
  const DEFAULT_ROW = 24;
  const HEADER_W = 48;
  const COL_WIDTH = 96;
  const ROW_HEIGHT = 24;

  let colWidths = null;
  let rowHeights = null;

  function grid() {
    if (window.SheetCore && window.SheetCore.getGrid) return window.SheetCore.getGrid();
    return window.SheetVirtualGrid || null;
  }

  function wsIndex() {
    if (window.SheetCore && window.SheetCore.wsIndex) return window.SheetCore.wsIndex();
    try {
      const v = parseInt(sessionStorage.getItem("sheet_active_index") || "0", 10);
      return isNaN(v) || v < 0 ? 0 : v;
    } catch (_) {
      return 0;
    }
  }

  function apply() {
    const sheet = window.__LOADED_SHEET;
    colWidths = {};
    rowHeights = {};
    if (sheet && sheet.worksheets && sheet.worksheets[wsIndex()]) {
      const ws = sheet.worksheets[wsIndex()];
      if (ws.column_widths) {
        for (const k in ws.column_widths) {
          const idx = parseInt(k, 10);
          if (!isNaN(idx)) colWidths[idx] = ws.column_widths[k];
        }
      }
      if (ws.row_heights) {
        for (const k in ws.row_heights) {
          const idx = parseInt(k, 10);
          if (!isNaN(idx)) rowHeights[idx] = ws.row_heights[k];
        }
      }
    }
  }

  function colWidth(idx) {
    return colWidths && colWidths[idx] ? colWidths[idx] : DEFAULT_COL;
  }

  function rowHeight(idx) {
    return rowHeights && rowHeights[idx] ? rowHeights[idx] : DEFAULT_ROW;
  }

  function colX(idx) {
    let x = HEADER_W;
    for (let c = 0; c < idx; c++) x += colWidth(c);
    return x;
  }

  function totalColWidth() {
    const g = grid();
    const n = g ? g.totalCols : 26;
    let w = HEADER_W;
    for (let c = 0; c < n; c++) w += colWidth(c);
    return w;
  }

  function patchRender() {
    const g = grid();
    if (!g || g.__widthsPatched) return;
    g.__widthsPatched = true;

    g.colWidth = function (idx) { return colWidth(idx); };
    g.rowHeight = function (idx) { return rowHeight(idx); };
    g.colX = function (idx) { return colX(idx); };

    const origIH = g.renderHeaders.bind(g);
    g.renderHeaders = function () {
      origIH();
      const h = g.headerRow;
      if (!h) return;
      h.innerHTML = "";
      const corner = document.createElement("div");
      corner.style.cssText = "width:" + HEADER_W + "px;flex-shrink:0;";
      h.appendChild(corner);
      for (let c = 0; c < g.totalCols; c++) {
        const hd = document.createElement("div");
        hd.textContent = g.colName ? g.colName(c) : String.fromCharCode(65 + c);
        hd.style.cssText =
          "width:" + colWidth(c) + "px;background:#0f172a;color:#94a3b8;text-align:center;line-height:24px;font-size:11px;" +
          "border-right:1px solid #334155;flex-shrink:0;";
        h.appendChild(hd);
      }
      g.headerColPool = [];
    };

    const origIR = g.renderRow.bind(g);
    g.renderRow = function (row) {
      for (let c = 0; c < g.totalCols; c++) {
        const ref = (g.colName ? g.colName(c) : String.fromCharCode(65 + c)) + (row + 1);
        const key = row + "," + c;
        const cellData = g.cells.get(key);
        const value = cellData ? (cellData.value || "") : "";
        const formula = cellData ? (cellData.formula || "") : "";
        const node = g.getOrCreateNode();
        node.style.display = "block";
        node.dataset.ref = ref;
        node.dataset.row = row;
        node.dataset.col = c;
        node.dataset.formula = formula;
        node.textContent = value;
        node.style.left = (g.colX(c)) + "px";
        node.style.top = (row * rowHeight(row)) + "px";
        node.style.width = colWidth(c) + "px";
        node.style.height = rowHeight(row) + "px";
        node.style.fontWeight = "";
        node.style.fontStyle = "";
        node.style.textDecoration = "";
        node.style.fontFamily = "";
        node.style.fontSize = "12px";
        node.style.color = "#f8fafc";
        node.style.backgroundColor = "#0f172a";
        if (g.applyCellStyle) g.applyCellStyle(node, cellData);
        if (g.editingCell !== node) {
          node.contentEditable = "false";
          node.style.outline = "none";
          node.style.zIndex = "1";
        }
      }
    };

    if (g.bodyInner) {
      g.bodyInner.style.width = totalColWidth() + "px";
      if (rowHeights && Object.keys(rowHeights).length) {
        let totalH = 0;
        for (let i = 0; i < g.totalRows; i++) totalH += rowHeight(i);
        g.bodyInner.style.height = totalH + "px";
      }
    }
  }

  function wire() {
    const g = grid();
    if (!g) {
      setTimeout(wire, 100);
      return;
    }
    apply();
    patchRender();
    g.render();
    if (window.SheetSort && window.SheetSort.refreshHandles) window.SheetSort.refreshHandles();
    if (window.SheetFilter && window.SheetFilter.renderHeads) window.SheetFilter.renderHeads();
  }

  window.SheetWidths = {
    apply: wire,
    colWidth: colWidth,
    rowHeight: rowHeight,
    colX: colX,
  };

  if (window.SheetCore) {
    window.SheetCore.colWidth = function (idx) { return colWidth(idx); };
    window.SheetCore.rowHeight = function (idx) { return rowHeight(idx); };
    window.SheetCore.colX = function (idx) { return colX(idx); };
    window.SheetCore.refreshWidths = wire;
  }

  document.addEventListener("gb-sheet-tab", function () {
    setTimeout(wire, 50);
  });

  setTimeout(wire, 0);
})();