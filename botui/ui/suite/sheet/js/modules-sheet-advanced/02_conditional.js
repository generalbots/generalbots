"use strict";
/* Sheet advanced module: 02_conditional — conditional formatting quick actions */

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

  function applyRule(rule) {
    const sel = currentSelection();
    if (!sel) return Promise.resolve(null);
    const payload = {
      sheet_id: sheetId(),
      worksheet_index: wsIndex(),
      start_row: sel.startRow,
      start_col: sel.startCol,
      end_row: sel.endRow,
      end_col: sel.endCol,
      rule_type: rule.rule_type || "cellValue",
      condition: rule.condition || ">0",
      style: rule.style || { background: "#ffeb3b", color: "#000000" },
    };
    return fetch("/api/sheet/conditional-format", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    })
      .then(function (r) { return r.json(); })
      .then(function (j) { return j.success !== false ? { success: true } : j; })
      .catch(function () { return null; });
  }

  function highlightGreaterThan(value) {
    return applyRule({ condition: ">" + value, style: { background: "#4caf50", color: "#ffffff" } });
  }

  function highlightLessThan(value) {
    return applyRule({ condition: "<" + value, style: { background: "#f44336", color: "#ffffff" } });
  }

  function highlightBetween(v1, v2) {
    return applyRule({
      condition: v1 + ":" + v2,
      style: { background: "#2196f3", color: "#ffffff" },
    });
  }

  function highlightDuplicates() {
    return applyRule({
      rule_type: "duplicates",
      condition: "countif",
      style: { background: "#ff9800", color: "#000000" },
    });
  }

  function colorScale(minColor, maxColor) {
    return applyRule({
      rule_type: "colorScale",
      condition: "min:max",
      style: {
        background: minColor || "#fff3cd",
        color: "#000000",
        gradient_max: maxColor || "#c8e6c9",
      },
    });
  }

  window.SheetConditional = {
    applyRule: applyRule,
    highlightGreaterThan: highlightGreaterThan,
    highlightLessThan: highlightLessThan,
    highlightBetween: highlightBetween,
    highlightDuplicates: highlightDuplicates,
    colorScale: colorScale,
  };
})();
