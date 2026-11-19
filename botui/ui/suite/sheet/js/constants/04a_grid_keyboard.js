"use strict";
/* Sheet grid keyboard model (#2493). Extends — never replaces — the existing
   arrow/edit behaviour with the full spreadsheet map: data-edge jumps, page
   moves, row/column selection, range extension, F2 and type-to-edit. */

(function () {
  var D = window.SheetDefaults;
  if (!D || !D.virtualGrid) return;
  var VirtualGrid = D.virtualGrid;
  var ROW_HEIGHT = D.ROW_HEIGHT;
  var MAX_CLEAR_CELLS = 1000;

  function clampRow(g, r) {
    return Math.max(0, Math.min((g.totalRows || 1) - 1, r));
  }
  function clampCol(g, c) {
    return Math.max(0, Math.min((g.totalCols || 1) - 1, c));
  }
  function hasContent(data) {
    return !!(data && ((data.value != null && String(data.value) !== "") || data.formula));
  }

  // Nearest populated cell beyond the cursor in a direction, or the sheet edge.
  // Only the loaded cell Map is scanned (never the 1M x 16K address space).
  function dataEdge(g, row, col, direction) {
    var found = null;
    try {
      g.cells.forEach(function (data, key) {
        if (!hasContent(data)) return;
        var parts = String(key).split(",");
        var r = parseInt(parts[0], 10);
        var c = parseInt(parts[1], 10);
        if (isNaN(r) || isNaN(c)) return;
        var ok = false;
        if (direction === "up") ok = r < row && c === col;
        else if (direction === "down") ok = r > row && c === col;
        else if (direction === "left") ok = c < col && r === row;
        else if (direction === "right") ok = c > col && r === row;
        if (!ok) return;
        if (!found || (direction === "up" && r > found.row) ||
            (direction === "down" && r < found.row) ||
            (direction === "left" && c > found.col) ||
            (direction === "right" && c < found.col)) {
          found = { row: r, col: c };
        }
      });
    } catch (e) {
      console.warn("[sheet-keyboard] data-edge scan failed", e);
    }
    if (found) return found;
    if (direction === "up") return { row: 0, col: col };
    if (direction === "down") return { row: (g.totalRows || 1) - 1, col: col };
    if (direction === "left") return { row: row, col: 0 };
    return { row: row, col: (g.totalCols || 1) - 1 };
  }

  function pageStep(g, row) {
    var h = (typeof g.rowHeightOf === "function") ? g.rowHeightOf(row) : ROW_HEIGHT;
    return Math.max(1, Math.floor((g.viewportHeight || ROW_HEIGHT * 20) / (h || ROW_HEIGHT)) - 1);
  }

  function currentAnchor(g) {
    if (g.rangeAnchor && typeof g.rangeAnchor.row === "number") return g.rangeAnchor;
    return {
      row: g.selectedRow == null ? 0 : g.selectedRow,
      col: g.selectedCol == null ? 0 : g.selectedCol,
    };
  }

  function selectRange(g, r1, c1, r2, c2, anchor) {
    if (window.SheetAdvanced && window.SheetAdvanced.setRange) {
      window.SheetAdvanced.setRange(r1, c1, r2, c2);
    }
    g.rangeAnchor = anchor;
    g.selectCell(clampRow(g, r2), clampCol(g, c2));
    g.rangeAnchor = anchor;
  }

  function clearCells(g, row, col) {
    var sel = window.SheetAdvanced && window.SheetAdvanced.getSelection
      ? window.SheetAdvanced.getSelection() : null;
    var r1 = row, r2 = row, c1 = col, c2 = col;
    if (sel && typeof sel.startRow === "number") {
      r1 = sel.startRow; r2 = sel.endRow; c1 = sel.startCol; c2 = sel.endCol;
    }
    if ((r2 - r1 + 1) * (c2 - c1 + 1) > MAX_CLEAR_CELLS) {
      g.announce(D.a11yT("sheet-a11y-clear-too-large",
        "Selection too large to clear with the keyboard", null));
      return;
    }
    var jobs = [];
    for (var r = r1; r <= r2; r++) {
      for (var c = c1; c <= c2; c++) {
        var ref = D.colName(c) + (r + 1);
        var key = r + "," + c;
        g.cells.set(key, {});
        jobs.push(D.SheetAPI.updateCell(ref, "", g.instance || g));
        if (typeof g.onCellValueChanged === "function") g.onCellValueChanged(r, c, "");
      }
    }
    g.lastRenderedRange = null;
    g.requestRange();
    Promise.all(jobs).catch(function (err) {
      console.warn("[sheet-keyboard] clear cells failed", err);
    });
  }

  VirtualGrid.handleGridKeyDown = function (e, g) {
    var cell = e.target;
    var row = parseInt(cell.dataset.row, 10);
    var col = parseInt(cell.dataset.col, 10);
    if (isNaN(row) || isNaN(col)) return;

    var editing = g.editingCell === cell;
    var mod = e.ctrlKey || e.metaKey;
    var shift = e.shiftKey;
    var key = e.key;

    if (key === "Escape" && editing) {
      e.preventDefault();
      cell.textContent = cell.dataset.origVal || "";
      g.editingCell = null;
      cell.contentEditable = "false";
      cell.style.outline = "none";
      return;
    }
    if (editing) {
      // Commit on Enter/Tab as the original editor did: blur fires onCellBlur,
      // then move the selection. F2 is the explicit edit key, Escape cancels.
      if (key === "Enter" || key === "Tab") {
        e.preventDefault();
        cell.blur();
        if (key === "Tab") {
          if (shift && col > 0) g.selectCell(row, col - 1);
          else if (!shift && col + 1 < (g.totalCols || 1)) g.selectCell(row, col + 1);
        } else if (shift) {
          if (row > 0) g.selectCell(row - 1, col);
        } else if (row + 1 < (g.totalRows || 1)) {
          g.selectCell(row + 1, col);
        }
        return;
      }
      if (window.GBCollab && window.GBCollab.isConnected && window.GBCollab.isConnected()) {
        var ref = cell.dataset.ref || "";
        var p = D.parseCellRef(ref);
        if (p) window.GBCollab.debouncedTypingStart(p.row, p.col);
      }
      return;
    }

    if (key === "F2") {
      e.preventDefault();
      g.startEditingCell(cell);
      return;
    }
    if (key === "Delete" || key === "Backspace") {
      e.preventDefault();
      clearCells(g, row, col);
      return;
    }
    if (!mod && !e.altKey && key.length === 1) {
      e.preventDefault();
      g.startEditingCell(cell);
      cell.textContent = key;
      var range = document.createRange();
      range.selectNodeContents(cell);
      range.collapse(false);
      var sel = window.getSelection();
      sel.removeAllRanges();
      sel.addRange(range);
      return;
    }

    if (shift && !mod && key === " ") {
      e.preventDefault();
      var rowAnchor = { row: row, col: col };
      selectRange(g, row, 0, row, (g.totalCols || 1) - 1, rowAnchor);
      return;
    }
    if (mod && !shift && key === " ") {
      e.preventDefault();
      var colAnchor = { row: row, col: col };
      selectRange(g, 0, col, (g.totalRows || 1) - 1, col, colAnchor);
      return;
    }
    if (mod && shift && key.toLowerCase() === "a") {
      e.preventDefault();
      selectRange(g, 0, 0, (g.totalRows || 1) - 1, (g.totalCols || 1) - 1,
        { row: 0, col: 0 });
      return;
    }

    var target = null;
    var extension = false;
    switch (key) {
      case "ArrowUp":
        target = mod ? dataEdge(g, row, col, "up") : { row: row - 1, col: col };
        extension = mod && shift;
        break;
      case "ArrowDown":
        target = mod ? dataEdge(g, row, col, "down") : { row: row + 1, col: col };
        extension = mod && shift;
        break;
      case "ArrowLeft":
        target = mod ? dataEdge(g, row, col, "left") : { row: row, col: col - 1 };
        extension = mod && shift;
        break;
      case "ArrowRight":
        target = mod ? dataEdge(g, row, col, "right") : { row: row, col: col + 1 };
        extension = mod && shift;
        break;
      case "Home":
        target = mod ? { row: 0, col: 0 } : { row: row, col: 0 };
        break;
      case "End":
        target = mod
          ? dataEdge(g, (g.totalRows || 1) - 1, (g.totalCols || 1) - 1, "right")
          : dataEdge(g, row, (g.totalCols || 1) - 1, "left");
        break;
      case "PageUp":
        target = { row: row - pageStep(g, row), col: col };
        break;
      case "PageDown":
        target = { row: row + pageStep(g, row), col: col };
        break;
      case "Tab":
        if (mod) return;
        e.preventDefault();
        if (shift) {
          if (col > 0) g.selectCell(row, col - 1);
          else if (row > 0) g.selectCell(row - 1, (g.totalCols || 1) - 1);
          return;
        }
        if (col + 1 < (g.totalCols || 1)) g.selectCell(row, col + 1);
        else if (row + 1 < (g.totalRows || 1)) g.selectCell(row + 1, 0);
        return;
      case "Enter":
        if (mod) { e.preventDefault(); cell.blur(); return; }
        e.preventDefault();
        if (shift) {
          if (row > 0) g.selectCell(row - 1, col);
          return;
        }
        if (row + 1 < (g.totalRows || 1)) g.selectCell(row + 1, col);
        return;
      default:
        return;
    }

    if (!target) return;
    var nr = clampRow(g, target.row);
    var nc = clampCol(g, target.col);
    if (extension) {
      var anchor = currentAnchor(g);
      selectRange(g, anchor.row, anchor.col, nr, nc, anchor);
    } else {
      e.preventDefault();
      g.rangeAnchor = { row: nr, col: nc };
      g.selectCell(nr, nc);
    }
  };

  // Every ordinary selection (mouse, deep link, keyboard) resets the range
  // anchor; Ctrl+Shift+arrow restores its saved anchor after setRange().
  var originalSelectCell = VirtualGrid.selectCell;
  if (typeof originalSelectCell === "function") {
    VirtualGrid.selectCell = function (row, col, preventScroll) {
      this.rangeAnchor = { row: row, col: col };
      return originalSelectCell.call(this, row, col, preventScroll);
    };
  }

  // Keep the prototype copy used by per-window grids (#2270) in sync.
  if (D.GridProto) {
    D.GridProto.handleGridKeyDown = VirtualGrid.handleGridKeyDown;
    if (typeof VirtualGrid.selectCell === "function") D.GridProto.selectCell = VirtualGrid.selectCell;
  }
})();
