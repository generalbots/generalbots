"use strict";

// botui/ui/suite/sheet/modules/25_validation_enforce.js
// Validation enforcement for Spreadsheet — SERVER-ONLY. All rule
// evaluation (list, numberRange, textLength, dateRange, regex, email,
// url, formula, custom) lives in botserver's /api/sheet/validate-cell
// handler. The client only renders UI: tooltip, red border, dropdown.
//
// Public API: window.SheetValidationEnforce = {
//   enforce, verify, attach, showDropdown, hideDropdown, getValidators
// }
(function () {
  function getState() { return window.state || null; }
  function getSheet() {
    const s = getState();
    if (!s) return null;
    return (s.worksheets || [])[s.currentSheet || 0];
  }
  function getValidationEngine() { return window.SheetValidation || null; }
  function getValidationForCell(row, col) {
    const engine = getValidationEngine();
    if (!engine || !engine.getValidationForRange) return null;
    const ref = ((window.SheetFormulaEngine || {}).indexToColName || function () { return ""; })(col) + (row + 1);
    return engine.getValidationForRange(ref);
  }
  function getSheetId() {
    const el = document.getElementById("sheetName");
    return (el && el.value) ? el.value : null;
  }
  function getCellRef(row, col) {
    return ((window.SheetFormulaEngine || {}).indexToColName || function () { return ""; })(col) + (row + 1);
  }

  function tooltip() {
    let t = document.getElementById("sheetValidationTooltip");
    if (t) return t;
    t = document.createElement("div");
    t.id = "sheetValidationTooltip";
    t.style.cssText = "position:fixed;background:#d93025;color:#fff;padding:6px 10px;border-radius:4px;font-size:12px;z-index:9999;display:none;box-shadow:0 2px 6px rgba(0,0,0,0.3);pointer-events:none;";
    document.body.appendChild(t);
    return t;
  }

  function showTooltip(message, x, y) {
    const t = tooltip();
    t.textContent = message;
    t.style.left = (x + 12) + "px";
    t.style.top = (y - 8) + "px";
    t.style.display = "block";
    clearTimeout(t._hideTimer);
    t._hideTimer = setTimeout(function () { t.style.display = "none"; }, 2500);
  }

  function hideTooltip() {
    const t = tooltip();
    t.style.display = "none";
  }

  function drawCellBorder(row, col, isError) {
    const editor = (window.state || {}).editor;
    if (!editor || !editor.cells) return;
    if (editor.cells[row] && editor.cells[row][col]) {
      editor.cells[row][col].style.outline = isError ? "2px solid #d93025" : "";
    }
  }

  function enforce(row, col, value, x, y) {
    const validator = getValidationForCell(row, col);
    if (!validator) return Promise.resolve({ ok: true, value });
    return verify(row, col, value).then(function (v) {
      if (!v) {
        return { ok: false, value, error: "Server unreachable; cannot validate" };
      }
      if (v.ok) {
        drawCellBorder(row, col, false);
        return { ok: true, value };
      }
      showTooltip(v.error || "Validation failed", x || 100, y || 100);
      drawCellBorder(row, col, true);
      if (validator.severity === "warning") return { ok: true, value, warning: v.error };
      if (validator.onInvalid === "allow") return { ok: true, value };
      return { ok: false, value, error: v.error };
    });
  }

  function verify(row, col, value) {
    const API = window.SheetAPI;
    const sheetId = getSheetId();
    if (!API || !sheetId) return Promise.resolve(null);
    const ref = getCellRef(row, col);
    return API.validateCell(sheetId, ref, value).then(function (r) {
      if (!r || !r.ok) return null;
      const data = r.data || {};
      return { ok: data.valid !== false, error: data.error || null, source: "server" };
    }).catch(function () { return null; });
  }

  function dropdownOverlay() {
    let d = document.getElementById("sheetValidationDropdown");
    if (d) return d;
    d = document.createElement("div");
    d.id = "sheetValidationDropdown";
    d.style.cssText = "position:absolute;background:#fff;border:1px solid #dadce0;border-radius:4px;box-shadow:0 4px 12px rgba(0,0,0,0.15);z-index:9998;display:none;min-width:160px;max-height:200px;overflow-y:auto;";
    document.body.appendChild(d);
    document.addEventListener("click", function (e) {
      if (!d.contains(e.target)) d.style.display = "none";
    });
    return d;
  }

  function showDropdown(row, col, anchorX, anchorY) {
    const validator = getValidationForCell(row, col);
    if (!validator || validator.type !== "list" || !validator.list || !validator.list.length) return false;
    const d = dropdownOverlay();
    d.innerHTML = "";
    for (const opt of validator.list) {
      const item = document.createElement("div");
      item.className = "vd-item";
      item.textContent = opt;
      item.style.cssText = "padding:6px 12px;cursor:pointer;font-size:13px;";
      item.addEventListener("mouseover", function () { item.style.background = "#f1f3f4"; });
      item.addEventListener("mouseout", function () { item.style.background = "#fff"; });
      item.addEventListener("click", function (e) {
        e.stopPropagation();
        d.style.display = "none";
        const editors = document.querySelectorAll("[data-editing-cell='1']");
        if (editors.length) editors[0].value = opt;
        enforce(row, col, opt, anchorX, anchorY).then(function (result) {
          if (result && result.ok) {
            if (typeof window.SheetCells === "object" && window.SheetCells.setValue) {
              window.SheetCells.setValue(row, col, opt);
            } else {
              const ws = getSheet();
              if (ws) {
                if (!ws.cells[row]) ws.cells[row] = [];
                ws.cells[row][col] = opt;
                if (window.SheetRender) window.SheetRender.repaint();
              }
            }
          }
        });
      });
      d.appendChild(item);
    }
    d.style.left = anchorX + "px";
    d.style.top = (anchorY + 24) + "px";
    d.style.display = "block";
    return true;
  }

  function hideDropdown() {
    const d = document.getElementById("sheetValidationDropdown");
    if (d) d.style.display = "none";
  }

  function attach() {
    const orig = window.setCellValue;
    window.setCellValue = function (row, col, value, x, y) {
      return enforce(row, col, value, x, y).then(function (result) {
        if (!result || !result.ok) return false;
        if (typeof orig === "function") return orig(row, col, result.value);
        const ws = getSheet();
        if (ws) {
          if (!ws.cells[row]) ws.cells[row] = [];
          ws.cells[row][col] = result.value;
          if (window.SheetRender) window.SheetRender.repaint();
        }
        return true;
      });
    };

    document.addEventListener("dblclick", function (e) {
      const cell = e.target.closest && e.target.closest("[data-row][data-col]");
      if (!cell) return;
      const row = parseInt(cell.dataset.row, 10);
      const col = parseInt(cell.dataset.col, 10);
      const rect = cell.getBoundingClientRect();
      setTimeout(function () { showDropdown(row, col, rect.left, rect.bottom); }, 80);
    });
  }

  function getValidators() {
    const engine = getValidationEngine();
    return engine && engine.getAll ? engine.getAll() : [];
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", attach);
  } else {
    setTimeout(attach, 50);
  }

  window.SheetValidationEnforce = { enforce, verify, attach, showDropdown, hideDropdown, getValidators };
})();
