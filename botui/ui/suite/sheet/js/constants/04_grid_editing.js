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
    if (!VirtualGrid.selectionOverlay) return;
    const cwFn = (window.SheetCore && window.SheetCore.colWidth) ? window.SheetCore.colWidth : function () { return COL_WIDTH; };
    const cxFn = (window.SheetCore && window.SheetCore.colX) ? window.SheetCore.colX : function (c) { return HEADER_WIDTH + c * COL_WIDTH; };
    VirtualGrid.selectionOverlay.style.display = "block";
    VirtualGrid.selectionOverlay.style.left = (cxFn(col) - 1) + "px";
    VirtualGrid.selectionOverlay.style.top = (rTop(row) - 1) + "px";
    VirtualGrid.selectionOverlay.style.width = (cwFn(col) + 2) + "px";
    VirtualGrid.selectionOverlay.style.height = (rH(row) + 2) + "px";
  }

  function updateFormulaBar() {
    const addrEl = document.getElementById("cellAddress");
    const formulaEl = document.getElementById("formulaInput");
    if (!addrEl) return;
    if (VirtualGrid.selectedRow == null || VirtualGrid.selectedCol == null) {
      addrEl.value = "";
      if (formulaEl) formulaEl.value = "";
      return;
    }
    const ref = colName(VirtualGrid.selectedCol) + (VirtualGrid.selectedRow + 1);
    addrEl.value = ref;
    if (formulaEl) {
      const key = VirtualGrid.selectedRow + "," + VirtualGrid.selectedCol;
      const cellData = VirtualGrid.cells.get(key);
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
    this.selectedRow = row;
    this.selectedCol = col;
    positionSelectionOverlay(row, col);
    updateFormulaBar();
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
    var cell = e.target;
    // Only process if this cell was being edited
    if (this.editingCell !== cell) return;
    this.editingCell = null;
    cell.contentEditable = "false";
    cell.style.outline = "none";
    cell.style.zIndex = "1";

    const ref = cell.dataset.ref;
    const val = cell.textContent || "";
    const origVal = cell.dataset.origVal || "";

    if (val === origVal) {
      if (cell.dataset.formula) {
        const key = cell.dataset.row + "," + cell.dataset.col;
        const cellData = VirtualGrid.cells.get(key);
        cell.textContent = cellData ? (cellData.value || "") : "";
      }
      return;
    }

    D.SheetAPI.updateCell(ref, val).then(function (res) {
      if (res && res.success) {
        VirtualGrid.lastRenderedRange = null;
        VirtualGrid.requestRange();
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

  // Format the selected cell, or the whole active selection range when one
  // spans multiple cells, so toolbar formatting applies where the user expects.
  VirtualGrid.formatSelectedCell = function (styleProps) {
    var row = this.selectedRow;
    var col = this.selectedCol;
    if (row == null || col == null) return Promise.resolve(null);
    var startRow = row, startCol = col, endRow = row, endCol = col;
    var adv = window.SheetAdvanced;
    if (adv && typeof adv.getSelection === "function") {
      var sel = adv.getSelection();
      if (sel && (sel.startRow !== sel.endRow || sel.startCol !== sel.endCol)) {
        startRow = sel.startRow;
        startCol = sel.startCol;
        endRow = sel.endRow;
        endCol = sel.endCol;
      }
    }
    var self = this;
    return fetch("/api/sheet/format", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        sheet_id: D.currentSheetId(),
        worksheet_index: D.WORKSHEET_INDEX,
        start_row: startRow,
        start_col: startCol,
        end_row: endRow,
        end_col: endCol,
        style: styleProps
      })
    }).then(function (r) { return r.json(); }).then(function () {
      for (var r0 = startRow; r0 <= endRow; r0++) {
        for (var c0 = startCol; c0 <= endCol; c0++) {
          var key = r0 + "," + c0;
          var existing = self.cells.get(key) || {};
          existing.style = Object.assign({}, existing.style || {}, styleProps);
          self.cells.set(key, existing);
          var node = self.bodyInner.querySelector(
            '[data-row="' + r0 + '"][data-col="' + c0 + '"]'
          );
          if (node) self.applyCellStyle(node, existing);
        }
      }
      if (typeof self.requestRange === "function") {
        self.lastRenderedRange = null;
        self.requestRange();
      }
    }).catch(function () { return null; });
  };
})();
