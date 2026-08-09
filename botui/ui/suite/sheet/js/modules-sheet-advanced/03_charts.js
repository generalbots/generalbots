"use strict";
/* Sheet advanced module: 03_charts — chart insertion helpers */

(function () {
  function sheetId() {
    return window.__SHEET_INITIAL_ID || "current";
  }

  function wsIndex() {
    try {
      const v = parseInt(sessionStorage.getItem("sheet_active_index") || "0", 10);
      return isNaN(v) || v < 0 ? 0 : v;
    } catch (_) {
      return 0;
    }
  }

  function currentSelection() {
    const adv = window.SheetAdvanced;
    if (adv && adv.getSelection) return adv.getSelection();
    return null;
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

  function rangeRef(sel) {
    if (!sel) return null;
    return colName(sel.startCol) + (sel.startRow + 1) + ":" + colName(sel.endCol) + (sel.endRow + 1);
  }

  function insertChart(opts) {
    const sel = currentSelection();
    const dataRange = (opts && opts.data_range) || rangeRef(sel);
    if (!dataRange) return Promise.resolve(null);
    const payload = {
      sheet_id: sheetId(),
      worksheet_index: wsIndex(),
      chart_type: (opts && opts.chart_type) || "bar",
      data_range: dataRange,
      label_range: (opts && opts.label_range) || null,
      title: (opts && opts.title) || null,
    };
    return fetch("/api/sheet/chart", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    })
      .then(function (r) { return r.json(); })
      .then(function (j) { return j && j.chart ? j.chart : j; })
      .catch(function () { return null; });
  }

  function insertBar(title) {
    return insertChart({ chart_type: "bar", title: title || null });
  }

  function insertLine(title) {
    return insertChart({ chart_type: "line", title: title || null });
  }

  function insertPie(title) {
    return insertChart({ chart_type: "pie", title: title || null });
  }

  function insertColumn(title) {
    return insertChart({ chart_type: "column", title: title || null });
  }

  window.SheetCharts = {
    insertChart: insertChart,
    insertBar: insertBar,
    insertLine: insertLine,
    insertPie: insertPie,
    insertColumn: insertColumn,
    rangeRef: rangeRef,
  };
})();
