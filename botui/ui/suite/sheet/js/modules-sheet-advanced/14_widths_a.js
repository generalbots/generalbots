"use strict";
/* Sheet advanced module: 14_widths — render imported column widths / row heights */

(function () {
  const DEFAULT_COL = 96;
  const DEFAULT_ROW = 24;
  const HEADER_W = 48;
  const COL_WIDTH = 96;
  const ROW_HEIGHT = 24;

  // The workbook stores geometry in its native units — OOXML column widths are
  // character counts and row heights are points — while the grid lays out in
  // pixels. Passing either unit verbatim to the renderer squeezed every
  // generated workbook into 3–11 px columns (#2152).
  const CHAR_WIDTH_PX = 7; // Calibri 11 average advance
  const COL_PADDING_PX = 5; // cell padding Excel adds to a column
  const MIN_COL_PX = 28;
  const POINTS_TO_PX = 96 / 72; // 1 pt = 1.333 px at 96 dpi
  const MIN_ROW_PX = 18;

  function charsToPx(chars) {
    const px = chars * CHAR_WIDTH_PX + COL_PADDING_PX;
    return Math.max(MIN_COL_PX, Math.round(px));
  }

  function colPxToChars(px) {
    return Math.round(((px - COL_PADDING_PX) / CHAR_WIDTH_PX) * 100) / 100;
  }

  function pointsToPx(points) {
    return Math.max(MIN_ROW_PX, Math.round(points * POINTS_TO_PX));
  }

  function rowPxToPoints(px) {
    return Math.round((px / POINTS_TO_PX) * 100) / 100;
  }


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
          const chars = Number(ws.column_widths[k]);
          if (!isNaN(idx) && isFinite(chars)) colWidths[idx] = charsToPx(chars);
        }
      }
      if (ws.row_heights) {
        for (const k in ws.row_heights) {
          const idx = parseInt(k, 10);
          const points = Number(ws.row_heights[k]);
          if (!isNaN(idx) && isFinite(points)) rowHeights[idx] = pointsToPx(points);
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

  // Cumulative vertical offset of the TOP of row `idx` (sum of all row
  // heights above it). Variable row heights shift every following row down,
  // so positioning MUST use this, never `row * ROW_HEIGHT`.
  function rowTop(idx) {
    let t = 0;
    const n = Math.max(0, idx);
    for (let r = 0; r < n; r++) t += rowHeight(r);
    return t;
  }

  // Given a y offset relative to the body's top, return the row index whose
  // vertical span contains y (binary search over cumulative row heights).
  function rowAt(y) {
    const g = grid();
    const n = g ? g.totalRows : 1000;
    let lo = 0, hi = n - 1, row = 0;
    while (lo <= hi) {
      const mid = (lo + hi) >> 1;
      if (rowTop(mid) <= y) { row = mid; lo = mid + 1; }
      else { hi = mid - 1; }
    }
    return row;
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

  // Column virtualization (#786): computes the visible column window for a
  // grid given its scrollLeft and viewportWidth. Kept here (and reused by the
  // shell grid) so the math is unit-testable outside the DOM.
  function computeVisibleColRange(g) {
    const viewLeft = g.scrollLeft || 0;
    const viewRight = viewLeft + (g.viewportWidth || 0);
    const colXOf = g.colXOf || function (c) { return HEADER_W + c * colWidth(c); };
    const colWidthOf = g.colWidthOf || colWidth;
    let lo = 0;
    let hi = g.totalCols - 1;
    let start = 0;
    while (lo <= hi) {
      const mid = (lo + hi) >> 1;
      if (colXOf(mid) < viewLeft) { start = mid; lo = mid + 1; }
      else { hi = mid - 1; }
    }
    start = Math.max(0, start);
    let end = start + 1;
    for (let c = start; c < g.totalCols; c++) {
      const x = colXOf(c);
      const w = colWidthOf(c);
      if (x <= viewRight) end = c + 1;
      else break;
    }
    return { start: start, end: Math.min(g.totalCols, end) };
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
      const cols = g.visibleColRange ? g.visibleColRange() : { start: 0, end: g.totalCols };
      for (let c = cols.start; c < cols.end; c++) {
        const hd = document.createElement("div");
        hd.className = "ss-col-head";
        hd.dataset.col = c;
        hd.textContent = g.colName ? g.colName(c) : String.fromCharCode(65 + c);
        hd.style.cssText =
          "position:absolute;left:" + colX(c) + "px;width:" + colWidth(c) + "px;background:var(--sheet-bg, #0f172a);color:var(--sheet-muted, #94a3b8);text-align:center;line-height:24px;font-size:11px;" +
          "border-right:1px solid var(--sheet-border, #334155);flex-shrink:0;box-sizing:border-box;";
        h.appendChild(hd);
      }
      g.headerColPool = [];
    };

    const origIR = g.renderRow.bind(g);
    g.renderRow = function (row) {
      const cols = g.visibleColRange ? g.visibleColRange() : { start: 0, end: g.totalCols };
      for (let c = cols.start; c < cols.end; c++) {
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
        node.style.top = rowTop(row) + "px";
        node.style.width = colWidth(c) + "px";
        node.style.height = rowHeight(row) + "px";
        node.style.fontWeight = "";
        node.style.fontStyle = "";
        node.style.textDecoration = "";
        node.style.fontFamily = "";
        node.style.fontSize = "12px";
        node.style.color = "var(--sheet-fg, #f8fafc)";
        node.style.backgroundColor = "var(--sheet-bg, #0f172a)";
        if (g.applyCellStyle) g.applyCellStyle(node, cellData);
        var condStyle = g.conditionalStyleFor ? g.conditionalStyleFor(row, c, cellData) : null;
        if (condStyle && g.applyCellStyle) g.applyCellStyle(node, { style: condStyle });
        var fmtColor = g.formatColorFor ? g.formatColorFor(cellData, value) : null;
        if (fmtColor) node.style.color = fmtColor;
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

  // Internals for split module (#2305).
  var _I = window.__SheetWidths = window.__SheetWidths || {};
  _I.grid = grid; _I.wsIndex = wsIndex; _I.apply = apply; _I.patchRender = patchRender;
  _I.colWidth = colWidth; _I.rowHeight = rowHeight; _I.colX = colX; _I.rowTop = rowTop; _I.rowAt = rowAt;
  _I.computeVisibleColRange = computeVisibleColRange;
  _I.setColWidth = setColWidth; _I.setRowHeight = setRowHeight; _I.persistResize = persistResize;
  _I.totalColWidth = totalColWidth; _I.charsToPx = charsToPx; _I.pointsToPx = pointsToPx;
  _I.colPxToChars = colPxToChars; _I.rowPxToPoints = rowPxToPoints; _I.sheetId = sheetId;
  _I.getColWidths = function(){return colWidths;}; _I.getRowHeights = function(){return rowHeights;};
})();
