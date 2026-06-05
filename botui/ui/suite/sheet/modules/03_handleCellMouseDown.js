// sheet/modules/03_handleCellMouseDown.js
"use strict";

// Functions: handleCellMouseDown, handleMouseMove, handleMouseUp, handleCellDoubleClick, selectCell, highlightVirtualCell, extendSelection, clearSelection, handleColumnHeaderClick, handleRowHeaderClick, startEditing, finishEditing, cancelEditing, setCellValue, getCellData, getCellValue, evaluateFormula, evaluateSum, evaluateAverage, evaluateCount, evaluateMax

    document
      .getElementById("printScale")
      ?.addEventListener("change", updatePrintPreview);
    document
      .getElementById("printGridlines")
      ?.addEventListener("change", updatePrintPreview);
    document
      .getElementById("printHeaders")
      ?.addEventListener("change", updatePrintPreview);

    document
      .getElementById("insertChartBtn")
      ?.addEventListener("click", () => showModal("chartModal"));
    document
      .getElementById("insertChartBtnConfirm")
      ?.addEventListener("click", insertChart);
    document
      .getElementById("cancelChartBtn")
      ?.addEventListener("click", () => hideModal("chartModal"));

    document
      .getElementById("insertImageBtn")
      ?.addEventListener("click", showInsertImageModal);
    document
      .getElementById("closeInsertImageModal")
      ?.addEventListener("click", () => hideModal("insertImageModal"));
    document
      .getElementById("insertImgBtn")
      ?.addEventListener("click", insertImage);
    document
      .getElementById("cancelImgBtn")
      ?.addEventListener("click", () => hideModal("insertImageModal"));
    document.querySelectorAll(".img-tab").forEach((tab) => {
      tab.addEventListener("click", () => switchImgTab(tab.dataset.tab));
    });

    document
      .getElementById("filterBtn")
      ?.addEventListener("click", toggleFilter);
    document
      .getElementById("sortAscBtn")
      ?.addEventListener("click", sortAscending);
    document
      .getElementById("sortDescBtn")
      ?.addEventListener("click", sortDescending);

    document
      .getElementById("closeCustomFormatModal")
      ?.addEventListener("click", () => hideModal("customNumberFormatModal"));
    document
      .getElementById("applyCnfBtn")
      ?.addEventListener("click", applyCustomNumberFormat);
    document
      .getElementById("cancelCnfBtn")
      ?.addEventListener("click", () => hideModal("customNumberFormatModal"));
    document.querySelectorAll(".cnf-format-item").forEach((item) => {
      item.addEventListener("click", () =>
        selectCustomFormat(item.dataset.format),
      );
    });
    document
      .getElementById("cnfFormatCode")
      ?.addEventListener("input", updateCnfPreview);



    document.querySelectorAll(".context-item").forEach((item) => {
      item.addEventListener("click", () =>
        handleContextAction(item.dataset.action),
      );
    });

    elements.sheetName?.addEventListener("change", (e) => {
      state.sheetName = e.target.value;
      scheduleAutoSave();
    });

    window.addEventListener("beforeunload", handleBeforeUnload);
  }

  function handleCellMouseDown(e) {
    const cell = e.target.closest(".cell");
    if (!cell) return;

    const row = parseInt(cell.dataset.row);
    const col = parseInt(cell.dataset.col);

    if (state.isEditing) {
      finishEditing();
    }

    if (e.shiftKey) {
      extendSelection(row, col);
    } else {
      selectCell(row, col);
      state.isSelecting = true;
    }
  }

  function handleMouseMove(e) {
    if (!state.isSelecting) return;

    const cell = document
      .elementFromPoint(e.clientX, e.clientY)
      ?.closest(".cell");
    if (cell) {
      const row = parseInt(cell.dataset.row);
      const col = parseInt(cell.dataset.col);
      extendSelection(row, col);
    }
  }

  function handleMouseUp() {
    state.isSelecting = false;
  }

  function handleCellDoubleClick(e) {
    const cell = e.target.closest(".cell");
    if (!cell) return;

    const row = parseInt(cell.dataset.row);
    const col = parseInt(cell.dataset.col);
    startEditing(row, col);
  }

  function selectCell(row, col) {
    clearSelection();

    state.activeCell = { row, col };
    state.selection = {
      start: { row, col },
      end: { row, col },
    };

    if (useVirtualScroll && virtualGrid) {
      virtualGrid.scrollToCell(row, col);
      setTimeout(() => highlightVirtualCell(row, col), 50);
    } else {
      const cell = elements.cells.querySelector(
        `[data-row="${row}"][data-col="${col}"]`,
      );
      if (cell) {
        cell.classList.add("selected");
        cell.scrollIntoView({ block: "nearest", inline: "nearest" });
      }
    }

    updateCellAddress();
    updateFormulaBar();
    updateSelectionInfo();
  }

  function highlightVirtualCell(row, col) {
    const cell = elements.cells.querySelector(`[data-row="${row}"][data-col="${col}"]`);
    if (cell && !cell.classList.contains('selected')) {
      clearSelection();
      cell.classList.add('selected');
    }
  }

  function extendSelection(row, col) {
    clearSelection();

    const start = state.activeCell;
    state.selection = {
      start: {
        row: Math.min(start.row, row),
        col: Math.min(start.col, col),
      },
      end: {
        row: Math.max(start.row, row),
        col: Math.max(start.col, col),
      },
    };

    for (let r = state.selection.start.row; r <= state.selection.end.row; r++) {
      for (
        let c = state.selection.start.col;
        c <= state.selection.end.col;
        c++
      ) {
        const cell = elements.cells.querySelector(
          `[data-row="${r}"][data-col="${c}"]`,
        );
        if (cell) {
          if (r === state.activeCell.row && c === state.activeCell.col) {
            cell.classList.add("selected");
          } else {
            cell.classList.add("in-range");
          }
        }
      }
    }

    updateSelectionInfo();
    updateCalculationResult();
  }

  function clearSelection() {
    elements.cells
      .querySelectorAll(".cell.selected, .cell.in-range")
      .forEach((cell) => {
        cell.classList.remove("selected", "in-range");
      });
  }

  function handleColumnHeaderClick(e) {
    const header = e.target.closest(".column-header");
    if (!header) return;

    const col = parseInt(header.dataset.col);
    clearSelection();

    state.activeCell = { row: 0, col };
    state.selection = {
      start: { row: 0, col },
      end: { row: CONFIG.ROWS - 1, col },
    };

    for (let row = 0; row < CONFIG.ROWS; row++) {
      const cell = elements.cells.querySelector(
        `[data-row="${row}"][data-col="${col}"]`,
      );
      if (cell) cell.classList.add("in-range");
    }

    header.classList.add("selected");
    updateSelectionInfo();
  }

  function handleRowHeaderClick(e) {
    const header = e.target.closest(".row-header");
    if (!header) return;

    const row = parseInt(header.dataset.row);
    clearSelection();

    state.activeCell = { row, col: 0 };
    state.selection = {
      start: { row, col: 0 },
      end: { row, col: CONFIG.COLS - 1 },
    };

    for (let col = 0; col < CONFIG.COLS; col++) {
      const cell = elements.cells.querySelector(
        `[data-row="${row}"][data-col="${col}"]`,
      );
      if (cell) cell.classList.add("in-range");
    }

    header.classList.add("selected");
    updateSelectionInfo();
  }

  function startEditing(row, col) {
    const cell = elements.cells.querySelector(
      `[data-row="${row}"][data-col="${col}"]`,
    );
    if (!cell) return;

    state.isEditing = true;
    const data = getCellData(row, col);
    if (window.SheetValidationEnforcement) {
      const rule = window.SheetValidationEnforcement.checkCell(row, col, "").rule;
      if (rule && rule.showInputMessage && rule.inputMessage) {
        window.SheetValidationEnforcement.showInputTooltip(row, col, rule.inputMessage);
      }
    }
    const input = document.createElement("input");
    input.type = "text";
    input.className = "cell-input";
    input.value = data?.formula || data?.value || "";
    cell.textContent = "";
    cell.classList.add("editing");
    cell.appendChild(input);
    input.focus();
    input.select();

    input.addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        finishEditing(true);
        navigateCell(1, 0);
      } else if (e.key === "Tab") {
        e.preventDefault();
        finishEditing(true);
        navigateCell(0, e.shiftKey ? -1 : 1);
      } else if (e.key === "Escape") {
        cancelEditing();
      }
    });

    input.addEventListener("blur", () => {
      if (state.isEditing) finishEditing(true);
    });
  }

  function finishEditing(save = true) {
    if (!state.isEditing) return;

    const { row, col } = state.activeCell;
    const cell = elements.cells.querySelector(
      `[data-row="${row}"][data-col="${col}"]`,
    );
    const input = cell?.querySelector(".cell-input");

    if (input && save) {
      const value = input.value.trim();
      setCellValue(row, col, value);
    }

    state.isEditing = false;
    cell?.classList.remove("editing");
    renderCell(row, col);
    updateFormulaBar();
  }

  function cancelEditing() {
    state.isEditing = false;
    const { row, col } = state.activeCell;
    const cell = elements.cells.querySelector(
      `[data-row="${row}"][data-col="${col}"]`,
    );
    cell?.classList.remove("editing");
    renderCell(row, col);
  }

  function setCellValue(row, col, value) {
    if (!permissions.canEdit()) {
      addChatMessage("system", "You don't have permission to edit this sheet");
      return;
    }
    if (window.SheetValidation && window.SheetValidationEnforcement) {
      const r = window.SheetValidationEnforcement.checkCell(row, col, value);
      if (!r.valid) {
        window.SheetValidationEnforcement.showValidationError(row, col, r.message);
        if (!r.rule || !r.rule.errorStyle || r.rule.errorStyle === "stop") return;
      }
    }
    const ws = state.worksheets[state.activeWorksheet];
    const key = `${row},${col}`;
    const oldValue = ws.data[key]?.value || ws.data[key]?.formula || '';
    saveToHistory();

    if (!value) {
      delete ws.data[key];
    } else if (value.startsWith("=")) {
      ws.data[key] = { formula: value };
    } else {
      ws.data[key] = { value };
    }

    if (useVirtualScroll && virtualGrid) {
      virtualGrid.setCellValue(row, col, ws.data[key]);
    }

    auditLog.log('cell_change', { row, col, oldValue, newValue: value, cellRef: getCellRef(row, col) });
    
    state.isDirty = true;
    scheduleAutoSave();
    broadcastChange("cell", { row, col, value });
  }

  function getCellData(row, col) {
    const ws = state.worksheets[state.activeWorksheet];
    return ws?.data[`${row},${col}`];
  }

  function getCellValue(row, col) {
    const data = getCellData(row, col);
    if (!data) return "";
    if (data.formula) return evaluateFormula(data.formula, row, col);
    return data.value || "";
  }

  function evaluateFormula(formula, sourceRow, sourceCol) {
    if (!formula.startsWith("=")) return formula;

    try {
      let expr = formula.substring(1).toUpperCase();

      expr = expr.replace(/([A-Z]+)(\d+)/g, (match, col, row) => {
        const r = parseInt(row) - 1;
        const c = parseColName(col);
        const val = getCellValue(r, c);
        const num = parseFloat(val);
        return isNaN(num) ? `"${val}"` : num;
      });

      if (expr.startsWith("SUM(")) {
        return evaluateSum(expr);
      } else if (expr.startsWith("AVERAGE(")) {
        return evaluateAverage(expr);
      } else if (expr.startsWith("COUNT(")) {
        return evaluateCount(expr);
      } else if (expr.startsWith("MAX(")) {
        return evaluateMax(expr);
      } else if (expr.startsWith("MIN(")) {
        return evaluateMin(expr);
      } else if (expr.startsWith("IF(")) {
        return evaluateIf(expr);
      }

      const result = safeEvalArithmetic(expr);
      return typeof result === "number"
        ? Math.round(result * 1000000) / 1000000
        : result;
    } catch (e) {
      return "#ERROR";
    }
  }

  function evaluateSum(expr) {
    const match = expr.match(/SUM\(([^)]+)\)/i);
    if (!match) return "#ERROR";
    const values = parseRange(match[1]);
    return values.reduce((a, b) => a + b, 0);
  }

  function evaluateAverage(expr) {
    const match = expr.match(/AVERAGE\(([^)]+)\)/i);
    if (!match) return "#ERROR";
    const values = parseRange(match[1]);
    return values.length
      ? values.reduce((a, b) => a + b, 0) / values.length
      : 0;
  }

  function evaluateCount(expr) {
    const match = expr.match(/COUNT\(([^)]+)\)/i);
    if (!match) return "#ERROR";
    const values = parseRange(match[1]);
    return values.length;
  }

  function evaluateMax(expr) {
    const match = expr.match(/MAX\(([^)]+)\)/i);
    if (!match) return "#ERROR";
    const values = parseRange(match[1]);
    return values.length ? Math.max(...values) : 0;
  }

  window.setCellValue = setCellValue;
  window.finishEditing = finishEditing;
  window.startEditing = startEditing;
  window.evaluateFormula = evaluateFormula;