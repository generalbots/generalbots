"use strict";
/* sheet grid editing — selection, keyboard, cell writes and formatting */

(function () {
  var D = window.SheetDefaults;
  var COL_WIDTH = D.COL_WIDTH;
  var HEADER_WIDTH = D.HEADER_WIDTH;
  var colName = D.colName;
  var rTop = D.rTop;
  var rH = D.rH;
  var parseCellRef = D.parseCellRef;
  const VirtualGrid = D.virtualGrid;

  function positionSelectionOverlay(row, col) {
    var grid = this && this.selectionOverlay ? this : ((D.getActiveSheetInstance && D.getActiveSheetInstance() && D.getActiveSheetInstance().grid) || VirtualGrid);
    if (!grid.selectionOverlay) return;
    const cwFn = (window.SheetCore && window.SheetCore.colWidth) ? window.SheetCore.colWidth : function () { return COL_WIDTH; };
    const cxFn = (window.SheetCore && window.SheetCore.colX) ? window.SheetCore.colX : function (c) { return HEADER_WIDTH + c * COL_WIDTH; };
    grid.selectionOverlay.style.display = "block";
    grid.selectionOverlay.style.left = (cxFn(col) - 1) + "px";
    grid.selectionOverlay.style.top = (rTop(row) - 1) + "px";
    grid.selectionOverlay.style.width = (cwFn(col) + 2) + "px";
    grid.selectionOverlay.style.height = (rH(row) + 2) + "px";
  }

  function qInGrid(grid, selector) {
    if (grid && grid.windowBody) return grid.windowBody.querySelector(selector);
    return document.querySelector(selector);
  }

  function updateFormulaBar(grid) {
    grid = grid || (D.getActiveSheetInstance && D.getActiveSheetInstance() && D.getActiveSheetInstance().grid) || VirtualGrid;
    const addrEl = qInGrid(grid, "#cellAddress");
    const formulaEl = qInGrid(grid, "#formulaInput");
    if (!addrEl) return;
    if (grid.selectedRow == null || grid.selectedCol == null) {
      addrEl.value = "";
      if (formulaEl) formulaEl.value = "";
      return;
    }
    const ref = colName(grid.selectedCol) + (grid.selectedRow + 1);
    addrEl.value = ref;
    if (formulaEl) {
      const key = grid.selectedRow + "," + grid.selectedCol;
      const cellData = grid.cells.get(key);
      const formula = cellData && cellData.formula ? cellData.formula : "";
      const val = cellData && cellData.value ? cellData.value : "";
      formulaEl.value = formula ? formula : val;
    }
  }

  VirtualGrid.onCellMouseDown = function (e) {
    if (e.button !== 0) return;
    var row = parseInt(e.target.dataset.row, 10);
    var col = parseInt(e.target.dataset.col, 10);
    if (!isNaN(row) && !isNaN(col)) {
      this.selectCell(row, col);
    }
  };

  VirtualGrid.selectCell = function (row, col, preventScroll) {
    if (D.setActiveSheetInstance && this.instance) D.setActiveSheetInstance(this.instance);
    this.selectedRow = row;
    this.selectedCol = col;
    positionSelectionOverlay.call(this, row, col);
    updateFormulaBar(this);
    if (!preventScroll) this.ensureVisible(row, col);
    // Focus the cell element so keyboard events reach it
    var cellEl = this.bodyInner.querySelector('[data-row="' + row + '"][data-col="' + col + '"]');
    if (cellEl) cellEl.focus({preventScroll: true});
    if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
      window.GBCollab.sendCursor(row, col);
    }
  };

  VirtualGrid.ensureVisible = function (row, col) {
    var viewTop = this.scrollTop;
    var viewBottom = viewTop + this.viewportHeight;
    var viewLeft = this.scrollLeft;
    var viewRight = viewLeft + this.viewportWidth;
    var cellTop = rTop(row);
    var cellBottom = cellTop + rH(row);
    var cellLeft = this.colXOf(col);
    var cellRight = cellLeft + this.colWidthOf(col);
    if (cellTop < viewTop) this.scrollArea.scrollTop = cellTop - 10;
    else if (cellBottom > viewBottom) this.scrollArea.scrollTop = cellBottom - this.viewportHeight + 10;
    if (cellLeft < viewLeft + HEADER_WIDTH) this.scrollArea.scrollLeft = cellLeft - HEADER_WIDTH - 10;
    else if (cellRight > viewRight) this.scrollArea.scrollLeft = cellRight - this.viewportWidth + 10;
  };

  VirtualGrid.onCellDblClick = function (e) {
    var cell = e.target;
    cell.contentEditable = "true";
    this.editingCell = cell;
    cell.dataset.origVal = cell.textContent || "";
    if (cell.dataset.formula) {
      cell.textContent = cell.dataset.formula;
    }
    // Focus and place cursor at end
    var range = document.createRange();
    var sel = window.getSelection();
    range.selectNodeContents(cell);
    range.collapse(false);
    sel.removeAllRanges();
    sel.addRange(range);
    cell.focus();
  };

  VirtualGrid.onCellBlur = function (e) {
    var self = this;
    var cell = e.target;
    // Only process if this cell was being edited
    if (self.editingCell !== cell) return;
    self.editingCell = null;
    cell.contentEditable = "false";
    cell.style.outline = "none";
    cell.style.zIndex = "1";

    const ref = cell.dataset.ref;
    const val = cell.textContent || "";
    const origVal = cell.dataset.origVal || "";

    if (val === origVal) {
      if (cell.dataset.formula) {
        const key = cell.dataset.row + "," + cell.dataset.col;
        const cellData = self.cells.get(key);
        cell.textContent = cellData ? (cellData.value || "") : "";
      }
      return;
    }

    D.SheetAPI.updateCell(ref, val, self.instance || self).then(function (res) {
      if (res && res.success) {
        self.lastRenderedRange = null;
        self.requestRange();
        return;
      }
      // A rejected edit used to stay on screen as if saved. Restore the
      // previous text and say so.
      cell.textContent = origVal;
      if (window.sheetNotify) {
        window.sheetNotify("Cell not saved: " + (res && res.error ? res.error : "server rejected the edit"), true);
      }
    });

    if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
      const p = parseCellRef(ref);
      if (p) {
        window.GBCollab.sendEdit({ row: p.row, col: p.col, content: val, length: val.length });
        window.GBCollab.sendTypingStop();
      }
    }
  };

  VirtualGrid.onCellKeyDown = function (e) {
    var cell = e.target;
    var row = parseInt(cell.dataset.row, 10);
    var col = parseInt(cell.dataset.col, 10);
    if (isNaN(row) || isNaN(col)) return;

    var isEditing = (this.editingCell === cell);

    if (e.key === "Enter") {
      e.preventDefault();
      if (isEditing) {
        // Finish editing, move down
        cell.blur();
        this.selectCell(row + 1, col);
        var next = this.bodyInner.querySelector('[data-row="' + (row + 1) + '"][data-col="' + col + '"]');
        if (!next && row + 1 < this.totalRows) {
          this.scrollArea.scrollTop += rH(row + 1);
        }
      } else {
        // Enter edit mode on selected cell
        this.startEditingCell(cell);
      }
    } else if (e.key === "Tab") {
      e.preventDefault();
      if (isEditing) cell.blur();
      this.selectCell(row, col + 1);
    } else if (e.key === "ArrowDown" && !isEditing) {
      e.preventDefault();
      this.selectCell(row + 1, col);
    } else if (e.key === "ArrowUp" && !isEditing) {
      e.preventDefault();
      if (row > 0) this.selectCell(row - 1, col);
    } else if (e.key === "ArrowRight" && !isEditing) {
      e.preventDefault();
      this.selectCell(row, col + 1);
    } else if (e.key === "ArrowLeft" && !isEditing) {
      e.preventDefault();
      if (col > 0) this.selectCell(row, col - 1);
    } else if ((e.key === "F2") && !isEditing) {
      e.preventDefault();
      this.startEditingCell(cell);
    } else if (isEditing) {
      if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
        var ref = cell.dataset.ref || "";
        var p = parseCellRef(ref);
        if (p) window.GBCollab.debouncedTypingStart(p.row, p.col);
      }
    }
  };

  VirtualGrid.startEditingCell = function (cell) {
    if (!cell) return;
    cell.contentEditable = "true";
    this.editingCell = cell;
    cell.dataset.origVal = cell.textContent || "";
    if (cell.dataset.formula) {
      cell.textContent = cell.dataset.formula;
    }
    cell.focus();
    var range = document.createRange();
    var sel = window.getSelection();
    range.selectNodeContents(cell);
    range.collapse(false);
    sel.removeAllRanges();
    sel.addRange(range);
  };

  // Format the currently selected cell with given style properties
  VirtualGrid.formatSelectedCell = function (styleProps) {
    var row = this.selectedRow;
    var col = this.selectedCol;
    if (row == null || col == null) return Promise.resolve(null);
    var self = this;
    var inst = self.instance || (D.getActiveSheetInstance && D.getActiveSheetInstance());
    return fetch("/api/sheet/format", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        sheet_id: D.currentSheetId(inst),
        worksheet_index: inst && typeof inst.worksheetIndex === "number" ? inst.worksheetIndex : D.WORKSHEET_INDEX,
        start_row: row,
        start_col: col,
        end_row: row,
        end_col: col,
        style: styleProps
      })
    }).then(function (r) { return r.json(); }).then(function () {
      // Update local cell data and re-render the cell
      var key = row + "," + col;
      var existing = self.cells.get(key) || {};
      var mergedStyle = Object.assign({}, existing.style || {}, styleProps);
      existing.style = mergedStyle;
      self.cells.set(key, existing);
      // Re-render just this cell
      var node = self.bodyInner.querySelector('[data-row="' + row + '"][data-col="' + col + '"]');
      if (node) self.applyCellStyle(node, existing);
    }).catch(function () { return null; });
  };

  // sync proto for per-window instances (#2270)
  var D2 = window.SheetDefaults;
  for (var _k4 in VirtualGrid) {
    if (typeof VirtualGrid[_k4] === "function") D2.GridProto[_k4] = VirtualGrid[_k4];
  }
})();
