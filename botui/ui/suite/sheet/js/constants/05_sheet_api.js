"use strict";
/* sheet API client and worksheet index state */

(function () {
  var D = window.SheetDefaults;
  var parseCellRef = D.parseCellRef;

  function ctxOf(ctx) {
    if (ctx && typeof ctx === "object" && (ctx.sheetId || ctx.windowId || ctx.host || ctx.body)) return ctx;
    return D.getActiveSheetInstance ? D.getActiveSheetInstance() : null;
  }
  function sheetIdFor(ctx) {
    if (ctx && ctx.sheetId) return ctx.sheetId;
    var inst = ctxOf(ctx);
    if (inst) {
      if (inst.loadedSheet && inst.loadedSheet.id) return inst.loadedSheet.id;
      if (inst.sheetId) return inst.sheetId;
    }
    return D.currentSheetId(ctx || inst);
  }
  function wsIndexFor(ctx) {
    if (ctx && typeof ctx.worksheetIndex === "number") return ctx.worksheetIndex;
    var inst = ctxOf(ctx);
    if (inst && typeof inst.worksheetIndex === "number") return inst.worksheetIndex;
    return D.WORKSHEET_INDEX;
  }

  const SheetAPI = {
    updateCell: function (ref, value, ctx) {
      const p = parseCellRef(ref);
      if (!p) return Promise.resolve(null);
      return fetch("/api/sheet/cell", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          sheet_id: sheetIdFor(ctx),
          worksheet_index: wsIndexFor(ctx),
          row: p.row,
          col: p.col,
          value: value
        })
      }).then(function (r) { return r.json(); }).catch(function () { return null; });
    },

    evaluateFormula: function (formula, ctx) {
      return fetch("/api/sheet/formula", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          sheet_id: sheetIdFor(ctx),
          worksheet_index: wsIndexFor(ctx),
          formula: formula
        })
      })
        .then(function (r) { return r.json(); })
        .then(function (j) { return j && j.value !== undefined ? j.value : (j && j.error ? "#ERR: " + j.error : ""); })
        .catch(function () { return "#ERR: network"; });
    },

    getRange: function (startRow, startCol, endRow, endCol, ctx) {
      return fetch("/api/sheet/range", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          sheet_id: sheetIdFor(ctx),
          worksheet_index: wsIndexFor(ctx),
          start_row: startRow,
          start_col: startCol,
          end_row: endRow,
          end_col: endCol
        })
      }).then(function (r) { return r.json(); }).catch(function () { return { cells: {} }; });
    },

    load: function (id) {
      const url = "/api/sheet/load" + (id ? "?id=" + encodeURIComponent(id) : "");
      return fetch(url, { method: "GET" })
        .then(function (r) { return r.json(); }).catch(function () { return null; });
    },

    addWorksheet: function (ctx) {
      return fetch("/api/sheet/worksheets/add", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: sheetIdFor(ctx) })
      }).then(function (r) { return r.text(); });
    },

    switchWorksheet: function (index, ctx) {
      D.WORKSHEET_INDEX = index;
      var inst = ctxOf(ctx);
      if (inst) inst.worksheetIndex = index;
      return fetch("/api/sheet/worksheets/switch", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: sheetIdFor(ctx), index: index })
      }).then(function (r) { return r.text(); });
    },

    deleteWorksheet: function (index, ctx) {
      return fetch("/api/sheet/worksheets/delete", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: sheetIdFor(ctx), index: index })
      }).then(function (r) { return r.text(); });
    },

    renameWorksheet: function (index, name, ctx) {
      return fetch("/api/sheet/worksheets/rename", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: sheetIdFor(ctx), index: index, name: name })
      }).then(function (r) { return r.json(); });
    }
  };

  window.addEventListener("gb-sheet-tab", function (e) {
    const idx = e && e.detail && typeof e.detail.index === "number" ? e.detail.index : 0;
    D.WORKSHEET_INDEX = idx;
    var inst = D.getActiveSheetInstance ? D.getActiveSheetInstance() : null;
    if (inst) inst.worksheetIndex = idx;
    try { sessionStorage.setItem("sheet_active_index", String(idx)); } catch (_) {}
  });

  try {
    const saved = parseInt(sessionStorage.getItem("sheet_active_index") || "0", 10);
    if (!isNaN(saved) && saved >= 0) D.WORKSHEET_INDEX = saved;
  } catch (_) {}

  D.SheetAPI = SheetAPI;
})();
