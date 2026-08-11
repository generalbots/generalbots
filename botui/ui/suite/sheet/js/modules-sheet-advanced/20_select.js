"use strict";
/* Sheet advanced module: 20_select — row/column/whole-sheet selection via header clicks */

(function () {
  function grid() {
    if (window.SheetCore && window.SheetCore.getGrid) return window.SheetCore.getGrid();
    return window.SheetVirtualGrid || null;
  }

  function selectRow(row) {
    const g = grid();
    if (!g) return;
    if (window.SheetAdvanced && window.SheetAdvanced.setRange) {
      window.SheetAdvanced.setRange(row, 0, row, g.totalCols - 1);
    }
  }

  function selectColumn(col) {
    const g = grid();
    if (!g) return;
    if (window.SheetAdvanced && window.SheetAdvanced.setRange) {
      window.SheetAdvanced.setRange(0, col, g.totalRows - 1, col);
    }
  }

  function selectAll() {
    const g = grid();
    if (!g) return;
    if (window.SheetAdvanced && window.SheetAdvanced.setRange) {
      window.SheetAdvanced.setRange(0, 0, g.totalRows - 1, g.totalCols - 1);
    }
  }

  function colIdx(name) {
    let n = 0;
    for (let i = 0; i < name.length; i++) n = n * 26 + (name.charCodeAt(i) - 64);
    return n - 1;
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

  function onHeaderClick(e) {
    const g = grid();
    const h = e.target;
    if (!g || !g.headerRow || !h || h.tagName !== "DIV") return;
    if (h.classList && (h.classList.contains("ss-sort-head") || h.classList.contains("ss-filter-head"))) return;
    // Column header cells contain only the letter(s) as textContent
    const text = h.textContent || "";
    if (!/^[A-Z]+$/.test(text)) return;
    const col = colIdx(text);
    if (col < 0 || col >= g.totalCols) return;
    e.preventDefault();
    if (e.ctrlKey || e.metaKey) {
      // Multi-range selection: Ctrl+click on a header adds that column (#786).
      if (window.SheetAdvanced && window.SheetAdvanced.addRange) {
        window.SheetAdvanced.addRange(0, col, g.totalRows - 1, col);
      }
      return;
    }
    if (e.shiftKey) return;
    selectColumn(col);
  }

  function onRowHeaderClick(e) {
    const g = grid();
    const t = e.target;
    if (!g || !t || !t.dataset || t.dataset.row === undefined) return;
    const row = parseInt(t.dataset.row, 10);
    if (isNaN(row)) return;
    if (e.ctrlKey || e.metaKey) {
      // Multi-range selection: Ctrl+click on a header adds that row (#786).
      if (window.SheetAdvanced && window.SheetAdvanced.addRange) {
        window.SheetAdvanced.addRange(row, 0, row, g.totalCols - 1);
      }
      return;
    }
    if (e.shiftKey) return;
    selectRow(row);
  }

  function wire() {
    const g = grid();
    if (!g) {
      setTimeout(wire, 100);
      return;
    }
    if (g.headerRow && !g.headerRow.__selBound) {
      g.headerRow.__selBound = true;
      g.headerRow.addEventListener("click", onHeaderClick, true);
    }
    if (g.bodyInner && !g.bodyInner.__rowSelBound) {
      g.bodyInner.__rowSelBound = true;
      g.bodyInner.addEventListener("click", onRowHeaderClick, true);
    }
  }

  window.SheetSelect = {
    selectRow: selectRow,
    selectColumn: selectColumn,
    selectAll: selectAll,
  };

  if (window.SheetCore) {
    window.SheetCore.selectColumn = selectColumn;
    window.SheetCore.selectRow = selectRow;
  }

  setTimeout(wire, 0);
})();