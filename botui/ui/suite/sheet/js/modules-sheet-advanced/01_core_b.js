"use strict";
/* Split from 01_core.js for #2305 (450-line limit): sync/init and exports. */
(function () {
  var _I = window.__SheetCore01 || {};
  var wire = _I.wire;
  var setSelection = _I.setSelection;
  var clearSelection = _I.clearSelection;
  var addMultiRange = _I.addMultiRange;
  var currentSheetId = _I.currentSheetId;
  var colName = _I.colName;
  var cellValue = _I.cellValue;
  var api = _I.api;
  var wsIndex = _I.wsIndex;
  var filterColumns = _I.filterColumns;

  function sync() {
    wire();
  }

  function init(host, opts) {
    hostEl = host;
    if (opts && opts.sheetId) window.__SHEET_INITIAL_ID = opts.sheetId;
    setTimeout(wire, 0);
    return {
      getSelection: function () { return sel; },
      sync: sync,
    };
  }

  if (window.SheetCore) {
    window.SheetCore.setSelection = setSelection;
    window.SheetCore.getGrid = function () { return grid; };
    window.SheetCore.setGrid = function (g) { grid = g; };
    window.SheetCore.getHost = function () { return hostEl; };
    window.SheetCore.setHost = function (h) { hostEl = h; };
    window.SheetCore.getTabBar = function () { return tabBar; };
    window.SheetCore.setTabBar = function (t) { tabBar = t; };
    window.SheetCore.wsIndex = wsIndex;
    window.SheetCore.filterColumns = filterColumns;
    window.SheetCore.api = api;
    window.SheetCore.currentSheetId = currentSheetId;
    window.SheetCore.colName = colName;
    window.SheetCore.cellValue = cellValue;
    window.SheetCore.setRange = setSelection;
    window.SheetCore.refreshGrid = function () {
      if (grid) {
        grid.lastRenderedRange = null;
        grid.requestRange();
      }
    };
  }

  window.SheetAdvanced = {
    init: init,
    sync: sync,
    getSelection: function () { return sel; },
    setRange: function (r1, c1, r2, c2) {
      setSelection(r1, c1, r2, c2);
      if (!grid) return;
      // A full-row or full-column selection must not force-scroll to the far
      // corner (~1M index) — that is a surprising, disorienting jump. Only
      // bring bounded (small) ranges into view.
      const nr = Math.max(r1, r2), nc = Math.max(c1, c2);
      const fullRow = r1 === 0 && r2 === grid.totalRows - 1;
      const fullCol = c1 === 0 && c2 === grid.totalCols - 1;
      const sparse = fullRow || fullCol;
      grid.selectCell(nr, nc, sparse);
    },
    clearSelection: function () {
      clearSelection();
    },
    getRanges: function () {
      return multiRanges.slice();
    },
    addRange: function (r1, c1, r2, c2) {
      addMultiRange(r1, c1, r2, c2);
      document.dispatchEvent(new CustomEvent("gb-sheet-multirange", { detail: multiRanges.slice(), bubbles: true }));
    },
  };
})();