"use strict";
/* #2172 — apply gb-doc-delta/v1 ops to the live grid. */
(function () {
  function apply(payload) {
    var VG = (window.SheetDefaults && window.SheetDefaults.virtualGrid) || window.VirtualGrid;
    var ops = payload.ops || [];
    ops.forEach(function (op) {
      if (!op) return;
      if (op.op === "set_rows" && Array.isArray(op.rows) && VG) {
        var start = op.start || 0;
        op.rows.forEach(function (row, i) {
          var r = start + i;
          (row || []).forEach(function (val, c) {
            var key = r + "," + c;
            if (VG.cells) VG.cells.set(key, Object.assign({}, VG.cells.get(key) || {}, { value: val == null ? "" : String(val) }));
          });
        });
        if (VG.lastRenderedRange != null) VG.lastRenderedRange = null;
        if (typeof VG.requestRange === "function") VG.requestRange();
      }
      if (op.op === "clear_rows" && VG && VG.cells) { VG.cells.clear(); if (typeof VG.requestRange === "function") VG.requestRange(); }
      if (op.op === "set_col_widths" && Array.isArray(op.widths) && window.SheetCore && window.SheetCore.refreshWidths) {
        // stash widths on loaded sheet and refresh
        var w = window.__LOADED_SHEET;
        if (w && w.worksheets && w.worksheets[0]) w.worksheets[0].column_widths = op.widths;
        window.SheetCore.refreshWidths();
      }
      if (op.op === "finalize" && window.GBToasts) {
        window.GBToasts.show("Sheet", "Generation complete", "success");
      }
    });
  }
  if (window.GBDocDelta) window.GBDocDelta.register("sheet", apply);
  else document.addEventListener("DOMContentLoaded", function () { if (window.GBDocDelta) window.GBDocDelta.register("sheet", apply); });
})();
