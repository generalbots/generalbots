"use strict";
/* sheet API client and worksheet index state */

(function () {
  var D = window.SheetDefaults;
  var parseCellRef = D.parseCellRef;

  const SheetAPI = {
    updateCell: function (ref, value) {
      const p = parseCellRef(ref);
      if (!p) return Promise.resolve(null);
      return fetch("/api/sheet/cell", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          sheet_id: D.currentSheetId(),
          worksheet_index: D.WORKSHEET_INDEX,
          row: p.row,
          col: p.col,
          value: value
        })
      }).then(function (r) { return r.json(); }).catch(function () { return null; });
    },

    evaluateFormula: function (formula) {
      return fetch("/api/sheet/formula", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          sheet_id: D.currentSheetId(),
          worksheet_index: D.WORKSHEET_INDEX,
          formula: formula
        })
      })
        .then(function (r) { return r.json(); })
        .then(function (j) { return j && j.value !== undefined ? j.value : (j && j.error ? "#ERR: " + j.error : ""); })
        .catch(function () { return "#ERR: network"; });
    },

    getRange: function (startRow, startCol, endRow, endCol) {
      return fetch("/api/sheet/range", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          sheet_id: D.currentSheetId(),
          worksheet_index: D.WORKSHEET_INDEX,
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

    addWorksheet: function () {
      return fetch("/api/sheet/worksheets/add", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: D.currentSheetId() })
      }).then(function (r) { return r.text(); });
    },

    switchWorksheet: function (index) {
      D.WORKSHEET_INDEX = index;
      return fetch("/api/sheet/worksheets/switch", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: D.currentSheetId(), index: index })
      }).then(function (r) { return r.text(); });
    },

    deleteWorksheet: function (index) {
      return fetch("/api/sheet/worksheets/delete", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: D.currentSheetId(), index: index })
      }).then(function (r) { return r.text(); });
    },

    renameWorksheet: function (index, name) {
      return fetch("/api/sheet/worksheets/rename", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id: D.currentSheetId(), index: index, name: name })
      }).then(function (r) { return r.json(); });
    }
  };

  window.addEventListener("gb-sheet-tab", function (e) {
    const idx = e && e.detail && typeof e.detail.index === "number" ? e.detail.index : 0;
    D.WORKSHEET_INDEX = idx;
    try { sessionStorage.setItem("sheet_active_index", String(idx)); } catch (_) {}
  });

  try {
    const saved = parseInt(sessionStorage.getItem("sheet_active_index") || "0", 10);
    if (!isNaN(saved) && saved >= 0) D.WORKSHEET_INDEX = saved;
  } catch (_) {}

  D.SheetAPI = SheetAPI;
})();
