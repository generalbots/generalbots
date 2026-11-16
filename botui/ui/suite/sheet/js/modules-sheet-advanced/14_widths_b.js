"use strict";
/* Split from 14_widths.js for #2305 (450-line limit): wire and exports. */
(function () {
  var _I = window.__SheetWidths || {};
  function grid(){return _I.grid?_I.grid():null;}
  var wsIndex=_I.wsIndex, apply=_I.apply, patchRender=_I.patchRender, colWidth=_I.colWidth, rowHeight=_I.rowHeight;
  var colX=_I.colX, rowTop=_I.rowTop, rowAt=_I.rowAt, computeVisibleColRange=_I.computeVisibleColRange;
  var wireDragResize=_I.wireDragResize, setColWidth=_I.setColWidth, setRowHeight=_I.setRowHeight, persistResize=_I.persistResize;
  var totalColWidth=_I.totalColWidth, charsToPx=_I.charsToPx, pointsToPx=_I.pointsToPx, colPxToChars=_I.colPxToChars, rowPxToPoints=_I.rowPxToPoints;

  function wire() {
    const g = grid();
    if (!g) {
      setTimeout(wire, 100);
      return;
    }
    apply();
    patchRender();
    g.render();
    if (g.renderHeaders) g.renderHeaders();
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
    }).catch(function (err) {
      console.error("[gb-sheet] Failed to persist column width:", err);
    });
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
    if (g.renderHeaders) g.renderHeaders();
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
      // Persist in the workbook's native units, never in the px the grid uses:
      // storing pixels made every reload multiply the size (#2152).
      if (mode === "col" && index >= 0)
        persistResize({ col: index, width: colPxToChars(colWidth(index)) });
      if (mode === "row" && index >= 0)
        persistResize({ row: index, height: rowPxToPoints(rowHeight(index)) });
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
    resizeColumn: function (idx, w) { setColWidth(idx, w); persistResize({ col: idx, width: colPxToChars(colWidth(idx)) }); },
    resizeRow: function (idx, h) { setRowHeight(idx, h); persistResize({ row: idx, height: rowPxToPoints(rowHeight(idx)) }); },
    charsToPx: charsToPx,
    pointsToPx: pointsToPx,
    colPxToChars: colPxToChars,
    rowPxToPoints: rowPxToPoints,
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