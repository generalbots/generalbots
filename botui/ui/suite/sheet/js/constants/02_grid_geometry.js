"use strict";
/* sheet grid state, geometry and virtualized ranges */

(function () {
  var D = window.SheetDefaults;
  var ROW_HEIGHT = D.ROW_HEIGHT;
  var COL_WIDTH = D.COL_WIDTH;
  var HEADER_WIDTH = D.HEADER_WIDTH;
  var OVERSCAN = D.OVERSCAN;
  var rTop = D.rTop;
  var rH = D.rH;
  var rowIdx = D.rowIdx;

  D.GridProto = D.GridProto || {};

  function freshGridState() {
    return {
      totalRows: D.DEFAULT_TOTAL_ROWS,
      totalCols: D.DEFAULT_TOTAL_COLS,
      scrollTop: 0,
      scrollLeft: 0,
      viewportHeight: 0,
      viewportWidth: 0,
      cells: new Map(),
      pool: [],
      usedCount: 0,
      root: null,
      headerRow: null,
      headerCol: null,
      body: null,
      requestSeq: 0,
      lastRenderedRange: null,
      selectedRow: 0,
      selectedCol: 0,
      editingCell: null,
      selectionOverlay: null,
      scrollArea: null,
      midRow: null,
      bodyInner: null,
      headerColPool: [],
      __totalColsWidth: 0,
      host: null,
      instance: null,
    };
  }

  D.createVirtualGrid = function () {
    var g = freshGridState();
    var p = D.GridProto;
    for (var k in p) {
      if (Object.prototype.hasOwnProperty.call(p, k)) g[k] = p[k];
    }
    return g;
  };

  const VirtualGrid = D.createVirtualGrid();

  VirtualGrid.updateViewport = function () {
    this.viewportHeight = this.scrollArea.clientHeight;
    this.viewportWidth = this.scrollArea.clientWidth;
  };

  VirtualGrid.onScroll = function () {
    this.scrollTop = this.scrollArea.scrollTop;
    this.scrollLeft = this.scrollArea.scrollLeft;
    this.headerRow.style.transform = "translateX(" + (-this.scrollLeft) + "px)";
    // Virtualized headers: the visible column window changes with the
    // horizontal position, so the header row must re-render on every scroll.
    this.renderHeaders();
    this.requestRange();
  };

  VirtualGrid.getOrCreateHeaderColNode = function (idx) {
    let node;
    if (idx < this.headerColPool.length) {
      node = this.headerColPool[idx];
    } else {
      node = document.createElement("div");
      node.className = "ss-row-header";
      node.style.cssText = "position:absolute;width:" + HEADER_WIDTH + "px;height:" + ROW_HEIGHT + "px;background:var(--sheet-bg, #0f172a);color:var(--sheet-muted, #94a3b8);text-align:center;line-height:" + ROW_HEIGHT + "px;font-size:11px;border-bottom:1px solid var(--sheet-border, #334155);border-right:1px solid var(--sheet-border, #334155);box-sizing:border-box;z-index:5;";
      this.bodyInner.appendChild(node);
      this.headerColPool.push(node);
    }
    return node;
  };

  VirtualGrid.visibleRowRange = function () {
    const start = Math.max(0, rowIdx(this.scrollTop) - OVERSCAN);
    // Advance through cumulative row heights until the viewport is covered.
    let end = start;
    let h = 0;
    for (let r = start; r < this.totalRows && h < this.viewportHeight; r++, end = r + 1) {
      h += rH(r);
    }
    end = Math.min(this.totalRows, end + OVERSCAN);
    return { start: start, end: end };
  };

  // Column virtualization (#786): only the columns intersecting the viewport
  // are rendered and fetched, exactly like the row window above. colX/colWidth
  // come from SheetCore (imported widths) or Defaults.
  VirtualGrid.colWidthOf = function (c) {
    if (window.SheetCore && window.SheetCore.colWidth) return window.SheetCore.colWidth(c);
    return COL_WIDTH;
  };

  VirtualGrid.rowHeightOf = function (r) {
    if (window.SheetCore && window.SheetCore.rowHeight) return window.SheetCore.rowHeight(r);
    return ROW_HEIGHT;
  };

  VirtualGrid.colXOf = function (c) {
    if (window.SheetCore && window.SheetCore.colX) return window.SheetCore.colX(c);
    return HEADER_WIDTH + c * COL_WIDTH;
  };

  VirtualGrid.totalColsWidth = function () {
    if (!this.__totalColsWidth) {
      this.__totalColsWidth = HEADER_WIDTH + this.totalCols * COL_WIDTH;
    }
    return this.__totalColsWidth;
  };

  // Column virtualization (#786): only columns intersecting the viewport
  // render. The math lives in the widths module (unit-testable); the shell
  // delegates to it when loaded and falls back to a local implementation.
  VirtualGrid.visibleColRange = function () {
    if (window.SheetCore && window.SheetCore.computeVisibleColRange) {
      return window.SheetCore.computeVisibleColRange({
        scrollLeft: this.scrollLeft,
        viewportWidth: this.viewportWidth,
        totalCols: this.totalCols,
        colXOf: this.colXOf.bind(this),
        colWidthOf: this.colWidthOf.bind(this),
      });
    }
    const viewLeft = this.scrollLeft + HEADER_WIDTH;
    const viewRight = viewLeft + this.viewportWidth;
    let lo = 0;
    let hi = this.totalCols - 1;
    let start = 0;
    while (lo <= hi) {
      const mid = (lo + hi) >> 1;
      if (this.colXOf(mid) < viewLeft) { start = mid; lo = mid + 1; }
      else { hi = mid - 1; }
    }
    start = Math.max(0, start);
    let end = start + 1;
    for (let c = start; c < this.totalCols; c++) {
      if (this.colXOf(c) <= viewRight) end = c + 1;
      else break;
    }
    return { start: start, end: Math.min(this.totalCols, end) };
  };

  VirtualGrid.requestRange = function () {
    if (!this.viewportHeight) return;
    const range = this.visibleRowRange();
    const cols = this.visibleColRange();
    const rangeKey = range.start + ":" + range.end + "|" + cols.start + ":" + cols.end;
    if (this.lastRenderedRange === rangeKey) return;
    this.lastRenderedRange = rangeKey;

    // Render row numbers IMMEDIATELY — don't wait for API
    this.renderRowHeaders(range);

    const seq = ++this.requestSeq;
    var self = this;
    D.SheetAPI.getRange(range.start, cols.start, range.end - 1, cols.end - 1, this.instance || this)
      .then(function (data) {
        if (seq !== self.requestSeq) return;
        // Apply the server's recalculated cells: dependents of an edited
        // cell are already cached locally, so skipping existing keys kept
        // stale values on screen until a reload.
        if (data.cells) {
          for (var k in data.cells) {
            self.cells.set(k, data.cells[k]);
          }
        }
        if (data.total_rows) self.totalRows = Math.max(self.totalRows, data.total_rows);
        if (data.total_cols) self.totalCols = Math.max(self.totalCols, data.total_cols);
        self.render();
      })
      .catch(function (err) {
        /* Best-effort range fetch; grid can retry on next scroll/request. */
        console.debug("[gb-sheet] Range fetch failed:", err);
      });
  };

  VirtualGrid.renderRowHeaders = function (range) {
    if (!this.headerColPool) this.headerColPool = [];
    let headerUsed = 0;
    for (let r = range.start; r < range.end; r++) {
      const node = this.getOrCreateHeaderColNode(headerUsed++);
      node.textContent = r + 1;
      node.style.display = "block";
      node.style.left = "0px";
      node.style.top = rTop(r) + "px";
      node.style.height = rH(r) + "px";
      node.style.lineHeight = rH(r) + "px";
      node.dataset.row = r;
    }
    for (let i = headerUsed; i < this.headerColPool.length; i++) {
      this.headerColPool[i].style.display = "none";
    }
  };

  D.virtualGrid = VirtualGrid;
  // keep proto in sync for future per-window instances (#2270)
  for (var _k in VirtualGrid) {
    if (typeof VirtualGrid[_k] === "function") D.GridProto[_k] = VirtualGrid[_k];
  }
})();
