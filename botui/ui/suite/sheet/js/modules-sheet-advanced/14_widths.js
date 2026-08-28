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
          "position:absolute;left:" + colX(c) + "px;width:" + colWidth(c) + "px;background:#0f172a;color:#94a3b8;text-align:center;line-height:24px;font-size:11px;" +
          "border-right:1px solid #334155;flex-shrink:0;box-sizing:border-box;";
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
    wireDragResize();
  }

  function sheetId() {
    return (window.__SHEET_INITIAL_ID) || "current";
  }

  function persistResize(payload) {
    const body = Object.assign({ sheet_id: sheetId(), worksheet_index: wsIndex() }, payload);
    fetch("/api/sheet/resize", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body)
    }).catch(function () {});
  }

  function setColWidth(idx, w) {
    if (!colWidths) colWidths = {};
    colWidths[idx] = Math.max(24, Math.round(w));
    if (window.SheetWidths && window.SheetWidths.onResize) window.SheetWidths.onResize(idx, null);
    const g = grid();
    if (!g) return;
    if (g.bodyInner) g.bodyInner.style.width = totalColWidth() + "px";
    g.lastRenderedRange = null;
    if (g.render) g.render();
  }

  function setRowHeight(idx, h) {
    if (!rowHeights) rowHeights = {};
    rowHeights[idx] = Math.max(12, Math.round(h));
    if (window.SheetWidths && window.SheetWidths.onResize) window.SheetWidths.onResize(null, idx);
    const g = grid();
    if (!g) return;
    if (g.bodyInner) g.bodyInner.style.height = totalRowHeight() + "px";
    g.lastRenderedRange = null;
    if (g.render) g.render();
  }

  function totalRowHeight() {
    const g = grid();
    const n = g ? g.totalRows : 1000;
    let h = 0;
    for (let r = 0; r < n; r++) h += rowHeight(r);
    return h;
  }

  // Grab zone (px) either side of a header boundary where resize is active.
  const RESIZE_GRIP = 8;

  // Returns the column whose RIGHT boundary is being resized, or -1.
  // Uses absolute header geometry so the cursor reacts BETWEEN column
  // headers, not just on the last few pixels of a cell.
  function headerColIndex(el, x) {
    const g = grid();
    if (!g || !el) return -1;
    const headRect = g.headerRow.getBoundingClientRect();
    const headLeft = headRect.left;
    // Only meaningful when hovering the column-header strip.
    if (x < headLeft + HEADER_W) return -1;
    const xLocal = x - headLeft;
    // Column boundaries sit at colX(c)+colWidth(c) in the header strip's
    // local coordinates (colX already includes HEADER_W).
    for (let c = 0; c < g.totalCols; c++) {
      const end = colX(c) + colWidth(c);
      // Resizeable zone: centred on the boundary between column c and c+1.
      if (Math.abs(xLocal - end) <= RESIZE_GRIP) return c;
    }
    return -1;
  }

  function rowHeaderIndex(el, y) {
    if (!el || !el.classList || !el.classList.contains("ss-row-header")) return -1;
    if (el.dataset.row === undefined) return -1;
    const r = parseInt(el.dataset.row, 10);
    if (isNaN(r)) return -1;
    const e = el.getBoundingClientRect();
    // Resizeable zone centred on the bottom boundary of this row (resize row r)
    if (Math.abs(y - e.bottom) <= RESIZE_GRIP) return r;
    // …or on its top boundary (resize the previous row r-1).
    if (Math.abs(y - e.top) <= RESIZE_GRIP && r > 0) return r - 1;
    return -1;
  }

  function wireDragResize() {
    const g = grid();
    if (!g) { setTimeout(wireDragResize, 100); return; }
    if (g.__dragWired) return;
    g.__dragWired = true;
    if (!g.headerRow || !g.headerRow.addEventListener || !g.bodyInner || !g.bodyInner.addEventListener) return;

    const root = g.root || document;
    let mode = null;
    let startX = 0, startY = 0, startW = 0, startH = 0, index = -1;

    function endDrag(e) {
      e.preventDefault();
      document.removeEventListener("mousemove", onMove);
      document.removeEventListener("mouseup", endDrag);
      if (mode === "col" && index >= 0) persistResize({ col: index, width: colWidth(index) });
      if (mode === "row" && index >= 0) persistResize({ row: index, height: rowHeight(index) });
      // A `click` fires on the header right after a drag-resize; the header
      // click handler would select the whole row/column and force-scroll to
      // the far corner (~1M index). Stamp the time so those handlers can
      // ignore the click that immediately follows a resize.
      window.__GB_SHEET_RESIZED_AT = Date.now();
      mode = null; index = -1;
    }

    function onMove(e) {
      e.preventDefault();
      if (mode === "col" && index >= 0) {
        setColWidth(index, startW + (e.clientX - startX));
      } else if (mode === "row" && index >= 0) {
        setRowHeight(index, startH + (e.clientY - startY));
      }
    }

    g.headerRow.addEventListener("mousedown", function (e) {
      const col = headerColIndex(e.target, e.clientX);
      if (col < 0) return;
      e.preventDefault();
      e.stopPropagation();
      mode = "col"; index = col;
      startX = e.clientX; startW = colWidth(col);
      document.addEventListener("mousemove", onMove);
      document.addEventListener("mouseup", endDrag);
    });

    g.bodyInner.addEventListener("mousedown", function (e) {
      const row = rowHeaderIndex(e.target, e.clientY);
      if (row < 0) return;
      e.preventDefault();
      e.stopPropagation();
      mode = "row"; index = row;
      startY = e.clientY; startH = rowHeight(row);
      document.addEventListener("mousemove", onMove);
      document.addEventListener("mouseup", endDrag);
    });

    // Affordance: show a resize cursor while hovering the boundary between
    // column headers (or the boundary below a row header), so the user can
    // discover that drag-resizing is available. Uses absolute geometry so the
    // cursor reacts even between two adjacent headers.
    g.headerRow.addEventListener("mousemove", function (e) {
      if (mode) return;
      const col = headerColIndex(e.target, e.clientX);
      g.headerRow.style.cursor = col >= 0 ? "col-resize" : "default";
    });
    g.headerRow.addEventListener("mouseleave", function () {
      g.headerRow.style.cursor = "default";
    });
    g.bodyInner.addEventListener("mousemove", function (e) {
      if (mode) return;
      // The row header is thin (48px) and has no gap between rows, so the
      // element under the pointer is whichever header contains the y — the
      // boundary can sit on the NEXT row's top edge. Set the cursor per
      // header so the row-resize affordance shows at every row boundary.
      const headers = g.bodyInner.querySelectorAll(".ss-row-header");
      headers.forEach(function (h) {
        const rp = parseInt(h.dataset.row, 10);
        if (isNaN(rp)) { h.style.cursor = "default"; return; }
        const rect = h.getBoundingClientRect();
        const near =
          Math.abs(e.clientY - rect.bottom) <= RESIZE_GRIP ||
          (Math.abs(e.clientY - rect.top) <= RESIZE_GRIP && rp > 0);
        h.style.cursor = near ? "row-resize" : "default";
      });
    });
    g.bodyInner.addEventListener("mouseleave", function () {
      const headers = g.bodyInner.querySelectorAll(".ss-row-header");
      headers.forEach(function (h) { h.style.cursor = "default"; });
      g.bodyInner.style.cursor = "default";
    });
  }

  window.SheetWidths = {
    apply: wire,
    colWidth: colWidth,
    rowHeight: rowHeight,
    colX: colX,
    rowTop: rowTop,
    rowAt: rowAt,
    computeVisibleColRange: computeVisibleColRange,
    resizeColumn: function (idx, w) { setColWidth(idx, w); persistResize({ col: idx, width: colWidth(idx) }); },
    resizeRow: function (idx, h) { setRowHeight(idx, h); persistResize({ row: idx, height: rowHeight(idx) }); },
  };

  if (window.SheetCore) {
    window.SheetCore.colWidth = function (idx) { return colWidth(idx); };
    window.SheetCore.rowHeight = function (idx) { return rowHeight(idx); };
    window.SheetCore.colX = function (idx) { return colX(idx); };
    window.SheetCore.rowTop = function (idx) { return rowTop(idx); };
    window.SheetCore.rowAt = function (y) { return rowAt(y); };
    window.SheetCore.refreshWidths = wire;
    // #786: share the virtualization math so the shell grid and every module
    // compute the identical column window.
    window.SheetCore.computeVisibleColRange = computeVisibleColRange;
  }

  document.addEventListener("gb-sheet-tab", function () {
    setTimeout(wire, 50);
  });

  setTimeout(wire, 0);
})();