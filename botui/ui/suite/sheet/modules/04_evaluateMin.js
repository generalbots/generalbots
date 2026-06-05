
"use strict";

  function evaluateMin(expr) {
    const match = expr.match(/MIN\(([^)]+)\)/i);
    if (!match) return "#ERROR";
    const values = parseRange(match[1]);
    return values.length ? Math.min(...values) : 0;
  }

  function safeEvalArithmetic(expr) {
    expr = expr.trim();
    if (/[^0-9+\-*/().%\s<>=!&|]/.test(expr)) return "#ERROR";
    const tokens = expr.match(/(\d+\.?\d*|[+\-*/().%<>=!&|]+)/g);
    if (!tokens) return "#ERROR";
    function evalTokens(tokens) {
      const values = [];
      const ops = [];
      const prec = { "+": 1, "-": 1, "*": 2, "/": 2, "%": 2, "<": 0, ">": 0, "<=": 0, ">=": 0, "==": 0, "!=": 0 };
      function applyOp() {
        const op = ops.pop();
        const b = values.pop();
        const a = values.pop();
        switch (op) {
          case "+": values.push(a + b); break;
          case "-": values.push(a - b); break;
          case "*": values.push(a * b); break;
          case "/": values.push(b === 0 ? "#DIV/0!" : a / b); break;
          case "%": values.push(b === 0 ? "#DIV/0!" : a % b); break;
          case "<": values.push(a < b ? 1 : 0); break;
          case ">": values.push(a > b ? 1 : 0); break;
          case "<=": values.push(a <= b ? 1 : 0); break;
          case ">=": values.push(a >= b ? 1 : 0); break;
          case "==": values.push(a === b ? 1 : 0); break;
          case "!=": values.push(a !== b ? 1 : 0); break;
        }
      }
      for (let i = 0; i < tokens.length; i++) {
        const t = tokens[i];
        if (t === "(") { ops.push(t); }
        else if (t === ")") { while (ops.length && ops[ops.length - 1] !== "(") applyOp(); ops.pop(); }
        else if (t in prec) {
          while (ops.length && ops[ops.length - 1] !== "(" && prec[ops[ops.length - 1]] >= prec[t]) applyOp();
          ops.push(t);
        } else { values.push(parseFloat(t) || 0); }
      }
      while (ops.length) applyOp();
      return values[0];
    }
    return evalTokens(tokens);
  }

  function safeEvalCondition(expr) {
    expr = expr.trim();
    const m = expr.match(/^(.+?)\s*(>=|<=|!=|>|<|==)\s*(.+)$/);
    if (m) {
      const a = safeEvalArithmetic(m[1]);
      const b = safeEvalArithmetic(m[3]);
      if (typeof a === "string" && a.startsWith("#")) return false;
      if (typeof b === "string" && b.startsWith("#")) return false;
      switch (m[2]) {
        case ">": return a > b;
        case "<": return a < b;
        case ">=": return a >= b;
        case "<=": return a <= b;
        case "==": return a === b;
        case "!=": return a !== b;
      }
    }
    return !!safeEvalArithmetic(expr);
  }

  function evaluateIf(expr) {
    const match = expr.match(/IF\(([^,]+),([^,]+),([^)]+)\)/i);
    if (!match) return "#ERROR";
    try {
      const condition = safeEvalCondition(match[1]);
      return condition
        ? safeEvalArithmetic(match[2])
        : safeEvalArithmetic(match[3]);
    } catch {
      return "#ERROR";
    }
  }

  function evaluateAnd(expr) {
    const match = expr.match(/AND\(([^)]+)\)/i);
    if (!match) return "#ERROR";
    const parts = match[1].split(",");
    for (let i = 0; i < parts.length; i++) {
      const val = safeEvalArithmetic(parts[i].trim());
      if (!val) return 0;
    }
    return 1;
  }

  function evaluateOr(expr) {
    const match = expr.match(/OR\(([^)]+)\)/i);
    if (!match) return "#ERROR";
    const parts = match[1].split(",");
    for (let i = 0; i < parts.length; i++) {
      const val = safeEvalArithmetic(parts[i].trim());
      if (val) return 1;
    }
    return 0;
  }

  function evaluateNot(expr) {
    const match = expr.match(/NOT\(([^)]+)\)/i);
    if (!match) return "#ERROR";
    const val = safeEvalArithmetic(match[1].trim());
    return val ? 0 : 1;
  }

  function parseRange(rangeStr) {
    const values = [];
    const parts = rangeStr.split(":");

    if (parts.length === 2) {
      const start = parseCellRef(parts[0].trim());
      const end = parseCellRef(parts[1].trim());
      if (start && end) {
        for (let r = start.row; r <= end.row; r++) {
          for (let c = start.col; c <= end.col; c++) {
            const val = parseFloat(getCellValue(r, c));
            if (!isNaN(val)) values.push(val);
          }
        }
      }
    } else {
      const ref = parseCellRef(parts[0].trim());
      if (ref) {
        const val = parseFloat(getCellValue(ref.row, ref.col));
        if (!isNaN(val)) values.push(val);
      }
    }

    return values;
  }

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
  function copySelection() {
    state.clipboard = getSelectionData();
    state.clipboardMode = "copy";
    showCopyBox();
  }
  function cutSelection() {
    state.clipboard = getSelectionData();
    state.clipboardMode = "cut";
    showCopyBox();
  }
  function pasteSelection() {
    if (!state.clipboard) return;
    saveToHistory();
    const { row, col } = state.activeCell;
    const ws = state.worksheets[state.activeWorksheet];
    state.clipboard.forEach((rowData, rOffset) => {
      rowData.forEach((cellData, cOffset) => {
        const targetRow = row + rOffset;
        const targetCol = col + cOffset;
        if (!cellData) return;
        if (typeof window.setCellValue === "function") {
          const v = cellData.formula ? cellData.formula : (cellData.value != null ? String(cellData.value) : "");
          window.setCellValue(targetRow, targetCol, v, { skipHistory: true });
        } else {
          const key = `${targetRow},${targetCol}`;
          ws.data[key] = { ...cellData };
          renderCell(targetRow, targetCol);
        }
      });
    });
    if (state.clipboardMode === "cut") {
      clearSourceCells();
      state.clipboardMode = null;
    }
    hideCopyBox();
    state.isDirty = true;
    scheduleAutoSave();
  }
  function getSelectionData() {
    const { start, end } = state.selection;
    const data = [];
    for (let r = start.row; r <= end.row; r++) {
      const rowData = [];
      for (let c = start.col; c <= end.col; c++) {
        rowData.push(getCellData(r, c) || null);
      }
      data.push(rowData);
    }
    return data;
  }
  function clearSourceCells() {
    const { start, end } = state.selection;
    const ws = state.worksheets[state.activeWorksheet];
    for (let r = start.row; r <= end.row; r++) {
      for (let c = start.col; c <= end.col; c++) {
        delete ws.data[`${r},${c}`];
        renderCell(r, c);
      }
    }
  }
  function clearCells() {
    saveToHistory();
    const { start, end } = state.selection;
    const ws = state.worksheets[state.activeWorksheet];
    for (let r = start.row; r <= end.row; r++) {
      for (let c = start.col; c <= end.col; c++) {
        delete ws.data[`${r},${c}`];
        renderCell(r, c);
      }
    }
    state.isDirty = true;
    scheduleAutoSave();
  }
  function showCopyBox() {
    const copyBox = document.getElementById("copyBox");
    if (copyBox) copyBox.classList.remove("hidden");
  }
  function hideCopyBox() {
    const copyBox = document.getElementById("copyBox");
    if (copyBox) copyBox.classList.add("hidden");
  }
  function formatCells(format, value) {
    saveToHistory();
    const { start, end } = state.selection;
    const ws = state.worksheets[state.activeWorksheet];
    for (let r = start.row; r <= end.row; r++) {
      for (let c = start.col; c <= end.col; c++) {
        const key = `${r},${c}`;
        if (!ws.data[key]) ws.data[key] = { value: "" };
        if (!ws.data[key].style) ws.data[key].style = {};
        const style = ws.data[key].style;
        switch (format) {
          case "bold":
            style.fontWeight = style.fontWeight === "bold" ? "normal" : "bold";
            break;
          case "italic":
            style.fontStyle =
              style.fontStyle === "italic" ? "normal" : "italic";
            break;
          case "underline":
            style.textDecoration =
              style.textDecoration === "underline" ? "none" : "underline";
            break;
          case "strikethrough":
            style.textDecoration =
              style.textDecoration === "line-through" ? "none" : "line-through";
            break;
          case "alignLeft":
            style.textAlign = "left";
            break;
          case "alignCenter":
            style.textAlign = "center";
            break;
          case "alignRight":
            style.textAlign = "right";
            break;
          case "fontFamily":
            style.fontFamily = value;
            break;
          case "fontSize":
            style.fontSize = value;
            break;
          case "color":
            style.color = value;
            break;
          case "backgroundColor":
            style.background = value;
            break;
          case "currency":
            if (ws.data[key].value) {
              const num = parseFloat(ws.data[key].value);
              if (!isNaN(num)) ws.data[key].value = "$" + num.toFixed(2);
            }
            break;
          case "percent":
            if (ws.data[key].value) {
              const num = parseFloat(ws.data[key].value);
              if (!isNaN(num))
                ws.data[key].value = (num * 100).toFixed(0) + "%";
            }
            break;
        }
        renderCell(r, c);
      }
    }
    state.isDirty = true;
    scheduleAutoSave();
  }

  window.pasteSelection = pasteSelection;
  window.cutSelection = cutSelection;
  window.getSelectionData = getSelectionData;
  window.clearCells = clearCells;
