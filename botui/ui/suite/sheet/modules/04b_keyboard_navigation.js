"use strict";

  function handleKeyDown(e) {
    if (e.target.closest(".chat-input, .modal input, .sheet-name-input"))
      return;

    const { row, col } = state.activeCell;

    if (e.ctrlKey || e.metaKey) {
      switch (e.key.toLowerCase()) {
        case "c":
          copySelection();
          return;
        case "x":
          cutSelection();
          return;
        case "v":
          pasteSelection();
          return;
        case "z":
          e.shiftKey ? redo() : undo();
          e.preventDefault();
          return;
        case "y":
          redo();
          e.preventDefault();
          return;
        case "b":
          formatCells("bold");
          e.preventDefault();
          return;
        case "i":
          formatCells("italic");
          e.preventDefault();
          return;
        case "u":
          formatCells("underline");
          e.preventDefault();
          return;
        case "a":
          selectAll();
          e.preventDefault();
          return;
      }
    }

    if (state.isEditing) return;

    switch (e.key) {
      case "ArrowUp":
        navigateCell(-1, 0);
        e.preventDefault();
        break;
      case "ArrowDown":
        navigateCell(1, 0);
        e.preventDefault();
        break;
      case "ArrowLeft":
        navigateCell(0, -1);
        e.preventDefault();
        break;
      case "ArrowRight":
        navigateCell(0, 1);
        e.preventDefault();
        break;
      case "Tab":
        navigateCell(0, e.shiftKey ? -1 : 1);
        e.preventDefault();
        break;
      case "Enter":
        if (e.shiftKey) {
          navigateCell(-1, 0);
        } else {
          startEditing(row, col);
        }
        e.preventDefault();
        break;
      case "Delete":
      case "Backspace":
        clearCells();
        e.preventDefault();
        break;
      case "F2":
        startEditing(row, col);
        e.preventDefault();
        break;
      default:
        if (e.key.length === 1 && !e.ctrlKey && !e.metaKey && !e.altKey) {
          startEditing(row, col);
          const cell = elements.cells.querySelector(
            `[data-row="${row}"][data-col="${col}"]`,
          );
          const input = cell?.querySelector(".cell-input");
          if (input) input.value = e.key;
        }
    }
  }

  function navigateCell(dRow, dCol) {
    const newRow = Math.max(
      0,
      Math.min(CONFIG.ROWS - 1, state.activeCell.row + dRow),
    );
    const newCol = Math.max(
      0,
      Math.min(CONFIG.COLS - 1, state.activeCell.col + dCol),
    );
    selectCell(newRow, newCol);
  }

  function selectAll() {
    clearSelection();
    state.selection = {
      start: { row: 0, col: 0 },
      end: { row: CONFIG.ROWS - 1, col: CONFIG.COLS - 1 },
    };

    elements.cells.querySelectorAll(".cell").forEach((cell) => {
      cell.classList.add("in-range");
    });

    const activeCell = elements.cells.querySelector(
      `[data-row="${state.activeCell.row}"][data-col="${state.activeCell.col}"]`,
    );
    if (activeCell) {
      activeCell.classList.remove("in-range");
      activeCell.classList.add("selected");
    }

    updateSelectionInfo();
  }

  function handleFormulaKey(e) {
    if (e.key === "Enter") {
      e.preventDefault();
      const value = elements.formulaInput.value;
      const { row, col } = state.activeCell;
      setCellValue(row, col, value);
      renderCell(row, col);
      elements.formulaInput.blur();
    } else if (e.key === "Escape") {
      updateFormulaBar();
      elements.formulaInput.blur();
    }
  }

  function updateFormulaPreview() {
    const value = elements.formulaInput.value;
    if (value.startsWith("=")) {
      const result = window.evaluateFormula(
        value,
        state.activeCell.row,
        state.activeCell.col,
      );
      elements.calculationResult.textContent = `= ${result}`;
    } else {
      elements.calculationResult.textContent = "";
    }
  }

  function updateCellAddress() {
    const ref = getCellRef(state.activeCell.row, state.activeCell.col);
    elements.cellAddress.textContent = ref;
  }

  function updateFormulaBar() {
    const data = getCellData(state.activeCell.row, state.activeCell.col);
    elements.formulaInput.value = data?.formula || data?.value || "";
  }

  function updateSelectionInfo() {
    const { start, end } = state.selection;
    const rows = end.row - start.row + 1;
    const cols = end.col - start.col + 1;
    const count = rows * cols;
    if (count === 1) {
      elements.selectionInfo.textContent = "Ready";
    } else {
      elements.selectionInfo.textContent = `${rows}R × ${cols}C = ${count} cells`;
    }
  }

  function updateCalculationResult() {
    const { start, end } = state.selection;
    const values = [];
    for (let r = start.row; r <= end.row; r++) {
      for (let c = start.col; c <= end.col; c++) {
        const val = parseFloat(getCellValue(r, c));
        if (!isNaN(val)) values.push(val);
      }
    }
    if (values.length > 1) {
      const sum = values.reduce((a, b) => a + b, 0);
      const avg = sum / values.length;
      elements.calculationResult.textContent = `Sum: ${sum.toFixed(2)} | Avg: ${avg.toFixed(2)} | Count: ${values.length}`;
    } else {
      elements.calculationResult.textContent = "";
    }
  }
