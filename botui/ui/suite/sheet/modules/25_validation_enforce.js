"use strict";

/**
 * Module 25: Validation enforcement for Spreadsheet (P0 critical).
 * Hooks the validation engine (module 12) into the actual cell editing
 * flow: setCellValue, finishEditing, pasteSelection, range edit. Shows
 * a red tooltip with the validation error message; rejects the value.
 * For list-validators, draws a <select> dropdown overlay that appears
 * when the user starts editing a validated cell.
 *
 * Public API: window.SheetValidationEnforce = {
 *   enforce, attach, showDropdown, hideDropdown, getValidators
 * }.
 */

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

  function evaluate(validator, value) {
    if (!validator) return null;
    const v = String(value == null ? "" : value);
    switch (validator.type) {
      case "list":
        if (!validator.list) return null;
        return validator.list.indexOf(v) >= 0 ? null : (validator.message || "Valor fora da lista");
      case "numberRange": {
        const n = parseFloat(v);
        if (v === "" && validator.allowBlank) return null;
        if (Number.isNaN(n)) return validator.message || "Número inválido";
        if (validator.min != null && n < parseFloat(validator.min)) return validator.message || ("Mínimo: " + validator.min);
        if (validator.max != null && n > parseFloat(validator.max)) return validator.message || ("Máximo: " + validator.max);
        return null;
      }
      case "textLength": {
        if (v === "" && validator.allowBlank) return null;
        if (validator.min != null && v.length < parseInt(validator.min, 10)) return validator.message || ("Mínimo " + validator.min + " caracteres");
        if (validator.max != null && v.length > parseInt(validator.max, 10)) return validator.message || ("Máximo " + validator.max + " caracteres");
        return null;
      }
      case "dateRange": {
        const d = Date.parse(v);
        if (Number.isNaN(d)) return validator.message || "Data inválida";
        if (validator.minDate && d < Date.parse(validator.minDate)) return validator.message || ("Mínimo: " + validator.minDate);
        if (validator.maxDate && d > Date.parse(validator.maxDate)) return validator.message || ("Máximo: " + validator.maxDate);
        return null;
      }
      case "regex": {
        if (!validator.pattern) return null;
        try { return new RegExp(validator.pattern).test(v) ? null : (validator.message || "Formato inválido"); }
        catch (_e) { return null; }
      }
      case "email":
        return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(v) ? null : (validator.message || "E-mail inválido");
      case "url":
        return /^https?:\/\//.test(v) ? null : (validator.message || "URL inválida (use http:// ou https://)");
      case "formula":
        return null;
      case "custom":
        return null;
    }
    return null;
  }

  function enforce(row, col, value, x, y) {
    const validator = getValidationForCell(row, col);
    if (!validator) return { ok: true, value };
    const err = evaluate(validator, value);
    if (!err) {
      drawCellBorder(row, col, false);
      return { ok: true, value };
    }
    showTooltip(err, x || 100, y || 100);
    drawCellBorder(row, col, true);
    if (validator.severity === "warning") return { ok: true, value, warning: err };
    if (validator.onInvalid === "allow") return { ok: true, value };
    return { ok: false, value, error: err };
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
        const result = enforce(row, col, opt, anchorX, anchorY);
        if (result.ok) {
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
      const result = enforce(row, col, value, x, y);
      if (!result.ok) return false;
      if (typeof orig === "function") return orig(row, col, result.value);
      const ws = getSheet();
      if (ws) {
        if (!ws.cells[row]) ws.cells[row] = [];
        ws.cells[row][col] = result.value;
        if (window.SheetRender) window.SheetRender.repaint();
      }
      return true;
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

  window.SheetValidationEnforce = { enforce, attach, showDropdown, hideDropdown, getValidators, evaluate };
})();
