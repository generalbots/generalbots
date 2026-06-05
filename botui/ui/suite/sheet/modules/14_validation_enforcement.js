"use strict";

/**
 * Module 14: Validation enforcement for Sheet.
 * Wraps setCellValue, finishEditing, pasteSelection, and clipboard handlers
 * to call validateCellValue() before accepting changes. Also displays
 * in-cell dropdown for list-type validation and error tooltips/messages.
 *
 * Uses IIFE + late binding via window.SheetValidation; if global hooks
 * are not yet defined when this module loads, it retries via setInterval.
 */

(function () {
  function showValidationError(row, col, message) {
    const cell = document.querySelector(
      `.cell[data-row="${row}"][data-col="${col}"]`
    );
    if (!cell) return;
    cell.classList.add("validation-error");
    cell.setAttribute("data-validation-message", message || "Invalid value");
    const tip = document.createElement("div");
    tip.className = "validation-tooltip";
    tip.textContent = message || "Invalid value";
    tip.style.position = "absolute";
    tip.style.background = "#fee";
    tip.style.color = "#900";
    tip.style.padding = "4px 8px";
    tip.style.borderRadius = "4px";
    tip.style.fontSize = "12px";
    tip.style.zIndex = "9999";
    tip.style.pointerEvents = "none";
    const rect = cell.getBoundingClientRect();
    tip.style.left = rect.left + "px";
    tip.style.top = rect.bottom + "px";
    document.body.appendChild(tip);
    setTimeout(function () {
      tip.remove();
    }, 2500);
    setTimeout(function () {
      cell.classList.remove("validation-error");
      cell.removeAttribute("data-validation-message");
    }, 3000);
  }

  function showInputTooltip(row, col, message) {
    if (!message) return;
    const cell = document.querySelector(
      `.cell[data-row="${row}"][data-col="${col}"]`
    );
    if (!cell) return;
    const tip = document.createElement("div");
    tip.className = "validation-input-tooltip";
    tip.textContent = message;
    tip.style.position = "absolute";
    tip.style.background = "#eef";
    tip.style.color = "#006";
    tip.style.padding = "4px 8px";
    tip.style.borderRadius = "4px";
    tip.style.fontSize = "12px";
    tip.style.zIndex = "9999";
    tip.style.pointerEvents = "none";
    const rect = cell.getBoundingClientRect();
    tip.style.left = rect.left + "px";
    tip.style.top = rect.top - 28 + "px";
    document.body.appendChild(tip);
    setTimeout(function () {
      tip.remove();
    }, 4000);
  }

  function listDropdown(row, col, source) {
    if (!source) return null;
    const cell = document.querySelector(
      `.cell[data-row="${row}"][data-col="${col}"]`
    );
    if (!cell) return null;
    const opts = Array.isArray(source)
      ? source
      : String(source).split(",").map(function (s) { return s.trim(); });
    const select = document.createElement("select");
    select.className = "cell-list-dropdown";
    select.style.position = "absolute";
    select.style.zIndex = "9999";
    select.style.fontSize = "13px";
    const rect = cell.getBoundingClientRect();
    select.style.left = rect.left + "px";
    select.style.top = rect.top + "px";
    select.style.width = rect.width + "px";
    select.style.height = rect.height + "px";
    const blank = document.createElement("option");
    blank.value = "";
    blank.textContent = "";
    select.appendChild(blank);
    for (const o of opts) {
      const opt = document.createElement("option");
      opt.value = String(o);
      opt.textContent = String(o);
      select.appendChild(opt);
    }
    document.body.appendChild(select);
    select.focus();
    return new Promise(function (resolve) {
      select.addEventListener("change", function () {
        const v = select.value;
        select.remove();
        resolve(v);
      });
      select.addEventListener("blur", function () {
        const v = select.value;
        select.remove();
        resolve(v);
      });
    });
  }

  function getValidationForCell(row, col) {
    if (!window.state) return null;
    const ws = window.state.worksheets && window.state.worksheets[window.state.activeWorksheet];
    if (!ws || !ws.validations) return null;
    return ws.validations.find(function (v) {
      const r = v.range || {};
      return row >= (r.startRow || 0) && row <= (r.endRow || 1e9)
        && col >= (r.startCol || 0) && col <= (r.endCol || 1e9);
    }) || null;
  }

  function checkCell(row, col, value) {
    const rule = getValidationForCell(row, col);
    if (!rule) return { valid: true };
    if (!window.SheetValidation) return { valid: true };
    const result = window.SheetValidation.buildValidator(rule)(value, {});
    if (result) return { valid: true };
    return {
      valid: false,
      message: rule.errorMessage || "Invalid value",
      rule: rule,
    };
  }

  function wrapSetCellValue() {
    if (typeof window.setCellValue !== "function" || window.setCellValue.__validationWrapped) return;
    const original = window.setCellValue;
    window.setCellValue = function (row, col, value, opts) {
      opts = opts || {};
      if (!opts.skipValidation) {
        const r = checkCell(row, col, value);
        if (!r.valid) {
          showValidationError(row, col, r.message);
          if (r.rule && r.rule.errorStyle === "stop") return;
        }
      }
      return original.call(this, row, col, value, opts);
    };
    window.setCellValue.__validationWrapped = true;
  }

  function wrapFinishEditing() {
    if (typeof window.finishEditing !== "function" || window.finishEditing.__validationWrapped) return;
    const original = window.finishEditing;
    window.finishEditing = function (save) {
      if (save !== false && window.state && window.state.activeCell) {
        const r = window.state.activeCell;
        const cell = document.querySelector(
          `.cell[data-row="${r.row}"][data-col="${r.col}"]`
        );
        const input = cell && cell.querySelector(".cell-input");
        if (input) {
          const v = input.value;
          const rule = getValidationForCell(r.row, r.col);
          if (rule) {
            if (rule.showInputMessage && rule.inputMessage) {
              showInputTooltip(r.row, r.col, rule.inputMessage);
            }
            const res = checkCell(r.row, r.col, v);
            if (!res.valid) {
              showValidationError(r.row, r.col, res.message);
              if (!rule.errorStyle || rule.errorStyle === "stop") {
                input.focus();
                return;
              }
            }
          }
        }
      }
      return original.call(this, save);
    };
    window.finishEditing.__validationWrapped = true;
  }

  function wrapPasteSelection() {
    if (typeof window.pasteSelection !== "function" || window.pasteSelection.__validationWrapped) return;
    const original = window.pasteSelection;
    window.pasteSelection = function () {
      if (window.state && window.state.selection) {
        const sel = window.state.selection;
        const startRow = sel.startRow || 0;
        const startCol = sel.startCol || 0;
        const clipboard = (navigator.clipboard && navigator.clipboard.readText && "") || "";
        const rows = String(clipboard).split(/\r?\n/);
        for (let r = 0; r < rows.length; r++) {
          const cells = rows[r].split(/\t/);
          for (let c = 0; c < cells.length; c++) {
            const absRow = startRow + r;
            const absCol = startCol + c;
            const res = checkCell(absRow, absCol, cells[c]);
            if (!res.valid && res.rule && (!res.rule.errorStyle || res.rule.errorStyle === "stop")) {
              showValidationError(absRow, absCol, res.message);
              return;
            }
          }
        }
      }
      return original.call(this);
    };
    window.pasteSelection.__validationWrapped = true;
  }

  function attachHooks() {
    if (typeof window.setCellValue === "function") wrapSetCellValue();
    if (typeof window.finishEditing === "function") wrapFinishEditing();
    if (typeof window.pasteSelection === "function") wrapPasteSelection();
  }

  function showInputMessagesOnSelect() {
    document.addEventListener("click", function (e) {
      const cell = e.target.closest && e.target.closest(".cell");
      if (!cell) return;
      const row = parseInt(cell.getAttribute("data-row"), 10);
      const col = parseInt(cell.getAttribute("data-col"), 10);
      if (isNaN(row) || isNaN(col)) return;
      const rule = getValidationForCell(row, col);
      if (rule && rule.showInputMessage && rule.inputMessage) {
        showInputTooltip(row, col, rule.inputMessage);
      }
    }, true);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      attachHooks();
      showInputMessagesOnSelect();
    });
  } else {
    attachHooks();
    showInputMessagesOnSelect();
  }

  let attempts = 0;
  const interval = setInterval(function () {
    attachHooks();
    attempts++;
    if (attempts > 30) clearInterval(interval);
  }, 200);

  window.SheetValidationEnforcement = {
    checkCell,
    showValidationError,
    showInputTooltip,
    listDropdown,
    attachHooks,
  };
})();
